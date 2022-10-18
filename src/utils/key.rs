pub fn gen_key(n1: &str, n2: &str) -> String {
  n1.to_owned() + "-" + n2
}

pub fn resolve_nsp(nsp: Option<String>) -> String {
  match nsp {
    None => String::from("global"),
    Some(nsp) => nsp,
  }
}

pub fn gen_key_from_nsp(nsp: Option<String>, n2: &str) -> String {
  let nsp = match nsp {
    None => String::from("default"),
    Some(nsp) => nsp,
  };
  nsp + "-" + n2
}
