use serde::{Serialize, Deserialize};
#[cfg(feature = "openapi")]
use utoipa::Component;

use crate::schema::cargoes;

use super::namespace::NamespaceItem;

/// Cargo partial
/// this structure ensure write in database
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct CargoPartial {
  pub(crate) name: String,
  pub(crate) image_name: String,
  pub(crate) environnements: Option<Vec<String>>,
  pub(crate) binds: Option<Vec<String>>,
  pub(crate) replicas: Option<i32>,
  pub(crate) dns_entry: Option<String>,
  pub(crate) domainname: Option<String>,
  pub(crate) hostname: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CargoPatchPartial {
  pub(crate) name: Option<String>,
  pub(crate) image_name: Option<String>,
  pub(crate) binds: Option<Vec<String>>,
  pub(crate) replicas: Option<i32>,
  pub(crate) dns_entry: Option<String>,
  pub(crate) domainname: Option<String>,
  pub(crate) hostname: Option<String>,
  pub(crate) environnements: Option<Vec<String>>,
}

#[derive(AsChangeset)]
#[table_name = "cargoes"]
pub struct CargoPatchItem {
  pub(crate) key: Option<String>,
  pub(crate) name: Option<String>,
  pub(crate) image_name: Option<String>,
  pub(crate) binds: Option<Vec<String>>,
  pub(crate) replicas: Option<i32>,
  pub(crate) dns_entry: Option<String>,
  pub(crate) domainname: Option<String>,
  pub(crate) hostname: Option<String>,
}

/// Cargo item is an definition to container create image and start them
/// this structure ensure read and write in database
#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  Queryable,
  Identifiable,
  Insertable,
  Associations,
  AsChangeset,
)]
#[primary_key(key)]
#[belongs_to(NamespaceItem, foreign_key = "namespace_name")]
#[table_name = "cargoes"]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct CargoItem {
  pub(crate) key: String,
  pub(crate) namespace_name: String,
  pub(crate) name: String,
  pub(crate) image_name: String,
  pub(crate) binds: Vec<String>,
  pub(crate) replicas: i32,
  pub(crate) dns_entry: Option<String>,
  pub(crate) domainname: Option<String>,
  pub(crate) hostname: Option<String>,
}

/// Cargo item with his relation
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct CargoItemWithRelation {
  pub(crate) key: String,
  pub(crate) namespace_name: String,
  pub(crate) name: String,
  pub(crate) image_name: String,
  pub(crate) binds: Vec<String>,
  pub(crate) replicas: i32,
  pub(crate) dns_entry: Option<String>,
  pub(crate) domainname: Option<String>,
  pub(crate) hostname: Option<String>,
  pub(crate) containers: Vec<bollard::models::ContainerSummary>,
}
