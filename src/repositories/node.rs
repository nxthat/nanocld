use ntex::web;
use diesel::prelude::*;

use crate::controllers;
use crate::models::{Pool, NodeItem, NodePartial};

use crate::errors::HttpResponseError;
use super::errors::db_blocking_error;

/// List existing nodes
///
/// # Arguments
///
/// * [pool](web::types::State<Pool>) - Posgresql database pool
///
/// # Examples
///
/// ```rust,norun
///
/// use crate::repositories;
///
/// let res = repositories::node::list(&pool).await;
/// ```
pub async fn _list(
  pool: &web::types::State<Pool>,
) -> Result<Vec<NodeItem>, HttpResponseError> {
  use crate::schema::nodes::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;
  let res = web::block(move || dsl::nodes.load(&mut conn)).await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(items) => Ok(items),
  }
}

/// Create a new node
///
/// # Arguments
///
/// * [node](NodePartial) - Partial node
/// * [pool](web::types::State<Pool>) - Posgresql database pool
///
/// # Examples
///
/// ```rust,norun
///
/// use crate::models::NodePartial;
/// use crate::repositories;
///
/// let node = NodePartial {
/// };
/// let res = repositories::node::create(node, &pool).await;
/// ```
pub async fn _create(
  node: NodePartial,
  pool: &web::types::State<Pool>,
) -> Result<NodeItem, HttpResponseError> {
  use crate::schema::nodes::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;
  let res = web::block(move || {
    let node: NodeItem = node.into();
    diesel::insert_into(dsl::nodes)
      .values(&node)
      .execute(&mut conn)?;
    Ok(node)
  })
  .await;

  let item = res.map_err(db_blocking_error)?;

  Ok(item)
}
