use ntex::web;
use std::collections::HashMap;

use crate::utils::key;
use crate::models::ContainerFilterQuery;
use crate::errors::HttpResponseError;

pub async fn list_cargo_instance(
  qs: ContainerFilterQuery,
  docker_api: &web::types::State<bollard::Docker>,
) -> Result<Vec<bollard::models::ContainerSummary>, HttpResponseError> {
  let namespace = key::resolve_nsp(&qs.namespace);
  let mut filters = HashMap::new();
  let default_label = format!("namespace={}", &namespace);
  let mut labels = vec![default_label];
  if let Some(ref cluster) = qs.cluster {
    let label = format!("cluster={}-{}", &namespace, &cluster);
    labels.push(label);
  }
  if let Some(ref cargo) = qs.cargo {
    let label = format!("cargo={}-{}", &namespace, &cargo);
    labels.push(label);
  }
  filters.insert(String::from("label"), labels);
  let options = Some(bollard::container::ListContainersOptions::<String> {
    all: true,
    filters,
    ..Default::default()
  });
  let containers = docker_api.list_containers(options).await?;

  Ok(containers)
}
