use ntex::web;
use diesel::prelude::*;

use crate::controllers;
use crate::models::{
  Pool, ClusterVariablePartial, ClusterVariableItem, GenericDelete,
};

use crate::errors::HttpResponseError;

use super::errors::db_blocking_error;

pub async fn create(
  cluster_key: String,
  item: ClusterVariablePartial,
  pool: &web::types::State<Pool>,
) -> Result<ClusterVariableItem, HttpResponseError> {
  use crate::schema::cluster_variables::dsl;

  let mut conn = controllers::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    let item = ClusterVariableItem {
      key: format!("{}-{}", cluster_key, item.name),
      cluster_key: cluster_key.to_owned(),
      name: item.name.to_owned(),
      value: item.value.to_owned(),
    };
    diesel::insert_into(dsl::cluster_variables)
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

pub async fn list_by_cluster(
  cluster_key: String,
  pool: &web::types::State<Pool>,
) -> Result<Vec<ClusterVariableItem>, HttpResponseError> {
  use crate::schema::cluster_variables::dsl;

  let mut conn = controllers::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::cluster_variables
      .filter(dsl::cluster_key.eq(cluster_key))
      .get_results(&mut conn)
  })
  .await;
  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(items) => Ok(items),
  }
}

pub async fn delete_by_cluster_key(
  cluster_key: String,
  pool: &web::types::State<Pool>,
) -> Result<GenericDelete, HttpResponseError> {
  use crate::schema::cluster_variables::dsl;

  let mut conn = controllers::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::delete(
      dsl::cluster_variables.filter(dsl::cluster_key.eq(cluster_key)),
    )
    .execute(&mut conn)
  })
  .await;
  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(result) => Ok(GenericDelete { count: result }),
  }
}

pub async fn delete_by_key(
  key: String,
  pool: &web::types::State<Pool>,
) -> Result<GenericDelete, HttpResponseError> {
  use crate::schema::cluster_variables::dsl;

  let mut conn = controllers::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::delete(dsl::cluster_variables.filter(dsl::key.eq(key)))
      .execute(&mut conn)
  })
  .await;
  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(result) => Ok(GenericDelete { count: result }),
  }
}

pub async fn find_by_key(
  key: String,
  pool: &web::types::State<Pool>,
) -> Result<ClusterVariableItem, HttpResponseError> {
  use crate::schema::cluster_variables::dsl;

  let mut conn = controllers::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::cluster_variables
      .filter(dsl::key.eq(key))
      .get_result(&mut conn)
  })
  .await;
  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}
