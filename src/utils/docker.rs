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

/// Get default gateway of a network
///
/// ## Arguments
/// - [docker_network](Network) docker network
///
/// ## Return
/// if success return [default gateway](String)
/// a [http response error](HttpResponseError) is returned if an error occur
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

/// Docker utils unit tests
#[cfg(test)]
mod tests {

  use super::*;

  use bollard::{network::InspectNetworkOptions, container::StopContainerOptions};

  use crate::utils::tests::*;

  /// Test to get default gateway of system-nano-internal0 network
  #[ntex::test]
  async fn get_nanocl_internal_gateway() -> TestRet {
    let docker = gen_docker_client();
    let network = docker
      .inspect_network(
        "system-nano-internal0",
        None::<InspectNetworkOptions<String>>,
      )
      .await?;
    let _gateway = get_default_gateway(&network);
    Ok(())
  }

  /// Test to get default gateway of host network
  /// This should fail because host network doesn't have a gateway
  #[ntex::test]
  async fn get_host_network_gateway() -> TestRet {
    let docker = gen_docker_client();
    let network = docker
      .inspect_network("host", None::<InspectNetworkOptions<String>>)
      .await?;
    let gateway = get_default_gateway(&network);
    assert!(gateway.is_err(), "Expect get_default_gateway to fail");
    Ok(())
  }

  /// Test to generate labels with a namespace gg
  #[ntex::test]
  async fn gen_labels_with_namespace_test() -> TestRet {
    let labels = gen_labels_with_namespace("gg");
    assert_eq!(labels.get("namespace"), Some(&"gg"));
    Ok(())
  }

  /// Test to get network state of system-nano-internal0 network
  /// This should return NetworkState::Ready
  #[ntex::test]
  async fn get_network_state_test() -> TestRet {
    let docker = gen_docker_client();
    let state = get_network_state("system-nano-internal0", &docker).await?;
    assert_eq!(state, NetworkState::Ready);
    Ok(())
  }

  /// Test to get network state of a non existing network
  /// This should return NetworkState::NotFound
  #[ntex::test]
  async fn get_network_state_not_found_test() -> TestRet {
    let docker = gen_docker_client();
    let state = get_network_state("non-existing-network", &docker).await?;
    assert_eq!(state, NetworkState::NotFound);
    Ok(())
  }

  /// Test to get component state of the store container
  /// This should return ComponentState::Running
  #[ntex::test]
  async fn get_component_state_test() -> TestRet {
    let docker = gen_docker_client();
    let state = get_component_state("store", &docker).await;
    assert_eq!(state, ComponentState::Running);
    Ok(())
  }

  /// Test to get component state of a non existing container
  /// This should return ComponentState::Uninstalled
  #[ntex::test]
  async fn get_component_state_not_found_test() -> TestRet {
    let docker = gen_docker_client();
    let state = get_component_state("non-existing-container", &docker).await;
    assert_eq!(state, ComponentState::Uninstalled);
    Ok(())
  }

  /// Test to get component state of a stopped container
  /// This should return ComponentState::Stopped
  /// TODO: download a specific image before
  async fn _get_component_state_stopped_test() -> TestRet {
    let docker = gen_docker_client();

    // Stop system-nano-dns container
    docker
      .stop_container("system-nano-dns", None::<StopContainerOptions>)
      .await?;

    // Get the state of system-nano-dns container
    let state = get_component_state("system-nano-dns", &docker).await;
    assert_eq!(state, ComponentState::Stopped);

    // Start system-nano-dns container
    docker
      .start_container("system-nano-dns", None::<StartContainerOptions<String>>)
      .await?;
    Ok(())
  }
}
