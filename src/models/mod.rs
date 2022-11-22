use r2d2::PooledConnection;
use serde::{Deserialize, Serialize};
use diesel::{r2d2::ConnectionManager, PgConnection};

#[cfg(feature = "dev")]
use utoipa::ToSchema;

mod config;
pub use config::*;

mod state;
pub use state::*;

mod namespace;
pub use namespace::*;

mod cargo;
pub use cargo::*;

mod cluster;
pub use cluster::*;

mod cluster_network;
pub use cluster_network::*;

mod cargo_instance;
pub use cargo_instance::*;

mod cluster_variable;
pub use cluster_variable::*;

mod cargo_env;
pub use cargo_env::*;

mod proxy_template;
pub use proxy_template::*;

mod nginx_log;
pub use nginx_log::*;

mod container;
pub use container::*;

mod cargo_image;
pub use cargo_image::*;

mod node;
pub use node::*;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DBConn = PooledConnection<ConnectionManager<PgConnection>>;

/// Generic delete response
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct GenericDelete {
  pub(crate) count: usize,
}

/// Generic count response
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct GenericCount {
  pub(crate) count: i64,
}

/// Generic namespace query filter
#[derive(Debug, Serialize, Deserialize)]
pub struct GenericNspQuery {
  pub(crate) namespace: Option<String>,
}
