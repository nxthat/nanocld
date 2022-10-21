//! File used to describe daemon boot

use crate::controllers;
use crate::errors::DaemonError;

pub async fn install_components(
  docker: &bollard::Docker,
) -> Result<(), DaemonError> {
  controllers::utils::install_component(
    "cockroachdb/cockroach:v21.2.17",
    docker,
  )
  .await?;
  controllers::utils::build_component("nanocl-dns-dnsmasq", docker).await?;
  controllers::utils::build_component("nanocl-proxy-nginx", docker).await?;
  Ok(())
}
