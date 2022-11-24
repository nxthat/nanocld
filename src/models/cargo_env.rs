use serde::{Serialize, Deserialize};

#[cfg(feature = "dev")]
use utoipa::ToSchema;

use crate::schema::cargo_environnements;

#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct CargoEnvPartial {
  pub(crate) cargo_key: String,
  pub(crate) name: String,
  pub(crate) value: String,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(primary_key(key))]
#[diesel(table_name = cargo_environnements)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct CargoEnvItem {
  pub(crate) key: String,
  pub(crate) cargo_key: String,
  pub(crate) name: String,
  pub(crate) value: String,
}
