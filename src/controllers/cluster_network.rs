use std::collections::HashMap;

use ntex::web;
use ntex::http::StatusCode;
use serde::{Serialize, Deserialize};

use crate::repositories;
use crate::models::{Pool, GenericNspQuery, ClusterNetworkPartial};
use crate::errors::HttpResponseError;

/// List network for given cluster
#[cfg_attr(feature = "openapi", utoipa::path(
  get,
  path = "/clusters/{c_name}/networks",
  params(
    ("c_name" = String, path, description = "name of the cluster"),
    ("namespace" = Option<String>, query, description = "Name of the namespace where the cluster is if empty we use 'default' as value"),
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
  let nsp = match qs.namespace {
    None => String::from("global"),
    Some(nsp) => nsp,
  };

  let gen_key = nsp + "-" + &c_name.into_inner();
  let item = repositories::cluster::find_by_key(gen_key, &pool).await?;
  let items =
    repositories::cluster_network::list_for_cluster(item, &pool).await?;

  Ok(web::HttpResponse::Ok().json(&items))
}

/// Create a network for given cluster
#[cfg_attr(feature = "openapi", utoipa::path(
  post,
  request_body = ClusterNetworkPartial,
  path = "/clusters/{c_name}/networks",
  params(
    ("c_name" = String, path, description = "name of the cluster"),
    ("namespace" = Option<String>, query, description = "Name of the namespace where the cluster is if empty we use 'default' as value"),
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
  let name = c_name.into_inner();
  let nsp = match qs.namespace {
    None => String::from("global"),
    Some(nsp) => nsp,
  };
  let gen_key = nsp.to_owned() + "-" + &name;
  let cluster =
    repositories::cluster::find_by_key(gen_key.clone(), &pool).await?;
  let mut labels = HashMap::new();
  labels.insert(String::from("cluster_key"), gen_key.clone());
  let gen_name = cluster.key.to_owned() + "-" + &payload.name;
  let network_existing =
    match repositories::cluster_network::find_by_key(gen_name.clone(), &pool)
      .await
    {
      Err(_) => false,
      Ok(_) => true,
    };
  if network_existing {
    return Err(HttpResponseError {
      status: StatusCode::CONFLICT,
      msg: format!("Unable to create network with name {} a similar network have same name", name),
    });
  }
  let config = bollard::network::CreateNetworkOptions {
    name: gen_name,
    driver: String::from("bridge"),
    labels,
    ..Default::default()
  };
  let id = match docker_api.create_network(config).await {
    Err(err) => {
      return Err(HttpResponseError {
        status: StatusCode::BAD_REQUEST,
        msg: format!("Unable to create network with name {} {}", name, err),
      })
    }
    Ok(result) => result.id,
  };
  let id = match id {
    None => {
      return Err(HttpResponseError {
        status: StatusCode::BAD_REQUEST,
        msg: format!("Unable to create network with name {}", name),
      })
    }
    Some(id) => id,
  };
  let network = docker_api
    .inspect_network(
      &id,
      None::<bollard::network::InspectNetworkOptions<String>>,
    )
    .await?;

  let ipam_config = network
    .ipam
    .ok_or(HttpResponseError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      msg: String::from("Unable to get ipam config from network"),
    })?
    .config
    .ok_or(HttpResponseError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      msg: String::from("Unable to get ipam config"),
    })?;

  let default_gateway = ipam_config
    .get(0)
    .ok_or(HttpResponseError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      msg: String::from("Unable to get ipam config"),
    })?
    .gateway
    .as_ref()
    .ok_or(HttpResponseError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      msg: String::from("Unable to get ipam config gateway"),
    })?;

  let new_network = repositories::cluster_network::create_for_cluster(
    nsp,
    name,
    payload,
    id,
    default_gateway.to_owned(),
    &pool,
  )
  .await?;
  Ok(web::HttpResponse::Created().json(&new_network))
}

#[derive(Serialize, Deserialize)]
struct InspectClusterNetworkPath {
  c_name: String,
  n_name: String,
}

/// Inspect network by it's name for given cluster in given namespace
#[cfg_attr(feature = "openapi", utoipa::path(
  get,
  path = "/clusters/{c_name}/networks/{n_name}/inspect",
  params(
    ("c_name" = String, path, description = "name of the cluster"),
    ("n_name" = String, path, description = "name of the network"),
    ("namespace" = Option<String>, query, description = "Name of the namespace where the cluster is if empty we use 'default' as value"),
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
  let nsp = match qs.namespace {
    None => String::from("global"),
    Some(nsp) => nsp,
  };
  let gen_key = nsp + "-" + &c_name + "-" + &n_name;
  let network =
    repositories::cluster_network::find_by_key(gen_key, &pool).await?;
  Ok(web::HttpResponse::Ok().json(&network))
}

/// Delete network by it's name for given cluster in given namespace
#[cfg_attr(feature = "openapi", utoipa::path(
  delete,
  path = "/clusters/{c_name}/networks/{n_name}",
  params(
    ("c_name" = String, path, description = "name of the cluster"),
    ("n_name" = String, path, description = "name of the network"),
    ("namespace" = Option<String>, query, description = "Name of the namespace where the cluster is if empty we use 'default' as value"),
  ),
  responses(
    (status = 200, description = "Pg delete response", body = PgDeleteGeneric),
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
  let nsp = match qs.namespace {
    None => String::from("global"),
    Some(nsp) => nsp,
  };
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

/// Count cluster
#[cfg_attr(feature = "openapi", utoipa::path(
  get,
  path = "/networks/count",
  params(
    ("namespace" = Option<String>, query, description = "Name of the namespace where the cargo is stored"),
  ),
  responses(
    (status = 200, description = "Generic delete", body = PgGenericCount),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Namespace name not valid", body = ApiError),
  ),
))]
#[web::get("/networks/count")]
async fn count_cluster_network_by_namespace(
  pool: web::types::State<Pool>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let nsp = match qs.namespace {
    None => String::from("global"),
    Some(nsp) => nsp,
  };
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
