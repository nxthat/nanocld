// std libs
use std::fs;
use std::path::Path;
use std::os::unix::prelude::FileExt;
use futures::StreamExt;
// imported libs
use url::Url;
use ntex::rt;
use ntex::web;
use ntex::util::Bytes;
use ntex::http::{Client, StatusCode};
use ntex::channel::mpsc::channel as nt_channel;
use serde::Deserialize;
use serde::Serialize;
use futures::{SinkExt, TryStreamExt};
use futures::channel::mpsc::{unbounded, UnboundedReceiver};
// local libs
use crate::config::DaemonConfig;
use crate::models::Pool;
use crate::models::VmImageImportPayload;
use crate::errors::HttpResponseError;
use crate::models::VmImagePartial;
use crate::services;

#[derive(Debug, Serialize, Deserialize)]
pub enum DownloadFileStatus {
  Downloading,
  Syncing,
  Done,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadFileInfo {
  pub(crate) percent: f64,
  pub(crate) status: DownloadFileStatus,
}

impl Default for DownloadFileInfo {
  fn default() -> Self {
    Self {
      percent: 0.0,
      status: DownloadFileStatus::Downloading,
    }
  }
}

impl DownloadFileInfo {
  fn new(percent: f64, status: DownloadFileStatus) -> Self {
    Self { percent, status }
  }
}

pub struct DownloadFileRes {
  pub(crate) path: String,
  pub(crate) stream:
    UnboundedReceiver<Result<DownloadFileInfo, HttpResponseError>>,
}

async fn download_file(
  url: &Url,
  download_dir: impl AsRef<Path>,
) -> Result<DownloadFileRes, HttpResponseError> {
  // Im a noob so what ?
  let file_name = url
    .path_segments()
    .ok_or_else(|| HttpResponseError {
      status: StatusCode::BAD_REQUEST,
      msg: String::from("url have empty path cannot determine file name lol."),
    })?
    .last()
    .ok_or_else(|| HttpResponseError {
      status: StatusCode::BAD_REQUEST,
      msg: String::from("url have empty path cannot determine file name lol."),
    })?;
  let client = Client::new();
  let res = client.get(url.to_string()).send().await.map_err(|err| {
    HttpResponseError {
      status: StatusCode::BAD_REQUEST,
      msg: format!("Unable to get {:?} {:?}", &url, &err),
    }
  })?;
  let status = res.status();
  if !status.is_success() {
    return Err(HttpResponseError {
      status,
      msg: format!(
        "Unable to get {:?} got response e&rror {:?}",
        &url, &status
      ),
    });
  }
  let total_size = res
    .header("content-length")
    .ok_or_else(|| HttpResponseError {
      status: StatusCode::BAD_REQUEST,
      msg: format!("Unable to download {:?} content-length not set.", &url),
    })?
    .to_str()
    .map_err(|err| HttpResponseError {
      status: StatusCode::BAD_REQUEST,
      msg: format!(
        "Unable to download {:?} cannot convert content-length got error {:?}",
        &url, &err
      ),
    })?
    .parse::<u64>()
    .map_err(|err| HttpResponseError {
      status: StatusCode::BAD_REQUEST,
      msg: format!(
        "Unable to download {:?} cannot convert content-length got error {:?}",
        &url, &err
      ),
    })?;
  let url = url.to_owned();
  // let (tx_body, rx_body) = channel::<Result<Bytes, web::error::Error>>();
  let file_path = Path::new(download_dir.as_ref()).join(file_name);
  let ret_file_path = file_name.to_owned();
  let (mut wtx, wrx) =
    unbounded::<Result<DownloadFileInfo, HttpResponseError>>();
  rt::spawn(async move {
    let mut stream = res.into_stream();
    let fp = file_path.to_owned();
    let file = web::block(move || {
      let file = fs::File::create(&fp).map_err(|err| HttpResponseError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!("Unable to create file {:?} got error {:?}", &fp, &err),
      })?;
      Ok::<_, HttpResponseError>(file)
    })
    .await
    .map_err(|err| HttpResponseError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      msg: format!("{}", err),
    })?;
    let mut offset: u64 = 0;
    while let Some(chunk) =
      stream.try_next().await.map_err(|err| HttpResponseError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!(
          "Unable to load stream from {:?} got error {:?}",
          &url, &err
        ),
      })?
    {
      file
        .write_at(&chunk, offset)
        .map_err(|err| HttpResponseError {
          status: StatusCode::INTERNAL_SERVER_ERROR,
          msg: format!(
            "Unable to write in file {:?} got error {:?}",
            &file_path, &err
          ),
        })?;
      offset += chunk.len() as u64;
      let percent = (offset as f64 / total_size as f64) * 100.0;
      log::debug!("Downloading file from {:?} status {:?}%", &url, &percent);
      let info =
        DownloadFileInfo::new(percent, DownloadFileStatus::Downloading);
      let send = wtx.send(Ok::<_, HttpResponseError>(info)).await;
      if let Err(_err) = send {
        break;
      }
    }

    if offset == total_size {
      file.sync_all().map_err(|err| HttpResponseError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!(
          "Unable to sync file {:?} got error {:?}",
          &file_path, &err
        ),
      })?;
      let info = DownloadFileInfo::new(100.0, DownloadFileStatus::Done);
      let _send = wtx.send(Ok::<_, HttpResponseError>(info)).await;
    } else {
      fs::remove_file(&file_path).map_err(|err| HttpResponseError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!("Unable to delete created file {:?}", err),
      })?;
    }
    Ok::<(), HttpResponseError>(())
  });
  let res = DownloadFileRes {
    path: ret_file_path,
    stream: wrx,
  };
  Ok(res)
}

#[web::post("/virtual_machine_images/import")]
async fn import_virtual_machine_images(
  web::types::Json(payload): web::types::Json<VmImageImportPayload>,
  pool: web::types::State<Pool>,
  config: web::types::State<DaemonConfig>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let url = Url::parse(&payload.url).map_err(|err| HttpResponseError {
    status: StatusCode::BAD_REQUEST,
    msg: format!(
      "The url {:?} seems not valid got parsing error {:?}",
      &payload.url, &err
    ),
  })?;
  let download_dir = Path::new(&config.state_dir).join("qemu/images");
  let (tx, rx) = nt_channel();
  rt::spawn(async move {
    let mut dw_res = download_file(&url, download_dir).await?;
    while let Some(chunk) = dw_res.stream.next().await {
      match chunk {
        Err(err) => {
          let _res =
            tx.send(Err::<_, web::error::Error>(web::error::Error::new(err)));
          break;
        }
        Ok(info) => {
          let data = serde_json::to_string(&info).unwrap();
          let buffer = Bytes::from(data);
          if let Err(err) = tx.send(Ok::<_, web::error::Error>(buffer)) {
            log::debug!("Vm image import download send error {:?}", &err);
            break;
          }
        }
      }
    }
    let item = VmImagePartial {
      name: payload.name,
      image_path: dw_res.path,
      is_base: true,
      parent_key: None,
    };
    services::virtual_machine_images::create(item, &pool, &config).await?;
    Ok::<_, HttpResponseError>(())
  });
  Ok(
    web::HttpResponse::Ok()
      .keep_alive()
      .content_type("nanocl/streaming-v1")
      .streaming(rx),
  )
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(import_virtual_machine_images);
}

#[cfg(test)]
mod test {
  use serde_json::json;
  use futures::TryStreamExt;

  use super::ntex_config;
  use crate::utils::test::*;

  #[ntex::test]
  async fn test_import_image() -> TestReturn {
    let srv = generate_server(ntex_config).await;

    let res = srv.post("/virtual_machine_images/import").send_json(&json!({
      "name": "test",
      "url": "https://cloud-images.ubuntu.com/releases/jammy/release/ubuntu-22.04-server-cloudimg-amd64.img",
    })).await?;

    println!("{:#?}", res);
    let mut stream = res.into_stream();

    while let Some(chunk) = stream.try_next().await? {
      println!("{:?}", chunk);
    }
    Ok(())
  }
}
