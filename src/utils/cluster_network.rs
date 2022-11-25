use std::collections::HashMap;

use ntex::web;
use ntex::http::StatusCode;

use super::key::gen_key;
use crate::repositories;
use crate::models::{Pool, ClusterNetworkItem, ClusterNetworkPartial};
use crate::errors::HttpResponseError;

/// ## Create network in given cluster
/// This function will create a network based on the settings
///
/// ## Arguments
/// - [cluster_key](str) The cluster_key to target
/// - [network_name](str) The name of the network to create
///
/// ## Return
/// A ClusterNetwork Item
pub async fn create_network(
  nsp: String,
  c_name: String,
  item: ClusterNetworkPartial,
  docker_api: &web::types::State<bollard::Docker>,
  pool: &web::types::State<Pool>,
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
