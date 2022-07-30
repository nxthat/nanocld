pub fn ensure_namespace(namespace: &Option<String>) -> String {
  match namespace {
    None => String::from("global"),
    Some(nsp) => nsp.to_owned(),
  }
}

pub fn format_key(parent_key: &String, name: &String) -> String {
  format!("{}-{}", parent_key, name)
}

pub fn gen_nsp_key_by_name(
  namespace: &Option<String>,
  name: &String,
) -> String {
  let namespace = ensure_namespace(namespace);
  format_key(&namespace, name)
}
