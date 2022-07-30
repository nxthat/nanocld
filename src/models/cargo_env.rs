use serde::{Serialize, Deserialize};
#[cfg(feature = "openapi")]
use utoipa::Component;

use crate::schema::cargo_environnements;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct CargoEnvPartial {
  pub(crate) cargo_key: String,
  pub(crate) name: String,
  pub(crate) value: String,
}

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
#[table_name = "cargo_environnements"]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct CargoEnvItem {
  pub(crate) key: String,
  pub(crate) cargo_key: String,
  pub(crate) name: String,
  pub(crate) value: String,
}
