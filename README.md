<p align="center">
  <picture>
    <img alt="RobustMQ Logo" src="docs/images/robustmq-logo.png" width="300">
  </picture>
</p>

<p align="center">
  <a href="https://deepwiki.com/robustmq/robustmq"><img src="https://deepwiki.com/badge.svg" alt="Ask DeepWiki"></a>
  <a href="https://zread.ai/robustmq/robustmq" target="_blank"><img src="https://img.shields.io/badge/Ask_Zread-_.svg?style=flat&color=00b0aa&labelColor=000000&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB3aWR0aD0iMTYiIGhlaWdodD0iMTYiIHZpZXdCb3g9IjAgMCAxNiAxNiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTQuOTYxNTYgMS42MDAxSDIuMjQxNTZDMS44ODgxIDEuNjAwMSAxLjYwMTU2IDEuODg2NjQgMS42MDE1NiAyLjI0MDFWNC45NjAxQzEuNjAxNTYgNS4zMTM1NiAxLjg4ODEgNS42MDAxIDIuMjQxNTYgNS42MDAxSDQuOTYxNTZDNS4zMTUwMiA1LjYwMDEgNS42MDE1NiA1LjMxMzU2IDUuNjAxNTYgNC45NjAxVjIuMjQwMUM1LjYwMTU2IDEuODg2NjQgNS4zMTUwMiAxLjYwMDEgNC45NjE1NiAxLjYwMDFaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik00Ljk2MTU2IDEwLjM5OTlIMi4yNDE1NkMxLjg4ODEgMTAuMzk5OSAxLjYwMTU2IDEwLjY4NjQgMS42MDE1NiAxMS4wMzk5VjEzLjc1OTlDMS42MDE1NiAxNC4xMTM0IDEuODg4MSAxNC4zOTk5IDIuMjQxNTYgMTQuMzk5OUg0Ljk2MTU2QzUuMzE1MDIgMTQuMzk5OSA1LjYwMTU2IDE0LjExMzQgNS42MDE1NiAxMy43NTk5VjExLjAzOTlDNS42MDE1NiAxMC42ODY0IDUuMzE1MDIgMTAuMzk5OSA0Ljk2MTU2IDEwLjM5OTlaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik0xMy43NTg0IDEuNjAwMUgxMS4wMzg0QzEwLjY4NSAxLjYwMDEgMTAuMzk4NCAxLjg4NjY0IDEwLjM5ODQgMi4yNDAxVjQuOTYwMUMxMC4zOTg0IDUuMzEzNTYgMTAuNjg1IDUuNjAwMSAxMS4wMzg0IDUuNjAwMUgxMy43NTg0QzE0LjExMTkgNS42MDAxIDE0LjM5ODQgNS4zMTM1NiAxNC4zOTg0IDQuOTYwMVYyLjI0MDFDMTQuMzk4NCAxLjg4NjY0IDE0LjExMTkgMS42MDAxIDEzLjc1ODQgMS42MDAxWiIgZmlsbD0iI2ZmZiIvPgo8cGF0aCBkPSJNNCAxMkwxMiA0TDQgMTJaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik00IDEyTDEyIDQiIHN0cm9rZT0iI2ZmZiIgc3Ryb2tlLXdpZHRoPSIxLjUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIvPgo8L3N2Zz4K&logoColor=ffffff" alt="zread"/></a>
  <img alt="Latest Release" src="https://img.shields.io/github/v/release/robustmq/robustmq?style=flat">
  <img alt="License" src="https://img.shields.io/github/license/robustmq/robustmq?style=flat">
  <img alt="GitHub issues" src="https://img.shields.io/github/issues/robustmq/robustmq?style=flat">
  <img alt="GitHub stars" src="https://img.shields.io/github/stars/robustmq/robustmq?style=flat">
  <a href="https://codecov.io/gh/robustmq/robustmq">
    <img src="https://codecov.io/gh/robustmq/robustmq/graph/badge.svg?token=MRFFAX9QZO" alt="Coverage"/>
  </a>
  <img alt="Build Status" src="https://img.shields.io/github/actions/workflow/status/robustmq/robustmq/ci.yml?branch=main&style=flat">
  <img alt="Rust Version" src="https://img.shields.io/badge/rust-1.70+-orange.svg">
</p>

<h3 align="center">
    Communication infrastructure for the AI era — one binary, one broker, one storage layer, any protocol
</h3>

<p align="center">
  <a href="#-what-is-robustmq">What is RobustMQ</a> •
  <a href="#-mq9--agent-mailbox-for-ai">mq9</a> •
  <a href="#-features">Features</a> •
  <a href="#%EF%B8%8F-roadmap">Roadmap</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-contributing">Contributing</a> •
  <a href="#-community">Community</a>
</p>

---

> **⚠️ Development Status**
> RobustMQ is in early development and **not yet production-ready**. MQTT core is stable and continuing to mature. Kafka, NATS, and AMQP are under active development. Production readiness is targeted for 0.4.0.

---

## 🌟 What is RobustMQ

RobustMQ is a unified messaging engine built with Rust. One binary, one broker, no external dependencies — deployable from edge devices to cloud clusters. It natively supports MQTT, Kafka, NATS, AMQP, and **mq9** on a **shared storage layer**: one message written once, consumed by any protocol.

![RobustMQ Architecture](docs/images/robustmq-architecture.jpg)

```
MQTT publish  →  RobustMQ unified storage  →  Kafka consume
                                           →  NATS subscribe
                                           →  AMQP consume
                                           →  mq9 Agent mailbox
```

**Five protocols, one system:**

| Protocol | Best for |
|----------|---------|
| **MQTT** | IoT devices, edge sensors |
| **Kafka** | Streaming data pipelines, analytics |
| **NATS** | Ultra-low-latency pub/sub |
| **AMQP** | Enterprise messaging, RabbitMQ migration |
| **mq9** | AI Agent async communication |

## 🤖 mq9 — Agent Mailbox for AI

**mq9** is RobustMQ's communication layer designed for AI Agents. Just like people have email — you send a message, the recipient reads it when they're available — Agents need the same. Today, when Agent A sends a message to Agent B and B is offline, the message is gone. Every team works around this with Redis pub/sub, database polling, or homegrown queues.

mq9 solves it directly: **send a message, the recipient gets it when they come online.**

<div align="center">

| Operation | Subject | What it does |
|-----------|---------|-------------|
| **MAILBOX.CREATE** | `$mq9.AI.MAILBOX.CREATE` | Create a private or public mailbox |
| **Send** | `$mq9.AI.MAILBOX.MSG.{mail_address}` / `.urgent` / `.critical` | Deliver a message — three levels: `critical` / `urgent` / `normal` (default, no suffix) |
| **Subscribe** | `$mq9.AI.MAILBOX.MSG.{mail_address}.*` | Receive all priorities; new arrivals pushed in real time |
| **Discover** | `$mq9.AI.PUBLIC.LIST` | Discover all public mailboxes |

</div>

```bash
# Create a private mailbox — returns mail_address
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":3600}'
# → {"mail_address":"mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag","is_new":true}

# Send to another Agent's mailbox (works even if they're offline)
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag' \
  '{"type":"task_result","payload":"done","ts":1234567890}'

# Create a public mailbox (task queue), discoverable via PUBLIC.LIST
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":3600,"public":true,"name":"task.queue","desc":"Task queue"}'
nats pub '$mq9.AI.MAILBOX.MSG.task.queue' '{"type":"data_analysis"}'

# Subscribe to your mailbox — receives all non-expired messages immediately, in priority order
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.*'
```

**Multiple integration paths:** any NATS client connects directly; the RobustMQ SDK covers Go, Python, Rust, JavaScript, Java, and C#; the `langchain-mq9` toolkit plugs into LangChain and LangGraph; and an MCP Server provides JSON-RPC 2.0 access for tools like Dify.

mq9 is RobustMQ's fifth native protocol, alongside MQTT, Kafka, NATS, and AMQP, built on the same unified storage layer. Deploy one RobustMQ instance — mq9 is ready.

> 📖 [mq9 Documentation](https://robustmq.com/en/mq9/)

---

## ✨ Features

<div align="center">
  <video src="https://robustmq.com/assets/demo.zRXM786t.mp4" controls width="100%"></video>
</div>

- 🤖 **mq9 — AI Agent communication**: Agent mailboxes with persistent store-first delivery, three-level priority (critical / urgent / normal), TTL auto-cleanup, and public mailbox discovery — async Agent-to-Agent messaging, no simultaneous online required
- 🦀 **Rust-native**: No GC, stable and predictable memory footprint, no periodic spikes — consistent from edge devices to cloud clusters
- 🗄️ **Unified storage layer**: All protocols share one storage engine — data written once, consumed by any protocol, no duplication
- 🔌 **Native multi-protocol**: MQTT 3.1/3.1.1/5.0, Kafka, NATS, AMQP, mq9 — natively implemented, full protocol semantics
- 🏢 **Native multi-tenancy**: Unified across all protocols — full data isolation and independent permission management per tenant
- 🌐 **Edge-to-cloud**: Single binary, zero dependencies, offline buffering with auto-sync — same runtime from edge gateways to cloud clusters
- ⚡ **Ultra-low-latency dispatch**: NATS pure in-memory routing — no disk writes, millisecond to sub-millisecond latency
- 💾 **Multi-mode storage**: Memory / RocksDB / File, per-topic configuration, automatic cold data tiering to S3
- 🔄 **Shared subscription**: Break the "concurrency = partition count" limit — consumers scale elastically at any time
- 🛠️ **Minimal operations**: Single binary, zero external dependencies, built-in Raft consensus, ready out of the box

## 🗺️ Roadmap

```
Phase 1 — MQTT (current)
  MQTT core production-ready, continuously refined to be the best MQTT Broker available
  Architecture and infrastructure hardened in parallel

Phase 2 — NATS + mq9 AI Agent (in progress)
  NATS protocol compatibility + mq9 Agent mailbox with priority & public discovery
  Native Agent async communication layer

Phase 3 — Kafka (in progress)
  Full Kafka protocol compatibility
  Complete the IoT-to-streaming data path, edge-to-cloud data flow

Phase 4 — AMQP (planned)
  Full AMQP protocol compatibility
  Traditional enterprise messaging migration path
```

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

## 🏗️ Architecture

RobustMQ has three components with fixed, clean boundaries:

- **Meta Service** — metadata management, Raft-based consensus
- **Broker** — protocol parsing and routing (MQTT / Kafka / NATS / AMQP / mq9)
- **Storage Engine** — unified data storage with pluggable backends

Adding a new protocol means implementing only the Broker parsing layer. Adding a new storage backend means implementing only the Storage Engine interface. The core architecture does not change.

## 🚀 Quick Start

### One-Line Installation

```bash
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
broker-server start
```

### Multi-Protocol in Action

```bash
# Publish via MQTT
mqttx pub -h localhost -p 1883 -t "robustmq.multi.protocol" -m "Hello RobustMQ!"

# Consume the same message via Kafka
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic robustmq.multi.protocol --from-beginning

# Consume the same message via NATS
nats sub "robustmq.multi.protocol"
```

### mq9 Agent Mailbox in Action

```bash
# Agent A creates a mailbox — returns mail_address
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":3600}'

# Agent B sends to Agent A (works even if A is offline)
nats pub '$mq9.AI.MAILBOX.MSG.{mail_address_a}' '{"type":"task","payload":"hello","ts":1234567890}'

# Agent A subscribes and receives all non-expired messages in priority order
nats sub '$mq9.AI.MAILBOX.MSG.{mail_address_a}.*'
```

### Web Dashboard

Access `http://localhost:8080` for cluster monitoring and management.

<div align="center">
  <img src="docs/images/web-ui.jpg" alt="Web UI" width="45%" style="margin-right: 2%;">
  <img src="docs/images/web-ui-cluster.jpg" alt="Web UI Cluster" width="45%">
</div>

### Try Online Demo

- **MQTT Server**: `117.72.92.117:1883` (admin/robustmq)
- **Web Dashboard**: http://demo.robustmq.com:8080

📚 **Full installation and usage guide: [Documentation](https://robustmq.com/)**

## 🔧 Development

```bash
git clone https://github.com/robustmq/robustmq.git
cd robustmq
cargo run --package cmd --bin broker-server

make build           # Basic build
make build-full      # With frontend
```

📚 **[Build Guide](https://robustmq.com/QuickGuide/Build-and-Package.html)**

## 📚 Documentation

- **📖 [Official Documentation](https://robustmq.com/)** — Comprehensive guides and API references
- **🤖 [mq9 Overview](https://robustmq.com/en/mq9/Overview.html)** — Design rationale and core concepts
- **⚡ [mq9 Quick Start](https://robustmq.com/en/mq9/QuickStart.html)** — CLI walkthrough in 10 minutes
- **🔌 [mq9 SDK Integration](https://robustmq.com/en/mq9/SDK.html)** — Python, Go, JavaScript, Java, Rust, C#
- **🧩 [mq9 NATS Client Usage](https://robustmq.com/en/mq9/NatsClient.html)** — Use any NATS client directly
- **🔗 [mq9 LangChain Integration](https://robustmq.com/en/mq9/LangChain.html)** — LangChain & LangGraph toolkit
- **🗺️ [mq9 Roadmap](https://robustmq.com/en/mq9/Roadmap.html)** — Semantic routing, intent policy, context awareness
- **🚀 [Quick Start Guide](https://robustmq.com/QuickGuide/Overview.html)** — Get up and running in minutes
- **🔧 [MQTT Documentation](https://robustmq.com/RobustMQ-MQTT/Overview.html)** — MQTT-specific features and configuration
- **💻 [Command Reference](https://robustmq.com/RobustMQ-Command/Mqtt-Broker.html)** — CLI commands and usage
- **🎛️ [Web Console](https://github.com/robustmq/robustmq-copilot)** — Management interface

### mq9 SDK

Install the mq9 SDK for your language:

```bash
# Python
pip install robustmq

# JavaScript / TypeScript
npm install @robustmq/sdk

# Rust
cargo add robustmq

# Go
go get github.com/robustmq/robustmq-sdk/go

# Java (Maven)
# <dependency><groupId>com.robustmq</groupId><artifactId>robustmq</artifactId><version>0.3.5</version></dependency>

# C# (.NET)
dotnet add package RobustMQ
```

Or use any NATS client library directly — no SDK required.

## 🤝 Contributing

We welcome contributions. See our [Contribution Guide](https://robustmq.com/en/ContributionGuide/GitHub-Contribution-Guide.html) and [Good First Issues](https://github.com/robustmq/robustmq/labels/good%20first%20issue).

## 🌐 Community

- **🎮 [Discord](https://discord.gg/sygeGRh5)** — Real-time chat and collaboration
- **🐛 [GitHub Issues](https://github.com/robustmq/robustmq/issues)** — Bug reports and feature requests
- **💡 [GitHub Discussions](https://github.com/robustmq/robustmq/discussions)** — General discussions

### 🇨🇳 Chinese Community

- **微信群**: Join our WeChat group for Chinese-speaking users

  <div align="center">
    <img src="docs/images/wechat-group.jpg" alt="WeChat Group QR Code" width="200" />
  </div>

- **开发者微信**: If the group QR code has expired, follow our official WeChat account

  <div align="center">
    <img src="docs/images/wechat.jpg" alt="WeChat Official Account QR Code" width="200" />
  </div>

## License

RobustMQ is licensed under the [Apache License 2.0](LICENSE). See [LICENSING.md](LICENSING.md) for details.

---

<div align="center">
  <sub>Built with ❤️ by the RobustMQ team and <a href="https://github.com/robustmq/robustmq/graphs/contributors">contributors</a>.</sub>
</div>
