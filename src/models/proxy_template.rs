use diesel_derive_enum::DbEnum;
use serde::{Serialize, Deserialize};

#[cfg(feature = "dev")]
use utoipa::ToSchema;

use crate::schema::proxy_templates;

/// Nginx template mode
/// # Examples
/// ```
/// ProxyTemplateModes::Http; // For http forward
/// ProxyTemplateModes::Stream; // For low level tcp/udp forward
/// ```
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, DbEnum, Clone)]
#[DieselTypePath = "crate::schema::sql_types::ProxyTemplateModes"]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub enum ProxyTemplateModes {
  Http,
  Stream,
}

#[derive(
  Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable,
)]
#[diesel(primary_key(name))]
#[diesel(table_name = proxy_templates)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct ProxyTemplateItem {
  pub(crate) name: String,
  pub(crate) mode: ProxyTemplateModes,
  pub(crate) content: String,
}
