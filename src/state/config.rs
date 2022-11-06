use crate::cli::Cli;
use crate::errors::DaemonError;
use crate::models::{DaemonConfig, DaemonConfigFile};

fn merge_config(args: &Cli, config: &DaemonConfigFile) -> DaemonConfig {
  let hosts = if let Some(ref hosts) = args.hosts {
    hosts.to_owned()
  } else if let Some(ref hosts) = config.hosts {
    hosts.to_owned()
  } else {
    vec![String::from("unix:///run/nanocl/nanocl.sock")]
  };

  let state_dir = if let Some(ref state_dir) = args.state_dir {
    state_dir.to_owned()
  } else if let Some(ref state_dir) = config.state_dir {
    state_dir.to_owned()
  } else {
    String::from("/var/lib/nanocl")
  };

  let docker_host = if let Some(ref docker_host) = args.docker_host {
    docker_host.to_owned()
  } else if let Some(ref docker_host) = config.docker_host {
    docker_host.to_owned()
  } else {
    String::from("/run/nanocl/docker.sock")
  };

  let github_user = if let Some(ref github_user) = args.github_user {
    github_user.to_owned()
  } else if let Some(ref github_user) = config.github_user {
    github_user.to_owned()
  } else {
    String::default()
  };

  let github_token = if let Some(ref github_token) = args.github_token {
    github_token.to_owned()
  } else if let Some(ref github_token) = config.github_token {
    github_token.to_owned()
  } else {
    String::default()
  };

  DaemonConfig {
    hosts,
    state_dir,
    docker_host,
    github_user,
    github_token,
  }
}

fn read_config_file(
  config_dir: &String,
) -> Result<DaemonConfigFile, DaemonError> {
  let config_path = std::path::Path::new(&config_dir).join("nanocl.conf");

  if !config_path.exists() {
    return Ok(DaemonConfigFile::default());
  }

  let content = std::fs::read_to_string(&config_path)?;
  let config = serde_yaml::from_str::<DaemonConfigFile>(&content)?;

  Ok(config)
}

/// Init Daemon config
/// It will read /etc/nanocl/nanocl.conf
/// and parse Cli arguments we merge them together with a priority to the config file
pub fn init(args: &Cli) -> Result<DaemonConfig, DaemonError> {
  let file_config = match read_config_file(&args.config_dir) {
    Err(err) => {
      log::error!("{}", err);
      std::process::exit(1);
    }
    Ok(file_config) => file_config,
  };

  // Merge cli args and config file with priority to args
  Ok(merge_config(&args, &file_config))
}
