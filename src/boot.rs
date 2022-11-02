//! File used to describe daemon boot
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::{thread, time};

use ntex::http::StatusCode;
use ntex::web;
use bollard::Docker;
use bollard::network::{CreateNetworkOptions, InspectNetworkOptions};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::config::DaemonConfig;
use crate::{controllers, repositories, utils};
use crate::models::{
  Pool, NamespacePartial, ClusterPartial, CargoPartial, ClusterNetworkPartial,
  CargoInstancePartial,
};

use crate::errors::{DaemonError, HttpResponseError};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[derive(Clone)]
pub struct BootState {
  pub(crate) pool: Pool,
  pub(crate) docker_api: Docker,
}

struct BootConfig {
  config: DaemonConfig,
  s_pool: web::types::State<Pool>,
  docker_api: Docker,
  default_namespace: String,
  sys_cluster: String,
  sys_network: String,
  sys_namespace: String,
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
  interface_name: &str,
  docker_api: &Docker,
) -> Result<(), DaemonError> {
  let network_state =
    utils::docker::get_network_state(network_name, docker_api).await?;
  if network_state == utils::docker::NetworkState::Ready {
    return Ok(());
  }
  let mut options: HashMap<String, String> = HashMap::new();
  options.insert(
    String::from("com.docker.network.bridge.name"),
    interface_name.to_owned(),
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
) -> Result<(), DaemonError> {
  if repositories::cluster::find_by_key(String::from("system-nano"), pool)
    .await
    .is_ok()
  {
    return Ok(());
  }
  let cluster = ClusterPartial {
    name: String::from("nano"),
    proxy_templates: None,
  };
  repositories::cluster::create_for_namespace(sys_nsp, cluster, pool).await?;
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
  boot_config: &BootConfig,
) -> Result<(), DaemonError> {
  let key = utils::key::gen_key(&boot_config.sys_namespace, "store");
  if repositories::cargo::find_by_key(key, &boot_config.s_pool)
    .await
    .is_ok()
  {
    return Ok(());
  }
  let path = Path::new(&boot_config.config.state_dir).join("store/data");
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
  let cargo = repositories::cargo::create(
    boot_config.sys_namespace.to_owned(),
    store_cargo,
    &boot_config.s_pool,
  )
  .await?;

  let cluster_key =
    utils::key::gen_key(&boot_config.sys_namespace, &boot_config.sys_cluster);
  let network_key = utils::key::gen_key(&cluster_key, &boot_config.sys_network);
  let cargo_instance = CargoInstancePartial {
    cargo_key: cargo.key,
    cluster_key,
    network_key,
  };

  repositories::cargo_instance::create(cargo_instance, &boot_config.s_pool)
    .await?;

  Ok(())
}

async fn create_proxy_cargo(
  system_nsp: &str,
  config: &DaemonConfig,
  s_pool: &web::types::State<Pool>,
) -> Result<(), DaemonError> {
  let key = utils::key::gen_key(system_nsp, "proxy");
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
    name: String::from("proxy"),
    image_name: String::from("nanocl-proxy-nginx"),
    environnements: None,
    binds,
    replicas: Some(1),
    dns_entry: None,
    domainname: Some(String::from("proxy")),
    hostname: Some(String::from("proxy")),
    network_mode: Some(String::from("host")),
    restart_policy: Some(String::from("unless-stopped")),
    cap_add: None,
  };

  repositories::cargo::create(system_nsp.to_owned(), proxy_cargo, s_pool)
    .await?;

  Ok(())
}

async fn create_dns_cargo(boot_config: &BootConfig) -> Result<(), DaemonError> {
  let key = utils::key::gen_key(&boot_config.sys_namespace, "dns");

  if repositories::cargo::find_by_key(key, &boot_config.s_pool)
    .await
    .is_ok()
  {
    return Ok(());
  }

  let config_file_path =
    Path::new(&boot_config.config.state_dir).join("dnsmasq/dnsmasq.conf");
  let dir_path =
    Path::new(&boot_config.config.state_dir).join("dnsmasq/dnsmasq.d/");
  let binds = Some(vec![
    format!("{}:/etc/dnsmasq.conf", config_file_path.display()),
    format!("{}:/etc/dnsmasq.d/", dir_path.display()),
  ]);
  let dns_cargo = CargoPartial {
    name: String::from("dns"),
    image_name: String::from("nanocl-dns-dnsmasq"),
    environnements: None,
    binds,
    replicas: Some(1),
    dns_entry: None,
    domainname: Some(String::from("dns")),
    hostname: Some(String::from("dns")),
    network_mode: Some(String::from("host")),
    restart_policy: Some(String::from("unless-stopped")),
    cap_add: Some(vec![String::from("NET_ADMIN")]),
  };

  repositories::cargo::create(
    boot_config.sys_namespace.to_owned(),
    dns_cargo,
    &boot_config.s_pool,
  )
  .await?;

  Ok(())
}

async fn create_daemon_cargo(
  boot_config: &BootConfig,
) -> Result<(), DaemonError> {
  let key = utils::key::gen_key(&boot_config.sys_namespace, "daemon");
  if repositories::cargo::find_by_key(key, &boot_config.s_pool)
    .await
    .is_ok()
  {
    return Ok(());
  }
  println!("state dir {}", &boot_config.config.state_dir);
  let path = Path::new(&boot_config.config.state_dir);
  let binds = vec![format!("{}:/var/lib/nanocl", path.display())];
  let store_cargo = CargoPartial {
    name: String::from("daemon"),
    image_name: String::from("nanocl-daemon:0.1.5"),
    environnements: None,
    binds: Some(binds),
    replicas: Some(1),
    dns_entry: None,
    domainname: Some(String::from("daemon")),
    hostname: Some(String::from("daemon")),
    network_mode: Some(String::from("host")),
    restart_policy: Some(String::from("unless-stopped")),
    cap_add: None,
  };
  let cargo = repositories::cargo::create(
    boot_config.sys_namespace.to_owned(),
    store_cargo,
    &boot_config.s_pool,
  )
  .await?;

  let cluster_key =
    utils::key::gen_key(&boot_config.sys_namespace, &boot_config.sys_cluster);
  let network_key = utils::key::gen_key(&cluster_key, &boot_config.sys_network);
  let cargo_instance = CargoInstancePartial {
    cargo_key: cargo.key,
    cluster_key,
    network_key,
  };

  repositories::cargo_instance::create(cargo_instance, &boot_config.s_pool)
    .await?;

  Ok(())
}

/// Register default system network in store
async fn register_system_network(
  boot_config: &BootConfig,
) -> Result<(), DaemonError> {
  let cluster_key =
    utils::key::gen_key(&boot_config.sys_namespace, &boot_config.sys_cluster);
  let key = utils::key::gen_key(&cluster_key, &boot_config.sys_network);

  if repositories::cluster_network::find_by_key(key, &boot_config.s_pool)
    .await
    .is_ok()
  {
    return Ok(());
  }

  let docker_network = boot_config
    .docker_api
    .inspect_network(
      "system-nano-internal0",
      None::<InspectNetworkOptions<&str>>,
    )
    .await?;
  let network = ClusterNetworkPartial {
    name: boot_config.sys_network.to_owned(),
  };

  let docker_network_id =
    docker_network
      .to_owned()
      .id
      .ok_or(DaemonError::HttpResponse(HttpResponseError {
        msg: String::from("Unable to get network ID of system-nano-internal0"),
        status: StatusCode::INTERNAL_SERVER_ERROR,
      }))?;

  let default_gateway = utils::docker::get_default_gateway(&docker_network)?;

  repositories::cluster_network::create_for_cluster(
    boot_config.sys_namespace.to_owned(),
    boot_config.sys_cluster.to_owned(),
    network,
    docker_network_id,
    default_gateway.to_owned(),
    &boot_config.s_pool,
  )
  .await?;

  Ok(())
}

async fn prepare_system(boot_config: &BootConfig) -> Result<(), DaemonError> {
  ensure_namespace(&boot_config.default_namespace, &boot_config.s_pool).await?;
  ensure_namespace(&boot_config.sys_namespace, &boot_config.s_pool).await?;
  create_system_cluster(
    boot_config.sys_namespace.to_owned(),
    &boot_config.s_pool,
  )
  .await?;
  register_system_network(boot_config).await?;
  create_store_cargo(boot_config).await?;
  create_daemon_cargo(boot_config).await?;
  create_proxy_cargo(
    &boot_config.sys_namespace,
    &boot_config.config,
    &boot_config.s_pool,
  )
  .await?;
  create_dns_cargo(boot_config).await?;
  Ok(())
}

/// Boot function called before http server start to
/// initialize his state and some background task
pub async fn boot(
  config: &DaemonConfig,
  docker_api: &Docker,
) -> Result<BootState, DaemonError> {
  const SYSTEM_NETWORK_KEY: &str = "system-nano-internal0";
  const SYSTEM_NETWORK: &str = "nanoclinternal0";
  create_system_network(SYSTEM_NETWORK_KEY, SYSTEM_NETWORK, docker_api).await?;
  let (pool, s_pool) = prepare_store(&config, &docker_api).await?;
  let boot_config = BootConfig {
    config: config.to_owned(),
    s_pool,
    docker_api: docker_api.to_owned(),
    default_namespace: String::from("global"),
    sys_cluster: String::from("nano"),
    sys_network: String::from("internal0"),
    sys_namespace: String::from("system"),
  };
  prepare_system(&boot_config).await?;
  // Return state
  Ok(BootState {
    pool,
    docker_api: docker_api.to_owned(),
  })
}
