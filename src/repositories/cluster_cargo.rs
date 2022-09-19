use ntex::web;
use diesel::prelude::*;

use crate::components;
use crate::models::{Pool, ClusterCargoPartial, ClusterCargoItem, GenericDelete};

use crate::errors::HttpResponseError;
use crate::repositories::errors::db_blocking_error;

pub async fn create(
  item: ClusterCargoPartial,
  pool: &web::types::State<Pool>,
) -> Result<ClusterCargoItem, HttpResponseError> {
  use crate::schema::cluster_cargoes::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    let item = ClusterCargoItem {
      key: format!("{}-{}", item.cluster_key, item.cargo_key),
      network_key: item.network_key,
      cluster_key: item.cluster_key,
      cargo_key: item.cargo_key,
    };
    diesel::insert_into(dsl::cluster_cargoes)
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

pub async fn get_by_cluster_key(
  cluster_key: String,
  pool: &web::types::State<Pool>,
) -> Result<Vec<ClusterCargoItem>, HttpResponseError> {
  use crate::schema::cluster_cargoes::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::cluster_cargoes
      .filter(dsl::cluster_key.eq(cluster_key))
      .get_results(&mut conn)
  })
  .await;
  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}

pub async fn delete_by_key(
  key: String,
  pool: &web::types::State<Pool>,
) -> Result<GenericDelete, HttpResponseError> {
  use crate::schema::cluster_cargoes::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::delete(dsl::cluster_cargoes.filter(dsl::cluster_key.eq(key)))
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
  use crate::schema::cluster_cargoes::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::delete(dsl::cluster_cargoes.filter(dsl::cargo_key.eq(cargo_key)))
      .execute(&mut conn)
  })
  .await;
  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(result) => Ok(GenericDelete { count: result }),
  }
}

pub async fn find_by_cargo_key(
  cargo_key: String,
  pool: &web::types::State<Pool>,
) -> Result<Vec<ClusterCargoItem>, HttpResponseError> {
  use crate::schema::cluster_cargoes::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::cluster_cargoes
      .filter(dsl::cargo_key.eq(cargo_key))
      .load(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(items) => Ok(items),
  }
}

pub async fn get_by_key(
  key: String,
  pool: &web::types::State<Pool>,
) -> Result<ClusterCargoItem, HttpResponseError> {
  use crate::schema::cluster_cargoes::dsl;

  let mut conn = components::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::cluster_cargoes
      .filter(dsl::key.eq(key))
      .get_result(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}
