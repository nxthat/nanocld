use serde::{Serialize, Deserialize};

// Virtual machine item
pub struct VirtualMachineItem {
  // The key is a concatenation of the namespace where the virtual machine is created and the name
  // that way we can use the same virtual machine name for different namespace.
  pub(crate) key: String,
  // Name of the virtual machine.
  pub(crate) name: String,
  // Path of the pid where is stored the pid of the vm
  pub(crate) pid_path: String,
  // Image of the virtual machine.
  // Note that when you create a virtual machine it will copy the base img.
  pub(crate) image_key: String,
}

#[derive(Serialize, Deserialize)]
pub struct VirtualMachinePartial {
  pub(crate) name: String,
  pub(crate) image: String,
}

pub struct VirtualMachineConfigItem {
  // Number of cpu used by the virtual machine
  pub(crate) cpu: i32,
  // Number of random access memory used by the virtual machine
  // eg: 2G 8G
  pub(crate) memory: String,
  // Network interface where the virtual machine is binded
  pub(crate) network: String,
  // Mac address of the virtual machine
  pub(crate) macaddr: String,
}
