use ntex::web;

use crate::services;
use crate::errors::HttpResponseError;
use crate::models::ContainerFilterQuery;

#[web::get("/containers")]
async fn list_containers(
  web::types::Query(qs): web::types::Query<ContainerFilterQuery>,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let containers = services::container::list_container(qs, &docker_api).await?;
  Ok(web::HttpResponse::Ok().json(&containers))
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(list_containers);
}
