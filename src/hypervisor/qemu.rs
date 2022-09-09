/// # Qemu Hypervisor
/// Hypervisor implementation for qemu compatibility
use std::fs;
use std::path::Path;
use std::process::Command;

use super::Hypervisor;
use super::lib::{VmInstance, VmConfig};

pub struct Qemu {}

impl Hypervisor for Qemu {
  fn new() -> Self {
    Qemu {}
  }

  fn resize_image(
    &self,
    image_path: String,
    size: String,
  ) -> Result<(), super::HypervisorError> {
    let file_meta = fs::metadata(&image_path)?;
    // Todo verify file perm
    let _file_perm = file_meta.permissions();
    let ouput = Command::new("qemu-img")
      .args(["resize", &image_path, &size])
      .output()?;

    println!("{:#?}", &ouput);
    Ok(())
  }

  fn create_image(&self) {}

  fn create_instance(&self, name: &str, image: &str) -> VmInstance {
    VmInstance::new(name, image)
  }

  fn stop_instance(
    &self,
    instance: &VmInstance,
  ) -> Result<(), super::HypervisorError> {
    let pidfile_path = self.get_instance_pidfile_path(instance);
    // This may look dumb to convert string into i32 to reconvert to string
    // but that way im sure the pid is a number inside the readed file.
    let pid = fs::read_to_string(&pidfile_path)?.trim().parse::<u64>()?;
    let output = Command::new("kill").args(vec![pid.to_string()]).output()?;
    fs::remove_file(pidfile_path)?;
    println!("{:#?}", output);
    Ok(())
  }

  /// Start virtual machine instance using qemu
  /// a basic command should be
  /// ```sh,norun
  /// qemu-system-x86_64 -machine accel=kvm,type=q35 -smp 2 -m 4G -net nic,macaddr=2c:4d:11:12:11:11 \
  /// -net bridge,br=nanoclvpn0 -drive if=virtio,format=qcow2,file=ubuntu-22.04-server-cloudimg-amd64.img \
  /// -cdrom seed.img --daemonize -display none
  /// ```
  /// # Todo
  /// Generate the macaddr
  fn start_instance(
    &self,
    instance: &VmInstance,
    config: &VmConfig,
  ) -> Result<(), super::HypervisorError> {
    log::debug!(
      "Starting qemu instance {:#?} with config {:#?}",
      &instance,
      &config
    );
    let image = format!(
      "if=virtio,format=qcow2,file=./vm_images/{image}.img",
      image = &instance.image
    );
    let cpu = config.cpu.to_string();
    let network_bridge = format!("bridge,br={}", config.network);
    let macaddr = format!("nic,macaddr={}", config.macaddr);
    let pidfile = self.get_instance_pidfile_path(instance);
    let args = vec![
      "-machine",
      "accel=kvm,type=q35",
      "-smp",
      &cpu,
      "-m",
      &config.memory,
      "-net",
      &macaddr,
      "-net",
      &network_bridge,
      "-drive",
      &image,
      // "-cdrom",
      // "seed.img",
      "-pidfile",
      &pidfile,
      "--daemonize",
      "-display",
      "none",
    ];
    let ouput = Command::new("qemu-system-x86_64").args(&args).output()?;
    log::debug!("Qemu instance started {:?}", &ouput);
    Ok(())
  }

  /// Todo delete instance
  fn delete_instance(
    &self,
    _instance: &VmInstance,
  ) -> Result<(), super::HypervisorError> {
    todo!("Delete vm instance that stop instance and remove his image");
  }
}

#[cfg(test)]
pub mod test {

  use super::*;

  use crate::utils::test::TestReturn;

  // #[ntex::test]
  async fn test_images() -> TestReturn {
    let qemu = Qemu::new();
    qemu.resize_image(
      String::from("/var/lib/nanoc/qemu/images/ubuntu-22.img"),
      String::from("50G"),
    )?;
    Ok(())
  }

  // #[ntex::test]
  async fn test_instances() -> TestReturn {
    let qemu = Qemu::new();
    let instance = qemu.create_instance("ubuntu", "ubuntu-22");
    let instance_config = VmConfig {
      cpu: 2,
      memory: String::from("2G"),
      network: String::from("nanoclservices0"),
      macaddr: String::from("2c:4d:11:12:11:11"),
    };
    assert_eq!(instance.name, "ubuntu");
    qemu.start_instance(&instance, &instance_config)?;
    qemu.stop_instance(&instance)?;
    // qemu.delete_instance(&instance)?;
    Ok(())
  }
}
