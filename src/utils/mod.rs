pub mod errors;

pub mod key;
pub mod cargo;
pub mod docker;
pub mod cluster;
pub mod cluster_network;
pub mod cargo_instance;
pub mod cluster_variable;

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

#[cfg(test)]
pub mod tests {
  use ntex::web::*;
  use ntex::http::client::ClientResponse;
  use ntex::http::client::error::SendRequestError;

  use std::env;
  use crate::controllers;
  use crate::models::{Pool, DaemonConfig};

  pub use ntex::web::test::TestServer;

  pub type TestRet = Result<(), Box<dyn std::error::Error + 'static>>;
  pub type TestReqRet = Result<ClientResponse, SendRequestError>;

  type Config = fn(&mut ServiceConfig);

  pub fn before() {
    // Build a test env logger
    let _ = env_logger::builder().is_test(true).try_init();
  }

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
    before();
    // Build a test daemon config
    let daemon_config = DaemonConfig {
      state_dir: String::from("/var/lib/nanocl"),
      ..Default::default()
    };
    // Create docker_api
    let docker_api = gen_docker_client();
    // Create postgres pool
    let pool = gen_postgre_pool().await;
    // Create test server
    test::server(move || {
      App::new()
        .state(daemon_config.clone())
        .state(pool.clone())
        .state(docker_api.clone())
        .configure(config)
    })
  }
}
