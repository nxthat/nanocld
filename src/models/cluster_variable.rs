use serde::{Serialize, Deserialize};
#[cfg(feature = "openapi")]
use utoipa::Component;

use crate::schema::cluster_variables;

use super::cluster::ClusterItem;

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
#[primary_key(key)]
#[table_name = "cluster_variables"]
#[belongs_to(ClusterItem, foreign_key = "cluster_key")]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct ClusterVariableItem {
  pub(crate) key: String,
  pub(crate) cluster_key: String,
  pub(crate) name: String,
  pub(crate) value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct ClusterVariablePartial {
  pub(crate) name: String,
  pub(crate) value: String,
}
