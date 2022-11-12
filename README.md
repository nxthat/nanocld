<div align="center">
  <img src="https://download.next-hat.com/ressources/images/logo.png" >
  <h1>Nanocld</h1>
  <h3>Hybrid Cloud Orchestrator</h3>
  <h3>DAEMON</h3>
  <p>

  [![Stars](https://img.shields.io/github/stars/nxthat/nanocld?label=%E2%AD%90%20stars%20%E2%AD%90)](https://github.com/nxthat/nanocld)
  [![Build With](https://img.shields.io/badge/built_with-Rust-dca282.svg?style=flat)](https://github.com/nxthat/nanocld)
  [![Chat on Discord](https://img.shields.io/discord/1011267493114949693?label=chat&logo=discord&style=flat)](https://discord.gg/WV4Aac8uZg)

  </p>

  <p>

  [![Tests](https://github.com/nxthat/nanocld/actions/workflows/tests.yml/badge.svg)](https://github.com/nxthat/nanocld/actions/workflows/tests.yml)
  [![Clippy](https://github.com/nxthat/nanocld/actions/workflows/clippy.yml/badge.svg)](https://github.com/nxthat/nanocld/actions/workflows/clippy.yml)

  </p>

  <p>

  [![Crate.io](https://img.shields.io/crates/v/nanocld?style=flat)](https://crates.io/crates/nanocld)
  [![Github](https://img.shields.io/github/v/release/nxthat/nanocld?style=flat)](https://github.com/nxthat/nanocld/releases/latest)

  </p>

</div>

<blockquote class="tags">
 <strong>Tags</strong>
 </br>
 <span id="nxtmdoc-meta-keywords">
  Test, Deploy, Monitor, Scale, Orchestrate
 </span>
</blockquote>

## ❓ What is nanocld ?

Nanocld, stand for `Nano Cloud Daemon` and it's a lie because you will be able to create big ones. <br />
I see it as an open source [Hybrid Cloud Orchestrator](https://docs.next-hat.com/docs/guides/nanocl/overview) ! <br />
To help orchestrate `containers` and `virtual machines` on multiple hosts. <br />
It provides basic mechanisms for deployment, maintenance, and scaling. <br />
You will be able to create an your own `Hybrid Cloud` with optional `Vpn` and `Dns` controllers. <br />
Deploying your `applications` and `servers` behind a `Vpn` as never been that easy !

Builds upon `Rust` to have the best performance and a smallest footprint. <br />
It's use the best ideas and practices from the community. <br />
You can build an entire CI/CD pipeline from `tests` to `high availability production`. <br />
See it as a `Kubernetes` alternative with more `features` and a `network security layer`.

This repository is the `DAEMON` version you can see the `CLI` [here](https://github.com/nxthat/nanocl).

## 📙 Overview

<img src="https://download.next-hat.com/ressources/images/infra.png" />

## ✨ Features
- [x] Manage clusters (CRUD)
- [x] Manage networks (CRUD)
- [x] Manage containers (CRUD)
- [x] Http proxy
- [x] Udp/Tcp proxy
- [x] Monitor http request
- [x] Single-node mode
- [x] Store a git repository state as image
- [ ] Highly-scalable distributed node mode
- [ ] Manage virtual machine (CRUD)
- [ ] Monitor tcp/udp packets

## 🎉 Let's get started

- [Installation](https://docs.next-hat.com/docs/setups/nanocl)
- [Tutorial](https://docs.next-hat.com/docs/guides/nanocl/get-started)

## 🔨 Contribution

If you want to contribute see [Build From Source](https://docs.next-hat.com/docs/setups/nanocl/linux/from-sources)
section on our official documentation to see how to setup a dev environnement for nanocl
