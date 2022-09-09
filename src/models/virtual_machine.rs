use serde::{Serialize, Deserialize};

use super::virtual_machine_image::VmImageItem;
use crate::schema::virtual_machines;

// Virtual machine item
// This structure is used to create new entry in database
#[derive(Debug, Insertable, Queryable, Identifiable, Associations)]
#[primary_key(key)]
#[table_name = "virtual_machines"]
#[belongs_to(VmImageItem, foreign_key = "image")]
#[cfg_attr(feature = "openapi", derive(Component))]
pub struct VmItem {
  // The key is a concatenation of the namespace where the virtual machine is created and the name
  // that way we can use the same virtual machine name for different namespace.
  pub(crate) key: String,
  // Name of the virtual machine.
  pub(crate) name: String,
  // Path of the pid where is stored the pid of the vm
  pub(crate) pid_path: String,
  // Image of the virtual machine.
  // Note that when you create a virtual machine it will copy the image.
  pub(crate) image: String,
  // Number of ram used by the virtual machine
  pub(crate) memory: i16,
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
  // Number of cpu used by the virtual machine
  pub(crate) cpu: i16,
  // Number of memory used by the virtual machine
  pub(crate) memory: i16,
  // Network to connect virtual machine to
  pub(crate) network: String,
}
