//! File used to describe daemon boot
use std::collections::HashMap;
use std::path::Path;
use std::{time, thread};

use ntex::web;
use ntex::http::StatusCode;

use bollard::Docker;
use bollard::network::{CreateNetworkOptions, InspectNetworkOptions};

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::cli::Cli;
use crate::{utils, controllers, repositories};
use crate::models::{
  Pool, NamespacePartial, ClusterPartial, CargoPartial, ClusterNetworkPartial,
  CargoInstancePartial, DaemonConfig, ArgState, DaemonState,
};

use crate::errors::{DaemonError, HttpResponseError};

use super::config;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

/// Ensure existance of the system network that controllers will use.
/// It's ensure existance of a network in your system called `nanoclinternal0`
/// Also registered inside docker as `system-nano-internal0`
async fn ensure_system_network(docker_api: &Docker) -> Result<(), DaemonError> {
  const SYSTEM_NETWORK_KEY: &str = "system-nano-internal0";
  const SYSTEM_NETWORK: &str = "nanoclinternal0";
  let network_state =
    utils::docker::get_network_state(SYSTEM_NETWORK_KEY, docker_api).await?;
  if network_state == utils::docker::NetworkState::Ready {
    return Ok(());
  }
  let mut options: HashMap<String, String> = HashMap::new();
  options.insert(
    String::from("com.docker.network.bridge.name"),
    SYSTEM_NETWORK.to_owned(),
  );
  let config = CreateNetworkOptions {
    name: SYSTEM_NETWORK_KEY.to_owned(),
    driver: String::from("bridge"),
    options,
    ..Default::default()
  };
  docker_api.create_network(config).await?;
  Ok(())
}

/// Ensure existance of a container for our store
/// we use cockroachdb with a postgresql connector.
/// we also run latest migration on our database to have the latest schema.
/// It will return a connection Pool that will be use in our State.
async fn ensure_store(
  config: &DaemonConfig,
  docker_api: &Docker,
) -> Result<(Pool, web::types::State<Pool>), DaemonError> {
  log::info!("Booting store");
  controllers::store::boot(config, docker_api).await?;
  // We wait 500ms to ensure store is booted
  // It's a tricky hack to avoid some error printed by postgresql connector for now.
  let sleep_time = time::Duration::from_millis(500);
  thread::sleep(sleep_time);
  log::info!("Store booted");
  let postgres_ip = controllers::store::get_store_ip_addr(docker_api).await?;
  log::info!("Connecting to store");
  // Connect to postgresql
  let pool = controllers::store::create_pool(postgres_ip.to_owned()).await;
  let s_pool = web::types::State::new(pool.to_owned());
  let mut conn = controllers::store::get_pool_conn(&s_pool)?;
  log::info!("Store connected");
  log::info!("Running migrations");
  // This will run the necessary migrations.
  // See the documentation for `MigrationHarness` for
  // all available methods.
  conn.run_pending_migrations(MIGRATIONS)?;
  Ok((pool, s_pool))
}

/// Ensure existance of specific namespace in our store.
/// We use it to be sure `system` and `global` namespace exists.
/// system is the namespace where controllers are registered.
/// where global is the namespace used by default.
/// User can registed they own namespace to ensure better encaptusation of projects.
async fn register_namespace(
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

/// Ensure existance of a cluster called `nano` in our store
/// This cluster is the default cluster where our controllers will be created.
async fn register_system_cluster(
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

/// Ensure existance of the system network in our store binded to `nanoclinternal0`
async fn register_system_network(
  boot_config: &ArgState,
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

/// Ensure exsistance of our deamon in the store.
/// We are running inside us it's that crazy ?
async fn register_daemon(boot_config: &ArgState) -> Result<(), DaemonError> {
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

/// Register all dependencies needed
/// Default Namespace, Cluster, Network, and Controllers will be registered in our store
async fn register_dependencies(
  boot_config: &ArgState,
) -> Result<(), DaemonError> {
  register_namespace(&boot_config.default_namespace, &boot_config.s_pool)
    .await?;
  register_namespace(&boot_config.sys_namespace, &boot_config.s_pool).await?;
  register_system_cluster(
    boot_config.sys_namespace.to_owned(),
    &boot_config.s_pool,
  )
  .await?;
  register_system_network(boot_config).await?;
  controllers::store::register(boot_config).await?;
  controllers::proxy::register(
    &boot_config.sys_namespace,
    &boot_config.config,
    &boot_config.s_pool,
  )
  .await?;
  controllers::dns::register(boot_config).await?;
  register_daemon(boot_config).await?;
  Ok(())
}

/// Init function called before http server start
/// to initialize our state
pub async fn init(args: &Cli) -> Result<DaemonState, DaemonError> {
  let config = config::init(args)?;
  let docker_api = bollard::Docker::connect_with_unix(
    &config.docker_host,
    120,
    bollard::API_DEFAULT_VERSION,
  )?;
  ensure_system_network(&docker_api).await?;
  let (pool, s_pool) = ensure_store(&config, &docker_api).await?;
  let boot_config = ArgState {
    config: config.to_owned(),
    s_pool,
    docker_api: docker_api.to_owned(),
    default_namespace: String::from("global"),
    sys_cluster: String::from("nano"),
    sys_network: String::from("internal0"),
    sys_namespace: String::from("system"),
  };
  register_dependencies(&boot_config).await?;
  Ok(DaemonState {
    pool,
    config,
    docker_api,
  })
}
