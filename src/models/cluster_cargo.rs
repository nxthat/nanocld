use serde::{Serialize, Deserialize};
#[cfg(feature = "openapi")]
use utoipa::Component;

use crate::schema::cluster_cargoes;

use super::cargo::CargoItem;
use super::cluster::ClusterItem;
use super::cluster_network::ClusterNetworkItem;

/// Structure definition of a cluster cargo item in database
#[derive(
  Debug,
  Serialize,
  Deserialize,
  Queryable,
  Insertable,
  Identifiable,
  Associations,
  AsChangeset,
)]
#[primary_key(key)]
#[table_name = "cluster_cargoes"]
#[belongs_to(CargoItem, foreign_key = "cargo_key")]
#[belongs_to(ClusterItem, foreign_key = "cluster_key")]
#[belongs_to(ClusterNetworkItem, foreign_key = "network_key")]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct ClusterCargoItem {
  pub(crate) key: String,
  pub(crate) cargo_key: String,
  pub(crate) cluster_key: String,
  pub(crate) network_key: String,
}

/// Structure used as body parameter to create a cluster cargo
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct ClusterCargoPartial {
  pub(crate) cargo_key: String,
  pub(crate) cluster_key: String,
  pub(crate) network_key: String,
}

/// Structure used to parse path parameter of cluster cargo patch method
#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterCargoPatchPath {
  pub(crate) cluster_name: String,
  pub(crate) cargo_name: String,
}
