use std::collections::HashMap;

use bollard::Docker;
use bollard::container::StartContainerOptions;
use bollard::errors::Error as DockerError;
use bollard::models::Network;
use bollard::network::InspectNetworkOptions;
use ntex::http::StatusCode;
use crate::errors::HttpResponseError;

#[derive(Debug, Eq, PartialEq)]
pub enum ComponentState {
  Uninstalled,
  Running,
  Stopped,
}

#[derive(Debug, Eq, PartialEq)]
pub enum NetworkState {
  NotFound,
  Ready,
}

/// ## Generate labels with a namespace
///
/// ## Arguments
/// - [namespace](str) the name of the namespace
///
/// ## Return
/// [labels](HashMap) a hashmap of strings with namespace key as given value
///
/// ## Examples
/// ```rust,norun
/// use crate::services;
///
/// services::utils::gen_labels_with_namespace("default");
/// ```
pub fn gen_labels_with_namespace(namespace: &str) -> HashMap<&str, &str> {
  let mut labels: HashMap<&str, &str> = HashMap::new();
  labels.insert("namespace", namespace);
  labels
}

/// ## Start a service
/// Start service by it's name
///
/// ## Arguments
/// - [name](str) name of the service to start
/// - [docker_api](Docker) bollard docker instance
///
/// ## Return
/// if sucess return nothing a [docker error](DockerError) is returned if an error occur
///
/// ## Examples
/// ```rust,norun
/// use crate::services;
///
/// services::utils::start_service(&docker, "nproxy").await;
/// ```
pub async fn start_component(
  name: &str,
  docker_api: &Docker,
) -> Result<(), DockerError> {
  docker_api
    .start_container(name, None::<StartContainerOptions<String>>)
    .await?;
  Ok(())
}

/// ## Get network state
///
/// ## Arguments
/// - [name](str) name of the network
/// - [docker_api](Docker) bollard docker instance
///
/// ## Return
/// /// if success return [network state](NetworkState)
/// a [docker error](DockerError) is returned if an error occur
///
/// ## Examples
/// ```rust,norun
/// use crate::services;
///
/// services::utils::get_network_state(&docker, "network-name").await;
/// ```
pub async fn get_network_state(
  network_name: &str,
  docker_api: &Docker,
) -> Result<NetworkState, DockerError> {
  let config = InspectNetworkOptions {
    verbose: true,
    scope: "local",
  };

  let res = docker_api.inspect_network(network_name, Some(config)).await;
  if let Err(err) = res {
    match err {
      DockerError::DockerResponseServerError {
        status_code,
        message,
      } => {
        if status_code == 404 {
          return Ok(NetworkState::NotFound);
        }
        return Err(DockerError::DockerResponseServerError {
          status_code,
          message,
        });
      }
      _ => return Err(err),
    }
  }
  Ok(NetworkState::Ready)
}

/// ## Get service state
/// Get state of a service by his name
///
/// ## Arguments
/// - [name](str) name of the service
/// - [docker_api](Docker) bollard docker instance
///
/// ## Return
/// if success return [service state](ServiceState)
/// a [docker error](DockerError) is returned if an error occur
///
/// ## Examples
/// ```rust,norun
/// use crate::services;
///
/// services::utils::get_component_state(&docker, "nproxy").await;
/// ```
pub async fn get_component_state(
  container_name: &'static str,
  docker_api: &Docker,
) -> ComponentState {
  let resp = docker_api.inspect_container(container_name, None).await;
  if resp.is_err() {
    return ComponentState::Uninstalled;
  }
  let body = resp.expect("ContainerInspectResponse");
  if let Some(state) = body.state {
    if let Some(running) = state.running {
      return if running {
        ComponentState::Running
      } else {
        ComponentState::Stopped
      };
    }
  }
  ComponentState::Stopped
}

pub fn get_default_gateway(
  docker_network: &Network,
) -> Result<String, HttpResponseError> {
  let ipam_config = docker_network
    .to_owned()
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
    })?
    .to_owned();

  Ok(default_gateway)
}
