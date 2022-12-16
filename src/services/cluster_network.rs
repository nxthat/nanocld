use ntex::web;

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
  let cluster_name = url_path.c_name.to_owned();
  let network_name = url_path.n_name.to_owned();
  let cluster_key = utils::key::gen_key_from_nsp(&qs.namespace, &cluster_name);
  let network_key = utils::key::gen_key(&cluster_key, &network_name);
  let res = utils::cluster_network::delete_network_by_key(
    network_key,
    &docker_api,
    &pool,
  )
  .await?;
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

/// Cluster network unit tests
#[cfg(test)]
pub mod tests {

  use ntex::http::StatusCode;

  use super::*;

  use crate::utils::tests::*;
  use crate::services::cluster;
  use crate::models::{ClusterNetworkItem, GenericDelete, GenericCount};

  /// Test utils function to list cluster networks
  pub async fn list(srv: &TestServer, cluster_name: &str) -> TestReqRet {
    srv
      .get(format!("/clusters/{cluster_name}/networks"))
      .send()
      .await
  }

  /// Test utils function to create a new cluster network
  pub async fn create(
    srv: &TestServer,
    cluster_name: &str,
    network: &ClusterNetworkPartial,
  ) -> TestReqRet {
    srv
      .post(format!("/clusters/{cluster_name}/networks"))
      .send_json(network)
      .await
  }

  /// Test utils function to count networks inside a namespace
  pub async fn count(srv: &TestServer) -> TestReqRet {
    srv.get("/networks/count").send().await
  }

  /// Test utils function to delete a cluster network by name
  pub async fn delete(
    srv: &TestServer,
    cluster_name: &str,
    network_name: &str,
  ) -> TestReqRet {
    srv
      .delete(format!("/clusters/{cluster_name}/networks/{network_name}"))
      .send()
      .await
  }

  /// Test utils function to inspect a cluster network by name
  pub async fn inspect(
    srv: &TestServer,
    cluster_name: &str,
    network_name: &str,
  ) -> TestReqRet {
    srv
      .get(format!(
        "/clusters/{cluster_name}/networks/{network_name}/inspect"
      ))
      .send()
      .await
  }

  /// Basic list when cluster a cluster doesn't exists that return a StatusCode::NOT_FOUND
  #[ntex::test]
  async fn basic_list_cluster_not_exists() -> TestRet {
    let srv = generate_server(ntex_config).await;
    let res = list(&srv, "test").await?;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    Ok(())
  }

  /// Basic network count inside a namespace
  #[ntex::test]
  async fn basic_count_namespace() -> TestRet {
    let srv = generate_server(ntex_config).await;
    let mut res = count(&srv).await?;
    assert_eq!(res.status(), StatusCode::OK);
    let _body: GenericCount = res.json().await?;
    Ok(())
  }

  /// Basic create when cluster doesn't exists that return a StatusCode::NOT_FOUND
  #[ntex::test]
  async fn basic_create_cluster_not_exists() -> TestRet {
    let srv = generate_server(ntex_config).await;
    let network = ClusterNetworkPartial {
      name: "test".to_string(),
    };
    let res = create(&srv, "test", &network).await?;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    Ok(())
  }

  /// Test to create, inspect and delete a cluster unit-test-net with a network test-net
  #[ntex::test]
  async fn basic_create_inspect_delete() -> TestRet {
    let srv_cluster = generate_server(cluster::ntex_config).await;
    let srv = generate_server(ntex_config).await;
    let cluster_name = "unit-test-net";
    let network_name = "test-net";

    // Create cluster
    let res = cluster::tests::create(&srv_cluster, cluster_name).await?;
    assert_eq!(
      res.status(),
      StatusCode::CREATED,
      "Expected status {} when creating a cluster {} but got {}",
      StatusCode::CREATED,
      cluster_name,
      res.status()
    );

    // Create network
    let network = ClusterNetworkPartial {
      name: network_name.to_string(),
    };
    let mut res = create(&srv, cluster_name, &network).await?;
    assert_eq!(res.status(), StatusCode::CREATED);
    let body: ClusterNetworkItem = res
      .json()
      .await
      .expect("Expect a valid cluster network json body when creating a cluster network");
    assert_eq!(
      body.name, network_name,
      "Expected network name {} but got {}",
      network_name, body.name
    );

    // Inspect network
    let mut res = inspect(&srv, cluster_name, network_name).await?;
    assert_eq!(res.status(), StatusCode::OK);
    let body: ClusterNetworkItem = res.json().await.expect("Expect a valid cluster network json body when inspecting a cluster network");
    assert_eq!(
      body.name, network_name,
      "Expected network name {} but got {}",
      network_name, body.name
    );

    // List cluster networks
    let mut res = list(&srv, cluster_name).await?;
    assert_eq!(res.status(), StatusCode::OK);
    let _body: Vec<ClusterNetworkItem> = res.json().await.expect(
      "Expect a valid cluster network json body when listing cluster networks",
    );

    // Delete network
    let mut res = delete(&srv, cluster_name, network_name).await?;
    assert_eq!(
      res.status(),
      StatusCode::OK,
      "Expected status {} when deleting a cluster network but got {}",
      StatusCode::OK,
      res.status()
    );

    let body: GenericDelete = res.json().await.expect(
      "Expect a generic delete json body when deleting a cluster network",
    );
    assert_eq!(
      body.count, 1,
      "Expected 1 deleted network but got {}",
      body.count
    );

    // Delete cluster
    let mut res = cluster::tests::delete(&srv_cluster, cluster_name).await?;
    assert_eq!(
      res.status(),
      StatusCode::OK,
      "Expected status {} when deleting a cluster but got {}",
      StatusCode::OK,
      res.status()
    );

    let _body: GenericDelete = res
      .json()
      .await
      .expect("Expect a generic delete json body when deleting a cluster");
    assert_eq!(
      _body.count, 1,
      "Expected 1 deleted cluster but got {}",
      _body.count
    );

    Ok(())
  }
}
