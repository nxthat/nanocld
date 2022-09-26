use std::path::Path;

use crate::config::DaemonConfig;

use bollard::{
  Docker,
  errors::Error as DockerError,
  container::{CreateContainerOptions, Config},
  service::HostConfig,
};

use super::utils::*;

pub fn gen_etcd_host_conf(config: &DaemonConfig) -> HostConfig {
  let dir_path = Path::new(&config.state_dir).join("etcd/data");
  let binds = Some(vec![format!("{}:/var/lib/etcd", dir_path.display())]);
  HostConfig {
    binds,
    network_mode: Some(String::from("nanoclservices0")),
    ..Default::default()
  }
}

async fn create_etc_container(
  name: &str,
  config: &DaemonConfig,
  docker_api: &Docker,
) -> Result<(), DockerError> {
  let image = Some("quay.io/coreos/etcd:v3.5.5");
  let labels = Some(gen_labels_with_namespace("nanocl"));
  let host_config = Some(gen_etcd_host_conf(config));
  let options = Some(CreateContainerOptions { name });
  let config = Config {
    image,
    labels,
    host_config,
    shell: Some(vec![
      "-initial-cluster-token",
      "nanocl-etcd-cluster",
      "-listen-peer-urls",
      "http://0.0.0.0:2380",
      "-initial-cluster",
      "etcd0=http://${HostIP}:2380",
    ]),
    tty: Some(true),
    attach_stdout: Some(true),
    attach_stderr: Some(true),
    ..Default::default()
  };
  docker_api.create_container(options, config).await?;
  Ok(())
}

async fn boot(
  config: &DaemonConfig,
  docker_api: &Docker,
) -> Result<(), DockerError> {
  let container_name = "nanocl-db-etcd";
  let s_state = get_component_state(container_name, docker_api).await;

  if s_state == ComponentState::Uninstalled {
    create_etc_container(container_name, config, docker_api).await?;
  }
  if s_state != ComponentState::Running {
    if let Err(err) = start_component(container_name, docker_api).await {
      log::error!("error while starting {} {}", container_name, err);
    }
  }
  Ok(())
}
