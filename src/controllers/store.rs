use std::path::Path;
use std::collections::HashMap;

use ntex::web;
use ntex::http::StatusCode;
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use bollard::{
  Docker,
  models::HostConfig,
  errors::Error as DockerError,
  container::{CreateContainerOptions, Config},
  service::{RestartPolicy, RestartPolicyNameEnum},
};

use crate::{utils, repositories, models::CargoInstanceState};
use crate::errors::{DaemonError, HttpResponseError};
use crate::models::{
  Pool, DBConn, ArgState, DaemonConfig, CargoPartial, CargoInstancePartial,
};

/// Generate HostConfig struct for container creation
///
/// ## Arguments
/// [config](DaemonConfig) Daemon config reference
///
/// ## Returns
/// [HostConfig](HostConfig) HostConfig struct for container creation
fn gen_store_host_conf(config: &DaemonConfig) -> HostConfig {
  let path = Path::new(&config.state_dir).join("store/data");

  let binds = vec![format!("{}:/cockroach/cockroach-data", path.display())];

  HostConfig {
    binds: Some(binds),
    restart_policy: Some(RestartPolicy {
      name: Some(RestartPolicyNameEnum::UNLESS_STOPPED),
      maximum_retry_count: None,
    }),
    network_mode: Some(String::from("system-nano-internal0")),
    ..Default::default()
  }
}

/// Generate container config for system store
/// This function will generate a container config for the system store
///
/// ## Arguments
/// [name](str) The name of the container
/// [config](DaemonConfig) Reference to daemon config
///
/// ## Returns
/// [Config](Config) The container config
fn gen_store_config<'a>(
  name: &'a str,
  config: &DaemonConfig,
) -> bollard::container::Config<&'a str> {
  let image = Some("cockroachdb/cockroach:v21.2.17");
  let mut labels = HashMap::new();
  labels.insert("namespace", "system");
  labels.insert("cluster", "system-nano");
  labels.insert("cargo", "system-store");
  let host_config = Some(gen_store_host_conf(config));
  Config {
    image,
    labels: Some(labels),
    host_config,
    hostname: Some(name),
    domainname: Some(name),
    cmd: Some(vec!["start-single-node", "--insecure"]),
    ..Default::default()
  }
}

/// Create system store cargo instance
///
/// ## Arguments
/// [name](str) The name of the cargo instance
/// [config](DaemonConfig) Reference to daemon config
/// [docker_api](Docker) Reference to docker api
async fn create_system_store(
  name: &str,
  config: &DaemonConfig,
  docker_api: &Docker,
) -> Result<(), DockerError> {
  let options = Some(CreateContainerOptions {
    name,
    ..Default::default()
  });
  let config = gen_store_config(name, config);
  docker_api
    .create_container(options, config.to_owned())
    .await?;
  Ok(())
}

/// Create a connection pool for postgres database
///
/// ## Arguments
/// [host](String) Host to connect to
///
/// ## Returns
/// - [Pool](Pool) R2d2 pool connection for postgres
pub async fn create_pool(host: String) -> Pool {
  web::block(move || {
    let db_url =
      "postgres://root:root@".to_owned() + &host + ":26257/defaultdb";
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    r2d2::Pool::builder().build(manager)
  })
  .await
  .expect("cannot connect to postgresql.")
}

/// Get connection from the connection pool
///
/// ## Arguments
/// [pool](web::types::State<Pool>) a pool wrapped in ntex State
///
/// ## Returns
/// - [DBConn](DBConn) A connection to the database
/// - [HttpResponseError](HttpResponseError) Error if unable to get connection
pub fn get_pool_conn(pool: &Pool) -> Result<DBConn, HttpResponseError> {
  let conn = match pool.get() {
    Ok(conn) => conn,
    Err(err) => {
      return Err(HttpResponseError {
        msg: format!("Cannot get connection from pool got error: {}", &err),
        status: StatusCode::INTERNAL_SERVER_ERROR,
      });
    }
  };
  Ok(conn)
}

/// Get store ip address
///
/// ## Arguments
/// [docker_api](Docker) Reference to docker api
///
/// ## Returns
/// - [String](String) Ip address of the store
/// - [HttpResponseError](HttpResponseError) Error if unable to get ip address
pub async fn get_store_ip_addr(
  docker_api: &Docker,
) -> Result<String, HttpResponseError> {
  let container = docker_api.inspect_container("store", None).await?;
  let networks = container
    .network_settings
    .ok_or(HttpResponseError {
      msg: String::from("unable to get store network nettings"),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    })?
    .networks
    .ok_or(HttpResponseError {
      msg: String::from("unable to get store networks"),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    })?;
  let ip_address = networks
    .get("system-nano-internal0")
    .ok_or(HttpResponseError {
      msg: String::from("unable to get store network nanocl"),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    })?
    .ip_address
    .as_ref()
    .ok_or(HttpResponseError {
      msg: String::from("unable to get store network nanocl"),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    })?;
  Ok(ip_address.to_owned())
}

/// Boot the store and ensure it's running
///
/// ## Arguments
/// [config](DaemonConfig) Reference to Daemon config
/// [docker_api](Docker) Reference to docker
///
/// ## Returns
/// - [Result](Result) Result of the boot process
/// - [DockerError](DockerError) Error if unable to boot store
pub async fn boot(
  config: &DaemonConfig,
  docker_api: &Docker,
) -> Result<(), DockerError> {
  let container_name = "store";
  let s_state =
    utils::cargo_instance::get_cargo_instance_state(container_name, docker_api)
      .await;

  if s_state == CargoInstanceState::Uninstalled {
    create_system_store(container_name, config, docker_api).await?;
  }
  if s_state != CargoInstanceState::Running {
    if let Err(err) =
      utils::cargo_instance::start_cargo_instance(container_name, docker_api)
        .await
    {
      log::error!("error while starting {} {}", container_name, err);
    }
  }
  Ok(())
}

/// Register store as a cargo
///
/// ## Arguments
/// [arg](ArgState) Reference to argument state
///
/// ## Returns
/// - [Result](Result) Result of the registration process
/// - [DaemonError](DaemonError) Error if unable to register store
pub async fn register(arg: &ArgState) -> Result<(), DaemonError> {
  let name = "store";
  let key = utils::key::gen_key(&arg.sys_namespace, name);
  if repositories::cargo::find_by_key(key, &arg.pool)
    .await
    .is_ok()
  {
    return Ok(());
  }

  let config = gen_store_config(name, &arg.config);

  let store_cargo = CargoPartial {
    name: name.into(),
    environnements: None,
    replicas: Some(1),
    dns_entry: None,
    config: serde_json::to_value(config).map_err(|err| HttpResponseError {
      msg: format!("unable to serialize store config: {}", err),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    })?,
  };
  let cargo = repositories::cargo::create(
    arg.sys_namespace.to_owned(),
    store_cargo,
    &arg.pool,
  )
  .await?;

  let cluster_key = utils::key::gen_key(&arg.sys_namespace, &arg.sys_cluster);
  let network_key = utils::key::gen_key(&cluster_key, &arg.sys_network);
  let cargo_instance = CargoInstancePartial {
    cargo_key: cargo.key,
    cluster_key,
    network_key,
  };

  repositories::cargo_instance::create(cargo_instance, &arg.pool).await?;

  Ok(())
}
