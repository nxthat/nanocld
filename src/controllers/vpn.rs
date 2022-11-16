use std::path::Path;

use crate::{utils, repositories};
use crate::models::{ArgState, CargoPartial};
use crate::errors::DaemonError;

/// Register ipsec as a cargo
///
/// ## Arguments
/// [arg](ArgState) Reference to argument state
pub async fn register(arg: &ArgState) -> Result<(), DaemonError> {
  let key = utils::key::gen_key(&arg.sys_namespace, "vpn");
  if repositories::cargo::find_by_key(key, &arg.s_pool)
    .await
    .is_ok()
  {
    return Ok(());
  }
  let path = Path::new(&arg.config.state_dir).join("store/data");
  let binds = vec![format!("{}:/cockroach/cockroach-data", path.display())];
  let store_cargo = CargoPartial {
    name: String::from("store"),
    image_name: String::from("cockroachdb/cockroach:v21.2.17"),
    environnements: None,
    binds: Some(binds),
    replicas: Some(1),
    dns_entry: None,
    domainname: Some(String::from("store")),
    hostname: Some(String::from("store")),
    network_mode: None,
    restart_policy: Some(String::from("unless-stopped")),
    cap_add: None,
  };
  repositories::cargo::create(
    arg.sys_namespace.to_owned(),
    store_cargo,
    &arg.s_pool,
  )
  .await?;

  Ok(())
}
