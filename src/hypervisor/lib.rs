use super::HypervisorError;

use std::path::Path;

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

impl VmInstance {
  pub fn new(name: &str, image: &str) -> Self {
    Self {
      name: name.to_owned(),
      image: image.to_owned(),
      pid: None,
      state: VmState::Stoped,
      pid_path: format!("/vm/pids/{name}.pid", name = name),
    }
  }
}

/// # Hypervisor
/// Generic trait to manage virtual machine
/// We must implement this trait for at least hyper-v and qemu
/// to have linux and windows compatibility
pub trait Hypervisor {
  fn new() -> Self;
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
    image_path: impl AsRef<Path>,
    size: String,
  ) -> Result<(), HypervisorError>;

  /// # Create image
  /// Create virtual machine image at given path with given size
  fn create_image(&self);

  fn create_instance(&self, name: &str, image: &str) -> VmInstance;

  fn delete_instance(&self, instance: &VmInstance);

  fn stop_instance(&self, instance: &VmInstance);

  fn start_instance(&self, instance: &VmInstance);
}
