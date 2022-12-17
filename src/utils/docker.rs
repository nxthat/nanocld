use bollard::Docker;
use bollard::container::StartContainerOptions;
use bollard::errors::Error as DockerError;
use bollard::network::InspectNetworkOptions;

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

/// Docker utils unit tests
#[cfg(test)]
mod tests {

  use super::*;

  use bollard::container::StopContainerOptions;

  use crate::utils::tests::*;

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
