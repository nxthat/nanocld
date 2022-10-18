pub enum ControllerType {
  Dns,
  GeoDns,
  Proxy,
  Vpn,
  Store,
}

pub struct Controller {
  pub r#type: ControllerType,
}
