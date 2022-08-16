use thiserror::Error;
use std::num::ParseIntError;

#[derive(Debug, Error)]
pub enum HypervisorError {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  ParseIntError(#[from] ParseIntError),
}
