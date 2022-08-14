use std::process::Command;

pub async fn ssh_conn() {
  // Command::new("ssh").args(vec![
  //   "-nNT",
  //   "-L", "/run/nanocl/proxies/proxy_1.sock",
  // ])
}

async fn test_connection() {}

pub async fn setup_proxy() {
  println!("Im called");

  Command::new("ssh")
    .args(vec!["ubuntu:ubuntu@nanocl-proxy-n1", "nanocld --version"])
    .output()
    .expect("failed to execute process");
}

pub async fn setup_worker() {}

pub async fn _setup_master() {}

#[cfg(test)]
pub mod tests {
  use super::*;

  #[ntex::test]
  async fn test_setup_proxy() {
    setup_proxy().await;
  }
}
