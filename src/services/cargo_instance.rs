use ntex::web;
use futures::{stream, StreamExt};

use crate::models::DaemonConfig;
use crate::{utils, repositories};
use crate::models::{Pool, GenericNspQuery, CargoInstancePath};
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

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(delete_cargo_instance_by_name);
  config.service(update_cargo_instance_by_name);
}
