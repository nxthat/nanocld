//! Functions to manipulate clusters in database
use ntex::web;
use diesel::prelude::*;

use crate::services;
use crate::errors::HttpResponseError;
use crate::repositories::errors::db_blocking_error;
use crate::models::{
  Pool, ClusterItem, ClusterPartial, PgDeleteGeneric, PgGenericCount,
};

/// # Create cluster for namespace
/// Return a fresh cluster with id and gen_id for given namespace
///
/// # Arguments
///
/// - [nsp](String) namespace of the cluster
/// - [item](ClusterPartial) - Cluster to create without id and other generated data
/// - [pool](web::types::State<Pool>) - Posgresql database pool
///
/// # Examples
///
/// ```
/// // Create a simple cluster
///
/// use crate::repositories::cluster;
/// let nsp = String::from("default");
/// let new_cluster = ClusterCreate {
///  name: String::from("test-cluster")
/// }
/// cluster::create_for_namespace(nsp, new_cluster, &pool).await;
/// ```
pub async fn create_for_namespace(
  nsp: String,
  item: ClusterPartial,
  pool: &web::types::State<Pool>,
) -> Result<ClusterItem, HttpResponseError> {
  use crate::schema::clusters::dsl;
  let conn = services::postgresql::get_pool_conn(pool)?;

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
      .execute(&conn)?;
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
  pool: &web::types::State<Pool>,
) -> Result<PgGenericCount, HttpResponseError> {
  use crate::schema::clusters::dsl;

  let conn = services::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::clusters
      .filter(dsl::namespace.eq(namespace))
      .count()
      .get_result(&conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(result) => Ok(PgGenericCount { count: result }),
  }
}

/// Return found cluster or an error otherwise
///
/// # Arguments
///
/// * `gen_id` - Generated id of the cluster
/// * `pool` - Posgresql database pool
///
/// # Examples
///
/// ```
/// // Find cluster by id
///
/// use crate::repositories::cluster;
/// cluster::find_by_key(gen_id, &pool).await;
/// ```
pub async fn find_by_key(
  key: String,
  pool: &web::types::State<Pool>,
) -> Result<ClusterItem, HttpResponseError> {
  use crate::schema::clusters::dsl;

  let conn = services::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::clusters.filter(dsl::key.eq(key)).get_result(&conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}

/// Return list of cluster of given namespace
///
/// # Arguments
///
/// * `nsp` - Namespace name
/// * `pool` - Posgresql database pool
///
/// # Examples
///
/// ```
/// // List cluster by namespace
///
/// use crate::repositories::cluster;
/// cluster::find_by_namespace(gen_id, &pool).await;
/// ```
pub async fn find_by_namespace(
  nsp: String,
  pool: &web::types::State<Pool>,
) -> Result<Vec<ClusterItem>, HttpResponseError> {
  use crate::schema::clusters::dsl;

  let conn = services::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::clusters.filter(dsl::namespace.eq(nsp)).load(&conn)
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
/// ```
/// // Delete cluster by generated id
///
/// use crate::repositories::cluster;
/// cluster::delete_by_gen_id(gen_id, &pool).await;
/// ```
pub async fn delete_by_key(
  key: String,
  pool: &web::types::State<Pool>,
) -> Result<PgDeleteGeneric, HttpResponseError> {
  use crate::schema::clusters::dsl;

  let conn = services::postgresql::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::delete(dsl::clusters)
      .filter(dsl::key.eq(key))
      .execute(&conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(result) => Ok(PgDeleteGeneric { count: result }),
  }
}

#[cfg(test)]
mod test_cluster {
  use ntex::web;

  use super::*;

  use crate::utils::test::*;

  #[ntex::test]
  async fn main() {
    const NSP_NAME: &str = "default";
    const CLUSTER_NAME: &str = "test-default-cluster";

    let pool = gen_postgre_pool().await;
    let pool_state = web::types::State::new(pool);

    // test list cluster
    let _res = find_by_namespace(String::from("default"), &pool_state)
      .await
      .unwrap();
    let item = ClusterPartial {
      name: String::from(CLUSTER_NAME),
      proxy_templates: None,
    };
    // test create cluster
    create_for_namespace(String::from(NSP_NAME), item, &pool_state)
      .await
      .unwrap();
    let key = NSP_NAME.to_owned() + "-" + CLUSTER_NAME;
    // test delete cluster
    delete_by_key(key, &pool_state).await.unwrap();
  }
}
