use serde::{Serialize, Deserialize};

#[cfg(feature = "dev")]
use utoipa::ToSchema;

use crate::schema::clusters;

use super::cargo::CargoItem;
use super::cluster_network::ClusterNetworkItem;
use super::cluster_variable::ClusterVariableItem;

/// Partial cluster
/// this structure ensure write in database
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct ClusterPartial {
  pub(crate) name: String,
  pub(crate) proxy_templates: Option<Vec<String>>,
}

/// Cluster used to encapsulate networks
/// this structure ensure read and write in database
#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  Identifiable,
  Insertable,
  AsChangeset,
  Queryable,
)]
#[diesel(primary_key(key))]
#[diesel(table_name = clusters)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct ClusterItem {
  pub(crate) key: String,
  pub(crate) name: String,
  pub(crate) namespace: String,
  pub(crate) proxy_templates: Vec<String>,
}

/// Cluster item with his relations
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct ClusterItemWithRelation {
  pub(crate) key: String,
  pub(crate) name: String,
  pub(crate) namespace: String,
  pub(crate) proxy_templates: Vec<String>,
  pub(crate) variables: Vec<ClusterVariableItem>,
  pub(crate) networks: Option<Vec<ClusterNetworkItem>>,
  pub(crate) cargoes: Option<Vec<CargoItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct ClusterJoinBody {
  pub(crate) cargo: String,
  pub(crate) network: String,
}
