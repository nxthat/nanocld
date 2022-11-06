pub mod errors;

pub mod key;
pub mod node;
pub mod cargo;
pub mod docker;
pub mod github;
pub mod cluster;
pub mod cluster_network;
pub mod container;
pub mod git_repository;
pub mod cluster_variable;

use std::{fs, io::Read};
use serde::Serialize;
use ntex::http::StatusCode;

use crate::errors::HttpResponseError;

/// Render a mustache template to string
pub fn render_template<T, D>(
  template: T,
  data: &D,
) -> Result<String, HttpResponseError>
where
  T: ToString,
  D: Serialize,
{
  let compiled =
    mustache::compile_str(&template.to_string()).map_err(|err| {
      HttpResponseError {
        msg: format!("{}", err),
        status: StatusCode::INTERNAL_SERVER_ERROR,
      }
    })?;

  let result =
    compiled
      .render_to_string(&data)
      .map_err(|err| HttpResponseError {
        msg: format!("{}", err),
        status: StatusCode::INTERNAL_SERVER_ERROR,
      })?;

  Ok(result)
}

// Should not be needed anymore
pub fn _get_free_port() -> Result<u16, HttpResponseError> {
  let socket = match std::net::UdpSocket::bind("127.0.0.1:0") {
    Err(err) => {
      return Err(HttpResponseError {
        msg: format!("unable to find a free port {:?}", err),
        status: StatusCode::INTERNAL_SERVER_ERROR,
      })
    }
    Ok(socket) => socket,
  };
  let port = match socket.local_addr() {
    Err(err) => {
      return Err(HttpResponseError {
        msg: format!("unable to find a free port {:?}", err),
        status: StatusCode::INTERNAL_SERVER_ERROR,
      })
    }
    Ok(local_addr) => local_addr.port(),
  };
  drop(socket);
  Ok(port)
}

// Should not be needed anymore
pub fn _generate_mac_addr() -> Result<String, HttpResponseError> {
  let mut mac: [u8; 6] = [0; 6];
  let mut urandom =
    fs::File::open("/dev/urandom").map_err(|err| HttpResponseError {
      msg: format!(
        "Unable to open /dev/urandom to generate a mac addr {:?}",
        &err
      ),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    })?;
  urandom
    .read_exact(&mut mac)
    .map_err(|err| HttpResponseError {
      msg: format!("Unable to read /dev/urandom to generate a mac addr ${err}"),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    })?;
  let mac_addr = format!(
    "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
  );

  Ok(mac_addr)
}

#[cfg(test)]
pub mod test {
  use ntex::web::*;

  use std::env;
  use crate::controllers;
  use crate::models::{Pool, DaemonConfig};

  pub use ntex::web::test::TestServer;

  pub type TestReturn = Result<(), Box<dyn std::error::Error + 'static>>;

  type Config = fn(&mut ServiceConfig);

  pub fn gen_docker_client() -> bollard::Docker {
    let socket_path = env::var("DOCKER_SOCKET_PATH")
      .unwrap_or_else(|_| String::from("/run/docker.sock"));
    bollard::Docker::connect_with_unix(
      &socket_path,
      120,
      bollard::API_DEFAULT_VERSION,
    )
    .unwrap()
  }

  pub async fn gen_postgre_pool() -> Pool {
    let docker_api = gen_docker_client();
    let ip_addr = controllers::store::get_store_ip_addr(&docker_api)
      .await
      .unwrap();

    controllers::store::create_pool(ip_addr).await
  }

  pub async fn generate_server(config: Config) -> test::TestServer {
    let docker_api = gen_docker_client();
    let daemon_config = DaemonConfig {
      state_dir: String::from("/var/lib/nanocl"),
      ..Default::default()
    };

    let pool = gen_postgre_pool().await;

    test::server(move || {
      App::new()
        .state(daemon_config.clone())
        .state(pool.clone())
        .state(docker_api.clone())
        .configure(config)
    })
  }
}
