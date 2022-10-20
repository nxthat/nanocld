use std::collections::HashMap;
use futures::StreamExt;
use bollard::{
  Docker,
  errors::Error as DockerError,
  image::{CreateImageOptions, BuildImageOptions},
  network::InspectNetworkOptions,
  container::StartContainerOptions,
};

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

type DockerBuildOutput = Result<bollard::models::BuildInfo, DockerError>;
type DockerCreateOutput = Result<bollard::models::CreateImageInfo, DockerError>;

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

/// ## Parse docker build output
/// Print parsed docker build output
///
/// ## Arguments
/// - [service_name](str) The name of the service being builded
/// - [output](DockerBuildOutput) The output to parse
///
/// ## Return
/// Ok in any case
fn parse_build_output(
  service_name: &'static str,
  output: DockerBuildOutput,
) -> Result<(), DockerError> {
  match output {
    Err(err) => return Err(err),
    Ok(build_info) => {
      if let Some(err) = build_info.error {
        log::error!("[{}] {:#?}", &service_name, &err);
        return Err(DockerError::DockerResponseServerError {
          status_code: 400,
          message: format!("Error while building {}: {}", &service_name, &err),
        });
      }
    }
  }
  Ok(())
}

/// ## Parse docker create output
/// Print parsed docker create output
///
/// ## Arguments
/// - [service_name](str) The name of the service being builded
/// - [output](DockerCreateOutput) The output to parse
///
/// ## Return
/// CreateImageInfo or DockerError
fn parse_create_output(
  service_name: &'static str,
  output: DockerCreateOutput,
) -> Result<bollard::models::CreateImageInfo, DockerError> {
  let output = match output {
    Err(err) => return Err(err),
    Ok(create_info) => {
      if let Some(err) = create_info.error {
        log::error!("[{}] {:#?}", &service_name, &err);
        return Err(DockerError::DockerResponseServerError {
          status_code: 400,
          message: format!("Error while building {}: {}", &service_name, &err),
        });
      }
      create_info
    }
  };
  Ok(output)
}

/// # Build a service
/// Build a nxthat service from github
///
/// # Arguments
/// - [name](str) name of the service to build
/// - [docker_api](Docker) bollard docker instance
///
/// # Return
/// if sucess return nothing a [docker error](DockerError) is returned if an error occur
///
/// /// # Examples
/// ```rust,norun
/// use crate::services;
///
/// services::utils::build_component(&docker, "nproxy").await;
/// ```
pub async fn build_component(
  service_name: &'static str,
  docker_api: &Docker,
) -> Result<(), DockerError> {
  if image_exists(service_name, docker_api).await {
    return Ok(());
  }
  let git_url = "https://github.com/nxthat/".to_owned();
  let image_url = git_url + service_name + ".git";
  let options = BuildImageOptions::<String> {
    dockerfile: String::from("Dockerfile"),
    t: service_name.to_string(),
    remote: image_url,
    rm: true,
    forcerm: true,
    ..Default::default()
  };
  log::info!("building service [{}]", &service_name);
  let mut stream = docker_api.build_image(options, None, None);
  while let Some(output) = stream.next().await {
    parse_build_output(service_name, output)?;
  }
  log::info!("successfully builded service [{}]", &service_name);
  Ok(())
}

/// # Install a service
/// Install a service from docker image
///
/// # Arguments
/// - [name](str) name of the service to install
/// - [docker_api](Docker) bollard docker instance
///
/// # Return
/// if sucess return nothing a [docker error](DockerError) is returned if an error occur
///
/// /// # Examples
/// ```rust,norun
/// use crate::services;
///
/// services::utils::install_component("postgresql", &docker_api).await;
/// ```
pub async fn install_component(
  image_name: &'static str,
  docker_api: &Docker,
) -> Result<(), DockerError> {
  if image_exists(image_name, docker_api).await {
    return Ok(());
  }
  log::info!("installing component [{}]", image_name);
  let mut stream = docker_api.create_image(
    Some(CreateImageOptions {
      from_image: image_name,
      ..Default::default()
    }),
    None,
    None,
  );
  while let Some(output) = stream.next().await {
    parse_create_output(image_name, output)?;
  }
  log::info!("successfully installed component [{}]", image_name);
  Ok(())
}

pub async fn image_exists(image_name: &str, docker: &Docker) -> bool {
  if docker.inspect_image(image_name).await.is_ok() {
    return true;
  }
  false
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
