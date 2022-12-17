use ntex::rt;
use ntex::util::Bytes;
use ntex::web;
use ntex::channel::mpsc::{self, Receiver};
use ntex::http::StatusCode;
use futures::StreamExt;
use bollard::Docker;
use bollard::models::{ImageInspect, ImageSummary};

use crate::errors::HttpResponseError;
use crate::models::GenericDelete;

/// List all cargo/container images
///
/// ## Arguments
/// - [docker_api](bollard::Docker) docker api client
///
/// ## Return
/// - [Result](Vec<ImageSummary>) - List of images
/// - [Result](HttpResponseError) - An http response error if something went wrong
pub async fn list(
  docker_api: &Docker,
) -> Result<Vec<ImageSummary>, HttpResponseError> {
  let items = docker_api
    .list_images(Some(bollard::image::ListImagesOptions::<String> {
      all: true,
      ..Default::default()
    }))
    .await?;

  Ok(items)
}

/// Inspect a cargo/container image
///
/// ## Arguments
/// - [image_name](str) name of the image to inspect
/// - [docker_api](bollard::Docker) docker api client
///
/// ## Return
/// - [Result](ImageInspect) - Image inspect
/// - [Result](HttpResponseError) - An http response error if something went wrong
pub async fn inspect(
  image_name: &str,
  docker_api: &Docker,
) -> Result<ImageInspect, HttpResponseError> {
  let image = docker_api.inspect_image(image_name).await?;

  Ok(image)
}

/// Download a cargo/container image
///
/// ## Arguments
/// - [image_name](str) name of the image to download
/// - [tag](str) tag of the image to download
/// - [docker_api](bollard::Docker) docker api client
///
/// ## Return
/// - [Result](HttpResponseError) - An http response error if something went wrong
///
pub async fn download(
  from_image: &str,
  tag: &str,
  docker_api: &Docker,
) -> Result<Receiver<Result<Bytes, web::error::Error>>, HttpResponseError> {
  let from_image = from_image.to_owned();
  let tag = tag.to_owned();
  let docker_api = docker_api.to_owned();
  let (tx, rx_body) = mpsc::channel();

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
          let _ = tx.send(Err::<_, web::error::Error>(err));
          break;
        }
        Ok(result) => {
          let data = match serde_json::to_string(&result) {
            Err(err) => {
              let err =
                ntex::web::Error::new(web::error::InternalError::default(
                  format!("{:?}", err),
                  StatusCode::INTERNAL_SERVER_ERROR,
                ));
              let _ = tx.send(Err::<_, web::error::Error>(err));
              break;
            }
            Ok(data) => data,
          };
          // Add the length of the data to the beginning of the stream
          // The length is an usize
          // The stream is terminated by a newline
          let len = data.len();
          let response = format!("{}\n{}\n", len, data);

          if tx
            .send(Ok::<_, web::error::Error>(Bytes::from(response)))
            .is_err()
          {
            break;
          }
        }
      }
    }
  });
  Ok(rx_body)
}

/// Delete a cargo/container image
///
/// ## Arguments
/// - [image_name](str) name of the image to delete
/// - [docker_api](bollard::Docker) docker api client
///
/// ## Return
/// - [Result](HttpResponseError) - An http response error if something went wrong
/// - [Result](GenericDelete) - Delete response
pub async fn delete(
  id_or_name: &str,
  docker_api: &Docker,
) -> Result<GenericDelete, HttpResponseError> {
  docker_api.remove_image(&id_or_name, None, None).await?;
  let res = GenericDelete { count: 1 };

  Ok(res)
}
