//! File used to describe daemon boot

use crate::utils;
use crate::errors::DaemonError;

pub async fn install_components(
  docker: &bollard::Docker,
) -> Result<(), DaemonError> {
  utils::docker::install_component("cockroachdb/cockroach:v21.2.17", docker)
    .await?;
  utils::docker::build_component("nanocl-dns-dnsmasq", docker).await?;
  utils::docker::build_component("nanocl-proxy-nginx", docker).await?;
  Ok(())
}
