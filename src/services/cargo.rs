use ntex::web;
use ntex::http::StatusCode;
use std::collections::HashMap;
use futures::{StreamExt, stream};

use crate::models::CargoItem;

use crate::errors::HttpResponseError;

#[derive(Debug)]
pub struct CreateCargoContainerOpts<'a> {
  pub(crate) cargo: &'a CargoItem,
  pub(crate) cluster_name: &'a str,
  pub(crate) network_key: &'a str,
  pub(crate) environnements: Vec<String>,
  pub(crate) labels: Option<&'a mut HashMap<String, String>>,
}

pub async fn create_containers<'a>(
  opts: CreateCargoContainerOpts<'a>,
  docker_api: &web::types::State<bollard::Docker>,
) -> Result<Vec<String>, HttpResponseError> {
  log::debug!(
    "creating containers for cargo {:?} with labels {:?}",
    &opts.cargo,
    &opts.labels,
  );
  let mut container_ids: Vec<String> = Vec::new();
  let image_name = opts.cargo.image_name.clone();
  if docker_api
    .inspect_image(&opts.cargo.image_name)
    .await
    .is_err()
  {
    return Err(HttpResponseError {
      msg: format!(
        "Unable to create cargo container image {} is not available.",
        &image_name,
      ),
      status: StatusCode::BAD_REQUEST,
    });
  }
  let image = Some(image_name.to_owned());
  let mut labels: HashMap<String, String> = match opts.labels {
    None => HashMap::new(),
    Some(labels) => labels.to_owned(),
  };
  let mut name = format!(
    "{}-{}-{}",
    &opts.cargo.namespace_name, &opts.cluster_name, &opts.cargo.name,
  );
  let len = container_ids.len();
  if len != 0 {
    name += &("-".to_owned() + &len.to_string());
  }
  labels.insert(
    String::from("namespace"),
    opts.cargo.namespace_name.to_owned(),
  );
  labels.insert(String::from("cargo"), opts.cargo.key.to_owned());
  let options = bollard::container::CreateContainerOptions { name };
  let config = bollard::container::Config {
    image,
    hostname: opts.cargo.hostname.to_owned(),
    domainname: opts.cargo.domainname.to_owned(),
    tty: Some(true),
    labels: Some(labels),
    env: Some(opts.environnements),
    attach_stdout: Some(true),
    attach_stderr: Some(true),
    host_config: Some(bollard::models::HostConfig {
      binds: Some(opts.cargo.binds.to_owned()),
      network_mode: Some(opts.network_key.to_owned()),
      ..Default::default()
    }),
    ..Default::default()
  };
  let res = docker_api.create_container(Some(options), config).await?;
  container_ids.push(res.id);
  Ok(container_ids)
}

pub async fn list_containers(
  cargo_key: String,
  docker_api: &web::types::State<bollard::Docker>,
) -> Result<Vec<bollard::models::ContainerSummary>, HttpResponseError> {
  let target_cluster = &format!("cargo={}", &cargo_key);
  let mut filters = HashMap::new();
  filters.insert("label", vec![target_cluster.as_str()]);
  let options = Some(bollard::container::ListContainersOptions {
    all: true,
    filters,
    ..Default::default()
  });
  let containers = docker_api.list_containers(options).await?;
  Ok(containers)
}

pub async fn delete_container(
  cargo_key: String,
  docker_api: &web::types::State<bollard::Docker>,
) -> Result<(), HttpResponseError> {
  let containers = list_containers(cargo_key, docker_api).await?;

  let mut stream = stream::iter(containers);

  while let Some(container) = stream.next().await {
    let id = container.id.ok_or(HttpResponseError {
      msg: String::from("unable to get container id"),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    let options = Some(bollard::container::RemoveContainerOptions {
      force: true,
      ..Default::default()
    });
    docker_api.remove_container(&id, options).await?;
  }

  // TODO test perf against stream
  // containers
  //   .into_iter()
  //   .map(|container| async move {
  //     let id = container.id.ok_or(HttpResponseError {
  //       msg: String::from("unable to get container id"),
  //       status: StatusCode::INTERNAL_SERVER_ERROR,
  //     })?;
  //     let options = Some(bollard::container::RemoveContainerOptions {
  //       force: true,
  //       ..Default::default()
  //     });
  //     docker_api.remove_container(&id, options).await?;
  //     Ok::<_, HttpResponseError>(())
  //   })
  //   .collect::<FuturesUnordered<_>>()
  //   .collect::<Vec<_>>()
  //   .await
  //   .into_iter()
  //   .collect::<Result<Vec<()>, HttpResponseError>>()?;

  Ok(())
}
