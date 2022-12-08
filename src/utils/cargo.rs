use bollard::service::{RestartPolicy, RestartPolicyNameEnum};
use futures::stream::FuturesUnordered;
use ntex::web;
use ntex::http::StatusCode;
use std::collections::HashMap;
use futures::{StreamExt, stream};

use crate::models::{DaemonConfig, CargoInstanceFilterQuery};
use crate::{repositories, utils};

use crate::models::{CargoItem, Pool};

use crate::errors::HttpResponseError;

use super::cluster::JoinCargoOptions;

pub async fn list_instances(
  qs: CargoInstanceFilterQuery,
  docker_api: &web::types::State<bollard::Docker>,
) -> Result<Vec<bollard::models::ContainerSummary>, HttpResponseError> {
  let namespace = utils::key::resolve_nsp(&qs.namespace);
  let mut filters = HashMap::new();
  let default_label = format!("namespace={}", &namespace);
  let mut labels = vec![default_label];
  if let Some(ref cluster) = qs.cluster {
    let label = format!("cluster={}-{}", &namespace, &cluster);
    labels.push(label);
  }
  if let Some(ref cargo) = qs.cargo {
    let label = format!("cargo={}-{}", &namespace, &cargo);
    labels.push(label);
  }
  filters.insert(String::from("label"), labels);
  let options = Some(bollard::container::ListContainersOptions::<String> {
    all: true,
    filters,
    ..Default::default()
  });
  let containers = docker_api.list_containers(options).await?;

  Ok(containers)
}

#[derive(Debug)]
pub struct CreateCargoInstanceOpts<'a> {
  pub(crate) cargo: &'a CargoItem,
  pub(crate) cluster_name: &'a str,
  pub(crate) network_key: &'a str,
  pub(crate) environnements: Vec<String>,
  pub(crate) labels: Option<&'a mut HashMap<String, String>>,
}

pub async fn create_instances<'a>(
  opts: CreateCargoInstanceOpts<'a>,
  docker_api: &web::types::State<bollard::Docker>,
) -> Result<Vec<String>, HttpResponseError> {
  log::debug!(
    "creating containers for cargo {:?} with labels {:?}",
    &opts.cargo,
    &opts.labels,
  );
  if docker_api
    .inspect_image(&opts.cargo.image_name)
    .await
    .is_err()
  {
    return Err(HttpResponseError {
      msg: format!(
        "Unable to create cargo container image {} is not available.",
        &opts.cargo.image_name,
      ),
      status: StatusCode::BAD_REQUEST,
    });
  }
  let mut count = 0;
  let mut container_ids: Vec<String> = Vec::new();
  let image_name = opts.cargo.image_name.clone();
  let image = Some(image_name.to_owned());
  let mut labels: HashMap<String, String> = match opts.labels {
    None => HashMap::new(),
    Some(labels) => labels.to_owned(),
  };
  labels.insert(
    String::from("namespace"),
    opts.cargo.namespace_name.to_owned(),
  );
  labels.insert(String::from("cargo"), opts.cargo.key.to_owned());
  while count < opts.cargo.replicas {
    let mut name = format!(
      "{}-{}-{}",
      &opts.cargo.namespace_name, &opts.cluster_name, &opts.cargo.name,
    );
    if count != 0 {
      name += &("-".to_owned() + &count.to_string());
    }

    log::debug!("passing env {:#?}", &opts.environnements);

    let mut network_mode = Some(opts.network_key.to_owned());
    if let Some(net_mode) = &opts.cargo.network_mode {
      network_mode = Some(net_mode.to_owned());
    }

    let options = bollard::container::CreateContainerOptions { name };
    let config = bollard::container::Config {
      image: image.to_owned(),
      hostname: opts.cargo.hostname.to_owned(),
      domainname: opts.cargo.domainname.to_owned(),
      tty: Some(true),
      labels: Some(labels.to_owned()),
      env: Some(opts.environnements.to_owned()),
      attach_stdout: Some(true),
      attach_stderr: Some(true),
      host_config: Some(bollard::models::HostConfig {
        restart_policy: Some(RestartPolicy {
          name: Some(RestartPolicyNameEnum::UNLESS_STOPPED),
          maximum_retry_count: None,
        }),
        binds: Some(opts.cargo.binds.to_owned()),
        cap_add: opts.cargo.cap_add.to_owned(),
        network_mode,
        ..Default::default()
      }),
      ..Default::default()
    };
    let res = docker_api.create_container(Some(options), config).await?;
    container_ids.push(res.id);
    count += 1;
  }
  Ok(container_ids)
}

pub async fn delete_instances(
  cargo_key: String,
  docker_api: &web::types::State<bollard::Docker>,
) -> Result<(), HttpResponseError> {
  let qs = CargoInstanceFilterQuery {
    cargo: Some(cargo_key.to_owned()),
    ..Default::default()
  };
  let instances = list_instances(qs, docker_api).await?;
  instances
    .into_iter()
    .map(|container| async move {
      let id = container.id.ok_or(HttpResponseError {
        msg: String::from("unable to get container id"),
        status: StatusCode::INTERNAL_SERVER_ERROR,
      })?;
      let options = Some(bollard::container::RemoveContainerOptions {
        force: true,
        ..Default::default()
      });
      docker_api.remove_container(&id, options).await?;
      Ok::<_, HttpResponseError>(())
    })
    .collect::<FuturesUnordered<_>>()
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .collect::<Result<Vec<()>, HttpResponseError>>()?;

  Ok(())
}

/// Regenerate containers for a given cargo
pub async fn update_instances(
  cargo_key: String,
  daemon_config: &web::types::State<DaemonConfig>,
  docker_api: &web::types::State<bollard::Docker>,
  pool: &web::types::State<Pool>,
) -> Result<(), HttpResponseError> {
  let cluster_cargoes =
    repositories::cargo_instance::find_by_cargo_key(cargo_key, pool).await?;
  let mut cluster_cargoes_stream = stream::iter(cluster_cargoes);
  while let Some(cluster_cargo) = cluster_cargoes_stream.next().await {
    let network = repositories::cluster_network::find_by_key(
      cluster_cargo.network_key,
      pool,
    )
    .await?;

    let cluster = repositories::cluster::find_by_key(
      cluster_cargo.cluster_key.to_owned(),
      pool,
    )
    .await?;
    let cargo = repositories::cargo::find_by_key(
      cluster_cargo.cargo_key.to_owned(),
      pool,
    )
    .await?;

    // Containers to remove after update
    let qs = CargoInstanceFilterQuery {
      cargo: Some(cluster_cargo.cargo_key.to_owned()),
      cluster: Some(cluster_cargo.cluster_key.to_owned()),
      ..Default::default()
    };
    let instances = list_instances(qs, docker_api).await?;

    let mut instance_stream = stream::iter(instances.to_owned());

    let mut count = 0;
    while let Some(instance) = instance_stream.next().await {
      let options = bollard::container::RenameContainerOptions {
        name: format!("{}-tmp-{}", &cargo.name, &count),
      };
      docker_api
        .rename_container(&instance.id.unwrap_or_default(), options)
        .await?;
      count += 1;
    }

    let opts = JoinCargoOptions {
      cluster: cluster.to_owned(),
      cargo,
      network,
      is_creating_relation: false,
    };

    utils::cluster::join_cargo(&opts, docker_api, pool).await?;

    utils::cluster::start(&cluster, daemon_config, pool, docker_api).await?;

    let mut scntr = stream::iter(instances);

    while let Some(container) = scntr.next().await {
      let options = Some(bollard::container::RemoveContainerOptions {
        force: true,
        ..Default::default()
      });
      docker_api
        .remove_container(&container.id.clone().unwrap_or_default(), options)
        .await?;
    }
  }
  Ok(())
}
