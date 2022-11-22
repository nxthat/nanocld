use ntex::rt;
use ntex::web;
use ntex::util::Bytes;
use futures::StreamExt;
use ntex::channel::mpsc;
use bollard::container::LogOutput;
use bollard::exec::{StartExecOptions, StartExecResults};

use crate::utils;
use crate::models::{CargoInstanceExecBody, CargoInstanceFilterQuery};
use crate::errors::HttpResponseError;

#[web::get("/cargoes/instances")]
async fn list_cargo_instance(
  web::types::Query(qs): web::types::Query<CargoInstanceFilterQuery>,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let containers =
    utils::cargo_instance::list_cargo_instance(qs, &docker_api).await?;
  Ok(web::HttpResponse::Ok().json(&containers))
}

#[web::post("/cargoes/instances/{name}/exec")]
async fn create_cargo_instance_exec(
  name: web::types::Path<String>,
  // mut stream: web::types::Payload,
  web::types::Json(body): web::types::Json<CargoInstanceExecBody>,
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

#[web::post("/cargoes/instances/exec/{id}/start")]
async fn start_cargo_instance_exec(
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
  config.service(list_cargo_instance);
  config.service(create_cargo_instance_exec);
  config.service(start_cargo_instance_exec);
}

#[cfg(test)]
mod tests {
  use bollard::exec::CreateExecResults;
  use bollard::service::ContainerSummary;

  use super::ntex_config;

  use crate::utils::test::*;
  use crate::models::{CargoInstanceFilterQuery, CargoInstanceExecBody};

  async fn list_cargo_instance(srv: &TestServer) -> TestReturn {
    let query = CargoInstanceFilterQuery {
      namespace: Some(String::from("system")),
      ..Default::default()
    };
    let mut resp = srv.get("/cargoes/instances").query(&query)?.send().await?;
    assert!(resp.status().is_success());
    let _: Vec<ContainerSummary> = resp.json().await?;

    Ok(())
  }

  async fn cargo_instance_exec(srv: &TestServer, name: &str) -> TestReturn {
    let exec = CargoInstanceExecBody {
      attach_stdin: Some(false),
      attach_stdout: Some(true),
      attach_stderr: Some(true),
      detach_keys: None,
      tty: Some(true),
      env: None,
      cmd: Some(vec![String::from("/usr/bin/ls")]),
      privileged: None,
      user: None,
      working_dir: None,
    };
    let mut resp = srv
      .post(format!("/cargoes/instances/{}/exec", &name))
      .send_json(&exec)
      .await?;
    assert!(resp.status().is_success());
    let resb: CreateExecResults = resp.json().await?;

    let resp = srv
      .post(format!("/cargoes/instances/exec/{}/start", &resb.id))
      .send()
      .await?;
    assert!(resp.status().is_success());
    Ok(())
  }

  #[ntex::test]
  async fn manipulate_cargo_instance() -> TestReturn {
    let srv = generate_server(ntex_config).await;
    list_cargo_instance(&srv).await?;
    cargo_instance_exec(&srv, "store").await?;
    Ok(())
  }
}
