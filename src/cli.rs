use clap::{AppSettings, Parser};

use crate::models::NodeMode;

/// nanocl daemon
/// vms and containers manager at scale for intranet
#[derive(Debug, Clone, Parser)]
#[clap(
  about,
  version,
  global_setting = AppSettings::DeriveDisplayOrder,
)]
pub struct Cli {
  /// Only available if nanocld have been builded with feature openapi
  #[clap(long)]
  pub(crate) genopenapi: bool,
  /// Only install required services this have to be called after fresh installation
  #[clap(long)]
  pub(crate) install_components: bool,
  /// Daemon host to listen to you can use tcp:// and unix://
  /// [default: unix:///run/nanocl/nanocl.sock]
  #[clap(short = 'H', long = "--host")]
  pub(crate) hosts: Option<Vec<String>>,
  /// Docker daemon socket to connect
  /// [default: unix:///run/nanocl/docker.sock]
  #[clap(long)]
  pub(crate) docker_host: Option<String>,
  /// State directory
  /// [default: /var/lib/nanocl]
  #[clap(long)]
  pub(crate) state_dir: Option<String>,
  /// Config directory
  #[clap(long, default_value = "/etc/nanocl")]
  pub(crate) config_dir: String,
  /// Github user used to make request with identity
  #[clap(long)]
  pub(crate) github_user: Option<String>,
  /// Generated token for given github user
  #[clap(long)]
  pub(crate) github_token: Option<String>,
  /// Node mode
  #[clap(long, default_value = "master")]
  pub(crate) mode: NodeMode,
}
