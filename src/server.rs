// use std::fs::File;
// use std::io::BufReader;

use ntex::web;
// use rustls::RootCertStore;
// use rustls::server::AllowAnyAuthenticatedClient;
// use rustls::{Certificate, PrivateKey, ServerConfig};
// use rustls_pemfile::{certs, pkcs8_private_keys};

use crate::services;
use crate::models::DaemonState;

// fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
//   let certfile = File::open(filename).expect("cannot open certificate file");
//   let mut reader = BufReader::new(certfile);
//   certs(&mut reader)
//     .unwrap()
//     .iter()
//     .map(|v| Certificate(v.clone()))
//     .collect()
// }

pub async fn start<'a>(daemon_state: DaemonState) -> std::io::Result<()> {
  // load ssl keys

  // let key_file = &mut BufReader::new(
  //   File::open("/home/leone/ssl_test/certs/node.key").unwrap(),
  // );
  // let key = PrivateKey(pkcs8_private_keys(key_file).unwrap().remove(0));
  // let cert_chain = load_certs("/home/leone/ssl_test/certs/node.crt");

  // let roots = load_certs("/home/leone/ssl_test/certs/ca.crt");
  // let mut client_auth_roots = RootCertStore::empty();
  // for root in roots {
  //   println!("{:#?}", &root);
  //   client_auth_roots.add(&root).unwrap();
  // }

  // let server_config = ServerConfig::builder()
  //   .with_cipher_suites(rustls::ALL_CIPHER_SUITES)
  //   .with_safe_default_kx_groups()
  //   .with_protocol_versions(rustls::ALL_VERSIONS)
  //   .expect("inconsistent cipher-suites/versions specified")
  //   .with_client_cert_verifier(AllowAnyAuthenticatedClient::new(
  //     client_auth_roots,
  //   ))
  //   .with_single_cert(cert_chain, key)
  //   .unwrap();

  log::info!("Preparing server");
  let hosts = daemon_state.config.hosts.to_owned();
  let mut server = web::HttpServer::new(move || {
    // App need to be mutable when feature dev is enabled
    #[allow(unused_mut)]
    let mut app = web::App::new()
      // bind config state
      .state(daemon_state.config.clone())
      // bind postgre pool to state
      .state(daemon_state.pool.clone())
      // bind docker api
      .state(daemon_state.docker_api.clone())
      // Default logger middleware
      .wrap(web::middleware::Logger::default())
      // Set Json body max size
      .app_state(web::types::JsonConfig::default().limit(4096))
      // configure system service
      .configure(services::system::ntex_config)
      // configure namespace service
      .configure(services::namespace::ntex_config)
      // configure cargo image service
      .configure(services::cargo_image::ntex_config)
      // configure cluster service
      .configure(services::cluster::ntex_config)
      // configure cluster variables service
      .configure(services::cluster_variable::ntex_config)
      // configure cluster network service
      .configure(services::cluster_network::ntex_config)
      // configure cargo instance service
      .configure(services::cargo_instance::ntex_config)
      // configure nginx template service
      .configure(services::proxy_template::ntex_config)
      // configure cargo service
      .configure(services::cargo::ntex_config);

    // configure openapi if dev feature is enabled
    #[cfg(feature = "dev")]
    {
      use crate::openapi;
      app = app.configure(openapi::ntex_config);
    }

    app
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
  // server = server.bind_rustls("0.0.0.0:8443", server_config)?;
  log::info!("Server ready");
  server.run().await
}
