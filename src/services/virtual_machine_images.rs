use std::fs;
use std::path::Path;
use ntex::web;
use ntex::http::StatusCode;

use crate::config::DaemonConfig;
use crate::errors::HttpResponseError;
use crate::models::{Pool, VmImagePartial, VmImageItem};
use crate::repositories;

pub async fn create(
  item: VmImagePartial,
  pool: &web::types::State<Pool>,
  config: &web::types::State<DaemonConfig>,
) -> Result<VmImageItem, HttpResponseError> {
  let path = Path::new(&config.state_dir)
    .join("qemu/images")
    .join(&item.image_path);
  let file_size = fs::metadata(path)
    .map_err(|err| HttpResponseError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      msg: format!("Unable to get file size {:#?}", err),
    })?
    .len()
    .try_into()
    .map_err(|err| HttpResponseError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      msg: format!("Unable to convert u64 to i64 xdlol {:#?}", err),
    })?;

  let item = VmImageItem {
    key: item.name.to_owned(),
    name: item.name,
    image_path: item.image_path,
    size: file_size,
    is_base: true,
    parent_key: None,
  };
  repositories::virtual_machine_image::create(item, pool).await
}
