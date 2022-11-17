use ntex::web;

use crate::repositories;
use crate::models::{Pool, ProxyTemplateItem};

use crate::errors::HttpResponseError;

/// List all proxy template
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  path = "/proxy/templates",
  responses(
      (status = 200, description = "Array of proxy templates", body = [ProxyTemplateItem]),
  ),
))]
#[web::get("/proxy/templates")]
async fn list_proxy_template(
  pool: web::types::State<Pool>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let items = repositories::proxy_template::list(&pool).await?;

  Ok(web::HttpResponse::Ok().json(&items))
}

/// Create proxy template
#[cfg_attr(feature = "dev", utoipa::path(
  post,
  path = "/proxy/templates",
  responses(
    (status = 201, description = "Nginx template created", body = ProxyTemplateItem)
  )
))]
#[web::post("/proxy/templates")]
async fn create_proxy_template(
  pool: web::types::State<Pool>,
  web::types::Json(payload): web::types::Json<ProxyTemplateItem>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let res = repositories::proxy_template::create(payload, &pool).await?;

  Ok(web::HttpResponse::Created().json(&res))
}

/// Delete proxy template by name
#[web::delete("/proxy/templates/{name}")]
async fn delete_proxy_template_by_name(
  pool: web::types::State<Pool>,
  name: web::types::Path<String>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let res =
    repositories::proxy_template::delete_by_name(name.into_inner(), &pool)
      .await?;

  Ok(web::HttpResponse::Ok().json(&res))
}

/// Inspect proxy template by name
#[web::get("/proxy/templates/{name}")]
async fn inspect_proxy_template_by_name(
  pool: web::types::State<Pool>,
  name: web::types::Path<String>,
) -> Result<web::HttpResponse, HttpResponseError> {
  let res =
    repositories::proxy_template::get_by_name(name.into_inner(), &pool).await?;

  Ok(web::HttpResponse::Ok().json(&res))
}

pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(list_proxy_template);
  config.service(create_proxy_template);
  config.service(delete_proxy_template_by_name);
  config.service(inspect_proxy_template_by_name);
}
