#[cfg(test)]
pub mod simple_scenario {
  use ntex::http::client::Client;
  use std::process::Command;

  async fn before_test() {
    Command::new("./target/debug/nanocld")
      .spawn()
      .expect("Start nanocld server")
      .wait()
      .expect("end");
  }

  #[ntex::test]
  async fn scenario() {
    before_test().await;
    let _client = Client::new();
  }
}
