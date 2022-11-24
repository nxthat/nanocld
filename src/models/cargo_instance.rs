use serde::{Serialize, Deserialize};

#[cfg(feature = "dev")]
use utoipa::ToSchema;

use crate::schema::cargo_instances;

/// Structure definition of a cluster cargo item in database
#[derive(Queryable, Insertable)]
#[diesel(primary_key(key))]
#[diesel(table_name = cargo_instances)]
#[diesel(belongs_to(CargoItem, foreign_key = cargo_key))]
#[diesel(belongs_to(ClusterItem, foreign_key = cluster_key))]
#[diesel(belongs_to(ClusterNetworkItem, foreign_key = network_key))]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct CargoInstanceItem {
  pub(crate) key: String,
  pub(crate) cargo_key: String,
  pub(crate) cluster_key: String,
  pub(crate) network_key: String,
}

/// Structure used as body parameter to create a cluster cargo
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct CargoInstancePartial {
  pub(crate) cargo_key: String,
  pub(crate) cluster_key: String,
  pub(crate) network_key: String,
}

/// Data use to filter containers by cluster cargo or namespace.
#[derive(Default, Serialize, Deserialize)]
pub struct CargoInstanceFilterQuery {
  pub(crate) cluster: Option<String>,
  pub(crate) cargo: Option<String>,
  pub(crate) namespace: Option<String>,
}

/// Structure used to create an exec instance inside a container
#[derive(Serialize, Deserialize)]
pub struct CargoInstanceExecBody {
  pub(crate) attach_stdin: Option<bool>,
  pub(crate) attach_stdout: Option<bool>,
  pub(crate) attach_stderr: Option<bool>,
  pub(crate) detach_keys: Option<String>,
  pub(crate) tty: Option<bool>,
  pub(crate) env: Option<Vec<String>>,
  pub(crate) cmd: Option<Vec<String>>,
  pub(crate) privileged: Option<bool>,
  pub(crate) user: Option<String>,
  pub(crate) working_dir: Option<String>,
}
