use serde::{Serialize, Deserialize};

#[cfg(feature = "dev")]
use utoipa::ToSchema;

use crate::schema::git_repository_branches;

/// Git repository branch
/// this structure ensure read and write entity in database
#[derive(
  Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable,
)]
#[diesel(primary_key(key))]
#[diesel(table_name = git_repository_branches)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct GitRepositoryBranchItem {
  pub(crate) key: String,
  pub(crate) name: String,
  pub(crate) last_commit_sha: String,
  pub(crate) repository_name: String,
}

/// Partial git repository branch
/// this structure ensure write in database
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct GitRepositoryBranchPartial {
  pub(crate) name: String,
  pub(crate) last_commit_sha: String,
  pub(crate) repository_name: String,
}
