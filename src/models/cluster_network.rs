use serde::{Serialize, Deserialize};

#[cfg(feature = "openapi")]
use utoipa::Component;

use crate::schema::cluster_networks;

use super::cluster::ClusterItem;

/// Cluster network partial
/// this structure ensure write in database
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct ClusterNetworkPartial {
  pub(crate) name: String,
}

/// Cluster network item
/// this structure ensure read and write in database
#[derive(
  Debug,
  Serialize,
  Deserialize,
  Queryable,
  Identifiable,
  Insertable,
  Associations,
  AsChangeset,
)]
#[diesel(primary_key(key))]
#[diesel(table_name = cluster_networks)]
#[diesel(belongs_to(ClusterItem, foreign_key = cluster_key))]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct ClusterNetworkItem {
  pub(crate) key: String,
  pub(crate) name: String,
  pub(crate) namespace: String,
  pub(crate) docker_network_id: String,
  pub(crate) default_gateway: String,
  pub(crate) cluster_key: String,
}
