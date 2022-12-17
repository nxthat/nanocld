use ntex::web;

use crate::utils;
use crate::errors::HttpResponseError;
use crate::models::{CargoInstanceExecBody, CargoInstanceFilterQuery};

/// Endpoint to list existing cargo instances
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  path = "/cargoes/instances",
  params(
    ("namespace" = Option<String>, Query, description = "Namespace to search in"),
    ("cluster" = Option<String>, Query, description = "Cluster to search in"),
    ("cargo" = Option<String>, Query, description = "Cargo to search in"),
  ),
  responses(
    (status = 200, description = "List of installed images", body = [ContainerSummary]),
    (status = 400, description = "Generic database error", body = ApiError),
  ),
))]
#[web::get("/cargoes/instances")]
async fn list_cargo_instance(
  web::types::Query(qs): web::types::Query<CargoInstanceFilterQuery>,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let containers = utils::cargo::list_instances(qs, &docker_api).await?;
  Ok(web::HttpResponse::Ok().json(&containers))
}

/// Endpoint to create a cargo instance command to execute
#[cfg_attr(feature = "dev", utoipa::path(
  post,
  path = "/cargoes/instances/{name}/exec",
  request_body = CargoInstanceExecBody,
  params(
    ("name" = String, Path, description = "Name of the cargo instance to execute command on"),
  ),
  responses(
    (status = 200, description = "Create exec result with his id", body = CreateExecResults),
    (status = 400, description = "Generic database error", body = ApiError),
  ),
))]
#[web::post("/cargoes/instances/{name}/exec")]
async fn create_cargo_instance_exec(
  name: web::types::Path<String>,
  // mut stream: web::types::Payload,
  web::types::Json(body): web::types::Json<CargoInstanceExecBody>,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let res = utils::cargo_instance::create_cargo_instance_exec(
    &name,
    &body,
    &docker_api,
  )
  .await?;

  Ok(web::HttpResponse::Created().json(&res))
}

/// Endpoint to start a cargo instance command by it's id
#[cfg_attr(feature = "dev", utoipa::path(
  post,
  path = "/cargoes/instances/exec/{id}/start",
  params(
    ("id" = String, Path, description = "Exec instance id to start"),
  ),
  responses(
    (status = 200, description = "Stream of the output of the command", content_type = "nanocl/streaming-v1", body = String),
    (status = 400, description = "Generic database error", body = ApiError),
  ),
))]
#[web::post("/cargoes/instances/exec/{id}/start")]
async fn start_cargo_instance_exec(
  id: web::types::Path<String>,
  docker_api: web::types::State<bollard::Docker>,
  // Todo pipe this stream with stdio
  #[allow(unused_variables)] stream: web::types::Payload,
) -> Result<web::HttpResponse, HttpResponseError> {
  utils::cargo_instance::exec_cargo_instance_exec(&id, &docker_api).await
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(list_cargo_instance);
  config.service(create_cargo_instance_exec);
  config.service(start_cargo_instance_exec);
}

/// Cargo instances unit tests
#[cfg(test)]
mod tests {
  use super::*;

  use bollard::exec::CreateExecResults;
  use bollard::service::ContainerSummary;
  use ntex::http::StatusCode;

  use crate::utils::tests::*;

  /// Test utils to list cargo instances
  pub async fn list(srv: &TestServer, namespace: Option<String>) -> TestReqRet {
    let query = CargoInstanceFilterQuery {
      namespace,
      ..Default::default()
    };
    srv
      .get("/cargoes/instances")
      .query(&query)
      .expect("Expect to bind cargo instance request query")
      .send()
      .await
  }

  #[ntex::test]
  async fn list_namespace_system() -> TestRet {
    let srv = generate_server(ntex_config).await;
    let mut resp = list(&srv, Some(String::from("system"))).await?;
    assert_eq!(
      resp.status(),
      StatusCode::OK,
      "Expect test list to return {}, got {}",
      StatusCode::OK,
      resp.status()
    );
    let _: Vec<ContainerSummary> = resp
      .json()
      .await
      .expect("Expect list to return Vector of ContainerSummary");

    Ok(())
  }

  #[ntex::test]
  async fn exec_ls_in_store(srv: &TestServer) -> TestRet {
    let instance_name = "store";
    let srv = generate_server(ntex_config).await;
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
      .post(format!("/cargoes/instances/{}/exec", &instance_name))
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
}
