/// # Qemu Hypervisor
/// Hypervisor implementation for qemu compatibility
use std::fs;
use std::path::Path;
use std::process::Command;

use super::Hypervisor;
use super::lib::VmInstance;

pub struct Qemu {}

impl Hypervisor for Qemu {
  fn new() -> Self {
    Qemu {}
  }

  fn resize_image(
    &self,
    image_path: impl AsRef<Path>,
    size: String,
  ) -> Result<(), super::HypervisorError> {
    let file_meta = fs::metadata(image_path.as_ref())?;
    // Todo verify file perm
    let _file_perm = file_meta.permissions();

    let ouput = Command::new("qemu-img")
      .args(["resize", image_path.as_ref().to_str().unwrap(), &size])
      .output()?;

    println!("{:#?}", &ouput);

    Ok(())
  }

  fn create_image(&self) {}

  fn create_instance(&self, name: &str, image: &str) -> VmInstance {
    VmInstance::new(name, image)
  }

  fn stop_instance(&self, instance: &VmInstance) {}

  fn start_instance(&self, instance: &VmInstance) {}

  fn delete_instance(&self, instance: &VmInstance) {}
}

#[cfg(test)]
pub mod test {

  use super::*;

  use crate::utils::test::TestReturn;

  #[ntex::test]
  async fn test_images() -> TestReturn {
    let qemu = Qemu::new();
    qemu.resize_image(
      "./vm_images/ubuntu-22.04-server-cloudimg-amd64.img",
      String::from("50G"),
    )?;
    Ok(())
  }

  #[ntex::test]
  async fn test_instances() {
    let qemu = Qemu::new();
    let instance = qemu.create_instance("ubuntu", "ubuntu-22");

    assert_eq!(instance.name, "ubuntu");
    qemu.start_instance(&instance);
    qemu.stop_instance(&instance);
    qemu.delete_instance(&instance);
  }
}
