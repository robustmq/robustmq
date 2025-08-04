<p align="center">
  <picture>
    <img alt="RobustMQ Logo" src="docs/images/robustmq-logo.png" width="300">
  </picture>
</p>

<p align="center">
  <a href="https://deepwiki.com/robustmq/robustmq"><img src="https://deepwiki.com/badge.svg" alt="Ask DeepWiki"></a>
  <img alt="Latest Release" src="https://img.shields.io/github/v/release/robustmq/robustmq?style=flat">
  <img alt="License" src="https://img.shields.io/github/license/robustmq/robustmq?style=flat">
  <img alt="GitHub issues" src="https://img.shields.io/github/issues/robustmq/robustmq?style=flat">
  <img alt="GitHub stars" src="https://img.shields.io/github/stars/robustmq/robustmq?style=flat">
  <a href="https://codecov.io/gh/robustmq/robustmq" >
  <img src="https://codecov.io/gh/robustmq/robustmq/graph/badge.svg?token=MRFFAX9QZO"/>
 </a>
</p>


<h3 align="center">
    A high-performance distributed message queue built with Rust.
</h3>

> Tips:<br/>
> - This project is currently in its early preview stage and is undergoing rapid iteration and testing. A stable release is expected in the second half of 2025.<br/>
> - We are still growing—please give us time to mature. Our ambition is for RobustMQ to become the next top-level Apache project in the message queue ecosystem.<br/>

## 🚀 Introduction

RobustMQ is a next-generation, high-performance, multi-protocol message queue system built in Rust. Our vision is to
create a unified messaging infrastructure tailored for AI systems. features:
- **High Performance**: Built with Rust, ensuring speed, efficiency and security.
- **Distributed architecture**: Computing, storage and scheduling are separated. Targeting cluster deployment, allowing for rapid expansion and contraction of capacity.
- **Multi-protocol**: Supports mainstream messaging protocols such as MQTT, AMQP, Kafka, and RocketMQ.
- **Plugin-based storage**: The storage layer is modularized, allowing for the selection of the appropriate storage engine based on the scenario, such as file storage, S3, HDFS, etc.
- **User-Friendly**: Designed with simplicity in mind, making it easy to deploy and manage.
- **Multi-tenancy**: In a physical cluster, multiple virtual clusters are allowed, thereby reducing deployment costs.

For more information, please refer to the official [website documentation](https://robustmq.com/).

## Architecture
![image](docs/images/robustmq-architecture.png)
- One binary, one process
- Support multiple network communication protocols
- Multi-protocol encoding and decoding，Different protocols use different ports: MQTT (1883/1884/8083/8084), Kafka (9092), Grpc (1228)
- Plugin-based storage configuration
- Metadata Service Based on Raft and RocksDB

## Quick Start
### Cargo startup
```
$ git clone https://github.com/robustmq/robustmq.git
$ cd robustmq
$ cargo run --package cmd --bin broker-server 
```

### Packaging
```
$ make build
```
Generated binary packages are located in the "build" directory.

### Binary startup
```
$ cd build
$ cd robustmq-0.1.25
$ bin/robust-server start
```

## RobustMQ MQTT
RobustMQ fully supports the MQTT 3/4/5 protocols, aiming to align with the functions of the EMQX enterprise edition.

1. [RobustMQ Quick Start](https://robustmq.com/QuickGuide/Overview.html)
2. [RobustMQ MQTT Doc](https://robustmq.com/RobustMQ-MQTT/Overview.html)
3. [RobustMQ MQTT Command](https://robustmq.com/RobustMQ-Command/Mqtt-Broker.html)
4. [RobustMQ Web UI](https://github.com/robustmq/robustmq-copilot)

![img](docs/images/web-ui.png)


## Contribution Guidelines

[GitHub Contribution Guide](https://robustmq.com/ContributionGuide/GitHub-Contribution-Guide.html)

## Contact Us

- **Discord Group**: Join our community on Discord for discussions, questions, and collaboration 👉 [Discord Link](https://discord.gg/sygeGRh5)
- **WeChat Group**: If you're interested in contributing to the project or discussing development topics, scan the QR code below to join our WeChat group for real-time discussion and collaboration:
<div align="center">
  <img src="docs/images/wechat-group.jpg" alt="WeChat Group QR Code" width=200 />
</div>

- **Personal WeChat**: The WeChat group QR code is updated periodically. If the group QR code has expired, you can add the developer's personal WeChat below to be invited directly:
<div align="center">
  <img src="docs/images/wechat.jpg" alt="WeChat QR Code" width=200 />
</div>

## License
RobustMQ uses the Apache 2.0 license to strike a balance between open contributions and allowing you to use the software however you want.