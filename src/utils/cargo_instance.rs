use bollard::container::StartContainerOptions;
use ntex::rt;
use ntex::web;
use ntex::http::StatusCode;
use ntex::util::Bytes;
use ntex::channel::mpsc;
use futures::StreamExt;
use bollard::Docker;
use bollard::errors::Error as DockerError;
use bollard::container::LogOutput;
use bollard::exec::{
  CreateExecOptions, CreateExecResults, StartExecOptions, StartExecResults,
};

use crate::errors::HttpResponseError;
use crate::models::CargoInstanceExecBody;
use crate::models::CargoInstanceState;

/// Create cargo instance exec
/// This function will create a exec instance for a container
///
/// ## Arguments
/// - [container_id_or_name](str) The id or name of the container
/// - [params](CargoInstanceExecBody) The exec body
/// - [docker_api](Docker) The docker api
///
/// ## Return
/// - [Result](CreateExecResults) The exec instance
/// - [Result](HttpResponseError) An http response error if something went wrong
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

/// Exec cargo instance
/// This function will exec a command in a container
///
/// ## Arguments
/// - [id](str) The id of the exec instance
/// - [docker_api](Docker) The docker api
///
/// ## Return
/// - [Result](web::HttpResponse) The response of the exec command
/// - [Result](HttpResponseError) An http response error if something went wrong
pub async fn exec_cargo_instance_exec(
  id: &str,
  docker_api: &Docker,
) -> Result<web::HttpResponse, HttpResponseError> {
  let res = docker_api.start_exec(id, None::<StartExecOptions>).await?;

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

/// ## Start a service
/// Start service by it's name
///
/// ## Arguments
/// - [name](str) name of the service to start
/// - [docker_api](Docker) bollard docker instance
///
/// ## Return
/// if sucess return nothing a [docker error](DockerError) is returned if an error occur
pub async fn start_cargo_instance(
  name: &str,
  docker_api: &Docker,
) -> Result<(), DockerError> {
  docker_api
    .start_container(name, None::<StartContainerOptions<String>>)
    .await?;
  Ok(())
}

/// ## Get service state
/// Get state of a service by his name
///
/// ## Arguments
/// - [name](str) name of the service
/// - [docker_api](Docker) bollard docker instance
///
/// ## Return
/// if success return [service state](ServiceState)
/// a [docker error](DockerError) is returned if an error occur
pub async fn get_cargo_instance_state(
  container_name: &'static str,
  docker_api: &Docker,
) -> CargoInstanceState {
  let resp = docker_api.inspect_container(container_name, None).await;
  if resp.is_err() {
    return CargoInstanceState::Uninstalled;
  }
  let body = resp.expect("ContainerInspectResponse");
  if let Some(state) = body.state {
    if let Some(running) = state.running {
      return if running {
        CargoInstanceState::Running
      } else {
        CargoInstanceState::Stopped
      };
    }
  }
  CargoInstanceState::Stopped
}

#[cfg(test)]
mod tests {
  use bollard::container::StopContainerOptions;

  use super::*;

  use crate::utils::tests::*;

  /// Test to get component state of the store container
  /// This should return ComponentState::Running
  #[ntex::test]
  async fn get_component_state_test() -> TestRet {
    let docker = gen_docker_client();
    let state = get_cargo_instance_state("store", &docker).await;
    assert_eq!(state, CargoInstanceState::Running);
    Ok(())
  }

  /// Test to get component state of a non existing container
  /// This should return ComponentState::Uninstalled
  #[ntex::test]
  async fn get_component_state_not_found_test() -> TestRet {
    let docker = gen_docker_client();
    let state =
      get_cargo_instance_state("non-existing-container", &docker).await;
    assert_eq!(state, CargoInstanceState::Uninstalled);
    Ok(())
  }

  /// Test to get component state of a stopped container
  /// This should return ComponentState::Stopped
  /// TODO: download a specific image before
  async fn _get_component_state_stopped_test() -> TestRet {
    let docker = gen_docker_client();

    // Stop system-nano-dns container
    docker
      .stop_container("system-nano-dns", None::<StopContainerOptions>)
      .await?;

    // Get the state of system-nano-dns container
    let state = get_cargo_instance_state("system-nano-dns", &docker).await;
    assert_eq!(state, CargoInstanceState::Stopped);

    // Start system-nano-dns container
    docker
      .start_container("system-nano-dns", None::<StartContainerOptions<String>>)
      .await?;
    Ok(())
  }
}
