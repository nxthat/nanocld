// std libs
use std::path::Path;
// imported libs
use url::Url;
use ntex::rt;
use ntex::web;
use ntex::util::Bytes;
use ntex::http::StatusCode;
use ntex::channel::mpsc::channel as nt_channel;
use futures::StreamExt;
// local libs
use crate::{utils, services};
use crate::config::DaemonConfig;
use crate::models::{Pool, VmImagePartial, VmImageImportPayload};
use crate::errors::HttpResponseError;

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
    let mut dw_res = utils::download_file(&url, download_dir).await?;
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

  // #[ntex::test]
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
