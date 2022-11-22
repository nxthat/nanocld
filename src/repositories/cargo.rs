use ntex::web;
use diesel::prelude::*;

use crate::controllers;
use crate::models::{
  Pool, CargoItem, CargoPartial, GenericDelete, NamespaceItem, GenericCount,
  CargoPatchPartial, CargoPatchItem,
};

use crate::errors::HttpResponseError;
use super::errors::db_blocking_error;

pub async fn find_by_namespace(
  nsp: NamespaceItem,
  pool: &web::types::State<Pool>,
) -> Result<Vec<CargoItem>, HttpResponseError> {
  let mut conn = controllers::store::get_pool_conn(pool)?;

  let res =
    web::block(move || CargoItem::belonging_to(&nsp).load(&mut conn)).await;
  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(items) => Ok(items),
  }
}

pub async fn create(
  nsp: String,
  item: CargoPartial,
  pool: &web::types::State<Pool>,
) -> Result<CargoItem, HttpResponseError> {
  use crate::schema::cargoes::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;
  let res = web::block(move || {
    let new_item = CargoItem {
      key: nsp.to_owned() + "-" + &item.name,
      name: item.name.clone(),
      namespace_name: nsp,
      image_name: item.image_name,
      binds: item.binds.unwrap_or_default(),
      replicas: item.replicas.unwrap_or(1),
      dns_entry: item.dns_entry,
      domainname: item.domainname,
      hostname: item.hostname,
      network_mode: item.network_mode,
      restart_policy: item.restart_policy,
      cap_add: item.cap_add,
    };
    diesel::insert_into(dsl::cargoes)
      .values(&new_item)
      .execute(&mut conn)?;
    Ok(new_item)
  })
  .await;
  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}

pub async fn count(
  namespace: String,
  pool: &web::types::State<Pool>,
) -> Result<GenericCount, HttpResponseError> {
  use crate::schema::cargoes::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::cargoes
      .filter(dsl::namespace_name.eq(namespace))
      .count()
      .get_result(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(result) => Ok(GenericCount { count: result }),
  }
}

pub async fn delete_by_key(
  key: String,
  pool: &web::types::State<Pool>,
) -> Result<GenericDelete, HttpResponseError> {
  use crate::schema::cargoes::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;
  let res = web::block(move || {
    diesel::delete(dsl::cargoes)
      .filter(dsl::key.eq(key))
      .execute(&mut conn)
  })
  .await;
  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(result) => Ok(GenericDelete { count: result }),
  }
}

pub async fn find_by_key(
  key: String,
  pool: &web::types::State<Pool>,
) -> Result<CargoItem, HttpResponseError> {
  use crate::schema::cargoes::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::cargoes.filter(dsl::key.eq(key)).get_result(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}

pub async fn _find_by_image_name(
  image_name: String,
  pool: &web::types::State<Pool>,
) -> Result<Vec<CargoItem>, HttpResponseError> {
  use crate::schema::cargoes::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;
  let res = web::block(move || {
    dsl::cargoes
      .filter(dsl::image_name.eq(image_name))
      .load(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(items) => Ok(items),
  }
}

pub async fn update_by_key(
  nsp: String,
  name: String,
  item: CargoPatchPartial,
  pool: &web::types::State<Pool>,
) -> Result<CargoItem, HttpResponseError> {
  use crate::schema::cargoes::dsl;

  let mut conn = controllers::store::get_pool_conn(pool)?;
  let res = web::block(move || {
    let mut key: Option<String> = None;

    if let Some(ref name) = item.name {
      key = Some(format!("{}-{}", &nsp, name));
    }

    let data = CargoPatchItem {
      key,
      name: item.name,
      image_name: item.image_name,
      binds: item.binds,
      replicas: item.replicas,
      dns_entry: item.dns_entry,
      domainname: item.domainname,
      hostname: item.hostname,
      network_mode: item.network_mode,
      restart_policy: item.restart_policy,
      cap_add: item.cap_add,
    };
    diesel::update(
      dsl::cargoes.filter(dsl::key.eq(&format!("{}-{}", &nsp, &name))),
    )
    .set(&data)
    .get_result(&mut conn)
  })
  .await;

  match res {
    Err(err) => Err(db_blocking_error(err)),
    Ok(item) => Ok(item),
  }
}
