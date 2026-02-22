# RobustMQ vs Existing Message Queues

## Background

Most existing message queues were built for specific historical contexts: Kafka for big data log streaming, MQTT brokers for IoT device connectivity, RabbitMQ for enterprise message routing. Each performs well in its own domain, but as business boundaries blur, organizations often end up maintaining multiple systems — adding operational complexity and creating data silos.

RobustMQ's starting point is: can a single unified architecture support IoT, big data, and AI scenarios simultaneously? This document offers an objective comparison of RobustMQ and mainstream message queues across architecture design and core capabilities.

---

## Side-by-Side Comparison

| Dimension | Kafka | Pulsar | NATS | RabbitMQ | RobustMQ |
|-----------|-------|--------|------|----------|----------|
| **Primary Language** | Java/Scala | Java | Go | Erlang | Rust |
| **Protocol Support** | Kafka | Kafka / Pulsar | NATS | AMQP | MQTT (available) / Kafka (in dev) / AMQP, RocketMQ, etc. (long-term plan) |
| **Architecture Dependencies** | ZooKeeper (legacy) / KRaft | Broker + BookKeeper + ZooKeeper | None | None | Meta Service + Broker + Storage Engine |
| **Compute-Storage Separation** | ❌ (coupled) | ✅ | ✅ | ❌ | ✅ |
| **Topic Scale** | ~10K (filesystem-bound) | Millions | Millions | ~10K | Millions (RocksDB KV) |
| **GC Pauses** | Yes (JVM) | Yes (JVM) | Minimal (Go GC) | No | No (Rust, zero GC) |
| **IoT Use Case** | Not suitable | Limited | Basic | Limited | ✅ Native MQTT |
| **Big Data Use Case** | ✅ Primary | ✅ Supported | ❌ | ❌ | In development (Kafka-compatible) |
| **Operational Complexity** | Medium (lower with KRaft) | High (3 process types) | Low | Low | Low (single binary, no external dependencies) |

---

## Architecture Differences

### Kafka

Kafka's core is the Partition + Replication model, with data persisted to local disk. Compute and storage are tightly coupled. Each Topic maps to directories and files on the filesystem, so Topic count is bounded by file descriptors and disk I/O — production environments typically stay within tens of thousands.

KRaft mode removed the ZooKeeper dependency and simplified operations, but the fundamental coupling of storage and compute remains.

### Pulsar

Pulsar uses a compute-storage separation design: stateless Brokers with data persisted to BookKeeper. The architecture is more flexible, supports millions of Topics, multi-tenancy, and geo-replication. However, the three-tier architecture (Broker / BookKeeper / ZooKeeper) makes deployment and operations more complex.

### NATS

NATS is lightweight and low-latency, well-suited for microservice messaging. JetStream added persistence capabilities. It supports large Topic counts but lacks native Kafka or MQTT protocol support, requiring extra work to integrate with existing ecosystems.

### RabbitMQ

RabbitMQ is based on the AMQP protocol and offers a flexible routing model (exchange / queue / binding), suited for enterprise messaging. Its throughput is relatively limited, and IoT or big data scenarios are not its design target.

### RobustMQ

RobustMQ's core design decisions:

**Fixed three-component architecture**: Meta Service handles metadata (consistency via Multi Raft); stateless Broker handles protocol processing; Storage Engine handles persistence. Clear component boundaries with independent scaling for each layer.

**Million-scale Topics**: RocksDB-based KV storage means all Topics share a single storage instance, distinguished by key prefixes. Creating a Topic only requires writing a metadata record — no physical files are created. This fundamentally removes the filesystem limit on Topic count.

**Pluggable storage engines**: The same Broker can work with memory, RocksDB, or File Segment storage backends, configurable per Topic to match different latency and cost requirements.

**Rust implementation**: Zero GC pauses, memory safety, predictable and stable latency.

**Extensible multi-protocol architecture**: Protocol handling in the Broker layer is pluggable by design, making it straightforward to add new protocols over time. The short-term focus is on MQTT and Kafka. AMQP, RocketMQ, and other protocols are part of the long-term architectural roadmap and will be added progressively.

---

## Scenario Recommendations

### IoT Device Connectivity

**Prefer RobustMQ or a dedicated MQTT broker** (e.g., EMQX). Kafka and Pulsar do not support MQTT natively and require a protocol gateway layer. RobustMQ's native MQTT support eliminates that middleware hop.

### Big Data / Log Streaming

**Kafka is still the recommended choice today.** The Kafka ecosystem is mature — Kafka Connect, Kafka Streams, and hundreds of connectors with stable native Flink/Spark integration. RobustMQ's Kafka protocol support is still in development and will gradually become available in 2026.

### IoT + Big Data Convergence

**RobustMQ's primary target scenario.** IoT devices write via MQTT; analytics platforms consume via Kafka protocol — sharing the same storage backend with no bridge layer required. This is where RobustMQ's differentiation is most pronounced compared to existing solutions.

### Microservice Communication

**NATS or RabbitMQ are better fits.** RobustMQ is not currently optimized for this use case.

### Million-Scale Lightweight Topics

**RobustMQ or Pulsar.** Kafka is unsuitable due to linear filesystem overhead growth with Topic count. RobustMQ's KV-based approach has a lower resource footprint than Pulsar's BookKeeper model.

---

## Current Limitations

To be objective, RobustMQ is still in early stages with the following clear limitations:

- **Kafka protocol incomplete**: Consumer Group, Rebalance, and transactions are still in development — not suitable for replacing Kafka in production yet
- **Immature ecosystem**: Far fewer connectors (8+) than Kafka (300+); deep integration with Flink/Spark needs time
- **No production cases yet**: No large-scale production validation; the current version (v0.3.0) is not recommended for production use
- **Small community**: Contributor and user base is significantly smaller than mature projects

MQTT scenarios are expected to be production-ready after v0.4.0 (May 2026). Kafka protocol support will mature progressively starting with v0.5.0 (September 2026).

---

## Detailed Comparison Documents

- [RobustMQ vs Kafka](./Diff-kafka.md)
- [RobustMQ vs Pulsar](./Diff-pulsar.md)
- [RobustMQ vs NATS](./Diff-nats.md)
- [RobustMQ vs Redpanda](./Diff-redpanda.md)
- [RobustMQ vs Iggy](./Diff-iggy.md)
- [Comprehensive Comparison](./Diff-MQ.md)
