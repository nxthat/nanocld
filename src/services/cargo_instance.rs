use ntex::rt;
use ntex::web;
use ntex::util::Bytes;
use ntex::channel::mpsc;
use futures::{stream, StreamExt};
use bollard::container::LogOutput;
use bollard::exec::{StartExecOptions, StartExecResults};

use crate::models::DaemonConfig;
use crate::{utils, repositories};
use crate::models::{
  Pool, GenericNspQuery, CargoInstancePath, ContainerExecBody,
  ContainerFilterQuery,
};
use crate::errors::HttpResponseError;
use crate::utils::cluster::JoinCargoOptions;

#[web::patch("/clusters/{cluster_name}/cargoes/{cargo_name}")]
async fn update_cargo_instance_by_name(
  req_path: web::types::Path<CargoInstancePath>,
  daemon_config: web::types::State<DaemonConfig>,
  pool: web::types::State<Pool>,
  docker_api: web::types::State<bollard::Docker>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let cluster_key =
    utils::key::gen_key_from_nsp(&qs.namespace, &req_path.cluster_name);
  let cargo_key =
    utils::key::gen_key_from_nsp(&qs.namespace, &req_path.cargo_name);

  let cluster_cargo = repositories::cargo_instance::get_by_key(
    format!("{}-{}", &cluster_key, &cargo_key),
    &pool,
  )
  .await?;

  let network = repositories::cluster_network::find_by_key(
    cluster_cargo.network_key,
    &pool,
  )
  .await?;

  let cluster =
    repositories::cluster::find_by_key(cluster_key.to_owned(), &pool).await?;
  let cargo =
    repositories::cargo::find_by_key(cargo_key.to_owned(), &pool).await?;
  let cnt_to_remove =
    utils::cluster::list_containers(&cluster_key, &cargo_key, &docker_api)
      .await?;

  let opts = JoinCargoOptions {
    cluster: cluster.to_owned(),
    cargo,
    network,
    is_creating_relation: false,
  };

  utils::cluster::join_cargo(&opts, &docker_api, &pool).await?;

  utils::cluster::start(&cluster, &daemon_config, &pool, &docker_api).await?;

  let mut stream = stream::iter(cnt_to_remove);

  while let Some(container) = stream.next().await {
    let options = Some(bollard::container::RemoveContainerOptions {
      force: true,
      ..Default::default()
    });
    docker_api
      .remove_container(&container.id.clone().unwrap_or_default(), options)
      .await?;
  }

  Ok(web::HttpResponse::Ok().into())
}

#[web::delete("/clusters/{cluster_name}/cargoes/{cargo_name}")]
async fn delete_cargo_instance_by_name(
  req_path: web::types::Path<CargoInstancePath>,
  pool: web::types::State<Pool>,
  docker_api: web::types::State<bollard::Docker>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let cluster_key =
    utils::key::gen_key_from_nsp(&qs.namespace, &req_path.cluster_name);
  let cargo_key =
    utils::key::gen_key_from_nsp(&qs.namespace, &req_path.cargo_name);

  let cargo_instance_key = utils::key::gen_key(&cluster_key, &cargo_key);

  log::info!("deleting cargo instance : {} ", &cargo_instance_key);
  // let cluster =
  //   repositories::cluster::find_by_key(cluster_key.to_owned(), &pool).await?;
  // let cargo =
  //   repositories::cargo::find_by_key(cargo_key.to_owned(), &pool).await?;
  repositories::cargo_instance::delete_by_key(cargo_instance_key, &pool)
    .await?;

  let res = repositories::cargo_instance::find_by_cargo_key(
    cargo_key.to_owned(),
    &pool,
  )
  .await?;

  println!("Instances : {:#?}", &res);
  let cnt_to_remove =
    utils::cluster::list_containers(&cluster_key, &cargo_key, &docker_api)
      .await?;

  let mut stream = stream::iter(cnt_to_remove);

  while let Some(container) = stream.next().await {
    let options = Some(bollard::container::RemoveContainerOptions {
      force: true,
      ..Default::default()
    });
    docker_api
      .remove_container(&container.id.clone().unwrap_or_default(), options)
      .await?;
  }

  log::info!("cargo instance deleted");

  Ok(web::HttpResponse::Ok().into())
}

#[web::get("/cargoes/instances")]
async fn list_cargo_instance(
  web::types::Query(qs): web::types::Query<ContainerFilterQuery>,
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
  config.service(delete_cargo_instance_by_name);
  config.service(update_cargo_instance_by_name);
  config.service(create_cargo_instance_exec);
  config.service(start_cargo_instance_exec);
}
