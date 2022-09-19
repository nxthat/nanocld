use diesel_derive_enum::DbEnum;
use serde::{Serialize, Deserialize};

use super::virtual_machine_image::VmImageItem;
use crate::schema::virtual_machines;

#[cfg(feature = "openapi")]
use utoipa::Component;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, DbEnum, Clone)]
#[DieselTypePath = "crate::models::exports::Virtual_machine_states"]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(Component))]
pub enum VirtualMachineState {
  Running,
  Stopped,
}

// Virtual machine item
// This structure is used to create new entry in database
#[derive(Debug, Insertable, Queryable, Identifiable, Associations)]
#[diesel(primary_key(key))]
#[diesel(table_name = virtual_machines)]
#[diesel(belongs_to(VmImageItem, foreign_key = image))]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct VmItem {
  // The key is a concatenation of the namespace where the virtual machine is created and the name
  // that way we can use the same virtual machine name for different namespace.
  pub(crate) key: String,
  // Name of the virtual machine.
  pub(crate) name: String,
  // State of the virtual machine Running / Stopped
  pub(crate) state: VirtualMachineState,
  // Path of the pid where is stored the pid of the vm
  pub(crate) pid_path: String,
  // Image of the virtual machine.
  // Note that when you create a virtual machine it will copy the image.
  pub(crate) image: String,
  // Number of ram used by the virtual machine
  pub(crate) memory: String,
  // Number of cpu used by the virtual machine
  pub(crate) cpu: i16,
  // Network where virtual machine is connected
  pub(crate) network: String,
  // Ip address assigned to the virtual machine
  pub(crate) ip_addr: String,
  // Mac address assigned to the virtual machine
  pub(crate) mac_addr: String,
}

// Virtual machine partial
// This structure is the payload send for a post request to create a new virtual machine
#[derive(Serialize, Deserialize)]
pub struct VmPartial {
  // Name of the virtual machine to create
  pub(crate) name: String,
  // Image to base the virtual machine on
  pub(crate) image: String,
  // Size of the image that will be size of the disk for the virtual machine
  pub(crate) image_size: String,
  // Number of cpu used by the virtual machine
  pub(crate) cpu: i16,
  // Number of memory used by the virtual machine
  pub(crate) memory: String,
  // Network to connect virtual machine to
  pub(crate) network: String,
}
