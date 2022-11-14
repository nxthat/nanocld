#[cfg(test)]
pub mod simple_scenario {
  use ntex::http::client::Client;
  use std::process::{Command, Child};

  async fn before_test() -> Child {
    Command::new("./target/debug/nanocld")
      .spawn()
      .expect("Start nanocld server")
  }

  #[ntex::test]
  async fn scenario() {
    let mut child = before_test().await;
    let _client = Client::new();
    child.kill().unwrap();
  }
}
