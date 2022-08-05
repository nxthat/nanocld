//! File used to describe daemon boot

use crate::components;
use crate::errors::DaemonError;

pub async fn install_components(
  docker: &bollard::Docker,
) -> Result<(), DaemonError> {
  components::utils::install_component("postgres:alpine3.16", docker).await?;
  components::utils::build_component("nanocl-dns-dnsmasq", docker).await?;
  components::utils::build_component("nanocl-proxy-nginx", docker).await?;

  Ok(())
}
