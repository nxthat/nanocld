use std::fs;

use super::HypervisorError;

/// State of virtual machine
#[derive(Debug, Clone)]
pub enum VmState {
  Running,
  Stoped,
}

/// A virtual machine instance in memory
#[derive(Debug, Clone)]
pub struct VmInstance {
  pub(crate) name: String,
  pub(crate) pid: Option<String>,
  pub(crate) state: VmState,
  pub(crate) image: String,
  pub(crate) pid_path: String,
}

/// TODO make vm image struct
#[derive(Debug, Clone)]
pub struct VmImage {
  name: String,
  path: String,
  size: u64,
  parent: Option<Box<VmImage>>,
}

impl VmImage {
  pub fn create(
    &self,
    name: String,
    path: String,
    parent: Option<Box<VmImage>>,
  ) -> Result<Self, HypervisorError> {
    let size = fs::metadata(&path)?.len();

    let image = VmImage {
      name,
      size,
      path,
      parent,
    };

    Ok(image)
  }
}

#[derive(Debug, Clone)]
pub struct VmConfig {
  pub(crate) cpu: i16,
  pub(crate) memory: String,
  pub(crate) network: String,
  pub(crate) mac_addr: String,
}

impl VmInstance {
  pub fn new(name: &str, image: &str) -> Self {
    Self {
      name: name.to_owned(),
      image: image.to_owned(),
      pid: None,
      state: VmState::Stoped,
      // we have to happend this with hypervisor directory
      pid_path: format!("/vm/pids/{name}.pid", name = name),
    }
  }
}

/// # Hypervisor
/// Generic trait to manage virtual machine
/// We must implement this trait for at least qemu and hyper-v
/// to have linux and windows compatibility
pub trait Hypervisor {
  /// Generic new that all implementation must have
  fn new() -> Self
  where
    Self: Sized;

  fn generate_seed(
    &self,
    instance: &VmInstance,
    config: String,
  ) -> Result<(), HypervisorError> {
    Ok(())
  }

  /// # Get instance pidfile path
  /// Mostly used for qemu when we run it as a daemon
  fn get_instance_pidfile_path(&self, instance: &VmInstance) -> String {
    format!("./{name}.pid", name = &instance.name)
  }

  /// # Delete pidfile path
  /// Mostly used for qemu we remove the pidfile before starting
  /// and after shutdown
  /// https://libvir-list.redhat.narkive.com/dKkAJrha/libvirt-patch-remove-pid-file-before-starting-qemu-and-after-shutdown
  fn delete_pidfile_path(
    &self,
    instance: &VmInstance,
  ) -> Result<(), HypervisorError> {
    fs::remove_file(self.get_instance_pidfile_path(instance))?;
    Ok(())
  }

  /// # Resize image
  /// Resize virtual machine image at given path with given size
  ///
  /// # Example
  /// ```rust,norun
  /// let hypervisor = Hypervisor::new();
  /// hypervisor.resize_image("./image_path", "50G");
  /// ```
  fn resize_image(
    &self,
    image_path: &str,
    size: &str,
  ) -> Result<(), HypervisorError>;

  /// # Create image
  /// Create virtual machine image at given path with given size
  fn create_image(&self, image_path: &str, size: &str);

  /// # Copy image
  /// Create a copy of an existing image with given size for a vm instance
  fn copy_image(
    &self,
    parent_img: &str,
    img: &str,
    size: &str,
  ) -> Result<(), HypervisorError>;

  /// # Create instance
  /// Create a virtual machine instance
  /// # Return
  /// Return a VmInstance structure
  fn create_instance(&self, name: &str, image: &str) -> VmInstance;

  /// # Delete instance
  /// Delete a virtual machine instance
  /// This will stop the virtual machine and delete its image
  fn delete_instance(
    &self,
    instance: &VmInstance,
  ) -> Result<(), HypervisorError>;

  /// # Stop instance
  /// Stop a virtual machine instance
  /// This will stop the virtual machine.
  fn stop_instance(&self, instance: &VmInstance)
    -> Result<(), HypervisorError>;

  /// # Start instance
  /// Start a virtual machine instance
  /// You must create an instance before starting it.
  fn start_instance(
    &self,
    instance: &VmInstance,
    config: &VmConfig,
  ) -> Result<(), HypervisorError>;
}
