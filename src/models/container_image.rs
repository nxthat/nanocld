use serde::{Serialize, Deserialize};

#[cfg(feature = "openapi")]
use utoipa::Component;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct ContainerImagePartial {
  pub(crate) name: String,
}
