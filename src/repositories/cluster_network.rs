use ntex::web;
use diesel::prelude::*;

use crate::controllers;
use crate::errors::HttpResponseError;
use crate::models::{
  Pool, ClusterNetworkPartial, ClusterNetworkItem, GenericDelete, ClusterItem,
  GenericCount,
};

use super::errors::db_blocking_error;

// Vec<ClusterNetworkItem>
pub async fn list_for_cluster(
  cluster: ClusterItem,
  pool: &web::types::State<Pool>,
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
  pool: &web::types::State<Pool>,
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
  pool: &web::types::State<Pool>,
) -> Result<ClusterNetworkItem, HttpResponseError> {
  use crate::schema::cluster_networks::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;

  let res = web::block(move || {
    let cluster_key = namespace_name.to_owned() + "-" + &cluster_name;
    let item = ClusterNetworkItem {
      key: cluster_key.to_owned() + "-" + &item.name,
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
  pool: &web::types::State<Pool>,
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
  pool: &web::types::State<Pool>,
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

#[cfg(test)]
mod cluster_networks {
  use ntex::web;
  use bollard::network::CreateNetworkOptions;

  use crate::repositories::cluster;
  use crate::models::ClusterPartial;

  use super::*;

  use crate::utils::test::*;

  #[ntex::test]
  async fn main() {
    const NET_NAME: &str = "test-cluster-network";

    let pool = gen_postgre_pool().await;
    let pool_state = web::types::State::new(pool);

    // Create cluster for relationship
    let new_cluster = ClusterPartial {
      name: String::from("dev"),
      proxy_templates: None,
    };
    let cluster = cluster::create_for_namespace(
      String::from("default"),
      new_cluster,
      &pool_state,
    )
    .await
    .unwrap();

    // create docker network for relationship
    let docker = bollard::Docker::connect_with_unix(
      "/run/docker.sock",
      120,
      bollard::API_DEFAULT_VERSION,
    )
    .unwrap();
    let net_config = CreateNetworkOptions {
      name: NET_NAME,
      ..Default::default()
    };
    let network = docker.create_network(net_config).await.unwrap();

    let id = match network.id {
      None => panic!("unable to bind network id"),
      Some(id) => id,
    };

    // create cluster network
    let new_network = ClusterNetworkPartial {
      name: String::from("test-dev"),
    };
    let network = create_for_cluster(
      cluster.namespace,
      cluster.name,
      new_network,
      id,
      String::from("127.0.0.1"),
      &pool_state,
    )
    .await
    .unwrap();

    let n_key = network.key.clone();
    // find cluster network
    find_by_key(n_key.clone(), &pool_state).await.unwrap();

    // delete cluster network
    delete_by_key(n_key.clone(), &pool_state).await.unwrap();

    // clean cluster
    cluster::delete_by_key("default-dev".to_string(), &pool_state)
      .await
      .unwrap();

    // clean docker network
    docker.remove_network(NET_NAME).await.unwrap();
  }
}
