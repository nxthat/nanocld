use std::collections::HashMap;

use bollard::Docker;
use bollard::container::StartContainerOptions;
use bollard::errors::Error as DockerError;
use bollard::models::Network;
use bollard::image::{BuildImageOptions, CreateImageOptions};
use bollard::network::InspectNetworkOptions;
use ntex::{web, rt};
use ntex::util::Bytes;
use ntex::channel::mpsc::{self, Receiver};
use ntex::http::StatusCode;
use futures::StreamExt;

use crate::models::{GitRepositoryItem, GitRepositoryBranchItem};
use crate::errors::HttpResponseError;

pub async fn build_git_repository(
  image_name: String,
  item: GitRepositoryItem,
  branch: GitRepositoryBranchItem,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<Receiver<Result<Bytes, web::error::Error>>, HttpResponseError> {
  let image_url = item.url + ".git#" + &branch.name;
  let mut labels: HashMap<String, String> = HashMap::new();
  labels.insert(String::from("commit"), branch.last_commit_sha);
  let options = bollard::image::BuildImageOptions::<String> {
    dockerfile: String::from("Dockerfile"),
    t: image_name,
    labels,
    remote: image_url,
    rm: true,
    forcerm: true,
    ..Default::default()
  };
  let (tx, rx_body) = mpsc::channel();
  rt::spawn(async move {
    let mut stream = docker_api.build_image(options, None, None);
    while let Some(result) = stream.next().await {
      match result {
        Err(err) => {
          let err = ntex::web::Error::new(web::error::InternalError::default(
            format!("{:?}", err),
            StatusCode::INTERNAL_SERVER_ERROR,
          ));
          let result = tx.send(Err::<_, web::error::Error>(err));
          if result.is_err() {
            break;
          }
        }
        Ok(result) => {
          let data = serde_json::to_string(&result).unwrap();
          let result = tx.send(Ok::<_, web::error::Error>(Bytes::from(data)));
          if result.is_err() {
            break;
          }
        }
      }
    }
  });

  Ok(rx_body)
}

#[allow(dead_code)]
pub async fn build_image(
  image_name: String,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<Receiver<Result<Bytes, web::error::Error>>, HttpResponseError> {
  let (tx, rx_body) = mpsc::channel();
  rt::spawn(async move {
    let mut stream = docker_api.create_image(
      Some(bollard::image::CreateImageOptions {
        from_image: image_name,
        ..Default::default()
      }),
      None,
      None,
    );
    while let Some(result) = stream.next().await {
      match result {
        Err(err) => {
          let err = ntex::web::Error::new(web::error::InternalError::default(
            format!("{:?}", err),
            StatusCode::INTERNAL_SERVER_ERROR,
          ));
          let _ = tx.send(Err::<_, web::error::Error>(err));
        }
        Ok(result) => {
          let data = serde_json::to_string(&result).unwrap();
          let _ = tx.send(Ok::<_, web::error::Error>(Bytes::from(data)));
        }
      }
    }
  });
  Ok(rx_body)
}

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
  let image_url = git_url + service_name + ".git#nightly";
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
