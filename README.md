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
    Next generation cloud-native converged message queue.
</h3>

> Tips:</br>
> - This project is currently in its early preview stage and is undergoing rapid iteration and testing. A stable release is expected in the second half of 2025.</br>
> - We are still growing—please give us time to mature. Our ambition is for RobustMQ to become the next top-level Apache project in the message queue ecosystem.</br>

## 🚀 Introduction

RobustMQ is a next-generation, high-performance, multi-protocol message queue system built in Rust. Our vision is to create a unified messaging infrastructure tailored for AI systems.

The goal of RobustMQ is to deliver a Rust-based message queue that supports multiple mainstream messaging protocols while embracing a fully serverless architecture. By integrating multi-protocol compatibility, high performance, and a distributed, elastic design, RobustMQ provides a unified and efficient communication foundation for AI applications—reducing architectural complexity while improving system performance and reliability.

We have long aimed to support multiple protocols within a truly serverless architecture. At the same time, we strive to keep the architecture simple and adaptable to various deployment scenarios and operational requirements, ultimately reducing the cost of deployment, operations, and usage.
<picture>

  <source
    media="(prefers-color-scheme: dark)"
    srcset="
      https://api.star-history.com/svg?repos=robustmq/robustmq&type=Date&theme=dark
    "
  />
  <source
    media="(prefers-color-scheme: light)"
    srcset="
      https://api.star-history.com/svg?repos=robustmq/robustmq&type=Date
    "
  />
  <img
    alt="Star History Chart"
    src="https://api.star-history.com/svg?repos=robustmq/robustmq&type=Date"
  />
</picture>

## 💡 Features

- **100% Rust**: The message queuing core is implemented entirely in Rust for safety, performance, and reliability.
- **Multi-Protocol Support**: Compatible with MQTT 3.1/3.1.1/5.0, AMQP, RocketMQ Remoting/gRPC, Kafka protocol, OpenMessaging, JNS, SQS, and other mainstream messaging protocols.
- **Layered Architecture**: Follows a three-tier design—compute, storage, and scheduling—where each layer can scale independently and supports clustered deployments for horizontal scalability.
- **Pluggable Storage Layer**: Offers a standalone, pluggable storage architecture. Users can choose the most suitable storage backend, with compatibility across traditional and cloud-native environments, supporting both cloud and IDC deployments.
- **High-Cohesion Design**: Includes built-in metadata storage and distributed journal storage services, enabling fast, cohesive, and easy deployment.
- **Feature-Rich Messaging**: Supports a wide range of advanced messaging features such as sequential messages, dead-letter messages, transactional messages, idempotent messages, and delayed messages.

> In the first phase (through the end of 2025), RobustMQ will initially support RobustMQ MQTT.

## Architecture

RobustMQ adopts a typical distributed, layered architecture composed of four main components:

- Control Layer (Placement Center)
- Computing Layer (Multi-Protocol Processing Layer)
- Storage Adapter Layer
- Standalone Remote Storage Engine

Each layer is independently designed, allowing for rapid horizontal scaling and dynamic resource allocation. This modular separation enables RobustMQ to achieve full serverless capabilities across the entire system, providing flexibility, scalability, and resilience in diverse deployment environments.

![image](docs/images/robustmq-architecture.png)

- Metadata Service

The metadata storage and scheduling component of the RobustMQ cluster. It is responsible for managing cluster metadata, node registration, configuration storage and distribution, as well as task scheduling. Key responsibilities include cluster node discovery, configuration propagation, and orchestration of cluster-wide behaviors.

- Multi-protocol computing layer

This is the Broker Cluster—the computing layer of the RobustMQ system. It handles adaptation and processing of various messaging protocols, and implements core messaging features. Incoming messages are forwarded to the storage layer via the Storage Adapter Layer after protocol-specific processing.

- Storage Adapter Layer

This layer provides abstraction and translation between messaging semantics and storage backends. It unifies various protocol concepts such as Topic, Queue, and Partition into the internal Shard model. It also interfaces with different storage systems including local file systems, HDFS, object storage, or custom-built storage engines—ensuring message data is persistently stored to the appropriate backend.

- Standalone storage engine

Refers to external, independent storage engines such as cloud object storage (e.g., AWS S3), HDFS clusters, or data lake platforms (e.g., Apache Iceberg, Hudi). Similar to RobustMQ's Journal Server or Apache BookKeeper, this layer provides distributed, segmented, high-performance, and reliable message data storage. It supports seamless horizontal scaling and is designed to operate transparently to upstream layers.

## RobustMQ MQTT

RobustMQ MQTT is a full-featured implementation of the MQTT protocol built on the RobustMQ platform. Designed in Rust, it aims to provide a high-performance, enterprise-grade message queuing solution that supports clustered deployment at scale. The long-term vision is to deliver a product that rivals leading enterprise MQTT brokers such as **EMQX** and **HiveMQ**.

![img](docs/images/console-start.png)

1. [RobustMQ Quick Start](https://robustmq.com/QuickGuide/Overview.html)
2. [RobsustMQ MQTT Doc](https://robustmq.com/RobustMQ-MQTT/Overview.html)
3. [RobustMQ MQTT Command](https://robustmq.com/RobustMQ-Command/Mqtt-Broker.html)
4. [Robustmq Web UI](https://github.com/robustmq/robustmq-copilot)

![img](docs/images/web-ui.png)
## Run Test Cases

[Run Test Cases](https://robustmq.com/Architect/Test-Case.html)

## Packaging

[Build & Packaging](https://robustmq.com/QuickGuide/Overview.html)

## Contribution Guidelines

[GitHub Contribution Guide](https://robustmq.com/ContributionGuide/GitHub-Contribution-Guide.html)

## Contact us

- **Discord Group**: Join our community on Discord for discussions, questions, and collaboration 👉 [Discord Link](https://discord.gg/sygeGRh5)
- **Wechat Group**: If you're interested in contributing to the project or discussing development topics, scan the QR code below to join our WeChat group for real-time discussion and collaboration:
<div align="center">
  <img src="docs/images/WechatGroup.jpg" alt="WeChat QR Code" width=200 />
</div>

- **Personal wechat**: The WeChat group QR code is updated periodically. If the group QR code has expired, you can add the developer's personal WeChat below to be invited directly:
<div align="center">
  <img src="docs/images/wechat.jpg" alt="WeChat QR Code" width=200 />
</div>

## License

RobustMQ uses the Apache 2.0 license to strike a balance between open contributions and allowing you to use the software however you want
