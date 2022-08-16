use std::fs;
use std::os::unix::prelude::FileExt;
use std::path::Path;
use ntex::rt;
use ntex::web;
use ntex::http::{Client, StatusCode};
use url::Url;
use futures::TryStreamExt;

use crate::config::DaemonConfig;
use crate::models::VmImageImportPayload;
use crate::errors::HttpResponseError;

// async fn download_file() {

// }

#[web::post("/virtual_machine_images/import")]
async fn import_virtual_machine_images(
  web::types::Json(payload): web::types::Json<VmImageImportPayload>,
  config: web::types::State<DaemonConfig>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let client = Client::new();

  let url = Url::parse(&payload.url).map_err(|err| HttpResponseError {
    status: StatusCode::BAD_REQUEST,
    msg: format!(
      "The url {:?} seems not valid got parsing error {:?}",
      &payload.url, &err
    ),
  })?;
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

  let file_path = Path::new(&config.state_dir)
    .join("qemu/images")
    .join(file_name);

  // Maybe security breach should check if url point to any local interfaces.
  // use url.host to check
  let res =
    client
      .get(&payload.url)
      .send()
      .await
      .map_err(|err| HttpResponseError {
        status: StatusCode::BAD_REQUEST,
        msg: format!("Unable to get {:?} {:?}", payload.url, err),
      })?;

  println!("{:#?}", res.headers());
  let total_size = res
    .header("content-length")
    .ok_or_else(|| HttpResponseError {
      status: StatusCode::BAD_REQUEST,
      msg: format!(
        "Unable to download {:?} content-length not set.",
        &payload.url
      ),
    })?
    .to_str()
    .map_err(|err| HttpResponseError {
      status: StatusCode::BAD_REQUEST,
      msg: format!(
        "Unable to download {:?} cannot convert content-length got error {:?}",
        &payload.url, &err
      ),
    })?
    .parse::<u64>()
    .map_err(|err| HttpResponseError {
      status: StatusCode::BAD_REQUEST,
      msg: format!(
        "Unable to download {:?} cannot convert content-length got error {:?}",
        &payload.url, &err
      ),
    })?;
  let status = res.status();
  if !status.is_success() {
    return Err(HttpResponseError {
      status,
      msg: format!(
        "Unable to get {:?} got response e&rror {:?}",
        &payload.url, &status
      ),
    });
  }

  rt::spawn(async move {
    let mut stream = res.into_stream();
    let file =
      fs::File::create(&file_path).map_err(|err| HttpResponseError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!(
          "Unable to create file {:?} got error {:?}",
          &file_path, &err
        ),
      })?;
    let mut offset: u64 = 0;
    while let Some(chunk) =
      stream.try_next().await.map_err(|err| HttpResponseError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!(
          "Unable to load stream from {:?} got error {:?}",
          &payload.url, &err
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
      println!("offset {:?} total_size {:?}", &offset, &total_size);
      let percent = offset / total_size * 100;
      println!("Download {:?} status {:?}%", &payload.url, &percent);
      log::debug!("Download {:?} status {:?}%", &payload.url, &percent);
    }

    file.sync_all().map_err(|err| HttpResponseError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      msg: format!("Unable to sync file {:?} got error {:?}", &file_path, &err),
    })?;
    Ok::<(), HttpResponseError>(())
  });

  Ok(web::HttpResponse::Ok().finish())
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(import_virtual_machine_images);
}

#[cfg(test)]
mod test {

  use serde_json::json;

  use super::ntex_config;
  use crate::utils::test::*;

  #[ntex::test]
  async fn test_import_image() -> TestReturn {
    let srv = generate_server(ntex_config).await;

    let mut res = srv.post("/virtual_machine_images/import").send_json(&json!({
      "name": "test",
      "url": "https://cloud-images.ubuntu.com/releases/jammy/release/ubuntu-22.04-server-cloudimg-amd64.img",
    })).await?;

    println!("{:#?}", res);
    println!("{:#?}", res.body().await);
    Ok(())
  }
}
