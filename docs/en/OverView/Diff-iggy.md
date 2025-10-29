# RobustMQ vs Apache Iggy Comparison

> **Core Difference**: Both RobustMQ and Apache Iggy are high-performance message queue systems built with Rust, but they differ significantly in their strategic focus. Iggy pursues extreme performance and simplicity with a custom protocol and proprietary SDK, making it ideal for embedded and edge scenarios. RobustMQ focuses on multi-protocol unification and cloud-native architecture, providing complete ecosystem compatibility and enterprise-grade scalability, suitable for complex production environments requiring multiple message queue protocol support.

RobustMQ and Apache Iggy are modern message queue systems built with Rust, emphasizing high performance, scalability, and developer friendliness. Iggy adopts a monolithic architecture with tightly coupled compute and storage, custom Kafka-style protocol, and proprietary SDK, targeting embedded and edge computing scenarios. RobustMQ adopts a layered decoupled architecture, natively supports multiple standard protocols (MQTT, Kafka, AMQP), uses community standard SDKs, implements compute-storage separation and pluggable storage design, and provides complete distributed capabilities. This document provides a detailed comparison of their positioning, architecture, features, and applicable scenarios.

---

## 1. Positioning & Strategic Goals

**Apache Iggy** is positioned as a high-performance, simplified message streaming platform, focusing on performance and developer experience. The project implements a Kafka-style message queue with Rust, providing millions of messages per second throughput and microsecond-level latency. Iggy's design philosophy emphasizes simplicity, reducing operational burden through a single-binary deployment and zero dependencies. Its architecture design makes it particularly suitable for edge computing, embedded scenarios, and scenarios requiring simple and efficient messaging systems. However, due to the use of a custom protocol and proprietary SDK, compatibility with the existing message queue ecosystem needs to be built from scratch.

**RobustMQ** is positioned as a cloud-native, multi-protocol unified message middleware. The project is implemented in Rust, leveraging Rust's memory safety and zero-cost abstraction features to provide high performance and reliability guarantees. RobustMQ's core goal is to solve the problem of protocol fragmentation in modern architecture, by supporting MQTT, Kafka, AMQP and other protocols in a single system, eliminating the complexity of maintaining multiple message queue systems. The layered architecture design enables independent scaling of compute and storage layers, while pluggable storage supports flexible adaptation from edge to cloud scenarios. RobustMQ provides complete enterprise-level features such as multi-tenancy, data integration, and cloud-native deployment support, suitable for large-scale production environments.

---

## 2. Architecture Design Comparison

**Apache Iggy** adopts a monolithic architecture design, with compute and storage tightly coupled, pursuing lightweight deployment and high performance. The project uses a custom Kafka-style protocol requiring a proprietary SDK for development. The storage layer is implemented based on local append-only logs, optimized for performance through memory mapping. Distributed capabilities are currently in the experimental stage, mainly providing basic master-slave replication. This architectural design makes Iggy suitable for edge computing and embedded scenarios, but ecosystem compatibility needs to be built from scratch.

**RobustMQ** adopts a layered decoupled architecture, completely separating Broker, Meta Service, and Journal Server to achieve compute-storage separation and independent scaling. The project is implemented in Rust, leveraging Rust's memory safety and zero-cost abstraction features to provide high performance and reliability guarantees. Single-binary deployment, with no external dependencies, makes deployment extremely simple. Native support for multiple standard protocols such as MQTT, Kafka, and AMQP, using community standard SDKs, ensures zero migration cost. The storage layer adopts a pluggable design, supporting multiple backends such as memory, SSD, and S3, meeting different performance and cost requirements. Raft consensus-based metadata management provides complete distributed capabilities, including automatic failover and elastic scaling. RobustMQ is specifically designed for cloud-native scenarios, providing Kubernetes Operator and Serverless support.

| Dimension | Apache Iggy | RobustMQ |
|------|------------|----------|
| **Architecture Pattern** | Monolithic architecture, compute-storage coupled<br>Lightweight, embedded deployment | Layered decoupling, Broker/Meta/Journal separation<br>Single binary, cloud-native K8s + Serverless |
| **Protocol & SDK** | Custom protocol (QUIC/TCP/HTTP)<br>Proprietary SDK, requires adaptation | MQTT/Kafka/AMQP multi-protocol<br>Standard SDK, zero learning cost |
| **Storage & Distribution** | Local Append-only log<br>Experimental clustering, master-slave replication | Pluggable storage (Memory/SSD/S3/HDFS)<br>Raft metadata, automatic failover |
| **Deployment Complexity** | Minimalist (single binary, no dependencies) | Minimalist (single binary, no dependencies) |
| **Ecosystem Compatibility** | New ecosystem, requires adapting existing systems | Fully compatible, can replace existing MQ |

---

## 3. Core Features & Characteristics Comparison

**Apache Iggy** features extreme performance with millions of messages per second throughput and microsecond-level latency. Uses zero-copy serialization and memory-mapped files to optimize I/O performance. Supports multiple network protocols (QUIC/TCP/HTTP), TLS encryption, and token-based authentication. Implements Kafka-style features including partitions, consumer groups, and offset management. The project is in active development, with experimental distributed cluster support and multi-tenancy features planned. Iggy's advantages are deployment simplicity (single binary, no dependencies) and performance excellence (sub-millisecond P99 latency). Its main challenges include: custom protocol requiring proprietary SDK, migrating from existing systems requires rewriting client code; distributed capabilities are immature and unsuitable for large-scale clusters; single protocol limits applicability in multi-scenario use.

**RobustMQ** features multi-protocol unification, achieving MQTT, Kafka, AMQP and other protocols within a single system, and using standard open-source SDKs to ensure protocol compatibility. Implements compute-storage separation, allowing independent scaling of Broker and Storage; pluggable storage supporting memory, SSD, S3, HDFS and other backends. The project is implemented in Rust, achieving zero GC and predictable latency. Provides over 8 data integration connectors (Kafka/Pulsar/MySQL/MongoDB, etc.), supporting complex data workflow orchestration. Optimized for AI scenarios, including AI training pipelines, real-time inference, and multi-modal data processing. Offers complete cloud-native support including single-binary deployment, Kubernetes Operator, Helm charts, and Serverless architecture. RobustMQ's advantages are multi-protocol unified management, reduced operational complexity; compute-storage separation architecture enabling elastic scaling; pluggable storage adapting from edge to cloud scenarios; complete distributed capabilities and enterprise-level support. Its main challenges include: the project is in active development, some advanced features are being implemented; the community size is smaller compared to established projects; production case accumulation is ongoing.

| Feature Dimension | Apache Iggy | RobustMQ |
|------|------------|----------|
| **Protocol Support** | Kafka-style (QUIC/TCP/HTTP)<br>Proprietary SDK | MQTT 3.1.1/5.0 (supported) / Kafka (in development) / AMQP (planned)<br>Standard open-source SDK, protocol compatible |
| **Performance** | Sub-millisecond latency, million msg/s<br>Zero-copy serialization | Microsecond latency (memory), million msg/s<br>Zero GC, Tokio async |
| **Storage Architecture** | Local Append-only log<br>Memory-mapped optimization | Pluggable: Memory/SSD/S3/HDFS<br>WAL consistency guarantee |
| **Data Integration** | Not supported | 8+ connectors (Kafka/Pulsar/MySQL/MongoDB, etc.) |
| **Distribution** | Experimental clustering, master-slave replication | Raft metadata, automatic failover, elastic scaling |
| **Cloud-Native** | Basic container support | Single binary deployment<br>K8s Operator + Helm + Serverless |
| **AI Scenarios** | Basic stream processing | AI workflow optimization, training pipelines, real-time inference |

---

## 4. Community & Development Stage

| Dimension | Apache Iggy | RobustMQ |
|------|------------|----------|
| **Project Status** | Currently incubating at Apache | Active open-source project |
| **Maturity** | Alpha stage, core features functional | Core MQTT supported, Kafka in development |
| **Community Size** | 3.2k+ stars, active development | 1.4K+ stars, rapidly growing community |

---

## 5. Performance Comparison

| Metric | Apache Iggy | RobustMQ |
|------|------------|----------|
| **Throughput** | Million msg/s | Million msg/s |
| **Latency** | Sub-millisecond P99 | Microsecond (memory)<br>Millisecond (SSD/S3) |
| **Concurrency** | High concurrency | High concurrency (Tokio async) |
| **Resource Usage** | Low memory footprint | Low memory footprint (Zero GC) |

---

## 6. Applicable Scenarios Comparison

**Selection Recommendations**

**Apache Iggy** is suitable for scenarios pursuing extreme performance and simple deployment, especially edge computing and embedded devices. Its single-binary, no-dependency deployment model and proprietary protocol make it very convenient for edge scenarios. However, migrating from existing Kafka/MQTT systems requires significant development investment, suitable for new projects or scenarios accepting custom protocols.

**RobustMQ** is suitable for production environments requiring multi-protocol support, particularly enterprises operating MQTT, Kafka, AMQP and other systems simultaneously. The compute-storage separation architecture and pluggable storage make it adaptable from edge to cloud, suitable for enterprises requiring flexible scaling. Additionally, RobustMQ's data integration capabilities make it suitable for IoT data pipelines and AI training scenarios.

| Scenario | Apache Iggy | RobustMQ |
|------|------------|----------|
| **Edge Computing** | Single binary, low latency, suitable | Suitable, supports pluggable storage and multi-protocol |
| **IoT Platform** | Requires custom integration | Native MQTT support, direct use |
| **Microservices Messaging** | Requires custom SDK | Can directly use Kafka/AMQP SDK |
| **Streaming Analytics** | Basic support | Suitable, supports connectors and AI workflows |
| **Multi-Cloud Deployment** | Requires custom adaptation | Native support for S3/HDFS/MinIO |

---

## 7. Migration Cost Comparison

**Migration Recommendations**

**Migrating from Kafka/RabbitMQ to Iggy** requires rewriting client code (SDK replacement) and protocol adaptation. Kafka Connect/RabbitMQ plugins cannot be directly used, requiring custom data integration. Suitable for new projects or scenarios willing to invest in complete system refactoring.

**Migrating from Kafka/RabbitMQ to RobustMQ** can reuse existing client SDKs (for MQTT/Kafka/AMQP), requiring minimal code changes. The multi-protocol architecture supports gradual migration, avoiding system interruptions caused by one-time switchovers. Pluggable storage allows retaining existing storage solutions, suitable for enterprises with high system stability requirements.

| Migration Path | Complexity | Cost | Risk |
|------|------------|----------|----------|
| **Kafka → Iggy** | High | High (rewrite client) | High |
| **RabbitMQ → Iggy** | High | High (protocol adaptation) | High |
| **MQTT → Iggy** | High | High (custom integration) | High |
| **Kafka → RobustMQ** | Low | Low (reuse SDK) | Low |
| **RabbitMQ → RobustMQ** | Low | Low (AMQP compatibility) | Low |
| **MQTT → RobustMQ** | Low | Low (native support) | Low |

---

## 8. Summary & Recommendations

**Apache Iggy** features extreme performance (sub-millisecond latency), deployment simplicity (single binary, no dependencies), and development-friendly. Implemented in Rust, it provides predictable performance and low resource consumption. The custom protocol design provides flexibility but brings migration costs, suitable for new projects and edge scenarios. Currently in the Apache incubation stage, distributed capabilities are being enhanced.

**RobustMQ** features multi-protocol unification (MQTT/Kafka/AMQP), compute-storage separation architecture, pluggable storage, and cloud-native support. Implemented in Rust, leveraging memory safety and high concurrency advantages. Uses community standard SDKs, providing zero migration cost, suitable for enterprises requiring integrated management of multiple message queue protocols. The data integration and AI optimization features provide significant value for IoT and AI scenarios.

**Recommended Scenarios**

- **Apache Iggy**: Edge computing, embedded devices, new projects, scenarios pursuing extreme performance and simple deployment. Suitable for development teams willing to use custom protocols and proprietary SDKs, and can accept distributed feature limitations.

- **RobustMQ**: Production environments requiring multi-protocol support (MQTT/Kafka/AMQP), enterprises needing flexible scaling (edge to cloud), IoT platforms and AI training pipelines. Suitable for enterprises with high system stability requirements and hoping to unify the management of multiple message queue systems.

---

## 9. References

- [Apache Iggy Official Site](https://iggy.apache.org)
- [Apache Iggy GitHub](https://github.com/apache/iggy)
- [RobustMQ Official Site](https://robustmq.com)
- [RobustMQ GitHub](https://github.com/robustmq/robustmq)
- [RobustMQ Documentation](https://robustmq.com/en/)
