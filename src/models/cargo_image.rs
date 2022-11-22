use serde::{Serialize, Deserialize};

#[cfg(feature = "dev")]
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct CargoImagePartial {
  pub(crate) name: String,
}
