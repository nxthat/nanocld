use std::path::Path;

use ntex::web;
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;

use ntex::http::StatusCode;
use bollard::{
  Docker,
  models::HostConfig,
  errors::Error as DockerError,
  container::{CreateContainerOptions, Config},
  service::{RestartPolicy, RestartPolicyNameEnum},
};

use crate::{
  utils,
  models::{Pool, DBConn},
  models::DaemonConfig,
};
use crate::errors::HttpResponseError;

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

async fn create_system_store(
  name: &str,
  config: &DaemonConfig,
  docker_api: &Docker,
) -> Result<(), DockerError> {
  let image = Some("cockroachdb/cockroach:v21.2.17");
  let mut labels = utils::docker::gen_labels_with_namespace("system");
  labels.insert("namespace", "system");
  labels.insert("cluster", "system-nano");
  labels.insert("cargo", "system-store");
  let host_config = Some(gen_store_host_conf(config));
  let options = Some(CreateContainerOptions { name });
  let config = Config {
    image,
    labels: Some(labels),
    host_config,
    hostname: Some(name),
    domainname: Some(name),
    cmd: Some(vec!["start-single-node", "--insecure"]),
    ..Default::default()
  };
  docker_api.create_container(options, config).await?;
  Ok(())
}

/// # Create pool
/// Create an pool connection to postgres database
///
/// # Returns
/// - [Pool](Pool) R2d2 pool connection for postgres
///
/// # Examples
/// ```
/// let pool = create_pool(String::from("localhost"));
/// ```
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

/// # Get connection from a pool
///
/// # Arguments
/// [pool](web::types::State<Pool>) a pool wrapped in ntex State
///
pub fn get_pool_conn(
  pool: &web::types::State<Pool>,
) -> Result<DBConn, HttpResponseError> {
  let conn = match pool.get() {
    Ok(conn) => conn,
    Err(_) => {
      return Err(HttpResponseError {
        msg: String::from("unable to connect to nanocl-db"),
        status: StatusCode::INTERNAL_SERVER_ERROR,
      });
    }
  };
  Ok(conn)
}

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

pub async fn boot(
  config: &DaemonConfig,
  docker_api: &Docker,
) -> Result<(), DockerError> {
  let container_name = "store";
  let s_state =
    utils::docker::get_component_state(container_name, docker_api).await;

  if s_state == utils::docker::ComponentState::Uninstalled {
    create_system_store(container_name, config, docker_api).await?;
  }
  if s_state != utils::docker::ComponentState::Running {
    if let Err(err) =
      utils::docker::start_component(container_name, docker_api).await
    {
      log::error!("error while starting {} {}", container_name, err);
    }
  }
  Ok(())
}
