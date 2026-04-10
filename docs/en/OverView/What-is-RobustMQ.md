<div align="center">
  <img src="../../images/robustmq-logo.png" width="200"/>
</div>

## What is RobustMQ

**Positioning: Communication infrastructure for the AI era**

**Vision: Become the foundation for data flow in the AI era — AI Agent collaboration, IoT device ingestion, edge-to-cloud sync, traditional messaging, real-time streaming pipelines, and ultra-low-latency dispatch, all running on a single communication layer.**

RobustMQ is a unified messaging engine built with Rust. One binary, one broker, no external dependencies — deployable from edge devices to cloud clusters. It natively supports MQTT, Kafka, NATS, and AMQP across six core scenarios: IoT device ingestion, edge-to-cloud data sync, traditional messaging, real-time streaming pipelines, ultra-low-latency real-time dispatch, and AI Agent communication. One message, one copy of data, consumed by any protocol.

---

## Why RobustMQ

Today's messaging infrastructure is a collection of protocol silos. IoT devices use MQTT brokers, data pipelines use Kafka, enterprise systems use RabbitMQ, and AI Agent communication has no native solution. Multiple systems mean duplicate data copies, overlapping operations, and bridging layers at every hop that add latency and failure points.

RobustMQ solves this at the architectural level: **unified storage layer + native multi-protocol support**. Not bridging, not routing — one copy of data written once, with MQTT, Kafka, NATS, and AMQP each reading through their own semantic lens. One system replaces multiple brokers. No data duplication, no operational overlap.

---

## Six Core Scenarios

### IoT Device Ingestion: MQTT in, Kafka out

IoT devices publish via MQTT into the unified storage layer. AI systems and big data platforms consume the same data directly using the Kafka protocol — no bridging or data forwarding required. One system replaces the MQTT Broker + Kafka dual-broker architecture.

```
IoT devices (MQTT) → RobustMQ unified storage → Big data platform (Kafka)
                                               → AI inference system (Kafka)
                                               → Real-time monitoring (NATS)
```

### Edge-to-Cloud Data Sync

RobustMQ deploys on edge nodes as a single binary with minimal memory footprint, supports offline local buffering, and automatically syncs to the cloud when connectivity is restored. Factory floors, retail stores, vehicle systems — a unified edge-to-cloud data path with no additional sync components.

### Traditional Messaging

Full AMQP protocol support with native Exchange, Queue, Binding, and vhost semantics. Existing RabbitMQ applications migrate at low cost while gaining multi-protocol interoperability from the unified storage layer.

### Real-Time Streaming Pipelines

Full Kafka protocol compatibility — existing Kafka applications connect using standard SDKs with zero migration cost. Multi-mode storage engine supports hot data at full speed and automatic cold data tiering to object storage. Millions of lightweight Topics support large-scale data partitioning.

### Ultra-Low-Latency Real-Time Dispatch

NATS-based pure in-memory message dispatch — messages are routed directly in memory without being persisted to disk. Designed for latency-critical scenarios: financial market data feeds, game state sync, industrial control commands, AI inference result distribution. Millisecond to sub-millisecond latency, throughput scales linearly with nodes.

```
Publisher → RobustMQ (in-memory routing) → Subscribers (real-time push)
No disk writes, no persistence, extreme low latency
Switch to JetStream mode when persistence is needed — unified storage layer takes over
```

### AI Agent Communication

mq9 is RobustMQ's fifth native protocol, purpose-built for AI Agents. Each Agent has a mailbox — messages are persisted on write and pushed to the recipient when they come online. Sender and receiver do not need to be online at the same time. Supports three priority levels (critical / urgent / normal), TTL-based auto-cleanup, and public mailbox discovery.

```
Create mailbox  → nats req  '$mq9.AI.MAILBOX.CREATE'                '{"ttl":3600}'
Send message    → nats pub  '$mq9.AI.MAILBOX.MSG.{mail_id}'         '{...}'
Urgent message  → nats pub  '$mq9.AI.MAILBOX.MSG.{mail_id}.urgent'  '{...}'
Subscribe       → nats sub  '$mq9.AI.MAILBOX.MSG.{mail_id}.*'
Discover public → nats req  '$mq9.AI.PUBLIC.LIST'                   ''
```

Any NATS client connects directly. Also available: RobustMQ SDK (Go/Python/Rust/JavaScript/Java/C#) and the `langchain-mq9` toolkit.

---

## Core Features

- 🤖 **mq9 — AI Agent communication**: Agent mailboxes, priority queuing, public discovery — async Agent-to-Agent messaging, no simultaneous online required
- 🦀 **Rust-native**: No GC, stable and predictable memory footprint, no periodic spikes, minimal resource usage — consistent from edge devices to cloud clusters
- 🗄️ **Unified storage layer**: All protocols share one storage engine — data written once, consumed by any protocol, no duplication
- 🔌 **Native multi-protocol**: MQTT 3.1/3.1.1/5.0, Kafka, NATS, AMQP, mq9 — natively implemented, full protocol semantics
- 🏢 **Native multi-tenancy**: Unified across all protocols — full data isolation and independent permission management per tenant
- 🌐 **Edge-to-cloud**: Single binary, zero dependencies, offline buffering with auto-sync — same runtime from edge gateways to cloud clusters
- ⚡ **Ultra-low-latency dispatch**: NATS pure in-memory routing — no disk writes, millisecond to sub-millisecond latency
- 💾 **Multi-mode storage engine**: Memory / RocksDB / File, per-Topic configuration, automatic cold data tiering to S3
- 🔄 **Shared subscription**: Break the "concurrency = partition count" limit — consumers scale elastically at any time
- 🛠️ **Minimal operations**: Single binary, zero external dependencies, built-in Raft consensus, ready out of the box

---

## Roadmap

The approach: slow is smooth, smooth is fast. Focused and disciplined. Each phase done properly before moving on.

```
Phase 1 (current)
  MQTT core production-ready, continuously refined to be the best MQTT Broker available
  Architecture and infrastructure hardened in parallel

Phase 2 (in progress)
  NATS protocol compatibility + AI Agent communication ($mq9.AI.API.* extension)
  Native Agent registration, discovery, invocation, and load balancing

Phase 3 (in progress)
  Full Kafka protocol compatibility
  Complete the IoT-to-streaming data path, edge-to-cloud data flow

Phase 4 (planned)
  Full AMQP protocol compatibility
  Traditional enterprise messaging migration path
```

---

## Current Status

| Feature | Status |
|---------|--------|
| MQTT 3.x / 5.0 core | ✅ Available |
| Session persistence and recovery | ✅ Available |
| Shared subscription | ✅ Available |
| Authentication and ACL | ✅ Available |
| Grafana + Prometheus monitoring | ✅ Available |
| Web management console | ✅ Available |
| Kafka protocol | 🚧 In development |
| NATS protocol | 🔬 Demo validated, in development |
| AMQP protocol | 🔬 Demo validated, in development |
| mq9 — AI Agent mailbox | 🔬 Demo validated, in development |

> **Notice**: The current version is still in early stage and not recommended for production use. Version 0.4.0 / 0.5.0 is targeted to reach MQTT production-ready status.

---

## Quick Start

```bash
# One-line install
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# Start service
robust-server start

# Publish via MQTT
mqttx pub -h localhost -p 1883 -t "robustmq.multi.protocol" -m "Hello RobustMQ!"

# Consume the same message via Kafka
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic robustmq.multi.protocol --from-beginning

# Consume the same message via NATS
nats sub "robustmq.multi.protocol"
```

Full documentation: [Quick Start Guide](../QuickGuide/Quick-Install.md)

---

## Project Info

- **Language**: Rust
- **License**: Apache 2.0
- **GitHub**: https://github.com/robustmq/robustmq
- **Website**: https://robustmq.com
