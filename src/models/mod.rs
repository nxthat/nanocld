use r2d2::PooledConnection;
use diesel::{r2d2::ConnectionManager, PgConnection};

use serde::{Deserialize, Serialize};

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

mod virtual_machine_image;
pub use virtual_machine_image::*;

mod virtual_machine;
pub use virtual_machine::*;

pub type DBConn = PooledConnection<ConnectionManager<PgConnection>>;
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[cfg(feature = "openapi")]
use utoipa::Component;

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

/// Re exports ours enums and diesel sql_types for schema.rs
pub mod exports {
  pub use diesel::sql_types::*;
  pub use super::node::{Node_modes, Ssh_auth_modes};
  pub use super::nginx_template::Nginx_template_modes;
  pub use super::git_repository::Git_repository_source_type;
}
