use ntex::web;

use crate::{repositories, services};
use crate::hypervisor::Hypervisor;
use crate::models::{Pool, VmPartial, VmItem};
use crate::errors::HttpResponseError;

// async fn create(
//   item: VmPartial,
//   pool: &web::types::State<Pool>,
//   hypervisor: &Box<dyn Hypervisor + 'static>,
// ) -> Result<VmItem, HttpResponseError> {
//   let image =
//     repositories::virtual_machine_image::find_by_id(item.image, pool).await?;

//   // services::virtual_machine_image::
//   // hypervisor
//   //   .start_instance(instance, config);

//   Ok(vm)
// }
