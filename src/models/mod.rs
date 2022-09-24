use r2d2::PooledConnection;
use diesel::{r2d2::ConnectionManager, PgConnection};
use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::Component;

mod namespace;
pub use namespace::*;

mod git_repository;
pub use git_repository::*;

mod git_repository_branch;
pub use git_repository_branch::*;

mod cargo;
pub use cargo::*;

mod cluster;
pub use cluster::*;

mod cluster_network;
pub use cluster_network::*;

mod cluster_cargo;
pub use cluster_cargo::*;

mod cluster_variable;
pub use cluster_variable::*;

mod cargo_env;
pub use cargo_env::*;

mod nginx_template;
pub use nginx_template::*;

mod nginx_log;
pub use nginx_log::*;

mod container;
pub use container::*;

mod container_image;
pub use container_image::*;

mod node;
pub use node::*;

pub type DBConn = PooledConnection<ConnectionManager<PgConnection>>;
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Generic postgresql delete response
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct GenericDelete {
  pub(crate) count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct GenericCount {
  pub(crate) count: i64,
}

/// Generic namespace query filter
#[derive(Debug, Serialize, Deserialize)]
pub struct GenericNspQuery {
  pub(crate) namespace: Option<String>,
}
