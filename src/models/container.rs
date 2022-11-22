use serde::{Serialize, Deserialize};

#[cfg(feature = "dev")]
use utoipa::ToSchema;

/// Data use to filter containers by cluster cargo or namespace.
#[derive(Default, Serialize, Deserialize)]
pub struct CargoInstanceFilterQuery {
  pub(crate) cluster: Option<String>,
  pub(crate) cargo: Option<String>,
  pub(crate) namespace: Option<String>,
}

/// Structure used to create an exec instance inside a container
#[derive(Serialize, Deserialize)]
pub struct CargoInstanceExecBody {
  pub(crate) attach_stdin: Option<bool>,
  pub(crate) attach_stdout: Option<bool>,
  pub(crate) attach_stderr: Option<bool>,
  pub(crate) detach_keys: Option<String>,
  pub(crate) tty: Option<bool>,
  pub(crate) env: Option<Vec<String>>,
  pub(crate) cmd: Option<Vec<String>>,
  pub(crate) privileged: Option<bool>,
  pub(crate) user: Option<String>,
  pub(crate) working_dir: Option<String>,
}
