use clap::Parser;

/// Nanocl daemon
/// Self Sufficient Hybrid Cloud Orchestrator
#[derive(Debug, Clone, Parser)]
#[command(name = "Nanocl")]
#[command(author = "nexthat team <team@next-hat.com>")]
#[command(version)]
pub struct Cli {
  /// Ensure state is inited
  #[clap(long)]
  pub(crate) init: bool,
  /// Daemon host to listen to you can use tcp:// and unix://
  /// [default: unix:///run/nanocl.sock]
  #[clap(short = 'H', long = "hosts")]
  pub(crate) hosts: Option<Vec<String>>,
  /// Docker daemon socket to connect
  /// [default: unix:///run/docker.sock]
  #[clap(long)]
  pub(crate) docker_host: Option<String>,
  /// State directory
  /// [default: /var/lib/nanocl]
  #[clap(long)]
  pub(crate) state_dir: Option<String>,
  /// Config directory
  #[clap(long, default_value = "/etc/nanocl")]
  pub(crate) config_dir: String,
}
