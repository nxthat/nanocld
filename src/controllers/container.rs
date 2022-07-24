use bollard::container::LogOutput;
use futures::{StreamExt, SinkExt};
use futures::channel::mpsc::unbounded;
use ntex::channel::mpsc;
use ntex::http::StatusCode;
use ntex::rt;
use ntex::util::Bytes;
use ntex::web;
use serde::{Serialize, Deserialize};

use bollard::exec::{StartExecOptions, StartExecResults};

use crate::services;
use crate::errors::HttpResponseError;
use crate::models::ContainerFilterQuery;

#[derive(Serialize, Deserialize)]
pub struct ContainerExecQuery {
  pub(crate) attach_stdin: Option<bool>,
  pub(crate) attach_stdout: Option<bool>,
  pub(crate) attach_stderr: Option<bool>,
  pub(crate) detach_keys: Option<String>,
  pub(crate) tty: Option<bool>,
  pub(crate) env: Option<Vec<String>>,
  pub(crate) cmd: Option<Vec<String>>,
  pub(crate) privileged: Option<bool>,
  pub(crate) user: Option<String>,
  pub(crate) working_dir: Option<String>,
}

#[web::get("/containers")]
async fn list_container(
  web::types::Query(qs): web::types::Query<ContainerFilterQuery>,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let containers = services::container::list_container(qs, &docker_api).await?;
  Ok(web::HttpResponse::Ok().json(&containers))
}

#[web::post("/containers/{name}/exec")]
async fn create_exec(
  name: web::types::Path<String>,
  // mut stream: web::types::Payload,
  web::types::Json(body): web::types::Json<ContainerExecQuery>,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<web::HttpResponse, HttpResponseError> {
  println!("im called");
  let container_name = name.into_inner();
  let config = bollard::exec::CreateExecOptions::<String> {
    attach_stdin: body.attach_stdin,
    attach_stdout: body.attach_stdout,
    attach_stderr: body.attach_stderr,
    detach_keys: body.detach_keys,
    tty: body.tty,
    env: body.env,
    privileged: body.privileged,
    user: body.user,
    working_dir: body.working_dir,
    cmd: body.cmd,
  };
  let exec_instance = docker_api.create_exec(&container_name, config).await?;

  Ok(web::HttpResponse::Created().json(&exec_instance))
}

#[derive(Serialize, Deserialize)]
pub enum LogOutputStreamTypes {
  StdErr,
  StdIn,
  StdOut,
  Console,
}

#[derive(Serialize, Deserialize)]
pub struct LogOutputStream {
  types: LogOutputStreamTypes,
  message: String,
}

#[web::post("/exec/{id}/start")]
async fn start_exec(
  id: web::types::Path<String>,
  docker_api: web::types::State<bollard::Docker>,
  stream: web::types::Payload,
) -> Result<web::HttpResponse, HttpResponseError> {
  let res = docker_api
    .start_exec(&id.into_inner(), None::<StartExecOptions>)
    .await?;

  match res {
    StartExecResults::Attached { mut output, input } => {
      let (tx, rx_body) = mpsc::channel();

      rt::spawn(async move {
        while let Some(output) = output.next().await {
          match output {
            Err(_err) => {
              todo!("catch error of stream.");
            }
            Ok(output) => match output {
              LogOutput::StdErr { message } => {
                let data = &LogOutputStream {
                  types: LogOutputStreamTypes::StdOut,
                  message: String::from_utf8(message.to_vec()).unwrap(),
                };
                let payload = serde_json::to_vec(&data).unwrap();

                if tx
                  .send(Ok::<_, web::error::Error>(Bytes::from(
                    message.to_vec(),
                  )))
                  .is_err()
                {
                  break;
                }
              }
              LogOutput::StdOut { message } => {
                let data = &LogOutputStream {
                  types: LogOutputStreamTypes::StdOut,
                  message: String::from_utf8(message.to_vec()).unwrap(),
                };
                // let payload = serde_json::to_string(&data).unwrap();
                // println!("{}", serde_json::to_string_pretty(&data).unw);

                let payload = serde_json::to_vec(&data).unwrap();

                if tx
                  .send(Ok::<_, web::error::Error>(Bytes::from(
                    message.to_vec(),
                  )))
                  .is_err()
                {
                  break;
                }
              }
              LogOutput::StdIn { message } => {
                let data = &LogOutputStream {
                  types: LogOutputStreamTypes::StdIn,
                  message: String::from_utf8(message.to_vec()).unwrap(),
                };
                let payload = serde_json::to_vec(&data).unwrap();

                if tx
                  .send(Ok::<_, web::error::Error>(Bytes::from(
                    message.to_vec(),
                  )))
                  .is_err()
                {
                  break;
                }
              }
              LogOutput::Console { message } => {
                let data = &LogOutputStream {
                  types: LogOutputStreamTypes::StdIn,
                  message: String::from_utf8(message.to_vec()).unwrap(),
                };
                let payload = serde_json::to_vec(&data).unwrap();

                if tx
                  .send(Ok::<_, web::error::Error>(Bytes::from(
                    message.to_vec(),
                  )))
                  .is_err()
                {
                  break;
                }
              }
            },
          }
        }
      });
      Ok(web::HttpResponse::Ok().streaming(rx_body))
    }
    StartExecResults::Detached => Ok(web::HttpResponse::Ok().into()),
  }
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(list_container);
  config.service(create_exec);
  config.service(start_exec);
}
