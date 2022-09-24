use ntex::web;
use diesel::prelude::*;

use crate::components;
use crate::models::{Pool, CargoEnvPartial, CargoEnvItem, GenericDelete};

use crate::errors::HttpResponseError;
use super::errors::db_blocking_error;

pub async fn create(
  item: CargoEnvPartial,
  pool: &web::types::State<Pool>,
) -> Result<CargoEnvItem, HttpResponseError> {
  use crate::schema::cargo_environnements::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    let item = CargoEnvItem {
      key: item.cargo_key.to_owned() + "-" + &item.name,
      cargo_key: item.cargo_key,
      name: item.name,
      value: item.value,
    };
    diesel::insert_into(dsl::cargo_environnements)
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

pub async fn exist_in_cargo(
  name: String,
  cargo_key: String,
  pool: &web::types::State<Pool>,
) -> Result<bool, HttpResponseError> {
  use crate::schema::cargo_environnements::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::select(diesel::dsl::exists(
      dsl::cargo_environnements
        .filter(dsl::name.eq(&name))
        .filter(dsl::cargo_key.eq(&cargo_key)),
    ))
    .get_result(&mut conn)
  })
  .await;
  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(data) => Ok(data),
  }
}

pub async fn patch_for_cargo(
  name: String,
  cargo_key: String,
  value: String,
  pool: &web::types::State<Pool>,
) -> Result<CargoEnvItem, HttpResponseError> {
  use crate::schema::cargo_environnements::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::update(
      dsl::cargo_environnements
        .filter(dsl::name.eq(&name))
        .filter(dsl::cargo_key.eq(&cargo_key)),
    )
    .set(dsl::value.eq(&value))
    .get_result(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}

pub async fn create_many(
  items: Vec<CargoEnvPartial>,
  pool: &web::types::State<Pool>,
) -> Result<Vec<CargoEnvItem>, HttpResponseError> {
  use crate::schema::cargo_environnements::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    let records = items
      .into_iter()
      .map(|item| CargoEnvItem {
        key: item.cargo_key.to_owned() + "-" + &item.name,
        cargo_key: item.cargo_key,
        name: item.name,
        value: item.value,
      })
      .collect::<Vec<CargoEnvItem>>();

    diesel::insert_into(dsl::cargo_environnements)
      .values(&records)
      .execute(&mut conn)?;
    Ok(records)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(items) => Ok(items),
  }
}

// May be needed later
pub async fn _delete_by_key(
  key: String,
  pool: &web::types::State<Pool>,
) -> Result<GenericDelete, HttpResponseError> {
  use crate::schema::cargo_environnements::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::delete(dsl::cargo_environnements.filter(dsl::key.eq(key)))
      .execute(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(result) => Ok(GenericDelete { count: result }),
  }
}

pub async fn delete_by_cargo_key(
  cargo_key: String,
  pool: &web::types::State<Pool>,
) -> Result<GenericDelete, HttpResponseError> {
  use crate::schema::cargo_environnements::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::delete(
      dsl::cargo_environnements.filter(dsl::cargo_key.eq(cargo_key)),
    )
    .execute(&mut conn)
  })
  .await;
  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(result) => Ok(GenericDelete { count: result }),
  }
}

pub async fn list_by_cargo_key(
  cargo_key: String,
  pool: &web::types::State<Pool>,
) -> Result<Vec<CargoEnvItem>, HttpResponseError> {
  use crate::schema::cargo_environnements::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::cargo_environnements
      .filter(dsl::cargo_key.eq(cargo_key))
      .get_results(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(items) => Ok(items),
  }
}
