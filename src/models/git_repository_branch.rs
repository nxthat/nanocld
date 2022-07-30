use serde::{Serialize, Deserialize};
#[cfg(feature = "openapi")]
use utoipa::Component;

use crate::schema::git_repository_branches;

/// Git repository branch
/// this structure ensure read and write entity in database
#[derive(
  Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable,
)]
#[primary_key(key)]
#[table_name = "git_repository_branches"]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct GitRepositoryBranchItem {
  pub(crate) key: String,
  pub(crate) name: String,
  pub(crate) last_commit_sha: String,
  pub(crate) repository_name: String,
}

/// Partial git repository branch
/// this structure ensure write in database
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct GitRepositoryBranchPartial {
  pub(crate) name: String,
  pub(crate) last_commit_sha: String,
  pub(crate) repository_name: String,
}
