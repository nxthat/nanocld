use ntex::web;

use crate::config::DaemonConfig;
use crate::models::{Pool, VirtualMachinePartial};

use crate::errors::HttpResponseError;

#[web::post("/virtual_machines")]
async fn create_virtual_machine(
  web::types::Json(payload): web::types::Json<VirtualMachinePartial>,
  pool: web::types::State<Pool>,
  config: web::types::State<DaemonConfig>,
) -> Result<web::HttpResponse, HttpResponseError> {
  Ok(web::HttpResponse::Ok().finish())
}
