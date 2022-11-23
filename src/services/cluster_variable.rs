use ntex::web;
use serde::{Serialize, Deserialize};

use crate::{utils, repositories};
use crate::models::{Pool, GenericNspQuery, ClusterVariablePartial};
use crate::errors::HttpResponseError;

/// Create cluster variable
#[cfg_attr(feature = "dev", utoipa::path(
  post,
  path = "/clusters/{c_name}/variables",
  request_body = ClusterVariablePartial,
  params(
    ("c_name" = String, Path, description = "name of the cluster"),
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cluster is stored is empty we use 'global' as value"),
  ),
  responses(
    (status = 200, description = "Fresh cluster variable", body = ClusterVariableItem),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Cluster name or Namespace not valid", body = ApiError),
  ),
))]
#[web::post("/clusters/{c_name}/variables")]
async fn create_cluster_variable(
  pool: web::types::State<Pool>,
  c_name: web::types::Path<String>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
  web::types::Json(payload): web::types::Json<ClusterVariablePartial>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let name = c_name.into_inner();
  let cluster_key = utils::key::gen_key_from_nsp(&qs.namespace, &name);

  repositories::cluster::find_by_key(cluster_key.to_owned(), &pool).await?;
  let cluster_var = repositories::cluster_variable::create(
    cluster_key.to_owned(),
    payload,
    &pool,
  )
  .await?;

  Ok(web::HttpResponse::Created().json(&cluster_var))
}

/// List variable of a cluster
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  path = "/clusters/{c_name}/variables",
  params(
    ("c_name" = String, Path, description = "name of the cluster"),
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cluster is stored is empty we use 'global' as value"),
  ),
  responses(
    (status = 200, description = "List of variable for given cluster", body = [ClusterVariableItem]),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Cluster name or Namespace not valid", body = ApiError),
  ),
))]
#[web::get("/clusters/{c_name}/variables")]
async fn list_cluster_variable(
  pool: web::types::State<Pool>,
  c_name: web::types::Path<String>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let name = c_name.into_inner();
  let cluster_key = utils::key::gen_key_from_nsp(&qs.namespace, &name);

  repositories::cluster::find_by_key(cluster_key.to_owned(), &pool).await?;
  let cluster_variables = repositories::cluster_variable::list_by_cluster(
    cluster_key.to_owned(),
    &pool,
  )
  .await?;

  Ok(web::HttpResponse::Ok().json(&cluster_variables))
}

#[derive(Serialize, Deserialize)]
pub struct ClusterVariablePath {
  c_name: String,
  v_name: String,
}

/// Delete cluster variable by it's name
#[cfg_attr(feature = "dev", utoipa::path(
  delete,
  path = "/clusters/{c_name}/variables/{v_name}",
  params(
    ("c_name" = String, Path, description = "name of the cluster"),
    ("v_name" = String, Path, description = "name of the variable"),
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cluster is stored is empty we use 'global' as value"),
  ),
  responses(
    (status = 200, description = "Generic delete response", body = GenericDelete),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Cluster name or Namespace not valid", body = ApiError),
  ),
))]
#[web::delete("/clusters/{c_name}/variables/{v_name}")]
async fn delete_cluster_variable(
  pool: web::types::State<Pool>,
  url_path: web::types::Path<ClusterVariablePath>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let var_name = format!("{}-{}", &url_path.c_name, &url_path.v_name);
  let var_key = utils::key::gen_key_from_nsp(&qs.namespace, &var_name);

  let res =
    repositories::cluster_variable::delete_by_key(var_key, &pool).await?;
  Ok(web::HttpResponse::Ok().json(&res))
}

/// Get cluster variable by it's name
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  path = "/clusters/{c_name}/variables/{v_name}",
  params(
    ("c_name" = String, Path, description = "name of the cluster"),
    ("v_name" = String, Path, description = "name of the variable"),
    ("namespace" = Option<String>, Query, description = "Name of the namespace where the cluster is stored is empty we use 'global' as value"),
  ),
  responses(
    (status = 200, description = "Generic delete response", body = GenericDelete),
    (status = 400, description = "Generic database error", body = ApiError),
    (status = 404, description = "Cluster name or Namespace not valid", body = ApiError),
  ),
))]
#[web::get("/clusters/{c_name}/variables/{v_name}")]
async fn get_cluster_variable_by_name(
  pool: web::types::State<Pool>,
  url_path: web::types::Path<ClusterVariablePath>,
  web::types::Query(qs): web::types::Query<GenericNspQuery>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let var_name = utils::key::gen_key(&url_path.c_name, &url_path.v_name);
  let var_key = utils::key::gen_key_from_nsp(&qs.namespace, &var_name);

  let res = repositories::cluster_variable::find_by_key(var_key, &pool).await?;
  Ok(web::HttpResponse::Ok().json(&res))
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(create_cluster_variable);
  config.service(list_cluster_variable);
  config.service(delete_cluster_variable);
  config.service(get_cluster_variable_by_name);
}

/// Cluster variable unit test
#[cfg(test)]
mod tests {
  use ntex::http::StatusCode;

  use super::*;
  use crate::models::ClusterVariableItem;
  use crate::utils::tests::*;

  use crate::services::cluster;

  /// Helper to list cluster variables
  pub async fn list(srv: &TestServer, cluster_name: &str) -> TestReqRet {
    srv
      .get(format!("/clusters/{cluster_name}/variables"))
      .send()
      .await
  }

  /// Helper to create a cluster variable
  pub async fn create(
    srv: &TestServer,
    cluster_name: &str,
    var: &ClusterVariablePartial,
  ) -> TestReqRet {
    srv
      .post(format!("/clusters/{cluster_name}/variables"))
      .send_json(var)
      .await
  }

  /// Helper to delete a cluster variable
  pub async fn delete(
    srv: &TestServer,
    cluster_name: &str,
    var_name: &str,
  ) -> TestReqRet {
    srv
      .delete(format!("/clusters/{cluster_name}/variables/{var_name}"))
      .send()
      .await
  }

  /// Helper to Get a cluster variable by his name
  pub async fn get_by_name(
    srv: &TestServer,
    cluster_name: &str,
    var_name: &str,
  ) -> TestReqRet {
    srv
      .get(format!("/clusters/{cluster_name}/variables/{var_name}"))
      .send()
      .await
  }

  /// Basic list when cluster a cluster doesn't exists that return a 404
  #[ntex::test]
  async fn basic_list_cluster_not_exists() -> TestRet {
    let srv = generate_server(ntex_config).await;
    let resp = list(&srv, "not-existing").await?;
    assert_eq!(
      resp.status(),
      StatusCode::NOT_FOUND,
      "Expect status 404 NOTFOUND when searching inside non existing cluster"
    );
    Ok(())
  }

  /// Basic create when cluster doesn't exist that return 404
  #[ntex::test]
  async fn basic_create_cluster_not_exists() -> TestRet {
    let srv = generate_server(ntex_config).await;
    let new_var = ClusterVariablePartial {
      name: String::from("test"),
      value: String::from("test"),
    };
    let resp = create(&srv, "not-existing", &new_var).await?;
    assert_eq!(
      resp.status(),
      StatusCode::NOT_FOUND,
      "Expect status 404 NOTFOUND when creating inside non existing cluster"
    );
    Ok(())
  }

  /// Basic delete when cluster doesn't exist that return 404
  #[ntex::test]
  async fn basic_delete_cluster_not_exists() -> TestRet {
    let srv = generate_server(ntex_config).await;
    let new_var = ClusterVariablePartial {
      name: String::from("test"),
      value: String::from("test"),
    };
    let resp = create(&srv, "not-existing", &new_var).await?;
    assert_eq!(
      resp.status(),
      StatusCode::NOT_FOUND,
      "Expect status 404 NOTFOUND when deleting inside non existing cluster"
    );
    Ok(())
  }

  /// Test to create read and destroy a variable when cluster exists
  #[ntex::test]
  async fn create_read_destroy_when_cluster_exists() -> TestRet {
    let cluster_name = "unit-test-var";
    let cluster_srv = generate_server(cluster::ntex_config).await;
    cluster::tests::create(&cluster_srv, cluster_name).await?;

    let srv = generate_server(ntex_config).await;

    let new_var = ClusterVariablePartial {
      name: String::from("unit-test"),
      value: String::from("yoloh"),
    };
    let resp = create(&srv, cluster_name, &new_var).await?;
    assert!(
      resp.status().is_success(),
      "Expect success when creating a variable in cluster {} with data {:#?}",
      cluster_name,
      new_var
    );
    let mut resp = get_by_name(&srv, &cluster_name, &new_var.name).await?;
    assert!(
      resp.status().is_success(),
      "Expect success when inspecting a variable in cluster {} with name {}",
      cluster_name,
      new_var.name,
    );
    let body: ClusterVariableItem = resp.json().await?;
    assert_eq!(
      body.value, new_var.value,
      "Expect cluster variable value {} to be {}",
      body.value, new_var.value
    );
    let resp = delete(&srv, &cluster_name, &new_var.name).await?;
    assert!(
      resp.status().is_success(),
      "Expect success when deleting variable {} in cluster {}",
      new_var.name,
      cluster_name
    );

    cluster::tests::delete(&cluster_srv, cluster_name).await?;
    Ok(())
  }
}
