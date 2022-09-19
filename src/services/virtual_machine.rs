use std::path::Path;

use ntex::web;
use ntex::http::StatusCode;

use crate::repositories;
use crate::config::DaemonConfig;
use crate::utils::generate_mac_addr;
use crate::hypervisor::{Hypervisor, VmConfig};
use crate::models::{Pool, VmPartial, VmItem, VirtualMachineState};
use crate::errors::HttpResponseError;

pub async fn create(
  item: VmPartial,
  pool: &web::types::State<Pool>,
  hypervisor: &dyn Hypervisor,
  config: &web::types::State<DaemonConfig>,
) -> Result<VmItem, HttpResponseError> {
  let image = repositories::virtual_machine_image::find_by_id(
    item.image.to_owned(),
    pool,
  )
  .await?;

  let vm_img_path = Path::new(&config.state_dir)
    .join("vm_images")
    .join(&item.name);

  let vm_img_path = vm_img_path.to_str().ok_or(HttpResponseError {
    msg: String::from("Unable to convert path to str"),
    status: StatusCode::INTERNAL_SERVER_ERROR,
  })?;

  hypervisor
    .copy_image(&image.path, vm_img_path, &item.image_size)
    .map_err(|err| HttpResponseError {
      msg: format!("Hypervisor error {:?}", &err),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

  let instance = hypervisor.create_instance(&item.name, vm_img_path);

  let mac_addr = generate_mac_addr()?;

  // TODO Generate a seed to set mac addr
  // hypervisor.generate_seed(instance, config);
  let instance_cfg = VmConfig {
    cpu: item.cpu.to_owned(),
    memory: item.memory.to_owned(),
    network: item.network.to_owned(),
    mac_addr: mac_addr.to_owned(),
  };
  hypervisor
    .start_instance(&instance, &instance_cfg)
    .map_err(|err| HttpResponseError {
      msg: format!("Unable to start virtual machine instance {:?}", &err),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

  let vm = VmItem {
    key: item.name.to_owned(),
    name: item.name.to_owned(),
    state: VirtualMachineState::Running,
    pid_path: instance.pid_path,
    image: item.image.to_owned(),
    memory: item.memory.to_owned(),
    cpu: item.cpu,
    network: item.network.to_owned(),
    ip_addr: String::from("0.0.0.0"),
    mac_addr,
  };

  let vm = repositories::virtual_machine::create(vm, pool).await?;

  Ok(vm)
}

// #[cfg(test)]
// pub mod tests {
//   use super::*;
//   use crate::{utils::test::TestReturn, hypervisor::Qemu};

//   async fn test_create_vm_instance() -> TestReturn {
//     let hypervisor = Qemu::new();

//     create(item, pool, &hypervisor, config);
//     Ok(())
//   }
// }
