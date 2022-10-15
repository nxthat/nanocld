use serde::{Serialize, Deserialize};

#[cfg(feature = "dev")]
use utoipa::ToSchema;

use crate::schema::namespaces;

/// Namespace to encapsulate clusters
/// this structure ensure read and write in database
#[derive(
  Debug, Serialize, Deserialize, Identifiable, Insertable, Queryable,
)]
#[diesel(primary_key(name))]
#[diesel(table_name = namespaces)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct NamespaceItem {
  pub(crate) name: String,
}

/// Partial namespace
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct NamespacePartial {
  pub(crate) name: String,
}
