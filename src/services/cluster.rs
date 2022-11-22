//! File to handle cluster routes
use ntex::web;
use futures::stream;
use futures::StreamExt;
use ntex::http::StatusCode;

use crate::models::DaemonConfig;
use crate::models::ClusterTemplatePartial;
use crate::models::DeleteClusterTemplatePath;
use crate::{utils, repositories};
use crate::utils::cluster::JoinCargoOptions;
use crate::models::{
  Pool, GenericNspQuery, ClusterJoinBody, ClusterPartial,
  ClusterItemWithRelation, CargoInstanceFilterQuery,
};

use crate::errors::HttpResponseError;

/// List all cluster
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  path = "/clusters",
  params(
    ("namespace" = Option<String>, Query, description = "Namespace to add cluster in if empty we use 'default' as value"),
  ),
  responses(
    (status = 200, description = "List of cluster for given namespace", body = ClusterItem),
    (status = 400, description = "Generic database error"),
    (status = 404, description = "Namespace name not valid"),
  ),
))]
#[web::get("/clusters")]
async fn list_cluster(
  pool: web::types::State<Pool>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let nsp = utils::key::resolve_nsp(&qs.namespace);
  let items = repositories::cluster::find_by_namespace(nsp, &pool).await?;
  Ok(web::HttpResponse::Ok().json(&items))
}

/// Create new cluster
#[cfg_attr(feature = "dev", utoipa::path(
  post,
  request_body = ClusterPartial,
  path = "/clusters",
  params(
    ("namespace" = Option<String>, Query, description = "Namespace to add cluster in if empty we use 'default' as value"),
  ),
  responses(
    (status = 201, description = "Fresh created cluster", body = ClusterItem),
    (status = 400, description = "Generic database error"),
    (status = 404, description = "Namespace name not valid"),
  ),
))]
#[web::post("/clusters")]
async fn create_cluster(
  pool: web::types::State<Pool>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
  web::types::Json(json): web::types::Json<ClusterPartial>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let nsp = utils::key::resolve_nsp(&qs.namespace);
  let res =
    repositories::cluster::create_for_namespace(nsp, json, &pool).await?;
  Ok(web::HttpResponse::Created().json(&res))
}

/// Delete cluster by it's name
#[cfg_attr(feature = "dev", utoipa::path(
  delete,
  path = "/clusters/{name}",
  params(
    ("name" = String, Path, description = "Name of the cluster"),
    ("namespace" = Option<String>, Query, description = "Namespace to add cluster in if empty we use 'default' as value"),
  ),
  responses(
    (status = 201, description = "Fresh created cluster", body = ClusterItem),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Namespace name not valid", body = ApiError),
  ),
))]
#[web::delete("clusters/{name}")]
async fn delete_cluster_by_name(
  pool: web::types::State<Pool>,
  docker_api: web::types::State<bollard::Docker>,
  name: web::types::Path<String>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let name = name.into_inner();
  let nsp = utils::key::resolve_nsp(&qs.namespace);
  let key = utils::key::gen_key(&nsp, &name);
  let item = repositories::cluster::find_by_key(key.to_owned(), &pool).await?;

  repositories::cluster_variable::delete_by_cluster_key(key.to_owned(), &pool)
    .await?;
  let qs = CargoInstanceFilterQuery {
    cluster: Some(name),
    namespace: Some(nsp),
    cargo: None,
  };

  repositories::cargo_instance::delete_by_cluster_key(key.to_owned(), &pool)
    .await?;
  let containers =
    utils::cargo_instance::list_cargo_instance(qs, &docker_api).await?;
  let mut stream = stream::iter(containers);
  while let Some(container) = stream.next().await {
    let options = bollard::container::RemoveContainerOptions {
      force: true,
      ..Default::default()
    };
    docker_api
      .remove_container(&container.id.unwrap(), Some(options))
      .await?;
  }

  utils::cluster::delete_networks(item, &docker_api, &pool).await?;
  log::debug!("deleting cluster cargo");
  let res = repositories::cluster::delete_by_key(key, &pool).await?;
  Ok(web::HttpResponse::Ok().json(&res))
}

/// Inspect cluster by it's name
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  path = "/clusters/{name}/inspect",
  params(
    ("name" = String, Path, description = "Name of the cluster"),
    ("namespace" = Option<String>, Query, description = "Namespace to add cluster in if empty we use 'default' as value"),
  ),
  responses(
    (status = 200, description = "Cluster information", body = ClusterItemWithRelation),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "id name or namespace name not valid", body = ApiError),
  ),
))]
#[web::get("/clusters/{name}/inspect")]
async fn inspect_cluster_by_name(
  pool: web::types::State<Pool>,
  name: web::types::Path<String>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let name = name.into_inner();
  let nsp = utils::key::resolve_nsp(&qs.namespace);
  let key = utils::key::gen_key(&nsp, &name);
  let item = repositories::cluster::find_by_key(key.to_owned(), &pool).await?;
  let proxy_templates = item.proxy_templates.to_owned();
  let networks =
    repositories::cluster_network::list_for_cluster(item, &pool).await?;

  let cargoes =
    repositories::cluster::list_cargo(key.to_owned(), &pool).await?;

  let variables =
    repositories::cluster::list_variable(key.to_owned(), &pool).await?;

  let res = ClusterItemWithRelation {
    name,
    key,
    namespace: nsp,
    proxy_templates,
    variables,
    cargoes: Some(cargoes),
    networks: Some(networks),
  };

  Ok(web::HttpResponse::Ok().json(&res))
}

/// Start all cargo inside cluster
#[cfg_attr(feature = "dev", utoipa::path(
  post,
  path = "/clusters/{name}/start",
  params(
    ("name" = String, Path, description = "Name of the cluster"),
    ("namespace" = Option<String>, Query, description = "Namespace to add cluster in if empty we use 'global' as value"),
  ),
  responses(
    (status = 200, description = "Cargos have been started"),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Cluster name of namespace invalid", body = ApiError),
  ),
))]
#[web::post("/clusters/{name}/start")]
async fn start_cluster_by_name(
  name: web::types::Path<String>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
  pool: web::types::State<Pool>,
  config: web::types::State<DaemonConfig>,
  docker_api: web::types::State<bollard::Docker>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let name = name.into_inner();
  let key = utils::key::gen_key_from_nsp(&qs.namespace, &name);
  let cluster = repositories::cluster::find_by_key(key, &pool).await?;
  utils::cluster::start(&cluster, &config, &pool, &docker_api).await?;
  Ok(web::HttpResponse::Ok().into())
}

/// join cargo inside a cluster
#[cfg_attr(feature = "dev", utoipa::path(
  post,
  path = "/clusters/{name}/join",
  request_body = ClusterJoinBody,
  params(
    ("name" = String, Path, description = "Name of the cluster"),
    ("namespace" = Option<String>, Query, description = "Namespace to add cluster in if empty we use 'global' as value"),
  ),
  responses(
    (status = 200, description = "Cargo joinned successfully"),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Cluster name of namespace invalid", body = ApiError),
  ),
))]
#[web::post("/clusters/{name}/join")]
async fn join_cargo_to_cluster(
  pool: web::types::State<Pool>,
  docker_api: web::types::State<bollard::Docker>,
  name: web::types::Path<String>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
  web::types::Json(payload): web::types::Json<ClusterJoinBody>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let name = name.into_inner();
  let nsp = utils::key::resolve_nsp(&qs.namespace);
  let cluster_key = utils::key::gen_key(&nsp, &name);
  let cargo_key = utils::key::gen_key(&nsp, &payload.cargo);

  if (repositories::cargo_instance::get_by_key(
    format!("{}-{}", &cluster_key, &cargo_key),
    &pool,
  )
  .await)
    .is_ok()
  {
    return Err(HttpResponseError {
      msg: format!(
        "Unable to join cargo {} to cluster {} in network {}, already exists",
        &payload.cargo, &name, &payload.network
      ),
      status: StatusCode::CONFLICT,
    });
  }

  let cluster = repositories::cluster::find_by_key(cluster_key, &pool).await?;
  let cargo = repositories::cargo::find_by_key(cargo_key, &pool).await?;
  let network_key = cluster.key.to_owned() + "-" + &payload.network;
  let network =
    repositories::cluster_network::find_by_key(network_key, &pool).await?;

  log::debug!(
    "joining cargo {:?} into cluster {:?}",
    cargo.key,
    cluster.key
  );
  let join_cargo_opts = JoinCargoOptions {
    cluster,
    cargo,
    network,
    is_creating_relation: true,
  };
  utils::cluster::join_cargo(&join_cargo_opts, &docker_api, &pool).await?;
  log::debug!("join success.");
  Ok(web::HttpResponse::Ok().into())
}

/// Count cluster
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  path = "/clusters/count",
  params(
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cargo is stored"),
  ),
  responses(
    (status = 200, description = "Generic delete", body = GenericCount),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Namespace name not valid", body = ApiError),
  ),
))]
#[web::get("/clusters/count")]
async fn count_cluster(
  pool: web::types::State<Pool>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let nsp = utils::key::resolve_nsp(&qs.namespace);
  let res = repositories::cluster::count(nsp, &pool).await?;

  Ok(web::HttpResponse::Ok().json(&res))
}

/// Add nginx template to cluster
#[web::post("/clusters/{name}/proxy/templates")]
async fn add_cluster_template(
  name: web::types::Path<String>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
  web::types::Json(payload): web::types::Json<ClusterTemplatePartial>,
  pool: web::types::State<Pool>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let key = utils::key::gen_key_from_nsp(&qs.namespace, &name.into_inner());
  repositories::proxy_template::get_by_name(payload.name.to_owned(), &pool)
    .await?;

  let cluster =
    repositories::cluster::find_by_key(key.to_owned(), &pool).await?;

  let mut proxy_templates = cluster.proxy_templates.to_owned();

  proxy_templates.push(payload.name);

  repositories::cluster::patch_proxy_templates(key, proxy_templates, &pool)
    .await?;

  Ok(web::HttpResponse::Created().into())
}

/// Delete nginx template to cluster
#[web::delete("/clusters/{cl_name}/proxy/templates/{nt_name}")]
async fn delete_cluster_template(
  req_path: web::types::Path<DeleteClusterTemplatePath>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
  pool: web::types::State<Pool>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let key = utils::key::gen_key_from_nsp(&qs.namespace, &req_path.cl_name);
  repositories::proxy_template::get_by_name(req_path.nt_name.to_owned(), &pool)
    .await?;

  let cluster =
    repositories::cluster::find_by_key(key.to_owned(), &pool).await?;

  let proxy_templates = cluster
    .proxy_templates
    .into_iter()
    .filter(|name| name != &req_path.nt_name)
    .collect::<Vec<String>>();

  repositories::cluster::patch_proxy_templates(key, proxy_templates, &pool)
    .await?;

  Ok(web::HttpResponse::Accepted().into())
}

/// # ntex config
/// Add cluster routes to ntex services
///
/// # Arguments
/// [config](web::ServiceConfig) mutable service config
///
/// # Examples
/// ```rust,norun
/// use ntex::web;
/// use crate::services;
///
/// web::App::new().configure(services::cluster::ntex_config)
/// ```
pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(list_cluster);
  config.service(create_cluster);
  config.service(inspect_cluster_by_name);
  config.service(delete_cluster_by_name);
  config.service(start_cluster_by_name);
  config.service(join_cargo_to_cluster);
  config.service(count_cluster);
  config.service(add_cluster_template);
  config.service(delete_cluster_template);
}

#[cfg(test)]
mod test_namespace_cluster {
  use crate::utils::test::*;

  use super::*;

  async fn test_list(srv: &TestServer) -> TestReturn {
    let resp = srv.get("/clusters").send().await?;

    assert!(resp.status().is_success());
    Ok(())
  }

  async fn test_list_with_nsp(srv: &TestServer) -> TestReturn {
    let resp = srv
      .get("/clusters")
      .query(&GenericNspQuery {
        namespace: Some(String::from("test")),
      })?
      .send()
      .await?;

    assert!(resp.status().is_success());
    Ok(())
  }

  async fn test_create(srv: &TestServer) -> TestReturn {
    let item = ClusterPartial {
      name: String::from("test_cluster"),
      proxy_templates: None,
    };
    let resp = srv.post("/clusters").send_json(&item).await?;

    assert!(resp.status().is_success());
    Ok(())
  }

  async fn test_delete(srv: &TestServer) -> TestReturn {
    let resp = srv.delete("/clusters/test_cluster").send().await?;
    assert!(resp.status().is_success());
    Ok(())
  }

  #[ntex::test]
  async fn main() -> TestReturn {
    let srv = generate_server(ntex_config).await;
    test_list(&srv).await?;
    test_list_with_nsp(&srv).await?;
    test_create(&srv).await?;
    test_delete(&srv).await?;
    Ok(())
  }
}
