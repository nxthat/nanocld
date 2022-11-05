//! nanocl daemon
//!
//! Provides an api to manage clusters network and containers
//! there are these advantages:
//! - Opensource
//! - [`Easy`]
//!
#[macro_use]
extern crate diesel;

use clap::Parser;

mod cli;
mod version;
mod controllers;

mod utils;
mod errors;
mod config;
mod state;
mod server;
mod schema;
mod models;
mod openapi;
mod services;
mod repositories;

/// nanocld is the daemon to manage your containers
///
/// # Example
/// ```sh
/// nanocld --version
/// ```
#[ntex::main]
async fn main() -> std::io::Result<()> {
  // Parsing command line arguments
  let args = cli::Cli::parse();

  // Building env logger
  if std::env::var("LOG_LEVEL").is_err() {
    std::env::set_var("LOG_LEVEL", "nanocld=info,warn,error,nanocld=debug");
  }
  env_logger::Builder::new().parse_env("LOG_LEVEL").init();

  let file_config = match config::read_config_file(&args.config_dir) {
    Err(err) => {
      log::error!("{}", err);
      std::process::exit(1);
    }
    Ok(file_config) => file_config,
  };

  // Merge cli args and config file with priority to args
  let daemon_config: config::DaemonConfig =
    config::merge_config(&args, &file_config);

  // Connect to docker daemon
  let docker_api = match bollard::Docker::connect_with_unix(
    &daemon_config.docker_host,
    120,
    bollard::API_DEFAULT_VERSION,
  ) {
    Err(err) => {
      log::error!("{}", err);
      std::process::exit(1);
    }
    Ok(docker_api) => docker_api,
  };

  // Init internal dependencies
  let boot_state = match state::init(&daemon_config, &docker_api).await {
    Err(err) => {
      let exit_code = errors::parse_main_error(&args, &daemon_config, err);
      std::process::exit(exit_code);
    }
    Ok(state) => state,
  };

  // start ntex http server
  server::start(daemon_config, boot_state).await?;
  log::info!("shutdown");
  Ok(())
}
