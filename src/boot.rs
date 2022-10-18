//! File used to describe daemon boot
use std::error::Error;
use std::{thread, time};

use ntex::web;

use bollard::errors::Error as DockerError;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::config::DaemonConfig;
use crate::{controllers, repositories};
use crate::models::{Pool, NamespacePartial, ClusterItem, ClusterPartial};

use crate::errors::DaemonError;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[derive(Clone)]
pub struct BootState {
  pub(crate) pool: Pool,
  pub(crate) docker_api: bollard::Docker,
}

/// # Create default namespace
/// Create a namespace with default as name if he doesn't exist
///
/// # Arguments
/// - [pool](web::types::State<Pool>) Postgres database pool
///
/// # Examples
/// ```rust,norun
/// ensure_namespace(&pool).await;
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

pub async fn create_system_network(
  docker_api: &bollard::Docker,
) -> Result<(), DockerError> {
  let network_name = "nanoclservices0";
  let state =
    controllers::utils::get_network_state(network_name, docker_api).await?;
  if state == controllers::utils::NetworkState::NotFound {
    controllers::utils::create_network(network_name, docker_api).await?;
  }
  Ok(())
}

async fn boot_controllers(
  config: &DaemonConfig,
  docker_api: &bollard::Docker,
) -> Result<(), DaemonError> {
  create_system_network(docker_api).await?;
  // Boot postgresql service to ensure database connection
  // controllers::store::boot(config, docker_api).await?;
  // Boot dnsmasq service to manage domain names
  controllers::dns::boot(config, docker_api).await?;
  // Boot nginx service to manage proxy
  controllers::proxy::boot(config, docker_api).await?;
  // services::ipsec::boot(config, docker_api).await?;
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
  let cluster = ClusterPartial {
    name: String::from(""),
    proxy_templates: None,
  };
  repositories::cluster::create_for_namespace(sys_nsp, cluster, pool).await?;
  Ok(())
}

/// Boot function called before http server start to
/// initialize his state and some background task
pub async fn boot(
  config: &DaemonConfig,
  docker_api: &bollard::Docker,
) -> Result<BootState, DaemonError> {
  const DEFAULT_NSP: &str = "global";
  const SYSTEM_NSP: &str = "system";
  log::info!("Booting store");
  boot_store(config, docker_api).await?;
  log::info!("Store booted");
  let postgres_ip = controllers::store::get_store_ip_addr(docker_api).await?;
  log::info!("Connecting to store");
  // Connect to postgresql
  let db_pool = controllers::store::create_pool(postgres_ip.to_owned()).await;
  let pool = web::types::State::new(db_pool.to_owned());
  let mut conn = controllers::store::get_pool_conn(&pool)?;
  log::info!("Store connected");
  log::info!("Running migrations");
  run_migrations(&mut conn)?;
  // Ensure required namespace to exists
  ensure_namespace(DEFAULT_NSP, &pool).await?;
  ensure_namespace(SYSTEM_NSP, &pool).await?;
  log::info!("Migrations success");
  log::info!("Booted");
  // Return state
  Ok(BootState {
    pool: db_pool,
    docker_api: docker_api.to_owned(),
  })
}
