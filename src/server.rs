use ntex::web;

use crate::openapi;
use crate::services;
use crate::boot::BootState;
use crate::config::DaemonConfig;

pub async fn start<'a>(
  config: DaemonConfig,
  boot_state: BootState,
) -> std::io::Result<()> {
  log::info!("Preparing server");
  let hosts = config.hosts.to_owned();
  let mut server = web::HttpServer::new(move || {
    web::App::new()
      // bind config state
      .state(config.clone())
      // bind postgre pool to state
      .state(boot_state.pool.clone())
      // bind docker api
      .state(boot_state.docker_api.clone())
      // Default logger middleware
      .wrap(web::middleware::Logger::default())
      // Set Json body max size
      .app_state(web::types::JsonConfig::default().limit(4096))
      // bind /explorer
      .configure(openapi::ntex_config)
      // bind controller system
      .configure(services::system::ntex_config)
      // bind controller namespace
      .configure(services::namespace::ntex_config)
      // bind controller git repository
      .configure(services::git_repository::ntex_config)
      // bind controller container_image
      .configure(services::container_image::ntex_config)
      // bind controller cluster
      .configure(services::cluster::ntex_config)
      // bind controller cluster variables
      .configure(services::cluster_variable::ntex_config)
      // bind controller cluster network
      .configure(services::cluster_network::ntex_config)
      // bind controller cluster cargo
      .configure(services::cargo_instance::ntex_config)
      // bind controller nginx template
      .configure(services::nginx_template::ntex_config)
      // bind controller container
      .configure(services::container::ntex_config)
      // bind controller cargo
      .configure(services::cargo::ntex_config)
  });
  let mut count = 0;
  let len = hosts.len();
  while count < len {
    let host = &hosts[count];
    if host.starts_with("unix://") {
      let addr = host.replace("unix://", "");
      server = match server.bind_uds(&addr) {
        Err(err) => {
          log::error!("Unable to bind server on {} got error {}", &addr, &err);
          std::process::exit(1);
        }
        Ok(server) => server,
      };
      log::info!("Listening on {}", &host);
    } else if host.starts_with("tcp://") {
      let addr = host.replace("tcp://", "");
      server = match server.bind(&addr) {
        Err(err) => {
          log::error!("Unable to bind server on {} got error {}", &addr, &err);
          std::process::exit(1);
        }
        Ok(server) => server,
      };
      log::info!("Listening on {}", &host);
    } else {
      log::warn!(
        "Warning {} is not valid use tcp:// or unix:// as protocol",
        host
      );
    }
    count += 1;
  }
  #[cfg(debug_assertions)]
  {
    server = server.bind("0.0.0.0:8383")?;
    log::info!("Listening on http://0.0.0.0:8383");
  }
  log::info!("Server ready");
  server.run().await
}
