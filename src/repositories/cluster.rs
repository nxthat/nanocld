//! Functions to manipulate clusters in database
use ntex::web;
use diesel::prelude::*;

use crate::controllers;
use crate::errors::HttpResponseError;
use crate::repositories::errors::db_blocking_error;
use crate::models::{
  Pool, ClusterItem, ClusterPartial, GenericDelete, GenericCount, CargoItem,
  CargoInstanceItem, ClusterVariableItem,
};

/// # Create cluster for namespace
/// Return a fresh cluster with id and gen_id for given namespace
///
/// # Arguments
///
/// - [nsp](String) namespace of the cluster
/// - [item](ClusterPartial) - Cluster to create without id and other generated data
/// - [pool](Pool) - Posgresql database pool
///
/// # Examples
///
/// ```rust,norun
/// use crate::repositories::cluster;

/// let nsp = String::from("default");
/// let new_cluster = ClusterCreate {
///  name: String::from("test-cluster")
/// }
/// let res = cluster::create_for_namespace(nsp, new_cluster, &pool).await;
/// ```
pub async fn create_for_namespace(
  nsp: String,
  item: ClusterPartial,
  pool: &Pool,
) -> Result<ClusterItem, HttpResponseError> {
  use crate::schema::clusters::dsl;
  let mut conn = controllers::store::get_pool_conn(pool)?;

  let res = web::block(move || {
    let k = nsp.to_owned() + "-" + &item.name;
    let new_cluster = ClusterItem {
      key: k,
      namespace: nsp,
      name: item.name,
      proxy_templates: item.proxy_templates.unwrap_or_default(),
    };

    diesel::insert_into(dsl::clusters)
      .values(&new_cluster)
      .execute(&mut conn)?;
    Ok(new_cluster)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}

pub async fn count(
  namespace: String,
  pool: &Pool,
) -> Result<GenericCount, HttpResponseError> {
  use crate::schema::clusters::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::clusters
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

/// # Find by ID
/// Return found cluster related to his ID or an error otherwise
///
/// # Arguments
///
/// * `gen_id` - Generated id of the cluster
/// * `pool` - Posgresql database pool
///
/// # Examples
///
/// ```rust,norun
/// use crate::repositories::cluster;
///
/// let res = cluster::find_by_key(gen_id, &pool).await;
/// ```
pub async fn find_by_key(
  key: String,
  pool: &Pool,
) -> Result<ClusterItem, HttpResponseError> {
  use crate::schema::clusters::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::clusters.filter(dsl::key.eq(key)).get_result(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}

/// # Find by namespace
/// Return from store a list of cluster for given namespace
///
/// # Arguments
///
/// * `nsp` - Namespace name
/// * `pool` - Posgresql database pool
///
/// # Examples
///
/// ```rust,norun
/// use crate::repositories::cluster;

/// let res = cluster::find_by_namespace(gen_id, &pool).await;
/// ```
pub async fn find_by_namespace(
  nsp: String,
  pool: &Pool,
) -> Result<Vec<ClusterItem>, HttpResponseError> {
  use crate::schema::clusters::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::clusters.filter(dsl::namespace.eq(nsp)).load(&mut conn)
  })
  .await;
  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(items) => Ok(items),
  }
}

/// Return number of deleted entries
///
/// # Arguments
///
/// * `gen_id` - Generated id of the cluster to delete
/// * `pool` - Posgresql database pool
///
/// # Examples
///
/// ```rust,norun
/// // Delete cluster by generated id
///
/// use crate::repositories::cluster;
/// cluster::delete_by_gen_id(gen_id, &pool).await;
/// ```
pub async fn delete_by_key(
  key: String,
  pool: &Pool,
) -> Result<GenericDelete, HttpResponseError> {
  use crate::schema::clusters::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::delete(dsl::clusters)
      .filter(dsl::key.eq(key))
      .execute(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(result) => Ok(GenericDelete { count: result }),
  }
}

pub async fn patch_proxy_templates(
  key: String,
  proxy_templates: Vec<String>,
  pool: &Pool,
) -> Result<ClusterItem, HttpResponseError> {
  use crate::schema::clusters::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;

  let cluster = web::block(move || {
    diesel::update(dsl::clusters.filter(dsl::key.eq(key)))
      .set(dsl::proxy_templates.eq(proxy_templates))
      .get_result::<ClusterItem>(&mut conn)
  })
  .await
  .map_err(db_blocking_error)?;

  Ok(cluster)
}

pub async fn list_cargo(
  key: String,
  pool: &Pool,
) -> Result<Vec<CargoItem>, HttpResponseError> {
  use crate::schema::cargo_instances::dsl;
  use crate::schema::cargoes;
  let mut conn = controllers::store::get_pool_conn(pool)?;

  let cargoes = web::block(move || {
    let data: Vec<(CargoInstanceItem, CargoItem)> = dsl::cargo_instances
      .filter(dsl::cluster_key.eq(key))
      .inner_join(cargoes::table)
      .load(&mut conn)?;
    let data = data
      .into_iter()
      .map(|item| item.1)
      .collect::<Vec<CargoItem>>();
    Ok(data)
  })
  .await
  .map_err(db_blocking_error)?;
  Ok(cargoes)
}

pub async fn list_variable(
  key: String,
  pool: &Pool,
) -> Result<Vec<ClusterVariableItem>, HttpResponseError> {
  use crate::schema::cluster_variables::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;

  let res = web::block(move || {
    dsl::cluster_variables
      .filter(dsl::cluster_key.eq(key))
      .get_results(&mut conn)
  })
  .await
  .map_err(db_blocking_error)?;

  Ok(res)
}
