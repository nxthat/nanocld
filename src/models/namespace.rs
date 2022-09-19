use serde::{Serialize, Deserialize};

#[cfg(feature = "openapi")]
use utoipa::Component;

use crate::schema::namespaces;

/// Namespace to encapsulate clusters
/// this structure ensure read and write in database
#[derive(
  Debug, Serialize, Deserialize, Identifiable, Insertable, Queryable,
)]
#[diesel(primary_key(name))]
#[diesel(table_name = namespaces)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct NamespaceItem {
  pub(crate) name: String,
}

/// Partial namespace
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct NamespacePartial {
  pub(crate) name: String,
}
