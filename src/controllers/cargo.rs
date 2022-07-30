use ntex::web;
use ntex::http::StatusCode;

use crate::{services, repositories};
use crate::models::{
  Pool, GenericNspQuery, CargoPartial, CargoEnvPartial, CargoItemWithRelation,
  ContainerFilterQuery,
};

use crate::errors::HttpResponseError;

/// List cargo
#[cfg_attr(feature = "openapi", utoipa::path(
  get,
  path = "/cargoes",
  params(
    ("namespace" = Option<String>, query, description = "Name of the namespace where the cargo are stored"),
  ),
  responses(
    (status = 200, description = "List of cargo", body = [CargoItem]),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Namespace name not valid", body = ApiError),
  ),
))]
#[web::get("/cargoes")]
async fn list_cargo(
  pool: web::types::State<Pool>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let nsp = match qs.namespace {
    None => String::from("global"),
    Some(nsp) => nsp,
  };

  let nsp = repositories::namespace::find_by_name(nsp, &pool).await?;
  let items = repositories::cargo::find_by_namespace(nsp, &pool).await?;
  Ok(web::HttpResponse::Ok().json(&items))
}

/// Create new cargo
#[cfg_attr(feature = "openapi", utoipa::path(
  post,
  request_body = CargoPartial,
  path = "/cargoes",
  params(
    ("namespace" = Option<String>, query, description = "Name of the namespace where the cargo will be stored"),
  ),
  responses(
    (status = 201, description = "New cargo", body = CargoItem),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Namespace name not valid", body = ApiError),
  ),
))]
#[web::post("/cargoes")]
async fn create_cargo(
  pool: web::types::State<Pool>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
  web::types::Json(payload): web::types::Json<CargoPartial>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let nsp = match qs.namespace {
    None => String::from("global"),
    Some(nsp) => nsp,
  };
  log::info!(
    "creating cargo for namespace {} with payload {:?}",
    &nsp,
    payload,
  );
  let environnements = payload.environnements.to_owned();
  let item = repositories::cargo::create(nsp, payload, &pool).await?;
  if let Some(environnements) = environnements {
    let mut envs: Vec<CargoEnvPartial> = Vec::new();
    let cargo_envs = environnements
      .into_iter()
      .try_fold(&mut envs, |acc, env_item| {
        let splited = env_item.split('=').collect::<Vec<&str>>();
        if splited.len() != 2 {
          return Err(HttpResponseError {
            msg: format!("env item {} is not a valid format", env_item),
            status: StatusCode::BAD_REQUEST,
          });
        }
        let env = CargoEnvPartial {
          cargo_key: item.key.to_owned(),
          name: splited[0].into(),
          value: splited[1].into(),
        };
        acc.push(env);
        Ok::<&mut Vec<CargoEnvPartial>, HttpResponseError>(acc)
      })?
      .to_vec();
    repositories::cargo_env::create_many(cargo_envs, &pool).await?;
  }
  log::info!("cargo succefully created");
  Ok(web::HttpResponse::Created().json(&item))
}

/// Delete cargo by it's name
#[cfg_attr(feature = "openapi", utoipa::path(
  delete,
  path = "/cargoes/{name}",
  params(
    ("name" = String, path, description = "Name of the cargo"),
    ("namespace" = Option<String>, query, description = "Name of the namespace where the cargo is stored"),
  ),
  responses(
    (status = 200, description = "Generic delete", body = PgDeleteGeneric),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Namespace name not valid", body = ApiError),
  ),
))]
#[web::delete("/cargoes/{name}")]
async fn delete_cargo_by_name(
  pool: web::types::State<Pool>,
  docker_api: web::types::State<bollard::Docker>,
  name: web::types::Path<String>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  log::info!("asking cargo deletion {}", &name);
  let nsp = match qs.namespace {
    None => String::from("global"),
    Some(nsp) => nsp,
  };
  let gen_key = nsp + "-" + &name.into_inner();

  repositories::cargo::find_by_key(gen_key.clone(), &pool).await?;
  repositories::cluster_cargo::delete_by_cargo_key(gen_key.to_owned(), &pool)
    .await?;
  let res =
    repositories::cargo::delete_by_key(gen_key.to_owned(), &pool).await?;
  repositories::cargo_env::delete_by_cargo_key(gen_key.to_owned(), &pool)
    .await?;
  services::cargo::delete_container(gen_key.to_owned(), &docker_api).await?;
  Ok(web::HttpResponse::Ok().json(&res))
}

/// Count cargo
#[cfg_attr(feature = "openapi", utoipa::path(
  get,
  path = "/cargoes/count",
  params(
    ("namespace" = Option<String>, query, description = "Name of the namespace where the cargo is stored"),
  ),
  responses(
    (status = 200, description = "Generic delete", body = PgGenericCount),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Namespace name not valid", body = ApiError),
  ),
))]
#[web::get("/cargoes/count")]
async fn count_cargo(
  pool: web::types::State<Pool>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let nsp = match qs.namespace {
    None => String::from("global"),
    Some(nsp) => nsp,
  };
  let res = repositories::cargo::count(nsp, &pool).await?;

  Ok(web::HttpResponse::Ok().json(&res))
}

/// Inspect cargo by it's name
#[cfg_attr(feature = "openapi", utoipa::path(
  get,
  path = "/cargoes/{name}/inspect",
  params(
    ("name" = String, path, description = "Name of the cargo"),
    ("namespace" = Option<String>, query, description = "Name of the namespace where the cargo is stored"),
  ),
  responses(
    (status = 200, description = "Generic delete", body = PgDeleteGeneric),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Namespace name not valid", body = ApiError),
  ),
))]
#[web::get("/cargoes/{name}/inspect")]
async fn inspect_cargo_by_name(
  name: web::types::Path<String>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
  pool: web::types::State<Pool>,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let name = name.into_inner();
  log::info!("asking cargo inspection {}", &name);
  let nsp = match &qs.namespace {
    None => String::from("global"),
    Some(nsp) => nsp.to_owned(),
  };
  let gen_key = nsp + "-" + &name;
  let res = repositories::cargo::find_by_key(gen_key.to_owned(), &pool).await?;

  let qs = ContainerFilterQuery {
    cargo: Some(name.to_owned()),
    cluster: None,
    namespace: qs.namespace.to_owned(),
  };

  let containers = services::container::list_container(qs, &docker_api).await?;

  let cargo = CargoItemWithRelation {
    key: res.key,
    name: res.name,
    namespace_name: res.namespace_name,
    binds: res.binds,
    image_name: res.image_name,
    domainname: res.domainname,
    dns_entry: res.dns_entry,
    hostname: res.hostname,
    containers,
  };

  Ok(web::HttpResponse::Ok().json(&cargo))
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(list_cargo);
  config.service(create_cargo);
  config.service(count_cargo);
  config.service(delete_cargo_by_name);
  config.service(inspect_cargo_by_name);
}

#[cfg(test)]
mod test_cargo {
  use crate::utils::test::*;

  use super::ntex_config;

  #[ntex::test]
  async fn test_list() -> TestReturn {
    let srv = generate_server(ntex_config).await;
    let res = srv.get("/cargoes").send().await?;
    assert!(res.status().is_success());
    Ok(())
  }
}
