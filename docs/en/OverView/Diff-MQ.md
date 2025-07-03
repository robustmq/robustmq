## Overview

This article aims to clarify the differences between **RobustMQ** and current mainstream message queues such as **Kafka, RabbitMQ, and RocketMQ**.

> **Note:** RobustMQ is still in early development. This article focuses on its **positioning, vision, and planned product capabilities** to highlight the differences. Some features of RobustMQ are still under development. The goal is to provide a clear comparison between RobustMQ and existing MQ systems. The content reflects personal understanding and may contain bias—feedback and discussion are welcome.

## In One Sentence

**RobustMQ** aims to be a **modern message queue platform** implemented in **Rust**, supporting **Kafka/AMQP/MQTT protocols** with a **highly elastic architecture**. Its goal is to replace and unify the functionality and ecosystems of multiple traditional MQ systems.

## Core Differences

| Feature / System                 | **RobustMQ**                                                | **Kafka**                          | **RabbitMQ**                       | **RocketMQ**                          |
| -------------------------------- | ----------------------------------------------------------- | ---------------------------------- | ---------------------------------- | ------------------------------------- |
| **Protocol Support**             | ✅ Kafka / AMQP / MQTT / Redis                               | ❌ Kafka only                       | ✅ AMQP, MQTT via plugins           | ❌ Custom protocol, partial Kafka SDK  |
| **Message Models**               | ✅ Pub/Sub, queue, delay, broadcast, P2P, priority           | Topic + offset (log-based)         | Pub/Sub, work queues               | Topic, delay, transaction, ordered    |
| **Architecture**                 | ✅ Decoupled storage/compute/scheduling, serverless-friendly | Monolithic (Zookeeper/KRaft-based) | Clustered, mirrored queues         | Broker + NameServer                   |
| **Storage Support**              | ✅ Pluggable (Redis, local, S3, MinIO, memory)               | Disk-based log                     | In-memory + disk (Mnesia + queues) | Custom CommitLog + internal buffering |
| **Language Implementation**      | Rust (high performance + memory safety)                     | Java / Scala                       | Erlang                             | Java                                  |
| **Ease of Deployment**           | ✅ Works on single node / containers / serverless            | Complex (Zookeeper/KRaft required) | Easy but plugin-heavy              | Requires NameServer + Broker config   |
| **Ecosystem Compatibility**      | ✅ Native support for Kafka / MQTT / AMQP tooling            | Rich Kafka ecosystem               | Rich AMQP ecosystem                | Limited compatibility with Kafka      |
| **Multi-tenancy / Protocol Mix** | ✅ Natively supported                                        | ❌ Requires separate clusters       | ❌ Plugin-based isolation           | ❌ No multi-protocol support           |

## Key Advantages

### **Unified Multi-Protocol Access**

* Kafka in, MQTT out; AMQP in, Kafka out — all combinations supported.
* No need for multiple MQ systems or protocol bridges.

### **Unified Support for Multiple Messaging Models**

* Built-in support for delayed messages, broadcast, priority queues, dead-letter queues, and peer-to-peer messaging.
* No plugins or external systems required.

### **Modern, Cloud-Native Architecture**

* Fully decoupled storage, compute, and scheduling layers.
* Stateless scheduler service supports serverless environments.
* Pluggable storage options (S3, MinIO, Redis, etc.).

### **Developer-Friendly**

* No need to learn new protocols — just use existing Kafka / MQTT / AMQP SDKs.
* Unified REST API, web console, and CLI tools available.


### **Easy Migration, Low Integration Cost**

* Seamlessly integrates with existing Kafka / RabbitMQ / MQTT clients — no code changes needed.
* Supports gradual migration from legacy MQ systems.


## Summary

| Comparison Area            | Kafka / RabbitMQ / RocketMQ         | **RobustMQ Advantages**                      |
| -------------------------- | ----------------------------------- | -------------------------------------------- |
| **Protocol Support**       | Single protocol                     | ✅ Unified multi-protocol                     |
| **Message Models**         | Partial, plugin-dependent           | ✅ Full native model support                  |
| **Developer Integration**  | SDKs or plugins required            | ✅ Use existing open-source SDKs directly     |
| **Operational Complexity** | Complex (ZK, NameServer, mirroring) | ✅ Lightweight, modular deployment            |
| **Migration Cost**         | High (client code rewrites needed)  | ✅ Low (supports native Kafka/MQTT/AMQP SDKs) |


