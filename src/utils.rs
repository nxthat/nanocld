use ntex::http::StatusCode;
use serde::Serialize;

use crate::errors::HttpResponseError;

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

#[cfg(test)]
pub mod test {
  use ntex::web::*;

  use crate::components;
  use crate::config::DaemonConfig;
  use crate::models::Pool;

  pub use ntex::web::test::TestServer;

  pub type TestReturn = Result<(), Box<dyn std::error::Error + 'static>>;

  type Config = fn(&mut ServiceConfig);

  pub fn gen_docker_client() -> bollard::Docker {
    bollard::Docker::connect_with_unix(
      "/run/docker.sock",
      120,
      bollard::API_DEFAULT_VERSION,
    )
    .unwrap()
  }

  pub async fn gen_postgre_pool() -> Pool {
    let docker = gen_docker_client();
    let ip_addr = components::postgresql::get_postgres_ip(&docker)
      .await
      .unwrap();

    components::postgresql::create_pool(ip_addr).await
  }

  pub async fn generate_server(config: Config) -> test::TestServer {
    let docker = gen_docker_client();

    let daemon_config = DaemonConfig {
      ..Default::default()
    };

    let pool = gen_postgre_pool().await;

    test::server(move || {
      App::new()
        .state(daemon_config.clone())
        .state(pool.clone())
        .state(docker.clone())
        .configure(config)
    })
  }
}
