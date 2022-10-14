use uuid::Uuid;
use chrono::{DateTime, FixedOffset};
use serde::{Serialize, Deserialize, Deserializer};

#[cfg(feature = "openapi")]
use utoipa::Component;

use crate::schema::nginx_logs;

fn deserialize_empty_string<'de, D>(
  deserializer: D,
) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  let buf = String::deserialize(deserializer)?;
  if buf.is_empty() {
    Ok(None)
  } else {
    Ok(Some(buf))
  }
}

fn deserialize_string_to_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
  D: Deserializer<'de>,
{
  let buf = String::deserialize(deserializer)?;
  let res = buf.parse::<i64>().unwrap_or_default();
  Ok(res)
}

fn deserialize_string_to_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
  D: Deserializer<'de>,
{
  let buf = String::deserialize(deserializer)?;
  let res = buf.parse::<i32>().unwrap_or_default();
  Ok(res)
}

fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
  D: Deserializer<'de>,
{
  let buf = String::deserialize(deserializer)?;
  let res = buf.parse::<f64>().unwrap_or_default();
  Ok(res)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NginxLogPartial {
  pub(crate) date_gmt: DateTime<FixedOffset>,
  pub(crate) uri: String,
  pub(crate) host: String,
  pub(crate) remote_addr: String,
  pub(crate) realip_remote_addr: String,
  pub(crate) server_protocol: String,
  pub(crate) request_method: String,
  #[serde(deserialize_with = "deserialize_string_to_i64")]
  pub(crate) content_length: i64,
  #[serde(deserialize_with = "deserialize_string_to_i64")]
  pub(crate) status: i64,
  #[serde(deserialize_with = "deserialize_string_to_f64")]
  pub(crate) request_time: f64,
  #[serde(deserialize_with = "deserialize_string_to_i64")]
  pub(crate) body_bytes_sent: i64,
  #[serde(deserialize_with = "deserialize_empty_string")]
  pub(crate) proxy_host: Option<String>,
  #[serde(deserialize_with = "deserialize_empty_string")]
  pub(crate) upstream_addr: Option<String>,
  #[serde(deserialize_with = "deserialize_empty_string")]
  pub(crate) query_string: Option<String>,
  #[serde(deserialize_with = "deserialize_empty_string")]
  pub(crate) request_body: Option<String>,
  #[serde(deserialize_with = "deserialize_empty_string")]
  pub(crate) content_type: Option<String>,
  #[serde(deserialize_with = "deserialize_empty_string")]
  pub(crate) http_user_agent: Option<String>,
  #[serde(deserialize_with = "deserialize_empty_string")]
  pub(crate) http_referrer: Option<String>,
  #[serde(deserialize_with = "deserialize_empty_string")]
  pub(crate) http_accept_language: Option<String>,
}

/// I REALLY THINK I HARD TROLLED THERE MY BAD XD
#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = nginx_logs)]
pub struct NginxLogItem {
  pub(crate) key: Uuid,
  pub(crate) date_gmt: DateTime<FixedOffset>,
  pub(crate) uri: String,
  pub(crate) host: String,
  pub(crate) remote_addr: String,
  pub(crate) realip_remote_addr: String,
  pub(crate) server_protocol: String,
  pub(crate) request_method: String,
  pub(crate) content_length: i64,
  pub(crate) status: i64,
  pub(crate) request_time: f64,
  pub(crate) body_bytes_sent: i64,
  pub(crate) proxy_host: Option<String>,
  pub(crate) upstream_addr: Option<String>,
  pub(crate) query_string: Option<String>,
  pub(crate) request_body: Option<String>,
  pub(crate) content_type: Option<String>,
  pub(crate) http_user_agent: Option<String>,
  pub(crate) http_referrer: Option<String>,
  pub(crate) http_accept_language: Option<String>,
}

impl From<NginxLogPartial> for NginxLogItem {
  fn from(partial: NginxLogPartial) -> Self {
    NginxLogItem {
      key: Uuid::new_v4(),
      date_gmt: partial.date_gmt,
      uri: partial.uri,
      host: partial.host,
      remote_addr: partial.remote_addr,
      realip_remote_addr: partial.realip_remote_addr,
      server_protocol: partial.server_protocol,
      request_method: partial.request_method,
      status: partial.status,
      request_time: partial.request_time,
      content_length: partial.content_length,
      body_bytes_sent: partial.body_bytes_sent,
      proxy_host: partial.proxy_host,
      upstream_addr: partial.upstream_addr,
      query_string: partial.query_string,
      request_body: partial.request_body,
      content_type: partial.content_type,
      http_user_agent: partial.http_user_agent,
      http_referrer: partial.http_referrer,
      http_accept_language: partial.http_accept_language,
    }
  }
}
