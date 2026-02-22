# RobustMQ 0.3.0 RELEASE

<p align="center">
  <img src="../../images/robustmq-logo.png" alt="RobustMQ Logo" width="200">
</p>

**Release Date**: February 2025  
**Version**: 0.3.0  
**GitHub**: [https://github.com/robustmq/robustmq](https://github.com/robustmq/robustmq)

> **Notice**: 0.3.0 is still in early stage and is not recommended for production use. We plan to reach production-ready status in version 0.4.0 (expected May 2025). Early testing and feedback are welcome.

---

## Highlights

RobustMQ 0.3.0 is a significant milestone — not just a feature iteration, but a fundamental re-alignment of the project's positioning and architecture.

Starting from 0.3.0, RobustMQ's positioning is clear: **Next-generation unified communication infrastructure for AI, IoT, and Big Data**. Through dual Kafka/MQTT protocol compatibility, million-scale Topics, direct object storage (S3/MinIO) integration, multi-mode storage engine, and intelligent data caching, RobustMQ provides high-performance, low-cost, stable communication infrastructure for AI training, Agent communication, IoT devices, and big data processing.

---

## Architecture Redesign

0.3.0 introduces a redesigned architecture with three clearly bounded components.

### Meta Service
Manages cluster metadata and coordination. All node states, Topic configurations, and client session information are stored here, with consistency and high availability guaranteed by a custom-built **Multi Raft** mechanism.

### Broker
Handles protocol processing and request routing. The Broker is stateless — it only handles client connections, protocol parsing, and message routing without holding any persistent data. This compute-storage separation design allows Broker nodes to scale horizontally at any time without data migration.

### Storage Engine
Handles data persistence with three pluggable storage backends:

| Engine | Characteristics | Use Case |
|--------|-----------------|----------|
| Memory | Pure in-memory, microsecond latency | Low-latency, ephemeral data |
| RocksDB | Unified KV storage | Million-scale Topics, general persistence |
| File Segment | Sequential write, high throughput | Kafka scenarios, big data streams |

The interface between upper-layer protocols and storage engines is plugin-based, allowing future backends to be added without changing the core architecture.

### Multi Raft
Meta Service uses a custom Multi Raft implementation that supports multiple independent Raft Groups. Different types of metadata can be managed by different Raft Groups, avoiding the performance bottleneck of a single Raft. This is a critical infrastructure investment for large-scale deployments.

---

## MQTT Broker Core Features

The MQTT Broker in 0.3.0 reaches an important milestone in functional completeness, covering the main scenarios of a production-grade MQTT Broker:

- **Full protocol support**: MQTT 3.1 / 3.1.1 / 5.0, including connect/disconnect, Keep-alive, Will messages, Retained messages, QoS 0/1/2
- **Session management**: Session persistence and recovery
- **Subscription modes**: Shared subscriptions, topic rewriting and filtering
- **Security**: Username/password authentication, ACL authorization
- **Message features**: Offline message storage, delayed messages
- **Rule engine**: Basic rule engine functionality

---

## Code Quality: Refactoring & Bug Fixes

0.3.0 represents a large amount of engineering work that is not directly visible as new features:

**Architecture Refactoring**
- Connection management layer refactored from a single implementation to an extensible abstraction
- Storage engine decoupled from the core and split into a plugin interface
- gRPC client layer now includes retry logic and timeout control
- Handler layer now has per-thread independent monitoring metrics

**Stability Fixes**
- Fixed multiple race conditions under high concurrency
- Fixed edge-case bugs in session recovery logic
- Fixed memory growth issues under large numbers of connections

**Performance Optimization**
- Optimized the critical path for connection establishment, reducing unnecessary gRPC calls
- Handler timeout mechanism prevents task stalls from collapsing throughput

---

## Ecosystem Tools

### Grafana + Prometheus
Built-in comprehensive monitoring metrics covering connections, message throughput, Handler latency, gRPC call duration, storage engine I/O, and queue backlog. An out-of-the-box Grafana Dashboard is provided — just import it after deployment to get full observability immediately.

### Command CLI (robust-ctl)
Enhanced command-line management tool supporting cluster status queries, Topic management, client session inspection, and user permission configuration.

### HTTP API
Full REST API covering cluster management, Topic operations, user management, and rule engine configuration. Designed for integration with existing operations platforms and automation scripts.

### Bench CLI (robust-bench)
Built-in benchmarking tool supporting MQTT connection, publish, and subscribe load tests with configurable concurrency, message rate, payload size, and duration. Quickly validate cluster performance and stability after deployment.

### RobustMQ Dashboard
Web-based management console providing cluster overview, node status, Topic list, client connections, and rule engine configuration — all operable from the UI.

### Website & Documentation
Restructured official website and documentation system, covering quick start, architecture overview, configuration reference, API manual, and benchmarking guide. Documentation is maintained in sync with the codebase.

---

## Installation & Usage

### One-line Install

```bash
# Install latest version (defaults to ~/robustmq-v0.3.0 with ~/robustmq symlink)
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

### Start Services

```bash
# Start Placement Center (Meta Service)
robust-server placement-center start

# Start MQTT Broker
robust-server mqtt-server start
```

### Quick Verification

```bash
# Install MQTTX CLI (https://mqttx.app/docs/cli)
# Subscribe
mqttx sub -h localhost -p 1883 -t "test/topic"

# Publish (in another terminal)
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"
```

Web console: visit `http://localhost:3000` (Dashboard)

Full documentation: [Quick Start Guide](../QuickGuide/Quick-Install.md)

### Docker

```bash
docker pull ghcr.io/robustmq/robustmq:v0.3.0
docker pull ghcr.io/robustmq/robustmq:latest
```

### Build from Source

```bash
git clone https://github.com/robustmq/robustmq.git
cd robustmq
git checkout v0.3.0
cargo build --release
```

---

## Roadmap

| Direction | Content |
|-----------|---------|
| Performance & Stability | MQTT connection path optimization, Raft write throughput, storage engine deep tuning |
| MQTT completeness | Full rule engine, Webhook integration, complete operations API |
| AI MQ | Topic direct object storage (S3/MinIO), three-tier intelligent cache, predictive prefetch |
| Kafka Protocol | Full Kafka protocol implementation, compatible with Flink, Spark, Kafka Connect |

See [2026 RoadMap](../OverView/RoadMap-2026.md) for details.

---

## Support & Feedback

- **GitHub**: [https://github.com/robustmq/robustmq](https://github.com/robustmq/robustmq)
- **Issues**: [https://github.com/robustmq/robustmq/issues](https://github.com/robustmq/robustmq/issues)
- **Discussions**: [https://github.com/robustmq/robustmq/discussions](https://github.com/robustmq/robustmq/discussions)
- **Docs**: [https://robustmq.com](https://robustmq.com)

---

**RobustMQ Team**  
February 2025
