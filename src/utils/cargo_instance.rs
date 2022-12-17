use ntex::web;
use ntex::rt;
use ntex::http::StatusCode;
use ntex::util::Bytes;
use ntex::channel::mpsc;
use futures::StreamExt;
use bollard::Docker;
use bollard::container::LogOutput;
use bollard::exec::{
  CreateExecOptions, CreateExecResults, StartExecOptions, StartExecResults,
};

use crate::errors::HttpResponseError;
use crate::models::CargoInstanceExecBody;

/// Create cargo instance exec
pub async fn create_cargo_instance_exec(
  container_id_or_name: &str,
  params: &CargoInstanceExecBody,
  docker_api: &Docker,
) -> Result<CreateExecResults, HttpResponseError> {
  let config = CreateExecOptions::<String> {
    attach_stdin: params.attach_stdin,
    attach_stdout: params.attach_stdout,
    attach_stderr: params.attach_stderr,
    detach_keys: params.detach_keys.to_owned(),
    tty: params.tty,
    env: params.env.to_owned(),
    privileged: params.privileged,
    user: params.user.to_owned(),
    working_dir: params.working_dir.to_owned(),
    cmd: params.cmd.to_owned(),
  };
  let exec_instance =
    docker_api.create_exec(container_id_or_name, config).await?;
  Ok(exec_instance)
}

pub async fn exec_cargo_instance_exec(
  id: &str,
  docker_api: &Docker,
) -> Result<web::HttpResponse, HttpResponseError> {
  let res = docker_api.start_exec(&id, None::<StartExecOptions>).await?;

  match res {
    StartExecResults::Attached {
      mut output,
      input: _,
    } => {
      let (tx, rx_body) = mpsc::channel();

      rt::spawn(async move {
        while let Some(output) = output.next().await {
          match output {
            Err(err) => {
              let err = web::error::Error::new(HttpResponseError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                msg: err.to_string(),
              });
              let _ = tx.send(Err::<_, web::error::Error>(err));
              tx.close();
              break;
            }
            Ok(output) => match output {
              LogOutput::Console { message } => {
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
