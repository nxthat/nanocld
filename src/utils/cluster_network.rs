use std::collections::HashMap;
use ntex::http::StatusCode;
use futures::StreamExt;
use futures::stream::FuturesUnordered;

use super::key::gen_key;
use crate::repositories;
use crate::errors::HttpResponseError;
use crate::models::{
  Pool, ClusterItem, ClusterNetworkItem, ClusterNetworkPartial, GenericDelete,
};

/// Create network in given cluster
/// This function will create a network based on the settings
///
/// ## Arguments
/// - [cluster_key](str) The cluster_key to target
/// - [network_name](str) The name of the network to create
///
/// ## Return
/// - [Result](ClusterNetworkItem) The created network
/// - [Result](HttpResponseError) An http response error if something went wrong
pub async fn create_network(
  nsp: String,
  c_name: String,
  item: ClusterNetworkPartial,
  docker_api: &bollard::Docker,
  pool: &Pool,
) -> Result<ClusterNetworkItem, HttpResponseError> {
  let cluster_key = gen_key(&nsp, &c_name);
  let mut labels = HashMap::new();
  labels.insert(String::from("cluster_key"), cluster_key.to_owned());
  let net_id = gen_key(&cluster_key, &item.name);
  let network_existing =
    repositories::cluster_network::find_by_key(net_id.clone(), pool)
      .await
      .is_ok();
  if network_existing {
    return Err(HttpResponseError {
      status: StatusCode::CONFLICT,
      msg: format!("Unable to create network with name {} a similar network have same name", &item.name),
    });
  }
  let config = bollard::network::CreateNetworkOptions {
    name: net_id,
    driver: String::from("bridge"),
    labels,
    ..Default::default()
  };
  let id = match docker_api.create_network(config).await {
    Err(err) => {
      return Err(HttpResponseError {
        status: StatusCode::BAD_REQUEST,
        msg: format!(
          "Unable to create network with name {} {}",
          &item.name, err
        ),
      })
    }
    Ok(result) => result.id,
  };
  let id = match id {
    None => {
      return Err(HttpResponseError {
        status: StatusCode::BAD_REQUEST,
        msg: format!("Unable to create network with name {}", &item.name),
      })
    }
    Some(id) => id,
  };
  let network = docker_api
    .inspect_network(
      &id,
      None::<bollard::network::InspectNetworkOptions<String>>,
    )
    .await?;

  let ipam_config = network
    .ipam
    .ok_or(HttpResponseError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      msg: String::from("Unable to get ipam config from network"),
    })?
    .config
    .ok_or(HttpResponseError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      msg: String::from("Unable to get ipam config"),
    })?;

  let default_gateway = ipam_config
    .get(0)
    .ok_or(HttpResponseError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      msg: String::from("Unable to get ipam config"),
    })?
    .gateway
    .as_ref()
    .ok_or(HttpResponseError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      msg: String::from("Unable to get ipam config gateway"),
    })?;

  let new_network = repositories::cluster_network::create_for_cluster(
    nsp.to_owned(),
    c_name.to_owned(),
    item,
    id,
    default_gateway.to_owned(),
    pool,
  )
  .await?;

  Ok(new_network)
}

/// Delete network in given cluster
/// This function will delete a network based on his key
///
/// ## Arguments
/// - [key](str) The key of the network to delete
/// - [docker_api](bollard::Docker) The docker api to use
/// - [pool](Pool) The database pool to use
///
/// ## Return
/// - [Result](GenericDelete) The number of deleted networks
/// - [Result](HttpResponseError) An http response error if something went wrong
///
/// ## Example
/// ```rust,norun
/// let docker_api = bollard::Docker::connect_with_local_defaults().unwrap();
/// let pool = db::get_pool();
/// let key = "network_key";
/// let result = delete_network(key.to_owned(), &docker_api, &pool).await;
/// ```
///
/// ## Note
/// This function will not return an error if the network is not found inside docker and will delete it from the database
pub async fn delete_network_by_key(
  key: String,
  docker_api: &bollard::Docker,
  pool: &Pool,
) -> Result<GenericDelete, HttpResponseError> {
  let network = repositories::cluster_network::find_by_key(key, pool).await?;
  if let Err(err) = docker_api.remove_network(&network.docker_network_id).await
  {
    log::warn!("Unable to delete network {} {}", network.name, err);
  };
  repositories::cluster_network::delete_by_key(network.key, pool).await?;
  Ok(GenericDelete { count: 1 })
}

/// Delete all networks in given cluster
/// This function will delete all networks in a given cluster
///
/// ## Arguments
/// - [cluster](ClusterItem) The cluster to target
/// - [docker_api](bollard::Docker) The docker api to use
/// - [pool](Pool) The database pool to use
///
/// ## Return
/// - [Result](()) If everything went well
/// - [Result](HttpResponseError) An http response error if something went wrong
///
/// ## Example
/// ```rust,norun
/// let cluster = repositories::cluster::find_by_key("key".to_owned(), pool).await?;
/// let docker_api = bollard::Docker::connect_with_local_defaults().unwrap();
/// let _ = utils::cluster_network::delete_networks(cluster, &docker_api, pool).await?;
/// ```
///
/// ## Note
/// This function will not return an error if a network is not found
pub async fn delete_networks(
  cluster: ClusterItem,
  docker_api: &bollard::Docker,
  pool: &Pool,
) -> Result<GenericDelete, HttpResponseError> {
  let networks =
    repositories::cluster_network::list_for_cluster(cluster, pool).await?;

  networks
    .iter()
    .map(|network| async move {
      delete_network_by_key(network.key.to_owned(), docker_api, pool).await?;
      Ok::<_, HttpResponseError>(())
    })
    .collect::<FuturesUnordered<_>>()
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .collect::<Result<Vec<_>, HttpResponseError>>()?;

  Ok(GenericDelete {
    count: networks.len(),
  })
}
