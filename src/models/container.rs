use serde::{Serialize, Deserialize};
#[cfg(feature = "openapi")]
use utoipa::Component;

/// Data use to filter containers by cluster cargo or namespace.
#[derive(Serialize, Deserialize)]
pub struct ContainerFilterQuery {
  pub(crate) cluster: Option<String>,
  pub(crate) cargo: Option<String>,
  pub(crate) namespace: Option<String>,
}
