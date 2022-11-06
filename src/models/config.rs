use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone)]
pub struct DaemonConfig {
  pub(crate) hosts: Vec<String>,
  pub(crate) state_dir: String,
  pub(crate) docker_host: String,
  pub(crate) github_user: String,
  pub(crate) github_token: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct DaemonConfigFile {
  pub(crate) hosts: Option<Vec<String>>,
  pub(crate) docker_host: Option<String>,
  pub(crate) state_dir: Option<String>,
  pub(crate) github_user: Option<String>,
  pub(crate) github_token: Option<String>,
}
