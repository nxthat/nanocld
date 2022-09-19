use clap::{Parser, arg_enum};
use diesel_derive_enum::DbEnum;
use serde::{Serialize, Deserialize};

#[cfg(feature = "openapi")]
use utoipa::Component;

use crate::schema::nodes;

arg_enum! {
  #[derive(Debug, Clone, Parser, Eq, PartialEq, Serialize, Deserialize, DbEnum)]
  #[DieselTypePath = "crate::models::exports::Node_modes"]
  #[serde(rename_all = "snake_case")]
  #[cfg_attr(feature = "openapi", derive(Component))]
  pub enum NodeMode {
    Master,
    Worker,
    Proxy,
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, DbEnum)]
#[DieselTypePath = "crate::models::exports::Ssh_auth_modes"]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(Component))]
pub enum SshAuthMode {
  Passwd,
  Rsa,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct NodePartial {
  pub(crate) name: String,
  pub(crate) mode: NodeMode,
  pub(crate) ip_address: String,
  pub(crate) ssh_auth_mode: SshAuthMode,
  pub(crate) ssh_user: String,
  pub(crate) ssh_credential: String,
}

#[derive(
  Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable,
)]
#[diesel(primary_key(name))]
#[diesel(table_name = nodes)]
pub struct NodeItem {
  pub(crate) name: String,
  pub(crate) mode: NodeMode,
  pub(crate) ip_address: String,
  pub(crate) ssh_auth_mode: SshAuthMode,
  pub(crate) ssh_user: String,
  pub(crate) ssh_credential: String,
}

impl From<NodePartial> for NodeItem {
  fn from(p: NodePartial) -> Self {
    NodeItem {
      name: p.name,
      mode: p.mode,
      ip_address: p.ip_address,
      ssh_auth_mode: p.ssh_auth_mode,
      ssh_user: p.ssh_user,
      ssh_credential: p.ssh_credential,
    }
  }
}
