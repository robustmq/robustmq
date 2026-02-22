<div align="center">
  <img src="../../images/robustmq-logo.png" width="200"/>
</div>

## What is RobustMQ

**Positioning: Next-generation unified communication infrastructure for AI, IoT, and Big Data.**

**Vision: Let data flow freely across AI training clusters, millions of Agents, IoT devices, and the cloud — at the optimal path, lowest latency, and minimum cost.**

RobustMQ is a next-generation unified messaging platform built with Rust. Through Kafka and MQTT dual-protocol compatibility, intelligent object storage caching, million-scale lightweight Topics, shared subscriptions, and a multi-mode storage engine, it enables data from AI training clusters, millions of Agents, IoT devices, and the cloud to flow freely at optimal paths, lowest latency, and minimum cost.

Fully compatible with Kafka and MQTT 3.1/3.1.1/5.0 protocols — existing applications can connect using standard Kafka SDKs with zero migration cost to gain all of RobustMQ's capabilities.

---

## Core Scenarios

### AI: Intelligent Data Scheduling and Caching Layer

Direct object storage (S3/MinIO) integration with three-tier intelligent caching eliminates the need to pre-import training data, removes I/O bottlenecks, and dramatically improves GPU utilization. A single cluster supports million-scale lightweight Topics, providing each AI Agent with an independent communication channel, fine-grained isolation, and per-Agent cost tracing. Shared subscriptions allow GPU training nodes to scale elastically without being constrained by Partition count.

### IoT: MQTT in / Kafka out — Unified Pipeline

A unified storage layer enables protocol interoperability — data ingested via MQTT from IoT devices can be consumed directly using the Kafka protocol by AI and big data systems. One system replaces the MQTT + Kafka dual-broker architecture. Minimal memory footprint supports edge deployment, and offline caching plus automatic sync covers the full pipeline from edge gateways to cloud clusters.

### Big Data: Compatible and Enhanced Kafka

RobustMQ is compatible with and enhances the Kafka protocol. The intelligent storage engine provides four modes — memory, hybrid, persistent, and tiered — configurable per Topic. Hot data is served at full speed; cold data is automatically tiered to S3. Performance and cost are balanced.

---

## Core Features

- 🚀 **Extreme Performance**: Built with Rust — microsecond latency, zero GC pauses, million-level QPS on a single node, tiny memory footprint for edge deployment
- 🔌 **Dual-Protocol Unification**: Fully compatible with MQTT 3.1/3.1.1/5.0 and Kafka. Unified storage layer enables MQTT in / Kafka out — one system replaces the dual-broker architecture
- 🎯 **AI Training Acceleration**: Direct object storage (S3/MinIO) integration, three-tier intelligent caching (memory/SSD/S3), no data pre-import needed — eliminates I/O bottlenecks and dramatically improves GPU utilization
- 🤖 **Agent Communication**: Million-scale lightweight Topics per cluster, each Agent gets an independent channel, fine-grained isolation and monitoring, per-Agent cost tracing
- 🔄 **Elastic Consumption**: Shared subscriptions break the Kafka "concurrency = Partition count" limit — GPU nodes scale freely without modifying Topic configurations
- 💾 **Intelligent Storage Engine**: Four modes — memory / hybrid / persistent / tiered — configurable per Topic. Hot data at full speed, cold data auto-tiered to S3. Balance performance and cost
- 🌐 **Edge to Cloud**: Minimal memory footprint enables unified deployment from edge gateways to cloud clusters. Offline caching + auto-sync covers the full IoT pipeline
- 🛠️ **Zero-Dependency Deployment**: Single binary, no external dependencies, built-in Raft consensus, ready to run out of the box with minimal operational overhead

---

## Architecture

![architecture](../../images/robustmq-architecture.jpg)

RobustMQ consists of three components with a fixed architecture and clear boundaries:

### Meta Service
Manages cluster metadata and coordination. All node states, Topic configurations, and client session information are stored here, with consistency and high availability guaranteed by a custom-built **Multi Raft** mechanism. Multiple independent Raft Groups are supported so that different types of metadata can be managed separately, avoiding the performance bottleneck of a single Raft group.

### Broker
Handles protocol processing and request routing. The Broker is **stateless** — it only handles client connections, protocol parsing, and message routing without holding any persistent data. This compute-storage separation design allows the Broker to scale horizontally at any time without data migration.

### Storage Engine
Handles data persistence with three pluggable backends, configurable per Topic:

| Engine | Latency | Use Case |
|--------|---------|----------|
| Memory | Microsecond | Gradient sync, real-time metrics, ephemeral notifications |
| RocksDB | Millisecond | Million-scale Topics, IoT device messages, offline storage |
| File Segment | Millisecond | High-throughput log streams, Kafka scenarios |

The storage engine interface is plugin-based, allowing future backends (HDFS, object storage, etc.) to be added without changing the core architecture.

---

## Roadmap

**Phase 1: Core Infrastructure (Complete)**

Build a scalable technical architecture with solid, concise, and well-abstracted code. Establishes the foundation for multi-protocol support, pluggable storage, scalability, and elasticity.

**Phase 2: MQTT Broker (Initially Complete)**

Deliver a stable, high-performance MQTT Broker with full MQTT 3.x/5.0 protocol support, optimized for edge deployment with an installation package under 20MB. Protocol capabilities are initially complete and will continue to evolve in future versions.

**Phase 3: Kafka Protocol & AI Capabilities (In Progress)**

Building on the completed MQTT Broker, this phase initiates Kafka protocol adaptation and AI capability development. Priority is placed on validating AI training data cache acceleration and million-scale lightweight Topics; Kafka protocol implementation is driven by AI scenario requirements, with standard Kafka protocol capabilities progressively completed.

---

## Current Status

| Feature | Status |
|---------|--------|
| MQTT 3.x / 5.0 core protocol | ✅ Available |
| Session persistence and recovery | ✅ Available |
| Shared subscriptions | ✅ Available |
| Authentication and ACL | ✅ Available |
| Rule engine | ✅ Basic |
| Grafana + Prometheus monitoring | ✅ Available |
| Web management console | ✅ Available |
| Kafka protocol | 🚧 In progress |
| AI training data cache | 🚧 In progress |

> **Notice**: The current version (0.3.0) is still in early stage and not recommended for production use. Version 0.4.0 (expected May 2025) is planned to reach production-ready status.

---

## Quick Start

```bash
# One-line install
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# Start services
robust-server start

# Verify MQTT
mqttx sub -h localhost -p 1883 -t "test/topic"
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"
```

Full documentation: [Quick Start Guide](../QuickGuide/Quick-Install.md)

---

## Project Philosophy

RobustMQ is a **non-commercial open source project** with no corporate backing and no paid edition — all core features are fully open source.

This is a project driven by technical conviction — the belief that rebuilding communication infrastructure in Rust is the right direction, that the AI era needs a messaging system truly designed for new scenarios, and that excellent infrastructure software should belong to the entire community.

The long-term goal is to become an **Apache Top-Level Project** and build a global developer community that continuously drives the project forward.

---

## Project Info

- **Language**: Rust
- **License**: Apache 2.0 (fully open source, no commercial edition)
- **GitHub**: https://github.com/robustmq/robustmq
- **Website**: https://robustmq.com
