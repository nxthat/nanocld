use ntex::web;
use diesel::prelude::*;

use crate::components;
use crate::models::{Pool, VmItem};
use crate::errors::HttpResponseError;

use super::errors::db_blocking_error;

pub async fn create(
  item: VmItem,
  pool: &web::types::State<Pool>,
) -> Result<VmItem, HttpResponseError> {
  use crate::schema::virtual_machines::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::insert_into(dsl::virtual_machines)
      .values(&item)
      .execute(&mut conn)?;
    Ok(item)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(res) => Ok(res),
  }
}
