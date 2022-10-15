use serde::{Serialize, Deserialize};

#[cfg(feature = "dev")]
use utoipa::ToSchema;

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
#[diesel(primary_key(key))]
#[diesel(table_name = cluster_variables)]
#[diesel(belongs_to(ClusterItem, foreign_key = cluster_key))]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct ClusterVariableItem {
  pub(crate) key: String,
  pub(crate) cluster_key: String,
  pub(crate) name: String,
  pub(crate) value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct ClusterVariablePartial {
  pub(crate) name: String,
  pub(crate) value: String,
}
