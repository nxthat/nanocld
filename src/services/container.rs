use ntex::rt;
use ntex::web;
use ntex::util::Bytes;
use ntex::channel::mpsc;
use futures::StreamExt;
use serde::{Serialize, Deserialize};

use bollard::container::LogOutput;
use bollard::exec::{StartExecOptions, StartExecResults};

use crate::utils;
use crate::errors::HttpResponseError;
use crate::models::{ContainerExecBody, ContainerFilterQuery};

#[web::get("/containers")]
async fn list_container(
  web::types::Query(qs): web::types::Query<ContainerFilterQuery>,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let containers = utils::container::list_container(qs, &docker_api).await?;
  Ok(web::HttpResponse::Ok().json(&containers))
}

#[web::post("/containers/{name}/exec")]
async fn create_exec(
  name: web::types::Path<String>,
  // mut stream: web::types::Payload,
  web::types::Json(body): web::types::Json<ContainerExecBody>,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<web::HttpResponse, HttpResponseError> {
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
  // Todo pipe this stream with stdio
  #[allow(unused_variables)] stream: web::types::Payload,
) -> Result<web::HttpResponse, HttpResponseError> {
  let res = docker_api
    .start_exec(&id.into_inner(), None::<StartExecOptions>)
    .await?;

  match res {
    StartExecResults::Attached {
      mut output,
      input: _,
    } => {
      let (tx, rx_body) = mpsc::channel();

      rt::spawn(async move {
        while let Some(output) = output.next().await {
          match output {
            Err(_err) => {
              log::error!("Todo catch error of exec stream.");
              break;
            }
            Ok(output) => match output {
              LogOutput::StdOut { message } => {
                if tx
                  .send(Ok::<_, web::error::Error>(Bytes::from(
                    message.to_vec(),
                  )))
                  .is_err()
                {
                  break;
                }
              }
              _ => log::debug!("todo exec command outputs"),
            },
          }
        }
        tx.close();
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
