use serde::{Serialize, Deserialize};

#[cfg(feature = "dev")]
use utoipa::ToSchema;

use crate::schema::cluster_networks;

use super::cluster::ClusterItem;

/// Enum used to represent network state
#[derive(Debug, Eq, PartialEq)]
pub enum NetworkState {
  NotFound,
  Ready,
}

/// Cluster network partial
/// this structure ensure write in database
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct ClusterNetworkPartial {
  pub(crate) name: String,
}

/// Cluster network item
/// this structure ensure read and write in database
#[derive(
  Serialize, Deserialize, Queryable, Identifiable, Insertable, Associations,
)]
#[diesel(primary_key(key))]
#[diesel(table_name = cluster_networks)]
#[diesel(belongs_to(ClusterItem, foreign_key = cluster_key))]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct ClusterNetworkItem {
  pub(crate) key: String,
  pub(crate) name: String,
  pub(crate) namespace: String,
  pub(crate) docker_network_id: String,
  pub(crate) default_gateway: String,
  pub(crate) cluster_key: String,
}

/// Structure used to parse inspect cluster network route path
#[derive(Serialize, Deserialize)]
pub struct InspectClusterNetworkPath {
  pub(crate) c_name: String,
  pub(crate) n_name: String,
}
