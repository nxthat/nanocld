use ntex::web;
use diesel::prelude::*;

use crate::{utils, controllers};
use crate::errors::HttpResponseError;
use crate::models::{
  Pool, ClusterNetworkPartial, ClusterNetworkItem, GenericDelete, ClusterItem,
  GenericCount,
};

use super::errors::db_blocking_error;

pub async fn list_for_cluster(
  cluster: ClusterItem,
  pool: &Pool,
) -> Result<Vec<ClusterNetworkItem>, HttpResponseError> {
  let mut conn = controllers::store::get_pool_conn(pool)?;

  let res = web::block(move || {
    ClusterNetworkItem::belonging_to(&cluster).load(&mut conn)
  })
  .await;
  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(items) => Ok(items),
  }
}

pub async fn count_by_namespace(
  namespace: String,
  pool: &Pool,
) -> Result<GenericCount, HttpResponseError> {
  use crate::schema::cluster_networks::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::cluster_networks
      .filter(dsl::namespace.eq(namespace))
      .count()
      .get_result(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(result) => Ok(GenericCount { count: result }),
  }
}

pub async fn create_for_cluster(
  namespace_name: String,
  cluster_name: String,
  item: ClusterNetworkPartial,
  docker_network_id: String,
  default_gateway: String,
  pool: &Pool,
) -> Result<ClusterNetworkItem, HttpResponseError> {
  use crate::schema::cluster_networks::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;

  let res = web::block(move || {
    let cluster_key = utils::key::gen_key(&namespace_name, &cluster_name);
    let item = ClusterNetworkItem {
      key: utils::key::gen_key(&cluster_key, &item.name),
      cluster_key,
      name: item.name,
      default_gateway,
      docker_network_id,
      namespace: namespace_name,
    };
    diesel::insert_into(dsl::cluster_networks)
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

pub async fn delete_by_key(
  key: String,
  pool: &Pool,
) -> Result<GenericDelete, HttpResponseError> {
  use crate::schema::cluster_networks::dsl;
  let mut conn = controllers::store::get_pool_conn(pool)?;

  let res = web::block(move || {
    diesel::delete(dsl::cluster_networks)
      .filter(dsl::key.eq(key))
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
  pool: &Pool,
) -> Result<ClusterNetworkItem, HttpResponseError> {
  use crate::schema::cluster_networks::dsl;
  let mut conn = controllers::store::get_pool_conn(pool)?;

  let res = web::block(move || {
    dsl::cluster_networks
      .filter(dsl::key.eq(key))
      .get_result(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}
