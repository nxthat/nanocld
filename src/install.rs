//! File used to describe daemon boot

use crate::components;
use crate::errors::DaemonError;

pub async fn install_components(
  docker_api: &bollard::Docker,
) -> Result<(), DaemonError> {
  components::utils::install_component(
    "quay.io/coreos/etcd:v3.5.5",
    docker_api,
  )
  .await?;
  components::utils::build_component("nanocl-dns-dnsmasq", docker_api).await?;
  components::utils::build_component("nanocl-proxy-nginx", docker_api).await?;
  Ok(())
}
