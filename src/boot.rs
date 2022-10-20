//! File used to describe daemon boot
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::{thread, time};

use ntex::web;
use bollard::Docker;
use bollard::network::CreateNetworkOptions;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::config::DaemonConfig;
use crate::controllers::utils::NetworkState;
use crate::utils::cluster::JoinCargoOptions;
use crate::{controllers, repositories, utils};
use crate::models::{Pool, NamespacePartial, ClusterPartial, CargoPartial};

use crate::errors::DaemonError;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[derive(Clone)]
pub struct BootState {
  pub(crate) pool: Pool,
  pub(crate) docker_api: bollard::Docker,
}

/// # Create default namespace
/// Create a namespace if he doesn't exist
///
/// # Arguments
/// - [name](str) The name of the namespace
/// - [pool](web::types::State<Pool>) Postgres database pool
///
/// # Examples
/// ```rust,norun
/// ensure_namespace("system", &pool).await;
/// ```
async fn ensure_namespace(
  name: &str,
  pool: &web::types::State<Pool>,
) -> Result<(), DaemonError> {
  match repositories::namespace::inspect_by_name(name.to_owned(), pool).await {
    Err(_err) => {
      let new_nsp = NamespacePartial {
        name: name.to_owned(),
      };
      repositories::namespace::create(new_nsp, pool).await?;
      Ok(())
    }
    Ok(_) => Ok(()),
  }
}

/// ## Create a system network
/// Create a system network by name with default settings using docker api
/// Creating a system network will use the same name for network bridge name
///
/// ## Arguments
/// - [name](str) name of the network to create
/// - [docker_api](Docker) bollard docker instance
///
/// ## Return
/// if sucess return nothing a [docker error](DockerError) is returned if an error occur
///
/// ## Examples
/// ```rust,norun
/// use crate::services;
///
/// services::utils::create_system_network(&docker, "network-name").await;
/// ```
async fn create_system_network(
  network_name: &str,
  docker_api: &Docker,
) -> Result<(), DaemonError> {
  let network_state =
    controllers::utils::get_network_state(network_name, docker_api).await?;
  if network_state == NetworkState::Ready {
    return Ok(());
  }
  let mut options: HashMap<String, String> = HashMap::new();
  options.insert(
    String::from("com.docker.network.bridge.name"),
    network_name.to_owned(),
  );
  let config = CreateNetworkOptions {
    name: network_name.to_owned(),
    driver: String::from("bridge"),
    options,
    ..Default::default()
  };
  docker_api.create_network(config).await?;
  Ok(())
}

async fn boot_store(
  config: &DaemonConfig,
  docker_api: &bollard::Docker,
) -> Result<(), DaemonError> {
  controllers::store::boot(config, docker_api).await?;
  // We wait 100ms to ensure store is booted
  // It's a tricky wait to avoid some error printed by postgresql connector.
  let sleep_time = time::Duration::from_millis(500);
  thread::sleep(sleep_time);
  Ok(())
}

/// ## Run diesel migration
/// This function ensure our store have correct datastructure based on our `migrations` folder
fn run_migrations(
  connection: &mut impl MigrationHarness<diesel::pg::Pg>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
  // This will run the necessary migrations.
  // See the documentation for `MigrationHarness` for
  // all available methods.
  connection.run_pending_migrations(MIGRATIONS)?;
  Ok(())
}

async fn create_system_cluster(
  sys_nsp: String,
  pool: &web::types::State<Pool>,
  docker_api: &bollard::Docker,
) -> Result<(), DaemonError> {
  if repositories::cluster::find_by_key(String::from("system-nano"), pool)
    .await
    .is_err()
  {
    let cluster = ClusterPartial {
      name: String::from("nano"),
      proxy_templates: None,
    };
    repositories::cluster::create_for_namespace(sys_nsp, cluster, pool).await?;
  }
  Ok(())
}

async fn prepare_store(
  config: &DaemonConfig,
  docker_api: &Docker,
) -> Result<(Pool, web::types::State<Pool>), DaemonError> {
  log::info!("Booting store");
  boot_store(config, docker_api).await?;
  log::info!("Store booted");
  let postgres_ip = controllers::store::get_store_ip_addr(docker_api).await?;
  log::info!("Connecting to store");
  // Connect to postgresql
  let pool = controllers::store::create_pool(postgres_ip.to_owned()).await;
  let s_pool = web::types::State::new(pool.to_owned());
  let mut conn = controllers::store::get_pool_conn(&s_pool)?;
  log::info!("Store connected");
  log::info!("Running migrations");
  run_migrations(&mut conn)?;
  Ok((pool, s_pool))
}

async fn create_store_cargo(
  system_nsp: &str,
  config: &DaemonConfig,
  s_pool: &web::types::State<Pool>,
) -> Result<(), DaemonError> {
  let key = utils::key::gen_key(system_nsp, "nstore");
  if repositories::cargo::find_by_key(key, s_pool).await.is_ok() {
    return Ok(());
  }
  let path = Path::new(&config.state_dir).join("store/data");
  let binds = vec![format!("{}:/cockroach/cockroach-data", path.display())];
  let store_cargo = CargoPartial {
    name: String::from("nstore"),
    image_name: String::from("cockroachdb/cockroach:v21.2.17"),
    environnements: Some(vec![String::from("test")]),
    binds: Some(binds),
    replicas: Some(1),
    dns_entry: None,
    domainname: Some(String::from("nstore")),
    hostname: Some(String::from("nstore")),
  };
  repositories::cargo::create(system_nsp.to_owned(), store_cargo, s_pool)
    .await?;

  Ok(())
}

async fn create_proxy_cargo(
  system_nsp: &str,
  config: &DaemonConfig,
  s_pool: &web::types::State<Pool>,
) -> Result<(), DaemonError> {
  let key = utils::key::gen_key(system_nsp, "nproxy");
  if repositories::cargo::find_by_key(key, s_pool).await.is_ok() {
    return Ok(());
  }

  let sites_path = Path::new(&config.state_dir).join("nginx/sites-enabled");
  let stream_path = Path::new(&config.state_dir).join("nginx/streams-enabled");
  let log_path = Path::new(&config.state_dir).join("nginx/log");
  let ssl_path = Path::new(&config.state_dir).join("nginx/ssl");
  let sock_path = Path::new(&config.state_dir).join("socket");
  let letsencrypt_path = Path::new(&config.state_dir).join("nginx/letsencrypt");
  let binds = Some(vec![
    format!("{}:/opt/nanocl-socket", sock_path.display()),
    format!("{}:/etc/nginx/sites-enabled", sites_path.display()),
    format!("{}:/var/log/nginx", log_path.display()),
    format!("{}:/etc/nginx/ssl", ssl_path.display()),
    format!("{}:/etc/nginx/streams-enabled", stream_path.display()),
    format!("{}:/etc/letsencrypt", letsencrypt_path.display()),
  ]);
  let proxy_cargo = CargoPartial {
    name: String::from("nproxy"),
    image_name: String::from("nanocl-proxy-nginx"),
    environnements: None,
    binds,
    replicas: Some(1),
    dns_entry: None,
    domainname: Some(String::from("nproxy")),
    hostname: Some(String::from("nproxy")),
  };

  repositories::cargo::create(system_nsp.to_owned(), proxy_cargo, s_pool)
    .await?;

  Ok(())
}

async fn create_dns_cargo(
  system_nsp: &str,
  config: &DaemonConfig,
  s_pool: &web::types::State<Pool>,
) -> Result<(), DaemonError> {
  let key = utils::key::gen_key(system_nsp, "ndns");
  if repositories::cargo::find_by_key(key, s_pool).await.is_ok() {
    return Ok(());
  }
  Ok(())
}

async fn prepare_system(
  config: &DaemonConfig,
  docker_api: &Docker,
  s_pool: &web::types::State<Pool>,
) -> Result<(), DaemonError> {
  const SYSTEM_NSP: &str = "system";
  const DEFAULT_NSP: &str = "global";
  const SYSTEM_NETWORK: &str = "nanoclinternal0";
  ensure_namespace(DEFAULT_NSP, &s_pool).await?;
  ensure_namespace(SYSTEM_NSP, &s_pool).await?;
  create_system_cluster(SYSTEM_NSP.to_owned(), s_pool, docker_api).await?;
  create_store_cargo(SYSTEM_NSP, config, s_pool).await?;
  create_proxy_cargo(SYSTEM_NSP, config, s_pool).await?;
  Ok(())
}

/// Boot function called before http server start to
/// initialize his state and some background task
pub async fn boot(
  config: &DaemonConfig,
  docker_api: &Docker,
) -> Result<BootState, DaemonError> {
  const SYSTEM_NETWORK: &str = "nanoclinternal0";
  create_system_network(SYSTEM_NETWORK, docker_api).await?;
  let (pool, s_pool) = prepare_store(&config, &docker_api).await?;
  // Ensure required namespace to exists
  prepare_system(config, docker_api, &s_pool).await?;
  // Return state
  Ok(BootState {
    pool,
    docker_api: docker_api.to_owned(),
  })
}
