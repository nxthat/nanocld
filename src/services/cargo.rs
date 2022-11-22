use ntex::web;
use ntex::http::StatusCode;
use futures::{stream, StreamExt};

use crate::models::DaemonConfig;
use crate::{repositories, utils};
use crate::models::{
  Pool, GenericNspQuery, CargoPartial, CargoEnvPartial, CargoItemWithRelation,
  CargoInstanceFilterQuery, CargoPatchPartial,
};

use crate::errors::HttpResponseError;

/// List cargo
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  path = "/cargoes",
  params(
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cargo are stored"),
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
  let nsp = utils::key::resolve_nsp(&qs.namespace);

  let nsp = repositories::namespace::find_by_name(nsp, &pool).await?;
  let items = repositories::cargo::find_by_namespace(nsp, &pool).await?;
  Ok(web::HttpResponse::Ok().json(&items))
}

/// Create new cargo
#[cfg_attr(feature = "dev", utoipa::path(
  post,
  request_body = CargoPartial,
  path = "/cargoes",
  params(
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cargo will be stored"),
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
  let nsp = utils::key::resolve_nsp(&qs.namespace);
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
#[cfg_attr(feature = "dev", utoipa::path(
  delete,
  path = "/cargoes/{name}",
  params(
    ("name" = String, Path, description = "Name of the cargo"),
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cargo is stored"),
  ),
  responses(
    (status = 200, description = "Generic delete", body = GenericDelete),
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
  let key = utils::key::gen_key_from_nsp(&qs.namespace, &name.into_inner());

  repositories::cargo::find_by_key(key.clone(), &pool).await?;
  repositories::cargo_instance::delete_by_cargo_key(key.to_owned(), &pool)
    .await?;
  let res = repositories::cargo::delete_by_key(key.to_owned(), &pool).await?;
  repositories::cargo_env::delete_by_cargo_key(key.to_owned(), &pool).await?;
  utils::cargo::delete_container(key.to_owned(), &docker_api).await?;
  Ok(web::HttpResponse::Ok().json(&res))
}

/// Count cargo
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  path = "/cargoes/count",
  params(
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cargo is stored"),
  ),
  responses(
    (status = 200, description = "Generic delete", body = GenericCount),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Namespace name not valid", body = ApiError),
  ),
))]
#[web::get("/cargoes/count")]
async fn count_cargo(
  pool: web::types::State<Pool>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let nsp = utils::key::resolve_nsp(&qs.namespace);
  let res = repositories::cargo::count(nsp, &pool).await?;

  Ok(web::HttpResponse::Ok().json(&res))
}

/// Inspect cargo by it's name
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  path = "/cargoes/{name}/inspect",
  params(
    ("name" = String, Path, description = "Name of the cargo"),
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cargo is stored"),
  ),
  responses(
    (status = 200, description = "Generic delete", body = GenericDelete),
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
  let key = utils::key::gen_key_from_nsp(&qs.namespace, &name);
  let res = repositories::cargo::find_by_key(key.to_owned(), &pool).await?;
  let qs = CargoInstanceFilterQuery {
    cargo: Some(name.to_owned()),
    cluster: None,
    namespace: qs.namespace.to_owned(),
  };
  let containers =
    utils::cargo_instance::list_cargo_instance(qs, &docker_api).await?;
  let environnements = if let Ok(envs) =
    repositories::cargo_env::list_by_cargo_key(key, &pool).await
  {
    Some(envs)
  } else {
    None
  };

  let cargo = CargoItemWithRelation {
    key: res.key,
    name: res.name,
    namespace_name: res.namespace_name,
    binds: res.binds,
    environnements,
    replicas: res.replicas,
    image_name: res.image_name,
    domainname: res.domainname,
    dns_entry: res.dns_entry,
    hostname: res.hostname,
    network_mode: res.network_mode,
    restart_policy: res.restart_policy,
    cap_add: res.cap_add,
    containers,
  };

  Ok(web::HttpResponse::Ok().json(&cargo))
}

#[web::patch("/cargoes/{name}")]
async fn patch_cargo_by_name(
  name: web::types::Path<String>,
  web::types::Json(mut payload): web::types::Json<CargoPatchPartial>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
  daemon_config: web::types::State<DaemonConfig>,
  pool: web::types::State<Pool>,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let name = name.into_inner();
  let nsp = utils::key::resolve_nsp(&qs.namespace);
  let key = utils::key::gen_key(&nsp, &name);

  let cargo = repositories::cargo::find_by_key(key.to_owned(), &pool).await?;

  // Add environement variables
  let mut env_stream =
    stream::iter(payload.environnements.to_owned().unwrap_or_default());
  while let Some(env) = env_stream.next().await {
    let arr = env.split('=').collect::<Vec<&str>>();
    if arr.len() != 2 {
      return Err(HttpResponseError {
        msg: format!("env variable formated incorrectly {}", &env),
        status: StatusCode::UNPROCESSABLE_ENTITY,
      });
    }
    let name = arr[0].to_owned();
    let value = arr[1].to_owned();
    let env = CargoEnvPartial {
      cargo_key: key.to_owned(),
      name: name.to_owned(),
      value: value.to_owned(),
    };
    let env_exists = repositories::cargo_env::exist_in_cargo(
      name.to_owned(),
      key.to_owned(),
      &pool,
    )
    .await?;
    if env_exists {
      // Update env variable if it exists
      repositories::cargo_env::patch_for_cargo(
        name.to_owned(),
        key.to_owned(),
        value,
        &pool,
      )
      .await?;
    } else {
      // Unless we create it
      repositories::cargo_env::create(env, &pool).await?;
    }
  }

  // Add binds
  let mut binds = cargo.binds.to_owned();
  if let Some(mut payload_binds) = payload.binds.to_owned() {
    binds.append(&mut payload_binds);
  }
  payload.binds = Some(binds);

  // Update entity
  let updated_cargo =
    repositories::cargo::update_by_key(nsp, name, payload, &pool).await?;

  // Update containers
  utils::cargo::update_containers(key, &daemon_config, &docker_api, &pool)
    .await?;
  Ok(web::HttpResponse::Accepted().json(&updated_cargo))
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(list_cargo);
  config.service(create_cargo);
  config.service(count_cargo);
  config.service(delete_cargo_by_name);
  config.service(inspect_cargo_by_name);
  config.service(patch_cargo_by_name);
}

#[cfg(test)]
mod tests {
  use ntex::http::StatusCode;

  use crate::utils::tests::*;
  use crate::services::cargo_image;
  use crate::models::{CargoPartial, CargoPatchPartial, CargoItem};

  use super::ntex_config;

  /// Ensure the cargo image exists for the test to run
  async fn ensure_test_image() -> TestRet {
    let srv = generate_server(cargo_image::ntex_config).await;
    cargo_image::tests::create_cargo_image(
      &srv,
      "nexthat/nanocl-get-started:latest",
    )
    .await?;
    Ok(())
  }

  /// Perform basic list against cargoes
  #[ntex::test]
  async fn basic_list() -> TestRet {
    let srv = generate_server(ntex_config).await;
    let resp = srv.get("/cargoes").send().await?;
    assert!(
      resp.status().is_success(),
      "Expect success while listing cargoes"
    );
    Ok(())
  }

  /// Perform count list against cargoes
  #[ntex::test]
  async fn basic_count() -> TestRet {
    let srv = generate_server(ntex_config).await;
    let resp = srv.get("/cargoes/count").send().await?;
    assert!(
      resp.status().is_success(),
      "Expect success while counting cargoes"
    );
    Ok(())
  }

  /// Perform CRUD Test against cargoes
  #[ntex::test]
  async fn crud() -> TestRet {
    ensure_test_image().await?;
    let srv = generate_server(ntex_config).await;

    // Create a new Cargo crud-test
    let new_cargo = CargoPartial {
      name: String::from("crud-test"),
      image_name: String::from("nexthat/nanocl-get-started"),
      ..Default::default()
    };
    let mut resp = srv.post("/cargoes").send_json(&new_cargo).await?;
    assert!(
      resp.status().is_success(),
      "Expect create new cargo to success with payload : {:#?}",
      new_cargo
    );
    let test_cargo: CargoItem = resp.json().await?;
    assert_eq!(
      test_cargo.name, new_cargo.name,
      "Expect create response name {} to match with payload name {}",
      test_cargo.name, new_cargo.name
    );

    // Inspect the created cargo
    let mut resp = srv
      .get(format!("/cargoes/{}/inspect", &test_cargo.name))
      .send()
      .await?;
    assert!(
      resp.status().is_success(),
      "Expect success while inspecting created cargo {}",
      test_cargo.name
    );
    let inspect_body: CargoItem = resp.json().await?;
    assert_eq!(
      inspect_body.key, test_cargo.key,
      "Expect inspect response key {} to match with create response key {}",
      inspect_body.key, test_cargo.key
    );

    // Patch the created cargo
    let cargo_patch_payload = CargoPatchPartial {
      domainname: Some(String::from("crud-test.internal")),
      ..Default::default()
    };
    let resp = srv
      .patch(format!("/cargoes/{}", &test_cargo.name))
      .send_json(&cargo_patch_payload)
      .await?;
    assert!(
      resp.status().is_success(),
      "Expect success while patching cargo domainname to {:?}",
      cargo_patch_payload.domainname
    );

    // Inspect the updated cargo
    let mut resp = srv
      .get(format!("/cargoes/{}/inspect", &test_cargo.name))
      .send()
      .await?;
    assert!(
      resp.status().is_success(),
      "Expect success while inspecting the updated cargo {}",
      test_cargo.name
    );
    let updated_cargo: CargoItem = resp.json().await?;

    assert_eq!(
      updated_cargo.domainname, cargo_patch_payload.domainname,
      "Expect updated cargo domainname {:?} to match {:?}",
      updated_cargo.domainname, cargo_patch_payload.domainname
    );

    // Delete crud-test cargo
    let resp = srv
      .delete(format!("/cargoes/{}", &test_cargo.name))
      .send()
      .await?;
    assert!(
      resp.status().is_success(),
      "Expect success while deleting cargo {}",
      test_cargo.name
    );

    // Inspect now just return a 404 not found
    let resp = srv
      .get(format!("/cargoes/{}", &test_cargo.name))
      .send()
      .await?;
    assert_eq!(
      resp.status(),
      StatusCode::NOT_FOUND,
      "Expect 404 not found while inspecting deleted cargo {}",
      test_cargo.name
    );

    Ok(())
  }
}
