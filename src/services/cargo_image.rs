use ntex::rt;
use ntex::web;
use ntex::http::StatusCode;
use ntex::util::Bytes;
use ntex::channel::mpsc;
use futures::StreamExt;

use crate::models::CargoImagePartial;
use crate::errors::HttpResponseError;

#[web::get("/cargoes/images")]
async fn list_cargo_image(
  docker_api: web::types::State<bollard::Docker>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let images = docker_api
    .list_images(Some(bollard::image::ListImagesOptions::<String> {
      all: false,
      ..Default::default()
    }))
    .await?;

  Ok(web::HttpResponse::Ok().json(&images))
}

#[web::get("/cargoes/images/{id_or_name}*")]
async fn inspect_cargo_image(
  name: web::types::Path<String>,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let image = docker_api.inspect_image(&name.into_inner()).await?;

  Ok(web::HttpResponse::Ok().json(&image))
}

#[web::post("/cargoes/images")]
async fn create_cargo_image(
  docker_api: web::types::State<bollard::Docker>,
  web::types::Json(payload): web::types::Json<CargoImagePartial>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let image_info = payload.name.split(':').collect::<Vec<&str>>();

  if image_info.len() != 2 {
    return Err(HttpResponseError {
      msg: String::from("missing tag in image name"),
      status: StatusCode::BAD_REQUEST,
    });
  }

  let (tx, rx_body) = mpsc::channel();

  let from_image = image_info[0].to_string();
  let tag = image_info[1].to_string();
  rt::spawn(async move {
    let mut stream = docker_api.create_image(
      Some(bollard::image::CreateImageOptions {
        from_image,
        tag,
        ..Default::default()
      }),
      None,
      None,
    );

    while let Some(result) = stream.next().await {
      match result {
        Err(err) => {
          let err = ntex::web::Error::new(web::error::InternalError::default(
            format!("{:?}", err),
            StatusCode::INTERNAL_SERVER_ERROR,
          ));
          let result = tx.send(Err::<_, web::error::Error>(err));
          if result.is_err() {
            break;
          }
        }
        Ok(result) => {
          let data = match serde_json::to_string(&result) {
            Err(err) => {
              log::error!("unable to stringify create image info {:#?}", err);
              break;
            }
            Ok(data) => data,
          };
          let result = tx.send(Ok::<_, web::error::Error>(Bytes::from(data)));
          if result.is_err() {
            break;
          }
        }
      }
    }
  });

  Ok(
    web::HttpResponse::Ok()
      .keep_alive()
      .content_type("nanocl/streaming-v1")
      .streaming(rx_body),
  )
}

#[web::delete("/cargoes/images/{id_or_name}*")]
async fn delete_cargo_image_by_name(
  docker_api: web::types::State<bollard::Docker>,
  id_or_name: web::types::Path<String>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let id_or_name = id_or_name.into_inner();
  docker_api.remove_image(&id_or_name, None, None).await?;
  Ok(web::HttpResponse::Ok().into())
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(list_cargo_image);
  config.service(create_cargo_image);
  config.service(delete_cargo_image_by_name);
  config.service(inspect_cargo_image);
}

#[cfg(test)]
pub mod tests {

  use futures::{TryStreamExt, StreamExt};

  use super::ntex_config;
  use crate::utils::tests::*;
  use crate::models::CargoImagePartial;

  #[ntex::test]
  pub async fn basic_list() -> TestRet {
    let srv = generate_server(ntex_config).await;

    let resp = srv.get("/cargoes/images").send().await?;
    let status = resp.status();
    assert!(status.is_success());
    Ok(())
  }

  pub async fn create_cargo_image(srv: &TestServer, name: &str) -> TestRet {
    println!("create cargo image {}", name);
    let payload = CargoImagePartial {
      name: name.to_owned(),
    };
    let resp = srv.post("/cargoes/images").send_json(&payload).await?;

    let status = resp.status();

    assert!(status.is_success());
    let content_type = resp.header("content-type").unwrap().to_str().unwrap();
    assert_eq!(content_type, "nanocl/streaming-v1");

    let mut stream = resp.into_stream();

    while let Some(chunk) = stream.next().await {
      if let Err(err) = chunk {
        panic!("Error while downloading image {}", &err);
      }
    }

    Ok(())
  }

  pub async fn inspect_image(srv: &TestServer, name: &str) -> TestRet {
    let resp = srv.get(format!("/cargoes/images/{}", &name)).send().await?;

    let status = resp.status();
    assert!(status.is_success());

    Ok(())
  }

  pub async fn delete_image(srv: &TestServer, name: &str) -> TestRet {
    let resp = srv
      .delete(format!("/cargoes/images/{}", &name))
      .send()
      .await?;

    let status = resp.status();

    assert!(status.is_success());

    Ok(())
  }

  /// Perform crud tests agains cargo images
  #[ntex::test]
  async fn crud() -> TestRet {
    const TEST_IMAGE: &str = "busybox:unstable-musl";
    let srv = generate_server(ntex_config).await;
    create_cargo_image(&srv, TEST_IMAGE).await?;
    inspect_image(&srv, TEST_IMAGE).await?;
    delete_image(&srv, TEST_IMAGE).await?;
    Ok(())
  }
}
