use ntex::rt;
use ntex::web;
use ntex::http::StatusCode;
use ntex::util::Bytes;
use ntex::channel::mpsc;
use futures::StreamExt;

use crate::models::CargoImagePartial;
use crate::errors::HttpResponseError;
use crate::models::GenericDelete;

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
  let res = GenericDelete { count: 1 };
  Ok(web::HttpResponse::Ok().json(&res))
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(list_cargo_image);
  config.service(create_cargo_image);
  config.service(delete_cargo_image_by_name);
  config.service(inspect_cargo_image);
}

/// Cargo image unit tests
#[cfg(test)]
pub mod tests {

  use bollard::service::ImageInspect;
  use ntex::http::StatusCode;
  use futures::{TryStreamExt, StreamExt};

  use super::*;
  use crate::{utils::tests::*, models::GenericDelete};

  /// Test utils to list cargo images
  pub async fn list(srv: &TestServer) -> TestReqRet {
    srv.get("/cargoes/images").send().await
  }

  /// Test utils to create cargo image
  pub async fn create(
    srv: &TestServer,
    payload: &CargoImagePartial,
  ) -> TestReqRet {
    srv.post("/cargoes/images").send_json(payload).await
  }

  /// Test utils to inspect cargo image
  pub async fn inspect(srv: &TestServer, id_or_name: &str) -> TestReqRet {
    srv
      .get(format!("/cargoes/images/{}", id_or_name))
      .send()
      .await
  }

  /// Test utils to delete cargo image
  pub async fn delete(srv: &TestServer, id_or_name: &str) -> TestReqRet {
    srv
      .delete(format!("/cargoes/images/{}", id_or_name))
      .send()
      .await
  }

  /// Basic test to list cargo images
  #[ntex::test]
  pub async fn basic_list() -> TestRet {
    let srv = generate_server(ntex_config).await;

    let resp = list(&srv).await?;
    let status = resp.status();
    assert_eq!(
      status,
      StatusCode::OK,
      "Expect basic to return status {} got {}",
      StatusCode::OK,
      status
    );

    Ok(())
  }

  /// Basic test to create cargo image with wrong name
  #[ntex::test]
  pub async fn basic_create_wrong_name() -> TestRet {
    let srv = generate_server(ntex_config).await;

    let payload = CargoImagePartial {
      name: "test".to_string(),
    };
    let resp = create(&srv, &payload).await?;
    let status = resp.status();
    assert_eq!(
      status,
      StatusCode::BAD_REQUEST,
      "Expect basic to return status {} got {}",
      StatusCode::BAD_REQUEST,
      status
    );

    Ok(())
  }

  /// Basic test to create, inspect and delete a cargo image
  #[ntex::test]
  async fn crud() -> TestRet {
    const TEST_IMAGE: &str = "busybox:unstable-musl";
    let srv = generate_server(ntex_config).await;

    // Create
    let payload = CargoImagePartial {
      name: TEST_IMAGE.to_owned(),
    };
    let res = create(&srv, &payload).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::OK,
      "Expect create to return status {} got {}",
      StatusCode::OK,
      status
    );
    let content_type = res
      .header("content-type")
      .expect("Expect create response to have content type header")
      .to_str()
      .unwrap();
    assert_eq!(
      content_type, "nanocl/streaming-v1",
      "Expect content type header to be nanocl/streaming-v1 got {}",
      content_type
    );
    let mut stream = res.into_stream();
    while let Some(chunk) = stream.next().await {
      if let Err(err) = chunk {
        panic!("Error while creating image {}", &err);
      }
    }

    // Inspect
    let mut res = inspect(&srv, TEST_IMAGE).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::OK,
      "Expect inspect to return status {} got {}",
      StatusCode::OK,
      status
    );
    let _body: ImageInspect = res
      .json()
      .await
      .expect("Expect inspect to return ImageInspect json data");

    // Delete
    let mut res = delete(&srv, TEST_IMAGE).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::OK,
      "Expect delete to return status {} got {}",
      StatusCode::OK,
      status
    );
    let body: GenericDelete = res
      .json()
      .await
      .expect("Expect delete to return GenericDelete json data");
    assert_eq!(
      body.count, 1,
      "Expect delete to return count 1 got {}",
      body.count
    );

    Ok(())
  }
}
