use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::Component;

use crate::schema::virtual_machine_images;

/// Virtual machine image
#[derive(
  Debug,
  Serialize,
  Deserialize,
  Insertable,
  Queryable,
  Identifiable,
  Associations,
)]
#[primary_key(key)]
#[table_name = "virtual_machine_images"]
#[belongs_to(Self, foreign_key = "parent_key")]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct VmImageItem {
  pub(crate) key: String,
  pub(crate) name: String,
  pub(crate) path: String,
  pub(crate) size: i64,
  pub(crate) is_base: bool,
  pub(crate) parent_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VmImagePartial {
  pub(crate) name: String,
  pub(crate) path: String,
  #[serde(default)]
  pub(crate) is_base: bool,
  pub(crate) parent_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VmImageImportPayload {
  pub(crate) name: String,
  pub(crate) url: String,
}
