use ntex::web;
use diesel::prelude::*;

use crate::components;
use crate::models::{Pool, VmImageItem, GenericDelete};
use crate::errors::HttpResponseError;

use super::errors::db_blocking_error;

pub async fn create(
  item: VmImageItem,
  pool: &web::types::State<Pool>,
) -> Result<VmImageItem, HttpResponseError> {
  use crate::schema::virtual_machine_images::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::insert_into(dsl::virtual_machine_images)
      .values(&item)
      .execute(&mut conn)?;
    Ok(item)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}

pub async fn find_by_id(
  key: String,
  pool: &web::types::State<Pool>,
) -> Result<VmImageItem, HttpResponseError> {
  use crate::schema::virtual_machine_images::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::virtual_machine_images
      .filter(dsl::key.eq(key))
      .get_result(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}

pub async fn delete_by_id(
  key: String,
  pool: &web::types::State<Pool>,
) -> Result<GenericDelete, HttpResponseError> {
  use crate::schema::virtual_machine_images::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::delete(dsl::virtual_machine_images.filter(dsl::key.eq(key)))
      .execute(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(count) => Ok(GenericDelete { count }),
  }
}
