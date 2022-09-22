use url::Url;
use std::{fs, path::Path, os::unix::prelude::FileExt, io::Read};
use serde::{Serialize, Deserialize};
use ntex::{
  web, rt,
  http::{StatusCode, Client},
};
use futures::{
  SinkExt, TryStreamExt,
  channel::mpsc::{UnboundedReceiver, unbounded},
};

use crate::errors::HttpResponseError;

#[derive(Debug, Serialize, Deserialize)]
pub enum DownloadFileStatus {
  Downloading,
  Syncing,
  Done,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadFileInfo {
  pub(crate) percent: f64,
  pub(crate) status: DownloadFileStatus,
}

impl Default for DownloadFileInfo {
  fn default() -> Self {
    Self {
      percent: 0.0,
      status: DownloadFileStatus::Downloading,
    }
  }
}

impl DownloadFileInfo {
  fn new(percent: f64, status: DownloadFileStatus) -> Self {
    Self { percent, status }
  }
}

pub struct DownloadFileRes {
  pub(crate) path: String,
  pub(crate) stream:
    UnboundedReceiver<Result<DownloadFileInfo, HttpResponseError>>,
}

/// Render a mustache template to string
pub fn render_template<T, D>(
  template: T,
  data: &D,
) -> Result<String, HttpResponseError>
where
  T: ToString,
  D: Serialize, {
  let compiled =
    mustache::compile_str(&template.to_string()).map_err(|err| {
      HttpResponseError {
        msg: format!("{}", err),
        status: StatusCode::INTERNAL_SERVER_ERROR,
      }
    })?;

  let result = compiled.render_to_string(&data).map_err(|err| {
    HttpResponseError {
      msg: format!("{}", err),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    }
  })?;

  Ok(result)
}

/// # Download file
/// Download a file over http protocol for given url in given directory
pub async fn download_file(
  url: &Url,
  download_dir: impl AsRef<Path>,
) -> Result<DownloadFileRes, HttpResponseError> {
  // ubuntu cloud server doesn't return any filename in headers so i use the path to dertermine the file name
  // a test should be made to see if the header containt filename to use it instead of the path
  let file_name = url
    .path_segments()
    .ok_or_else(|| {
      HttpResponseError {
        status: StatusCode::BAD_REQUEST,
        msg: String::from(
          "url have empty path cannot determine file name lol.",
        ),
      }
    })?
    .last()
    .ok_or_else(|| {
      HttpResponseError {
        status: StatusCode::BAD_REQUEST,
        msg: String::from(
          "url have empty path cannot determine file name lol.",
        ),
      }
    })?;
  let client = Client::new();
  let res = client.get(url.to_string()).send().await.map_err(|err| {
    HttpResponseError {
      status: StatusCode::BAD_REQUEST,
      msg: format!("Unable to get {:?} {:?}", &url, &err),
    }
  })?;
  let status = res.status();
  if !status.is_success() {
    return Err(HttpResponseError {
      status,
      msg: format!(
        "Unable to get {:?} got response e&rror {:?}",
        &url, &status
      ),
    });
  }
  let total_size = res
    .header("content-length")
    .ok_or_else(|| {
      HttpResponseError {
        status: StatusCode::BAD_REQUEST,
        msg: format!("Unable to download {:?} content-length not set.", &url),
      }
    })?
    .to_str()
    .map_err(|err| {
      HttpResponseError {
        status: StatusCode::BAD_REQUEST,
        msg: format!(
        "Unable to download {:?} cannot convert content-length got error {:?}",
        &url, &err
      ),
      }
    })?
    .parse::<u64>()
    .map_err(|err| {
      HttpResponseError {
        status: StatusCode::BAD_REQUEST,
        msg: format!(
        "Unable to download {:?} cannot convert content-length got error {:?}",
        &url, &err
      ),
      }
    })?;
  let url = url.to_owned();
  let file_path = Path::new(download_dir.as_ref()).join(file_name);
  let ret_file_path = file_name.to_owned();
  let (mut wtx, wrx) =
    unbounded::<Result<DownloadFileInfo, HttpResponseError>>();
  rt::spawn(async move {
    let mut stream = res.into_stream();
    let fp = file_path.to_owned();
    let file = web::block(move || {
      let file = fs::File::create(&fp).map_err(|err| {
        HttpResponseError {
          status: StatusCode::INTERNAL_SERVER_ERROR,
          msg: format!("Unable to create file {:?} got error {:?}", &fp, &err),
        }
      })?;
      Ok::<_, HttpResponseError>(file)
    })
    .await
    .map_err(|err| {
      HttpResponseError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!("{}", err),
      }
    })?;
    let mut offset: u64 = 0;
    while let Some(chunk) = stream.try_next().await.map_err(|err| {
      HttpResponseError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        msg: format!(
          "Unable to load stream from {:?} got error {:?}",
          &url, &err
        ),
      }
    })? {
      file.write_at(&chunk, offset).map_err(|err| {
        HttpResponseError {
          status: StatusCode::INTERNAL_SERVER_ERROR,
          msg: format!(
            "Unable to write in file {:?} got error {:?}",
            &file_path, &err
          ),
        }
      })?;
      offset += chunk.len() as u64;
      let percent = (offset as f64 / total_size as f64) * 100.0;
      log::debug!("Downloading file from {:?} status {:?}%", &url, &percent);
      let info =
        DownloadFileInfo::new(percent, DownloadFileStatus::Downloading);
      let send = wtx.send(Ok::<_, HttpResponseError>(info)).await;
      if let Err(_err) = send {
        break;
      }
    }

    if offset == total_size {
      file.sync_all().map_err(|err| {
        HttpResponseError {
          status: StatusCode::INTERNAL_SERVER_ERROR,
          msg: format!(
            "Unable to sync file {:?} got error {:?}",
            &file_path, &err
          ),
        }
      })?;
      let info = DownloadFileInfo::new(100.0, DownloadFileStatus::Done);
      let _send = wtx.send(Ok::<_, HttpResponseError>(info)).await;
    } else {
      fs::remove_file(&file_path).map_err(|err| {
        HttpResponseError {
          status: StatusCode::INTERNAL_SERVER_ERROR,
          msg: format!("Unable to delete created file {:?}", err),
        }
      })?;
    }
    Ok::<(), HttpResponseError>(())
  });
  let res = DownloadFileRes {
    path: ret_file_path,
    stream: wrx,
  };
  Ok(res)
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

pub fn generate_mac_addr() -> Result<String, HttpResponseError> {
  let mut mac: [u8; 6] = [0; 6];
  let mut urandom = fs::File::open("/dev/urandom").map_err(|err| {
    HttpResponseError {
      msg: format!(
        "Unable to open /dev/urandom to generate a mac addr {:?}",
        &err
      ),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    }
  })?;
  urandom.read_exact(&mut mac).map_err(|err| {
    HttpResponseError {
      msg: String::from("Unable to red /dev/urandom to generate a mac addr"),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    }
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
  use crate::components;
  use crate::config::DaemonConfig;
  use crate::models::Pool;

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
    let docker = gen_docker_client();
    let ip_addr = components::postgresql::get_postgres_ip(&docker)
      .await
      .unwrap();

    components::postgresql::create_pool(ip_addr).await
  }

  pub async fn generate_server(config: Config) -> test::TestServer {
    let docker = gen_docker_client();
    let daemon_config = DaemonConfig {
      state_dir: String::from("/var/lib/nanocl"),
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
