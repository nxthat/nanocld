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
  let envs = vec![
    String::from("VPN_DNS_SRV1=155.0.0.1"),
    String::from("VPN_PUBLIC_IP=155.0.0.1"),
    "VPN_L2TP_NET=192.168.84.0/16",
    "VPN_L2TP_LOCAL=192.168.84.1",
    "VPN_L2TP_POOL=192.168.84.10-192.168.84.254",
    "VPN_XAUTH_NET=192.168.85.0/16",
    "VPN_XAUTH_POOL=192.168.85.10-192.168.85.254",
  ];
  let store_cargo = CargoPartial {
    name: String::from("system-nano-vpn"),
    image_name: String::from("hwdsl2/ipsec-vpn-server"),
    environnements: Some(envs),
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
