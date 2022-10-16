//! File used to describe daemon boot
use std::error::Error;

use ntex::web;

use bollard::errors::Error as DockerError;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::config::DaemonConfig;
use crate::{controllers, repositories};
use crate::models::{Pool, NamespacePartial};

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
/// create_default_nsp(&pool).await;
/// ```
async fn create_default_nsp(
  pool: &web::types::State<Pool>,
) -> Result<(), DaemonError> {
  const NSP_NAME: &str = "global";
  match repositories::namespace::inspect_by_name(NSP_NAME.to_string(), pool)
    .await
  {
    Err(_err) => {
      let new_nsp = NamespacePartial {
        name: NSP_NAME.to_string(),
      };
      repositories::namespace::create(new_nsp, pool).await?;
      Ok(())
    }
    Ok(_) => Ok(()),
  }
}

pub async fn create_default_network(
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

async fn boot_docker_services(
  config: &DaemonConfig,
  docker_api: &bollard::Docker,
) -> Result<(), DaemonError> {
  create_default_network(docker_api).await?;
  // Boot postgresql service to ensure database connection
  controllers::store::boot(config, docker_api).await?;
  // Boot dnsmasq service to manage domain names
  controllers::dns::boot(config, docker_api).await?;
  // Boot nginx service to manage proxy
  controllers::proxy::boot(config, docker_api).await?;

  // services::ipsec::boot(config, docker_api).await?;
  Ok(())
}

fn run_migrations(
  connection: &mut impl MigrationHarness<diesel::pg::Pg>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
  // This will run the necessary migrations.
  // See the documentation for `MigrationHarness` for
  // all available methods.

  log::info!("runing migration");
  connection.run_pending_migrations(MIGRATIONS)?;
  Ok(())
}

/// Boot function called before http server start to
/// initialize his state and some background task
pub async fn boot(
  config: &DaemonConfig,
  docker_api: &bollard::Docker,
) -> Result<BootState, DaemonError> {
  log::info!("booting");
  boot_docker_services(config, docker_api).await?;
  let postgres_ip = controllers::store::get_postgres_ip(docker_api).await?;
  log::info!("creating postgresql state pool");
  // Connect to postgresql
  let db_pool = controllers::store::create_pool(postgres_ip.to_owned()).await;
  let pool = web::types::State::new(db_pool.to_owned());
  let mut conn = controllers::store::get_pool_conn(&pool)?;
  // wrap into state to be abble to use our functions
  run_migrations(&mut conn)?;
  // Create default namesapce
  create_default_nsp(&pool).await?;

  log::info!("booted");
  // Return state
  Ok(BootState {
    pool: db_pool,
    docker_api: docker_api.to_owned(),
  })
}
