use diesel_derive_enum::DbEnum;
use serde::{Serialize, Deserialize};
#[cfg(feature = "openapi")]
use utoipa::Component;

use crate::schema::git_repositories;

/// Git repository source types
/// # Examples
/// ```
/// GitRepositorySourceType::Github; // For github.com
/// GitRepositorySourceType::Gitlab; // for gitlab.com
/// GitRepositorySourceType::Local; // for nanocl managed git repository
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, DbEnum, Clone)]
#[serde(rename_all = "snake_case")]
#[DieselType = "Git_repository_source_type"]
#[cfg_attr(feature = "openapi", derive(Component))]
pub enum GitRepositorySourceType {
  Github,
  Gitlab,
  Local,
}

/// Git repository are used to have project definition to deploy cargo
/// this structure ensure read and write entity in database
/// we also support git hooks such as create/delete branch
#[derive(
  Clone, Serialize, Deserialize, Insertable, Queryable, Identifiable,
)]
#[primary_key(name)]
#[table_name = "git_repositories"]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct GitRepositoryItem {
  pub(crate) name: String,
  pub(crate) url: String,
  pub(crate) default_branch: String,
  pub(crate) source: GitRepositorySourceType,
}

/// Partial Git repository
/// this structure ensure write entity in database
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct GitRepositoryPartial {
  pub(crate) url: String,
  pub(crate) name: String,
}

/// Structure used as query to build a git repository by branch name
#[derive(Debug, Serialize, Deserialize)]
pub struct GitRepositoryBuildQuery {
  pub(crate) branch: Option<String>,
}
