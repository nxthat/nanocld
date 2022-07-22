use std::collections::HashMap;

use crate::models::ClusterVariableItem;

pub fn cluster_vars_to_hashmap(
  vars: Vec<ClusterVariableItem>,
) -> HashMap<String, String> {
  let hashmap: HashMap<String, String> = HashMap::new();
  vars.into_iter().fold(hashmap, |mut acc, item| {
    acc.insert(item.name, item.value);
    acc
  })
}
