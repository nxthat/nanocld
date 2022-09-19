use std::path::Path;

use ntex::{web, rt};
use notify::{Watcher, RecursiveMode};
use futures::channel::mpsc::{unbounded, UnboundedReceiver};
use bollard::{
  Docker,
  models::HostConfig,
  errors::Error as DockerError,
  container::{CreateContainerOptions, Config},
  exec::{CreateExecOptions, StartExecOptions},
};

use crate::config::DaemonConfig;
use crate::models::{Pool, NginxLogItem};

use super::utils::*;

pub async fn reload_config(docker_api: &Docker) -> Result<(), DockerError> {
  let container_name = "nanocl-proxy-nginx";
  let config = CreateExecOptions {
    cmd: Some(vec!["nginx", "-s", "reload"]),
    attach_stdout: Some(true),
    attach_stderr: Some(true),
    ..Default::default()
  };
  let res = docker_api.create_exec(container_name, config).await?;
  let config = StartExecOptions {
    detach: false,
    ..Default::default()
  };
  docker_api.start_exec(&res.id, Some(config)).await?;

  Ok(())
}

fn gen_nginx_host_conf(config: &DaemonConfig) -> HostConfig {
  let sites_path = Path::new(&config.state_dir).join("nginx/sites-enabled");
  let stream_path = Path::new(&config.state_dir).join("nginx/streams-enabled");
  let log_path = Path::new(&config.state_dir).join("nginx/log");
  let ssl_path = Path::new(&config.state_dir).join("nginx/ssl");
  let sock_path = Path::new(&config.state_dir).join("socket");
  let letsencrypt_path = Path::new(&config.state_dir).join("nginx/letsencrypt");
  let binds = Some(vec![
    format!("{}:/opt/nanocl-socket", sock_path.display()),
    format!("{}:/etc/nginx/sites-enabled", sites_path.display()),
    format!("{}:/var/log/nginx", log_path.display()),
    format!("{}:/etc/nginx/ssl", ssl_path.display()),
    format!("{}:/etc/nginx/streams-enabled", stream_path.display()),
    format!("{}:/etc/letsencrypt", letsencrypt_path.display()),
  ]);
  let network_mode = Some(String::from("host"));
  HostConfig {
    binds,
    network_mode,
    ..Default::default()
  }
}

async fn create_nginx_container(
  name: &str,
  config: &DaemonConfig,
  docker_api: &Docker,
) -> Result<(), DockerError> {
  let image = Some("nanocl-proxy-nginx:latest");
  let labels = Some(gen_labels_with_namespace("nanocl"));
  let host_config = Some(gen_nginx_host_conf(config));
  let options = Some(CreateContainerOptions { name });
  let config = Config {
    image,
    labels,
    host_config,
    tty: Some(true),
    attach_stdout: Some(true),
    attach_stderr: Some(true),
    ..Default::default()
  };
  docker_api.create_container(options, config).await?;
  Ok(())
}

/// This function must disapear.
pub fn watch_nginx_logs(
  state_dir: String,
  pool: web::types::State<Pool>,
) -> UnboundedReceiver<NginxLogItem> {
  // Create a channel to receive the events.
  let (mut _tx, rx) = unbounded::<NginxLogItem>();
  // Create a watcher object, delivering raw events.
  // The notification back-end is selected based on the platform.
  rt::Arbiter::new().exec_fn(move || {
    rt::spawn(async move {
      // Add a path to be watched. All files and directories at that path and
      // below will be monitored for changes.
      let dir_path = Path::new(&state_dir).join("nginx/log");
      let mut watcher = notify::recommended_watcher(|res| {
        match res {
          Ok(event) => println!("nginx event: {:?}", event),
          Err(e) => log::error!("Received error event {}", e),
        }
      })
      .unwrap();
      // Add a path to be watched. All files and directories at that path and
      // below will be monitored for changes.
      watcher.watch(&dir_path, RecursiveMode::Recursive).unwrap();
    });
  });
  rx
}

pub async fn boot(
  config: &DaemonConfig,
  docker_api: &Docker,
) -> Result<(), DockerError> {
  let container_name = "nanocl-proxy-nginx";
  let s_state = get_component_state(container_name, docker_api).await;
  if s_state == ComponentState::Uninstalled {
    create_nginx_container(container_name, config, docker_api).await?;
  }
  if s_state != ComponentState::Running {
    if let Err(err) = start_component(container_name, docker_api).await {
      log::error!("error while starting {} {}", container_name, err);
    }
  }
  Ok(())
}
