# RobustMQ Overall Architecture

RobustMQ is a next-generation unified messaging platform built with Rust, designed for AI, IoT, and Big Data scenarios. The architecture goal is: **full separation of compute, storage, and coordination — each component is stateless and independently scalable, supporting multi-protocol ingestion and pluggable storage with zero external dependencies.**

---

## High-Level Architecture

![architecture overview](../../images/robustmq-architecture-overview.jpg)

The system is built around three core components:

| Component | Responsibility |
|-----------|---------------|
| **Meta Service** | Cluster metadata management, node coordination, cluster controller |
| **Broker** | Multi-protocol parsing and message processing (MQTT, Kafka, etc.) |
| **Storage Engine** | Built-in storage engine with Memory / RocksDB / File Segment backends |

All three components are delivered as a single binary. The `roles` configuration determines which are active:

```toml
roles = ["meta", "broker", "engine"]
```

Any combination is supported — all-in-one on a single node, or each role on independent nodes.

---

## Cluster Deployment View

![architecture](../../images/robustmq-architecture.jpg)

In a typical three-node cluster, each Node contains four modules:

### 1. Common Server Layer

Shared infrastructure for all upper-layer modules:

| Component | Description |
|-----------|-------------|
| Inner gRPC Server | Inter-node communication for Meta Service and Broker |
| Admin HTTP Server | Exposes REST management API |
| Prometheus Server | Metrics endpoint for monitoring systems |

### 2. Meta Service

Cluster metadata management and controller. Core technology: **gRPC + Multi Raft + RocksDB**.

**Responsibilities:**
- **Cluster coordination**: Node discovery, join/leave management, data distribution
- **Metadata storage**: Broker info, Topic config, Connector config, and other cluster metadata
- **KV business data**: MQTT Session, Retained Messages, Will Messages, and other runtime data
- **Controller**: Failover, Connector task scheduling, and other cluster-level coordination

**Multi Raft Design:** Meta Service runs multiple independent Raft state machines (Metadata / Offset / Data), with multiple Leaders serving in parallel to avoid single-Raft write bottlenecks. Data is persisted via RocksDB with strict memory controls, supporting million-scale Topics and billion-scale Sessions.

> Meta Service plays a role similar to ZooKeeper for Kafka or NameServer for RocketMQ — but simultaneously serves as a metadata coordinator, KV store, and cluster controller. This is what enables RobustMQ's zero-external-dependency architecture.

### 3. Broker

Stateless protocol processing layer with a layered architecture:

```
┌─────────────────────────────────────┐
│           Network Layer              │  TCP / TLS / WebSocket / WSS / QUIC
├─────────────────────────────────────┤
│           Protocol Layer             │  MQTT / Kafka / AMQP / RocketMQ
├─────────────────────────────────────┤
│         Protocol Logic Layer         │  mqtt-broker / kafka-broker / ...
├─────────────────────────────────────┤
│        Common Message Logic Layer    │  pub/sub / expiry / delay / security / schema / metrics
├─────────────────────────────────────┤
│         Storage Adapter              │  Shard abstraction + storage engine routing
└─────────────────────────────────────┘
```

- **Network Layer**: Accepts connections over TCP, TLS, WebSocket, WebSockets, and QUIC
- **Protocol Layer**: MQTT is fully supported; Kafka is in progress; AMQP and RocketMQ are in long-term planning
- **Protocol Logic Layer**: Each protocol has an independent module for its specific business logic
- **Common Message Logic Layer**: Cross-protocol shared capabilities — pub/sub, expiry, delayed publish, authentication, Schema validation, metrics
- **Storage Adapter**: Unifies MQTT Topic, Kafka Partition, and AMQP Queue into a single **Shard** abstraction, routing data to the configured storage backend

### 4. Storage Engine

Built-in storage engine with three storage types, configurable at **Topic level**:

| Type | Config Value | Latency | Characteristics | Use Case |
|------|-------------|---------|-----------------|----------|
| Memory | `EngineMemory` | Microseconds | Pure in-memory, lost on restart | Real-time data, ephemeral messages |
| RocksDB | `EngineRocksDB` | Milliseconds | Local KV persistent | IoT device messages, offline messages |
| File Segment | `EngineSegment` | Milliseconds | Segmented log, high throughput | Big data streams, high-throughput writes |

Storage Adapter also supports external backends (MinIO, S3, MySQL, etc.) — routing target is determined by configuration.

---

## Startup Sequence

Modules initialize in the following order:

```
Common Server → Meta Service → Storage Engine → Broker
```

1. **Common Server**: Starts first to establish inter-node communication
2. **Meta Service**: Completes leader election via Raft; the leader also starts the controller thread
3. **Storage Engine**: Depends on Meta Service for cluster formation and metadata registration
4. **Broker**: Starts last; depends on Meta Service for coordination and Storage Engine for data writes

---

## Mixed Storage

RobustMQ supports mixing multiple storage engines within the same cluster at Topic granularity:

| Storage Choice | Use Case |
|----------------|----------|
| Memory | High throughput, ultra-low latency, tolerates minimal data loss |
| RocksDB | Low-latency persistence, no data loss |
| File Segment | High-throughput persistence, Big Data / Kafka scenarios |
| MinIO / S3 | Large volume, cost-sensitive, latency-tolerant |

> In practice, most deployments use a single storage type. Mixed storage is primarily used for advanced scenarios like AI training that require tiered caching.

---

## Design Principles

| Principle | Implementation |
|-----------|---------------|
| **Zero external dependencies** | Meta Service built-in (replaces ZooKeeper/etcd); Storage Engine built-in |
| **Compute-storage separation** | Stateless Broker; Storage Engine scales independently |
| **Multi-protocol extensibility** | Protocol layer decoupled from storage; new protocol = implement protocol logic layer only |
| **Pluggable storage** | Storage Adapter's Shard abstraction allows any backend to be added or swapped |
| **Topic-level configuration** | Storage type, QoS, expiry policy — all independently configurable per Topic |
