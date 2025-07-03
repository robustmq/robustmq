**Apache Iggy** and **RobustMQ** are both modern message queue systems written in **Rust**, emphasizing high performance, scalability, and developer-friendliness. Currently, Iggy has joined the Apache Software Foundation. RobustMQ is often compared with Iggy, and people frequently ask about the differences between them.

> **Note:** RobustMQ is in the early stages of development. This article mainly discusses the differences between Iggy and RobustMQ from the perspectives of RobustMQ's positioning, vision, and product capability planning. Some of RobustMQ's features are still under development. The goal of this article is to clarify the differences between Iggy and RobustMQ. The comparisons are based on personal understanding and may be biased—feedback and discussion are welcome.

## Summary

### Positioning

* **Iggy** is a high-performance, high-throughput streaming platform that can be seen as an alternative to Kafka. More precisely, it's a simplified and modern Kafka-style message queue system. It is a **new message queue platform with its own protocol and SDK**.

* **RobustMQ** is designed as an **all-in-one message queue** that adapts to current mainstream messaging protocols (Kafka, AMQP, MQTT, etc.) and is built with a fully **serverless architecture**. Its goal is to be compatible with existing protocols, SDKs, and ecosystems, while solving current messaging system problems related to elasticity, cost, and fragmented features through architecture upgrades (Rust performance/safety, compute/storage/scheduling separation, pluggable storage).


### Technical Philosophy

* **Architecture**: Iggy uses a monolithic (compute+storage) architecture, while RobustMQ adopts a **decoupled and layered architecture** (compute/storage/scheduling separation), providing greater elasticity.
* **Protocol and SDK**: Iggy uses a **custom protocol** and requires custom multi-language SDKs. RobustMQ uses standard open protocols and does **not** require private SDKs.
* **Storage Layer**: Iggy uses **local append-only log storage**, while RobustMQ supports **pluggable storage**, including HDFS, S3, MinIO, memory, and local storage. RobustMQ offers more flexibility and elasticity.
* **Client Integration**: Iggy requires clients to use its specific SDK. RobustMQ supports out-of-the-box use of open-source **Kafka, AMQP, or MQTT SDKs**, making it easier to integrate.
* **Ecosystem Compatibility**: Iggy requires users to adapt to a new ecosystem. RobustMQ is **fully compatible with mainstream MQ ecosystems and tools**, with near-zero switching cost.

## Core Design Philosophy Comparison

### Iggy's Philosophy

* A **simple yet modern Kafka alternative**
* Lightweight and suitable for embedded or self-hosted use
* Avoids complexity from ZooKeeper/KRaft
* Focused on performance and Rust-native safety

### RobustMQ's Philosophy

* A **unified enterprise message backbone** for multi-protocol, multi-model messaging
* One system to handle **Kafka / MQTT / AMQP** protocols
* Supports complex deployments (cloud-edge collaboration, decoupled architecture)
* More like a **modern enterprise service bus**

## Feature Comparison Table

| Feature / Item          | **Apache Iggy**                           | **RobustMQ**                                                 |
| ----------------------- | ----------------------------------------- | ------------------------------------------------------------ |
| **Language**            | Rust                                      | Rust                                                         |
| **Positioning**         | Kafka replacement (modern, lightweight)   | Unified message hub with multi-protocol support              |
| **Protocol Support**    | Custom protocol (lightweight), TCP, HTTP  | Kafka / AMQP / MQTT and more                                 |
| **SDKs**                | Iggy's own SDK                            | Kafka / AMQP / MQTT standard open-source SDKs                |
| **Topic & Partition**   | Supported                                 | Supported with fine-grained control                          |
| **Storage Layer**       | Local file-based append-only log          | Pluggable: local file, S3, MinIO, memory, etc.               |
| **Architecture**        | Leader-follower, stream-oriented          | Layered: storage/compute/scheduling separated                |
| **Distributed Support** | Experimental (basic clustering)           | Full distributed support with scaling and replication        |
| **Performance**         | Memory-mapped files, Rust-optimized       | Same (Rust, mmap, optimized pipeline)                        |
| **Message Model**       | Kafka-style (topic + offset)              | Unified model (Pub/Sub, queue, delayed, broadcast)           |
| **Use Cases**           | High-throughput log streaming, Kafka alt. | Multi-source integration, edge computing, IoT, microservices |
| **Advanced Features**   | Focused on throughput                     | Supports broadcast, private messages, priority queues        |
| **Ecosystem**           | CLI, HTTP, SDK in Iggy's ecosystem        | Fully compatible with Kafka / AMQP / MQTT tools              |
| **Maturity**            | Medium (actively developed)               | Medium (actively developed)                                  |
| **Integration Cost**    | High (business must adopt Iggy ecosystem) | Low (drop-in compatible with existing protocol SDKs)         |
| **License**             | Apache 2.0                                | Apache 2.0                                                   |
