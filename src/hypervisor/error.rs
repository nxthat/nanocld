use thiserror::Error;

#[derive(Debug, Error)]
pub enum HypervisorError {
  #[error(transparent)]
  Io(#[from] std::io::Error),
}
