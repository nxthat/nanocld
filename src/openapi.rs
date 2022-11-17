use ntex::web;
#[cfg(feature = "dev")]
use serde_json::json;
#[cfg(feature = "dev")]
use utoipa::OpenApi;
#[cfg(feature = "dev")]
use crate::models::*;
#[cfg(feature = "dev")]
use crate::services::*;
#[cfg(feature = "dev")]
use crate::errors::ApiError;

#[cfg_attr(feature = "dev", derive(OpenApi))]
#[cfg_attr(feature = "dev", openapi(
  paths(
    // Namespace
    namespace::list_namespace,
    namespace::create_namespace,
    namespace::delete_namespace_by_name,
    namespace::inspect_namespace_by_name,

    // proxy template
    proxy_template::list_proxy_template,

    // Cargo
    cargo::list_cargo,
    cargo::create_cargo,
    cargo::delete_cargo_by_name,
    cargo::count_cargo,

    // Cluster
    cluster::list_cluster,
    cluster::count_cluster,
    cluster::create_cluster,
    cluster::delete_cluster_by_name,
    cluster::inspect_cluster_by_name,
    cluster::start_cluster_by_name,
    cluster::join_cargo_to_cluster,

    // Cluster variable
    cluster_variable::list_cluster_variable,
    cluster_variable::create_cluster_variable,
    cluster_variable::delete_cluster_variable,

    // Cluster network
    cluster_network::list_cluster_network,
    cluster_network::create_cluster_network,
    cluster_network::delete_cluster_network_by_name,
    cluster_network::inspect_cluster_network_by_name,
    cluster_network::count_cluster_network_by_namespace,
  ),
  components(
    schemas(ApiError),
    schemas(GenericDelete),
    schemas(GenericCount),

    // Proxy template
    schemas(ProxyTemplateItem),
    schemas(ProxyTemplateModes),

    // Namespace
    schemas(NamespaceItem),
    schemas(NamespacePartial),

    // Cargo
    schemas(CargoItem),
    schemas(CargoPartial),

    // Cluster
    schemas(ClusterItem),
    schemas(ClusterPartial),
    schemas(ClusterJoinBody),

    // Cluster variable
    schemas(ClusterVariableItem),
    schemas(ClusterVariablePartial),
    schemas(ClusterItemWithRelation),

    // Cluster network
    schemas(ClusterNetworkItem),
    schemas(ClusterNetworkPartial),
    // ClusterItemWithRelation,

    // Todo Docker network struct bindings
    // Network,
    // Ipam,
    // IpamConfig,
    // NetworkContainer,
  )
))]
#[cfg(feature = "dev")]
pub struct ApiDoc;

#[cfg(feature = "dev")]
pub fn to_json() -> String {
  ApiDoc::openapi().to_pretty_json().unwrap()
}

#[cfg(feature = "dev")]
#[web::get("/explorer/swagger.json")]
async fn get_api_specs() -> Result<web::HttpResponse, web::Error> {
  let api_spec = to_json();
  return Ok(
    web::HttpResponse::Ok()
      .header("Access-Control-Allow", "*")
      .content_type("application/json")
      .body(api_spec),
  );
}

pub fn ntex_config(_config: &mut web::ServiceConfig) {
  #[cfg(feature = "dev")]
  {
    _config.service(get_api_specs);
  }
}
