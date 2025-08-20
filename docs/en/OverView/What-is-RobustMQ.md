# RobustMQ — A Next-Generation Cloud-Native Message Queue Built with Rust

## Logo

![image](../../../docs/images/robustmq-logo.png)

## Background & Motivation: Why Build RobustMQ

With the rapid development of distributed systems and cloud-native infrastructure, **Message Queues (MQs)** have become a core component of modern enterprise IT architectures. Whether in finance, e-commerce, IoT, or real-time data processing, message queues are indispensable.
However, today’s mainstream MQs (Kafka, RocketMQ, Pulsar, etc.) have revealed several persistent issues over long-term use:

* **The contradiction between performance and cost**
  Kafka is powerful, but in large-scale cluster scenarios, its deployment and maintenance costs are extremely high, and elasticity in scaling remains limited.

* **Fragmented protocols, scattered ecosystem**
  Different business scenarios often require different protocols (e.g., MQTT for IoT, Kafka for stream processing, AMQP for traditional systems), forcing enterprises to maintain multiple MQ systems in parallel.

* **Tightly coupled architectures, lack of flexibility**
  Most MQs tightly couple compute and storage, making it difficult to adapt to diverse storage backends like object storage (S3) or distributed file systems (HDFS).

* **Poor user experience**
  Complex deployment and operations demand extra manpower and steep learning curves, lacking a true “out-of-the-box” cloud-native experience.

RobustMQ was created to address these pain points. It is not only a high-performance MQ experiment, but also an exploration of **how Rust can be fused with cloud-native message queues**.

---

## Architecture & Design Principles

The core design philosophy of RobustMQ can be summarized as:
**High performance, multi-protocol unification, compute–storage decoupling, pluggable storage, cloud-native friendliness.**

### 1. High-Performance Core — Powered by Rust

RobustMQ is fully written in **Rust 🦀**. Rust’s zero-cost abstractions, memory safety, and powerful concurrency model give RobustMQ inherent advantages in performance and reliability.

* Networking built on **Tokio Runtime** + Reactor model, supporting massive concurrent connections.
* Storage and consensus modules based on **RocksDB + Raft (OpenRaft)** for high availability and consistency.
* No GC pauses, low and predictable latency, ideal for latency-sensitive domains like finance and IoT.

### 2. Multi-Protocol Unification — One System, Many Languages

RobustMQ integrates multiple protocol codecs, with different protocols accessed via dedicated ports:

* **MQTT** (1883/1884/8083/8084, supporting MQTT 3/4/5, IoT-friendly)
* **Kafka** (9092, planned Producer/Consumer support)
* **gRPC** (1228, for management and control plane)
* **RocketMQ, AMQP** (planned)

This design eliminates the need to deploy separate MQ systems for different scenarios — truly **“one deployment, multi-protocol availability.”**

### 3. Compute–Storage Separation — Cloud-Native Decoupling

Unlike traditional MQs with tightly coupled compute and storage, RobustMQ adopts a **Broker + Journal + Metadata Service** architecture with natural separation:

* **Broker**: Protocol parsing, client connection management, message routing.
* **Journal**: Message persistence and log management, supporting multiple storage backends.
* **Metadata Service**: Built on Raft and RocksDB, ensures high availability for metadata such as topics, offsets, and sessions.

This decoupled design brings stronger scalability and native support for **multi-tenancy and elastic scaling**.

### 4. Pluggable Storage — Adapt to Any Scenario

RobustMQ abstracts its storage layer via a unified Trait, enabling pluggable backends:

* Local file storage (high performance, low cost)
* S3 object storage (cloud-native)
* HDFS (big data)
* More engines in the future

A **WAL (Write-Ahead Log)** mechanism boosts read/write performance and consistency, especially for object storage.

### 5. Cloud-Native Friendly — Simplified Deployment & Ops

* **One-command start**: A single binary runs everything (`bin/robust-server start`).
* **Container-ready**: Planned Docker images and Kubernetes Operator for large-scale cluster management.
* **Visual Dashboard**: Cluster monitoring, topic/session/connector management, user management, reducing operational complexity.

---

## Vision: Building the “Unified Messaging Infrastructure” for the Cloud-Native Era

RobustMQ’s long-term vision is:

* **To become the unified messaging platform of cloud-native infrastructure** — freeing users from choosing between Kafka and MQTT by delivering one system that covers all.
* **To make message queues as easy to use as databases** — simpler deployment, lighter operations, elastic scaling, and dramatically lower barriers to adoption.
* **To drive Rust adoption in infrastructure** — exploring Rust’s best practices in distributed MQ systems for greater safety and efficiency.

Milestones:

1. **Short term (before 2025)**: Replace widely used community MQTT brokers with a stable production-ready version.
2. **Mid term**: Add Kafka, RocketMQ, and AMQP support to form a true multi-protocol unified platform.
3. **Long term**: Optimize the storage layer, fully embrace object and distributed storage, and evolve into a universal messaging hub.

---

## Core Competitiveness

1. **Rust performance & safety** — Memory-safe, zero-cost abstractions, highly concurrent.
2. **Multi-protocol unification** — One cluster supports multiple protocols, avoiding fragmentation.
3. **Pluggable storage** — Flexibly adapts to business scenarios, cloud-native ready.
4. **Decoupled distributed architecture** — Independent compute, storage, and metadata layers with excellent scalability and elasticity.
5. **Simplified user experience** — Single binary, one-click deployment, visual dashboard, Kubernetes-native in the future.
6. **Community-driven** — Already 1000+ stars, 50+ contributors, 2100+ commits, and growing rapidly.

---

📌 **In one sentence**:
RobustMQ is not “reinventing Kafka,” but rather leveraging **Rust + cloud-native architecture** to build a faster, more modern, and more user-friendly **next-generation unified messaging platform**.

