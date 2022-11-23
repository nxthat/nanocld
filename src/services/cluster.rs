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

/// Add proxy template to cluster
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

/// Delete proxy template to cluster
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

  Ok(web::HttpResponse::Ok().into())
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(list_cluster);
  config.service(count_cluster);
  config.service(create_cluster);
  config.service(inspect_cluster_by_name);
  config.service(start_cluster_by_name);
  config.service(join_cargo_to_cluster);
  config.service(add_cluster_template);
  config.service(delete_cluster_template);
  config.service(delete_cluster_by_name);
}

/// Cluster unit tests
#[cfg(test)]
pub mod tests {
  use super::*;

  use crate::utils::tests::*;
  use crate::services::{cargo, cargo_image, cluster_network, proxy_template};
  use crate::models::{
    ClusterItem, GenericCount, CargoPartial, ClusterNetworkPartial,
    ClusterNetworkItem, ProxyTemplateItem, ProxyTemplateModes,
  };

  /// Test utils to list clusters
  pub async fn list(srv: &TestServer, namespace: Option<String>) -> TestReqRet {
    let query = &GenericNspQuery { namespace };
    srv
      .get("/clusters")
      .query(query)
      .expect(&format!("List cluster with query {:#?}", query))
      .send()
      .await
  }

  /// Test utils to count clusters
  pub async fn count(
    srv: &TestServer,
    namespace: Option<String>,
  ) -> TestReqRet {
    let query = &GenericNspQuery { namespace };
    srv
      .get("/clusters/count")
      .query(&query)
      .expect("Expect to bind count query")
      .send()
      .await
  }

  /// Test utils to create cluster
  pub async fn create(srv: &TestServer, name: &str) -> TestReqRet {
    let item = ClusterPartial {
      name: name.to_owned(),
      proxy_templates: None,
    };
    srv.post("/clusters").send_json(&item).await
  }

  /// Test utils to delete a cluster
  pub async fn delete(srv: &TestServer, name: &str) -> TestReqRet {
    srv.delete(format!("/clusters/{}", name)).send().await
  }

  /// Test utils to inspect a cluster
  pub async fn inspect(srv: &TestServer, name: &str) -> TestReqRet {
    srv.get(format!("/clusters/{}/inspect", name)).send().await
  }

  /// Test utils to start a cluster
  pub async fn start(srv: &TestServer, name: &str) -> TestReqRet {
    srv.post(format!("/clusters/{}/start", name)).send().await
  }

  /// Test utils to join a cargo to a cluster
  pub async fn join_cargo(
    srv: &TestServer,
    name: &str,
    payload: &ClusterJoinBody,
  ) -> TestReqRet {
    srv
      .post(format!("/clusters/{}/join", name))
      .send_json(&payload)
      .await
  }

  /// Test utils to add a nginx template to a cluster
  pub async fn add_template(
    srv: &TestServer,
    name: &str,
    template_name: &str,
  ) -> TestReqRet {
    let item = ClusterTemplatePartial {
      name: template_name.to_owned(),
    };
    srv
      .post(format!("/clusters/{}/proxy/templates", name))
      .send_json(&item)
      .await
  }

  /// Test utils to remove a nginx template to a cluster
  pub async fn remove_template(
    srv: &TestServer,
    name: &str,
    template_name: &str,
  ) -> TestReqRet {
    srv
      .delete(format!(
        "/clusters/{}/proxy/templates/{}",
        name, template_name
      ))
      .send()
      .await
  }

  /// Basic test to list clusters
  #[ntex::test]
  async fn basic_list() -> TestRet {
    let srv = generate_server(ntex_config).await;
    let mut res = list(&srv, None).await?;
    assert_eq!(
      res.status(),
      StatusCode::OK,
      "Expect list with namespace system to return with status {}, got {}",
      StatusCode::OK,
      res.status()
    );
    let _body: Vec<ClusterItem> = res.json().await.expect("Expect list with namespace system to return with a Vector of ClusterItem as json data");
    Ok(())
  }

  /// Test list cluster with namespace system
  #[ntex::test]
  async fn list_with_namespace_system() -> TestRet {
    let srv = generate_server(ntex_config).await;
    let mut res = list(&srv, Some(String::from("system"))).await?;
    assert_eq!(
      res.status(),
      StatusCode::OK,
      "Expect list with namespace system to return with status {}, got {}",
      StatusCode::OK,
      res.status()
    );
    let _body: Vec<ClusterItem> = res.json().await.expect("Expect list with namespace system to return with a Vector of ClusterItem as json data");
    Ok(())
  }

  /// Basic test to count clusters
  #[ntex::test]
  async fn basic_count() -> TestRet {
    let srv = generate_server(ntex_config).await;
    let mut res = count(&srv, None).await?;
    assert_eq!(
      res.status(),
      StatusCode::OK,
      "Expect count with namespace system to return with status {}, got {}",
      StatusCode::OK,
      res.status()
    );
    let _body: GenericCount = res.json().await.expect(
      "Expect count with namespace system to return with a GenericCount as json data",
    );
    Ok(())
  }

  /// Test join cargo to cluster
  #[ntex::test]
  async fn basic_join() -> TestRet {
    cargo_image::tests::ensure_test_image().await?;
    let srv = generate_server(ntex_config).await;
    let cargo_srv = generate_server(cargo::ntex_config).await;
    let network_srv = generate_server(cluster_network::ntex_config).await;
    let proxy_srv = generate_server(proxy_template::ntex_config).await;
    // Create cluster
    let cluster_name = "utcj";
    let res = create(&srv, cluster_name).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::CREATED,
      "Expect create cluster to return with status {}, got {}",
      StatusCode::CREATED,
      status
    );

    // Inspect cluster
    let res = inspect(&srv, cluster_name).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::OK,
      "Expect inspect cluster to return with status {}, got {}",
      StatusCode::OK,
      status
    );

    // Create network
    let network = ClusterNetworkPartial {
      name: "utcj".to_owned(),
    };
    let mut res =
      cluster_network::tests::create(&network_srv, cluster_name, &network)
        .await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::CREATED,
      "Expect create network to return with status {}, got {}",
      StatusCode::CREATED,
      status
    );

    let _body: ClusterNetworkItem = res.json().await.expect(
      "Expect create network to return with a ClusterNetworkItem as json data",
    );

    // Create cargo
    let cargo = CargoPartial {
      name: "utcj".to_owned(),
      image_name: "nexthat/nanocl-get-started:latest".to_owned(),
      ..Default::default()
    };
    let res = cargo::tests::create(&cargo_srv, &cargo).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::CREATED,
      "Expect create cargo to return with status {}, got {}",
      StatusCode::CREATED,
      status
    );

    // Create proxy template
    let proxy_template = ProxyTemplateItem {
      name: "utcj".to_owned(),
      mode: ProxyTemplateModes::Http,
      content: "server {\n\
          server_name test.get-started.internal;\n\
          listen 127.0.0.1:80;\n\
          \n\
          if ($host != test.get-started.internal) {\n\
              return 404;\n\
          }\n\
          \n\
          location / {\n\
            proxy_set_header upgrade $http_upgrade;\n\
            proxy_set_header connection \"upgrade\";\n\
            proxy_http_version 1.1;\n\
            proxy_set_header x-forwarded-for $proxy_add_x_forwarded_for;\n\
            proxy_set_header host $host;\n\
            proxy_pass http://{{cargoes.unit-test-nginx-cluster-advenced.target_ip}}:9000;\n\
        }\n\
      }\n"
        .to_owned(),
    };
    let res =
      proxy_template::tests::create(&proxy_srv, &proxy_template).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::CREATED,
      "Expect create proxy template to return with status {}, got {}",
      StatusCode::CREATED,
      status
    );

    // Add proxy template to cluster
    let res = add_template(&srv, &cluster_name, &proxy_template.name).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::CREATED,
      "Expect add proxy template to cluster to return with status {}, got {}",
      StatusCode::CREATED,
      status
    );

    // Join
    let payload = ClusterJoinBody {
      cargo: cargo.name.to_owned(),
      network: network.name.to_owned(),
    };
    let res = join_cargo(&srv, cluster_name, &payload).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::OK,
      "Expect join cargo with name {} to cluster with name {} to return with status {}, got {}",
      cargo.name,
      cluster_name,
      StatusCode::OK,
      status
    );

    // Start
    let res = start(&srv, cluster_name).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::OK,
      "Expect start cluster with name {} to return with status {}, got {}",
      cluster_name,
      StatusCode::OK,
      status
    );

    // Remove proxy template to cluster
    let res =
      remove_template(&srv, &cluster_name, &proxy_template.name).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::OK,
      "Expect remove proxy template to cluster to return with status {}, got {}",
      StatusCode::OK,
      status
    );

    // Delete cluster
    let res = delete(&srv, cluster_name).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::OK,
      "Expect delete cluster with name {} to return with status {}, got {}",
      cluster_name,
      StatusCode::OK,
      status
    );

    // Delete cargo
    let res = cargo::tests::delete(&cargo_srv, &cargo.name).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::OK,
      "Expect delete cargo with name {} to return with status {}, got {}",
      cargo.name,
      StatusCode::OK,
      status
    );

    // Delete proxy template
    let res =
      proxy_template::tests::delete(&proxy_srv, &proxy_template.name).await?;
    let status = res.status();
    assert_eq!(
      status,
      StatusCode::OK,
      "Expect delete proxy template with name {} to return with status {}, got {}",
      proxy_template.name,
      StatusCode::OK,
      status
    );
    Ok(())
  }
}
