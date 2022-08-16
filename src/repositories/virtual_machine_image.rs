use ntex::web;
use diesel::prelude::*;

use crate::components;
use crate::models::{Pool, VmImageItem};
use crate::errors::HttpResponseError;

use super::errors::db_blocking_error;

pub async fn create(
  item: VmImageItem,
  pool: &web::types::State<Pool>,
) -> Result<VmImageItem, HttpResponseError> {
  use crate::schema::virtual_machine_images::dsl;

  let conn = components::postgresql::get_pool_conn(pool)?;

  let res = web::block(move || {
    diesel::insert_into(dsl::virtual_machine_images)
      .values(&item)
      .execute(&conn)?;
    Ok(item)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}
