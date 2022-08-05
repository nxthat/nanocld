//! nanocl daemon
//!
//! Provides an api to manage clusters network and containers
//! there are these advantages:
//! - Opensource
//! - [`Easy`]
//!
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use clap::Parser;
use errors::DaemonError;

mod cli;
mod boot;
mod events;
mod version;
mod components;

mod utils;
mod errors;
mod config;
mod install;
mod server;
mod schema;
mod models;
mod openapi;
mod services;
mod controllers;
mod repositories;

fn parse_main_error(
  #[allow(unused)] args: &cli::Cli,
  config: &config::DaemonConfig,
  err: errors::DaemonError,
) -> i32 {
  match err {
    DaemonError::Docker(err) => match err {
      bollard::errors::Error::HyperResponseError { err } => {
        if err.is_connect() {
          log::error!(
            "unable to connect to docker host {}",
            &config.docker_host,
          );
          return 1;
        }
        log::error!("{}", err);
        1
      }
      _ => {
        log::error!("{}", err);
        1
      }
    },
    _ => {
      log::error!("{}", err);
      1
    }
  }
}

/// nanocld is the daemon to manage your self hosted instranet
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

  // if we build with openapi feature
  // with args genopenapi we print the json on output
  // in order to generate a file with a pipe.
  #[cfg(feature = "openapi")]
  {
    if args.genopenapi {
      let result = openapi::to_json();
      println!("{}", result);
      std::process::exit(0);
    }
  }

  let file_config = match config::read_config_file(&args.config_dir) {
    Err(err) => {
      log::error!("{}", err);
      std::process::exit(1);
    }
    Ok(file_config) => file_config,
  };

  // Merge cli args and config file
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

  // Download, configure and boot internal services
  if args.install_components {
    if let Err(err) = install::install_components(&docker_api).await {
      let exit_code = parse_main_error(&args, &daemon_config, err);
      std::process::exit(exit_code);
    }
    if let Err(err) = boot::boot(&daemon_config, &docker_api).await {
      let exit_code = parse_main_error(&args, &daemon_config, err);
      std::process::exit(exit_code);
    };
    return Ok(());
  }

  // Start internal services
  let boot_state = match boot::boot(&daemon_config, &docker_api).await {
    Err(err) => {
      let exit_code = parse_main_error(&args, &daemon_config, err);
      std::process::exit(exit_code);
    }
    Ok(state) => state,
  };

  // Start background event_system
  let event_system = events::system::start(
    daemon_config.to_owned(),
    docker_api.to_owned(),
    boot_state.pool.to_owned(),
  )
  .await;

  // start ntex http server
  server::start(daemon_config, event_system, boot_state).await?;
  log::info!("kill received exiting.");
  Ok(())
}
