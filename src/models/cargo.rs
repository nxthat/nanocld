use serde::{Serialize, Deserialize};

#[cfg(feature = "dev")]
use utoipa::ToSchema;

use crate::schema::cargoes;

use super::cargo_env::CargoEnvItem;
use super::namespace::NamespaceItem;

/// Cargo partial
/// this structure ensure write in database
#[derive(Default, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct CargoPartial {
  pub(crate) name: String,
  pub(crate) config: serde_json::Value,
}

#[derive(Default, Serialize, Deserialize)]
pub struct CargoPatchPartial {
  pub(crate) name: Option<String>,
  pub(crate) config: serde_json::Value,
}

#[derive(AsChangeset)]
#[diesel(table_name = cargoes)]
pub struct CargoPatchItem {
  pub(crate) key: Option<String>,
  pub(crate) name: Option<String>,
  pub(crate) config: serde_json::Value,
}

/// Cargo item is an definition to container create image and start them
/// this structure ensure read and write in database
#[derive(
  Debug,
  Serialize,
  Deserialize,
  Queryable,
  Identifiable,
  Insertable,
  Associations,
)]
#[diesel(primary_key(key))]
#[diesel(table_name = cargoes)]
#[diesel(belongs_to(NamespaceItem, foreign_key = namespace_name))]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct CargoItem {
  pub(crate) key: String,
  pub(crate) namespace_name: String,
  pub(crate) name: String,
  pub(crate) config: serde_json::Value,
}

/// Cargo item with his relation
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct CargoItemWithRelation {
  pub(crate) key: String,
  pub(crate) namespace_name: String,
  pub(crate) name: String,
  pub(crate) config: serde_json::Value,
  pub(crate) environnements: Option<Vec<CargoEnvItem>>,
  pub(crate) containers: Vec<bollard::models::ContainerSummary>,
}
