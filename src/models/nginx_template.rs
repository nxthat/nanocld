use diesel_derive_enum::DbEnum;
use serde::{Serialize, Deserialize};

#[cfg(feature = "dev")]
use utoipa::ToSchema;

use crate::schema::nginx_templates;

/// Nginx template mode
/// # Examples
/// ```
/// NginxTemplateModes::Http; // For http forward
/// NginxTemplateModes::Stream; // For low level tcp/udp forward
/// ```
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, DbEnum, Clone)]
#[DieselTypePath = "crate::schema::sql_types::NginxTemplateModes"]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub enum NginxTemplateModes {
  Http,
  Stream,
}

#[derive(
  Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable,
)]
#[diesel(primary_key(name))]
#[diesel(table_name = nginx_templates)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct NginxTemplateItem {
  pub(crate) name: String,
  pub(crate) mode: NginxTemplateModes,
  pub(crate) content: String,
}
