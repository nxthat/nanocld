/// TODO Make a ssh client to wrap Command::new()
/// Add node parameter eg: proxy, master, worker
use std::process::Command;

pub async fn _ssh_conn() {
  // Command::new("ssh").args(vec![
  //   "-nNT",
  //   "-L", "/run/nanocl/proxies/proxy_1.sock",
  // ])
}

async fn _test_connection() {}

pub async fn _setup_proxy() {
  Command::new("ssh")
    .args(vec!["ubuntu:ubuntu@nanocl-proxy-n1", "nanocld --version"])
    .output()
    .expect("failed to execute process");
}

pub async fn _setup_worker() {}

pub async fn _setup_master() {}
