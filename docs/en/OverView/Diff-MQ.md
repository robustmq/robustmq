# RobustMQ vs Mainstream Message Queues (Kafka/RocketMQ/Pulsar) Comparison

> **Core Difference**: Traditional message queues (Kafka, RocketMQ, Pulsar) each focus on specific protocols and scenarios, typically adopting Java/JVM implementations and tightly coupled architectures. RobustMQ, as an emerging cloud-native message middleware, focuses on multi-protocol unification (MQTT/Kafka/AMQP/RocketMQ) and compute-storage separation, implemented in Rust, featuring zero GC and low resource consumption, single-binary deployment with no external dependencies, providing a simpler architecture and lower operational complexity.

Kafka, RocketMQ, and Pulsar represent different generations and design philosophies in message queue technology. Kafka focuses on streaming data and log aggregation, RocketMQ focuses on transactional messages and order guarantee, and Pulsar focuses on compute-storage separation and multi-tenancy. These mature projects have extensive production validation but also face challenges of complex architecture, high operational costs, and protocol fragmentation. RobustMQ, emerging from cloud-native requirements, features multi-protocol unification (MQTT/Kafka/AMQP/RocketMQ), single-binary deployment with zero dependencies, Rust zero GC, and pluggable storage design, providing simpler architecture and lower operational thresholds. This document provides a detailed comparison of their positioning, architecture, features, and applicable scenarios.

---

## 1. Positioning & Design Philosophy

**Kafka** is positioned as a distributed streaming platform, designed for high-throughput log aggregation and real-time data pipelines. The project uses Scala/Java, with Broker + ZooKeeper architecture, supporting the Kafka protocol. Adopts local disk sequential write, optimizing throughput performance. Rich ecosystem including Kafka Connect, Kafka Streams, KSQL. Suitable for big data pipelines, log collection, streaming analytics.

**RocketMQ** is positioned as a distributed message middleware, designed for transactional messages and order guarantee. The project uses Java, with Broker + NameServer architecture, supporting the RocketMQ protocol. Provides transactional messages, delayed messages, sequential messages, and other enterprise features. Suitable for financial transactions, order processing, system decoupling.

**Pulsar** is positioned as a next-generation distributed messaging and streaming platform, designed for multi-tenancy and compute-storage separation. The project uses Java, with Broker + BookKeeper + ZooKeeper architecture, supporting the Pulsar protocol. Provides streaming + queueing dual model, tiered storage, geo-replication. Suitable for large enterprise multi-tenant scenarios, IoT platforms.

**RobustMQ** is positioned as a cloud-native, multi-protocol unified message middleware. The project is implemented in Rust, leveraging Rust's memory safety, zero-cost abstraction, and concurrency performance advantages to achieve zero GC and low resource consumption. RobustMQ's core goal is to solve the protocol fragmentation problem, by supporting MQTT, Kafka, AMQP, RocketMQ and other protocols in a single system, eliminating the complexity of maintaining multiple message queue systems. Single-binary deployment, with no external dependencies, makes deployment extremely simple. The layered architecture design enables independent scaling of compute and storage layers, while pluggable storage supports flexible adaptation from edge to cloud scenarios.

---

## 2. Architecture Design Comparison

**Traditional Message Queue** architectural characteristics: Kafka adopts Broker + ZooKeeper architecture, with messages stored in local disk logs; RocketMQ adopts Broker + NameServer architecture, with storage also based on local disks; Pulsar adopts Broker + BookKeeper + ZooKeeper architecture, implementing compute-storage separation but with many components. These architectures typically tightly bind specific protocols to storage models, requiring additional bridging solutions for cross-protocol support.

**RobustMQ** architectural characteristics: Adopts Broker + Journal + Metadata Service three-tier architecture, implementing compute-storage separation. The project is implemented in Rust, leveraging Rust's memory safety, zero-cost abstraction, and concurrency performance advantages to achieve zero GC and low resource consumption. Single-binary deployment, with no external dependencies, makes deployment extremely simple. Broker handles protocol processing and message routing, Journal Server handles message persistence, Metadata Service manages cluster metadata based on Raft. The storage layer adopts a pluggable design, supporting multiple backends such as local files, S3, HDFS, and MinIO, allowing flexible selection based on business needs.

| Dimension | Kafka | RocketMQ | Pulsar | RobustMQ |
|------|-------|----------|--------|----------|
| **Architecture Pattern** | Broker + ZooKeeper<br>Compute-storage coupled | Broker + NameServer<br>Compute-storage coupled | Broker + BookKeeper + ZK<br>Compute-storage separated | Broker + Journal + Meta<br>Compute-storage separated |
| **Development Language** | Scala/Java | Java | Java | Rust |
| **Protocol Support** | Kafka protocol | RocketMQ protocol | Pulsar protocol<br>Plugin support for Kafka/MQTT/AMQP | MQTT/Kafka/AMQP/RocketMQ<br>Natively unified |
| **Storage Model** | Local disk log (sequential write) | Local disk | BookKeeper (distributed storage) | Pluggable: local/S3/HDFS/MinIO |
| **Scalability** | Supports partition scaling<br>Compute-storage coupled | Supports scaling<br>Cluster operations complex | Independent scaling<br>Multiple components | Independent scaling<br>Multi-tenancy support |
| **Deployment Complexity** | K8s Operator<br>Complex configuration | K8s Operator<br>Complex configuration | K8s Operator<br>Multiple components | Single binary, no dependencies<br>K8s Operator + Dashboard |

---

## 3. Core Features & Characteristics Comparison

**Kafka** features high throughput (millions msg/s), low latency (milliseconds), and sequential write optimization. Adopts partition + replica mechanism, supports horizontal scaling. Provides rich ecosystem: Kafka Connect (data integration), Kafka Streams (stream processing), KSQL (streaming SQL). Uses Java/JVM, GC may affect latency spikes. Suitable for log collection, streaming analytics, event-driven architecture. Challenges include: complex cluster configuration and operations; ZooKeeper dependency adds operational burden; only supports Kafka protocol.

**RocketMQ** features transactional messages, ordered messages, delayed messages, and high reliability. Adopts NameServer for routing, master-slave replication. Provides rich message filtering, retry mechanisms, dead letter queues. Uses Java/JVM, GC may affect latency spikes. Suitable for order processing, financial transactions, system decoupling. Challenges include: complex cluster operations, lacking compute-storage separation; only supports RocketMQ protocol; ecosystem smaller than Kafka.

**Pulsar** features compute-storage separation (Broker + BookKeeper), multi-tenancy isolation (Tenant/Namespace), tiered storage (hot/cold data separation). Provides streaming + queueing dual model, geo-replication, Pulsar Functions. Uses Java/JVM, GC may affect latency spikes. Suitable for large enterprise multi-tenant scenarios, IoT platforms, streaming analytics. Challenges include: complex architecture, multiple components, high resource consumption; Kafka/MQTT support requires additional plugins; operations and maintenance require dedicated teams.

**RobustMQ** features multi-protocol unification (MQTT/Kafka/AMQP/RocketMQ), compute-storage separation, pluggable storage, single-binary deployment. Implemented in Rust, achieving zero GC and predictable latency. Provides over 8 data integration connectors (Kafka/Pulsar/MySQL/MongoDB, etc.). Optimized for AI scenarios, including AI training pipelines, real-time inference. Offers complete cloud-native support including Kubernetes Operator, Helm charts, and Serverless architecture. Suitable for IoT platforms, microservices communication, AI data pipelines. Challenges include: the project is in active development; the community size is smaller compared to established projects.

| Feature Dimension | Kafka | RocketMQ | Pulsar | RobustMQ |
|------|-------|----------|--------|----------|
| **Protocol Support** | Kafka protocol | RocketMQ protocol | Pulsar protocol<br>Plugin support for other protocols | MQTT (supported) / Kafka (in development)<br>AMQP (planned) / RocketMQ (planned)|
| **Performance** | High throughput, million msg/s<br>Java GC may affect latency | High throughput, hundred thousand msg/s<br>Java GC may affect latency | High throughput, million msg/s<br>Java GC may affect latency | High throughput, million msg/s<br>Zero GC, predictable latency |
| **Message Model** | Pub/Sub (partitioned) | Pub/Sub + P2P<br>Ordered/transactional/delayed messages | Streaming + queueing dual model<br>Multiple subscription modes | Pub/Sub + queue<br>Delayed messages |
| **Storage Architecture** | Local disk log<br>Sequential write optimization | Local disk<br>Master-slave sync | BookKeeper tiered storage<br>Supports long-term storage | Pluggable storage<br>Supports local/S3/HDFS |
| **Enterprise Features** | Mature stream processing API<br>Rich connectors | Transactional messages<br>Ordered messages | Multi-tenant isolation<br>Geo-replication | Multi-protocol unified<br>Basic multi-tenancy in development |
| **Cloud-Native** | K8s Operator<br>Complex configuration | K8s Operator<br>Complex configuration | K8s Operator<br>Multiple components | Single binary deployment, no dependencies<br>K8s Operator + Dashboard |
| **Data Integration** | Kafka Connect<br>Rich connectors | RocketMQ Connect | Pulsar IO + Functions | 8+ Bridge connectors |

---

## 4. Community & Development Stage

| Dimension | Kafka | RocketMQ | Pulsar | RobustMQ |
|------|-------|----------|--------|----------|
| **Project Status** | Apache top-level project | Apache top-level project | Apache top-level project | Active open-source project |
| **Maturity** | Production-ready, mature | Production-ready, mature | Production-ready, mature | Core MQTT supported, Kafka in development |
| **Community Size** | 28k+ stars, extensive production use | 21k+ stars, widely used domestically | 14k+ stars, large enterprise user base | 1.4K+ stars, rapidly growing community |

---

## 5. Performance Comparison

| Metric | Kafka | RocketMQ | Pulsar | RobustMQ |
|------|-------|----------|--------|----------|
| **Latency** | Millisecond<br>GC may cause spikes | Millisecond<br>GC may cause spikes | Millisecond<br>GC may cause spikes | Microsecond (memory)<br>Millisecond (SSD/S3) |
| **Throughput** | Million msg/s | Hundred thousand msg/s | Million msg/s | Million msg/s |
| **Concurrency** | High concurrency | High concurrency | High concurrency | High concurrency (Tokio async) |
| **Resource Usage** | High (JVM + ZooKeeper) | High (JVM + NameServer) | High (JVM + multiple components) | Low (Zero GC, single binary) |

---

## 6. Applicable Scenarios Comparison

**Kafka** features mature stream processing API (Kafka Streams), rich ecosystem (Connect/KSQL), and high throughput. Suitable for big data pipelines, log aggregation, real-time analytics, event-driven architecture. Less suitable for scenarios requiring low latency (GC impact), transactional messages (needs third-party components), multi-protocol support (only Kafka protocol).

**RocketMQ** features transactional messages, ordered messages, delayed messages, and high reliability. Suitable for order processing, financial transactions, system decoupling, payment scenarios. Less suitable for streaming analytics (weaker than Kafka), multi-protocol support (only RocketMQ protocol), scenarios requiring compute-storage separation.

**Pulsar** features compute-storage separation, multi-tenancy isolation, tiered storage, and geo-replication. Suitable for large enterprise multi-tenant scenarios, IoT platforms, streaming analytics, scenarios requiring long-term storage. Less suitable for small teams (complex operations), resource-constrained scenarios (high resource consumption), scenarios requiring simple deployment.

**RobustMQ** features multi-protocol unification (MQTT/Kafka/AMQP), single-binary deployment, zero GC, and pluggable storage. Suitable for IoT platforms (native MQTT), microservices communication (multi-protocol support), AI data pipelines (data integration), edge computing (low resource consumption). Less suitable for scenarios requiring mature multi-tenancy (in development), scenarios requiring rich stream processing APIs (in development).

---

## 7. Summary & Recommendations

**Kafka** is positioned as a distributed streaming platform, featuring high throughput, mature ecosystem, rich connectors. Uses Scala/Java, with Broker + ZooKeeper architecture. Suitable for big data pipelines, log aggregation, streaming analytics. Operations are relatively complex, only supports Kafka protocol.

**RocketMQ** is positioned as a distributed message middleware, featuring transactional messages, ordered messages, delayed messages. Uses Java, with Broker + NameServer architecture. Suitable for order processing, financial transactions, system decoupling. Ecosystem is smaller than Kafka, only supports RocketMQ protocol.

**Pulsar** is positioned as a next-generation distributed messaging and streaming platform, featuring compute-storage separation, multi-tenancy, tiered storage. Uses Java, with Broker + BookKeeper + ZooKeeper architecture. Suitable for large enterprise multi-tenant scenarios, IoT platforms. Architecture is complex, operations require dedicated teams.

**RobustMQ** is positioned as a cloud-native, multi-protocol unified message middleware, featuring multi-protocol support (MQTT/Kafka/AMQP), single-binary deployment, zero GC, pluggable storage. Uses Rust, with Broker + Journal + Meta architecture. Suitable for IoT platforms, microservices communication, AI data pipelines. Project is in active development, community is growing.

**Recommended Scenarios**

- **Kafka**: Big data pipelines, log aggregation, real-time analytics, event-driven architecture. Suitable for scenarios requiring mature stream processing APIs and rich ecosystem.

- **RocketMQ**: Order processing, financial transactions, payment scenarios, system decoupling. Suitable for scenarios requiring transactional messages and ordered messages.

- **Pulsar**: Large enterprise multi-tenant scenarios, IoT platforms, scenarios requiring long-term storage. Suitable for scenarios requiring compute-storage separation and geo-replication.

- **RobustMQ**: IoT platforms, microservices communication, AI data pipelines, edge computing. Suitable for scenarios requiring multi-protocol support and simple deployment.

---

## 8. References

- [Apache Kafka Official Site](https://kafka.apache.org)
- [Apache RocketMQ Official Site](https://rocketmq.apache.org)
- [Apache Pulsar Official Site](https://pulsar.apache.org)
- [RobustMQ Official Site](https://robustmq.com)
- [RobustMQ GitHub](https://github.com/robustmq/robustmq)
- [RobustMQ Documentation](https://robustmq.com/en/)
