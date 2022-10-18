use ntex::web;
use ntex::http::StatusCode;

use crate::{utils, repositories};
use crate::models::{
  Pool, GenericNspQuery, ClusterNetworkPartial, InspectClusterNetworkPath,
};
use crate::errors::HttpResponseError;

/// List network for given cluster
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  path = "/clusters/{c_name}/networks",
  params(
    ("c_name" = String, Path, description = "name of the cluster"),
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cluster is if empty we use 'default' as value"),
  ),
  responses(
    (status = 201, description = "List of networks", body = [ClusterNetworkItem]),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Namespace name not valid", body = ApiError),
  ),
))]
#[web::get("/clusters/{c_name}/networks")]
async fn list_cluster_network(
  pool: web::types::State<Pool>,
  c_name: web::types::Path<String>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let c_name = c_name.into_inner();
  let key = utils::key::gen_key_from_nsp(&qs.namespace, &c_name);
  let item = repositories::cluster::find_by_key(key, &pool).await?;
  let items =
    repositories::cluster_network::list_for_cluster(item, &pool).await?;
  Ok(web::HttpResponse::Ok().json(&items))
}

/// Create a network for given cluster
#[cfg_attr(feature = "dev", utoipa::path(
  post,
  request_body = ClusterNetworkPartial,
  path = "/clusters/{c_name}/networks",
  params(
    ("c_name" = String, Path, description = "name of the cluster"),
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cluster is if empty we use 'default' as value"),
  ),
  responses(
    (status = 201, description = "List of networks", body = ClusterNetworkItem),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Namespace name not valid", body = ApiError),
  ),
))]
#[web::post("/clusters/{c_name}/networks")]
async fn create_cluster_network(
  pool: web::types::State<Pool>,
  docker_api: web::types::State<bollard::Docker>,
  c_name: web::types::Path<String>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
  web::types::Json(payload): web::types::Json<ClusterNetworkPartial>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let c_name = c_name.into_inner();
  let nsp = utils::key::resolve_nsp(&qs.namespace);
  let key = utils::key::gen_key(&nsp, &c_name);
  // Verify if the repository exist
  // NOTE: Should use a is_exist method instead.
  repositories::cluster::find_by_key(key, &pool).await?;

  // Create the new network
  let new_network = utils::cluster_network::create_network(
    nsp,
    c_name,
    payload,
    &docker_api,
    &pool,
  )
  .await?;
  Ok(web::HttpResponse::Created().json(&new_network))
}

/// Inspect network by it's name for given cluster in given namespace
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  path = "/clusters/{c_name}/networks/{n_name}/inspect",
  params(
    ("c_name" = String, Path, description = "name of the cluster"),
    ("n_name" = String, Path, description = "name of the network"),
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cluster is if empty we use 'default' as value"),
  ),
  responses(
    (status = 200, description = "Network item", body = ClusterNetworkItem),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Namespace name not valid", body = ApiError),
  ),
))]
#[web::get("/clusters/{c_name}/networks/{n_name}/inspect")]
async fn inspect_cluster_network_by_name(
  pool: web::types::State<Pool>,
  url_path: web::types::Path<InspectClusterNetworkPath>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let c_name = url_path.c_name.to_owned();
  let n_name = url_path.n_name.to_owned();
  let nsp = utils::key::resolve_nsp(&qs.namespace);
  let gen_key = nsp + "-" + &c_name + "-" + &n_name;
  let network =
    repositories::cluster_network::find_by_key(gen_key, &pool).await?;
  Ok(web::HttpResponse::Ok().json(&network))
}

/// Delete network by it's name for given cluster in given namespace
#[cfg_attr(feature = "dev", utoipa::path(
  delete,
  path = "/clusters/{c_name}/networks/{n_name}",
  params(
    ("c_name" = String, Path, description = "name of the cluster"),
    ("n_name" = String, Path, description = "name of the network"),
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cluster is if empty we use 'default' as value"),
  ),
  responses(
    (status = 200, description = "Pg delete response", body = GenericDelete),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Cluster network not found", body = ApiError),
  ),
))]
#[web::delete("/clusters/{c_name}/networks/{n_name}")]
async fn delete_cluster_network_by_name(
  pool: web::types::State<Pool>,
  docker_api: web::types::State<bollard::Docker>,
  url_path: web::types::Path<InspectClusterNetworkPath>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let c_name = url_path.c_name.to_owned();
  let n_name = url_path.n_name.to_owned();
  let nsp = utils::key::resolve_nsp(&qs.namespace);
  let gen_key = nsp + "-" + &c_name + "-" + &n_name;
  let network =
    repositories::cluster_network::find_by_key(gen_key, &pool).await?;

  if let Err(err) = docker_api.remove_network(&network.docker_network_id).await
  {
    return Err(HttpResponseError {
      status: StatusCode::BAD_REQUEST,
      msg: format!("Unable to delete network {:?}", err),
    });
  }

  let res =
    repositories::cluster_network::delete_by_key(network.key, &pool).await?;
  Ok(web::HttpResponse::Ok().json(&res))
}

/// Count cluster networks
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  path = "/networks/count",
  params(
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cargo is stored"),
  ),
  responses(
    (status = 200, description = "Generic delete", body = GenericCount),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Namespace name not valid", body = ApiError),
  ),
))]
#[web::get("/networks/count")]
async fn count_cluster_network_by_namespace(
  pool: web::types::State<Pool>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let nsp = utils::key::resolve_nsp(&qs.namespace);
  let res =
    repositories::cluster_network::count_by_namespace(nsp, &pool).await?;

  Ok(web::HttpResponse::Ok().json(&res))
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(list_cluster_network);
  config.service(create_cluster_network);
  config.service(inspect_cluster_network_by_name);
  config.service(delete_cluster_network_by_name);
  config.service(count_cluster_network_by_namespace);
}
