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
    Next-generation unified communication infrastructure for AI, IoT, and big data
</h3>

<p align="center">
  <a href="#-introduction--vision">Introduction & Vision</a> •
  <a href="#-features">Features</a> •
  <a href="#%EF%B8%8F-robustmq-development-roadmap">Roadmap</a> •
  <a href="#%EF%B8%8F-architecture">Architecture</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-contributing">Contributing</a> •
  <a href="#-community">Community</a>
</p>

---

> **⚠️ Development Status**
> RobustMQ is in early development and **not production-ready**. We are currently in **Phase 1** focusing on building a solid architectural foundation. See [Roadmap](#%EF%B8%8F-robustmq-development-roadmap) for detailed development plan and timeline.

## 🚀 Introduction
RobustMQ is a next-generation unified messaging platform built with Rust, designed for AI, IoT, and big data workloads. Fully compatible with Kafka and MQTT 3.1/3.1.1/5.0 protocols, existing applications can seamlessly connect using standard Kafka SDKs with zero migration cost to unlock the full power of RobustMQ.

![RobustMQ Architecture](docs/images/robustmq-architecture.jpg)

For AI workloads, RobustMQ serves as an intelligent data scheduling and caching layer. Direct object storage (S3/MinIO) integration with three-tier intelligent caching eliminates the need for data pre-import, removes I/O bottlenecks, and dramatically improves GPU utilization. A single cluster supports millions of lightweight topics, providing each AI Agent with an independent communication channel with fine-grained isolation and monitoring. Shared subscription mode enables GPU training nodes to scale elastically without being constrained by partition count.

For IoT workloads, a unified storage layer enables MQTT in / Kafka out — data ingested from IoT devices via MQTT can be consumed directly by AI and big data systems using the Kafka protocol, replacing dual MQTT + Kafka broker architectures with a single system. Minimal memory footprint supports edge deployment, with offline caching and automatic sync covering the full pipeline from edge gateways to cloud clusters.

For big data workloads, RobustMQ enhances the Kafka protocol with an intelligent storage engine offering four modes — memory, hybrid, persistent, and tiered. Each topic can be independently configured, with hot data served at maximum speed and cold data automatically tiered to S3, balancing performance and cost.


## 🗺️ RobustMQ Development Roadmap

**🚀 Long-term Vision**

Enable data to flow freely across AI training clusters, millions of Agents, IoT devices, and the cloud — via the optimal path, at the lowest latency, and with minimal cost.

**✨ Roadmap**
- **Phase 1**: Foundation (Completed) — Built a scalable technical architecture with solid, streamlined, and abstraction-friendly code implementation. Established a robust foundation for multi-protocol adaptation, pluggable storage, extensibility, and elasticity.

- **Phase 2**: MQTT Broker (Initial Release) — Delivered a stable, high-performance MQTT Broker with MQTT 3.x/5.0 protocol support, optimized for edge deployment with package size under 20MB. Core protocol capabilities are in place and will continue to evolve in future releases.

- **Phase 3**: Kafka Protocol & AI Capabilities (Starting) — With the MQTT Broker initially complete, now launching Kafka protocol adaptation and AI capability development. Prioritizing validation of AI training data caching acceleration and million-level lightweight topic feasibility, using AI workloads to drive Kafka protocol implementation; progressively building out full standard Kafka protocol compatibility on this foundation.

## ✨ Features

- 🚀 **Extreme Performance**: Built with Rust, microsecond latency, zero GC pauses, million-level QPS per node, minimal memory footprint for edge deployment
- 🔌 **Dual Protocol Unification**: Fully compatible with MQTT 3.1/3.1.1/5.0 and Kafka protocols, unified storage layer enables MQTT in / Kafka out, one system replaces two
- 🎯 **AI Training Acceleration**: Direct object storage (S3/MinIO) integration, three-tier intelligent caching (memory/SSD/S3), no data pre-import required, eliminates I/O bottlenecks, dramatically improves GPU utilization
- 🤖 **Agent Communication**: Millions of lightweight topics per cluster, independent channel per Agent, fine-grained isolation and monitoring, per-Agent cost attribution
- 🔄 **Elastic Consumption**: Shared subscription mode breaks Kafka's "concurrency = partition count" constraint, GPU training nodes scale freely without topic reconfiguration
- 💾 **Intelligent Storage Engine**: Four modes — memory, hybrid, persistent, tiered — independently configurable per topic, hot data at full speed, cold data auto-tiered to S3, balancing performance and cost
- 🌐 **Edge to Cloud**: Minimal memory footprint, unified deployment from edge gateways to cloud clusters, offline caching + automatic sync covering the full IoT pipeline
- 🛠️ **Minimal Deployment**: Single binary, zero external dependencies, built-in Raft consensus, ready out of the box with minimal operational overhead

## 🚀 Quick Start

### One-Line Installation

```bash
# Install and start RobustMQ
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
broker-server start
```

### Quick Test

```bash
# Test MQTT connection
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"
mqttx sub -h localhost -p 1883 -t "test/topic"
```

### Web Dashboard

Access `http://localhost:8080` for cluster monitoring and management.

<div align="center">
  <img src="docs/images/web-ui.jpg" alt="Web UI" width="45%" style="margin-right: 2%;">
  <img src="docs/images/web-ui-cluster.jpg" alt="Web UI Cluster" width="45%">
</div>

### Try Online Demo

- **MQTT Server**: `117.72.92.117:1883` (admin/robustmq)
- **Web Dashboard**: http://demo.robustmq.com

📚 **For detailed installation and usage guides, see our [Documentation](https://robustmq.com/)**

## 🔧 Development

```bash
# Clone and build
git clone https://github.com/robustmq/robustmq.git
cd robustmq
cargo run --package cmd --bin broker-server

# Build packages
make build              # Basic build
make build-full         # With frontend
```

📚 **For detailed build options, see our [Build Guide](https://robustmq.com/QuickGuide/Build-and-Package.html)**

## 📚 Documentation

- **📖 [Official Documentation](https://robustmq.com/)** - Comprehensive guides and API references
- **🚀 [Quick Start Guide](https://robustmq.com/QuickGuide/Overview.html)** - Get up and running in minutes
- **🔧 [MQTT Documentation](https://robustmq.com/RobustMQ-MQTT/Overview.html)** - MQTT-specific features and configuration
- **💻 [Command Reference](https://robustmq.com/RobustMQ-Command/Mqtt-Broker.html)** - CLI commands and usage
- **🎛️ [Web Console](https://github.com/robustmq/robustmq-copilot)** - Management interface

## 🤝 Contributing

We welcome contributions! Check out our [Contribution Guide](https://robustmq.com/en/ContributionGuide/GitHub-Contribution-Guide.html) and [Good First Issues](https://github.com/robustmq/robustmq/labels/good%20first%20issue).


## 🌐 Community

- **🎮 [Discord](https://discord.gg/sygeGRh5)** - Real-time chat and collaboration
- **🐛 [GitHub Issues](https://github.com/robustmq/robustmq/issues)** - Bug reports and feature requests
- **💡 [GitHub Discussions](https://github.com/robustmq/robustmq/discussions)** - General discussions

### 🇨🇳 Chinese Community

- **微信群**: Join our WeChat group for Chinese-speaking users

  <div align="center">
    <img src="docs/images/wechat-group.jpg" alt="WeChat Group QR Code" width="200" />
  </div>

- **开发者微信**: If the group QR code has expired, welcome to follow our official WeChat account!

  <div align="center">
    <img src="docs/images/wechat.jpg" alt="WeChat Official Account QR Code" width="200" />
  </div>
## 📄 License

RobustMQ is licensed under the [Apache License 2.0](LICENSE), which  strikes a balance between open collaboration and allowing you to use the software in your projects, whether open source or proprietary.

---

<div align="center">
  <sub>Built with ❤️ by the RobustMQ team and <a href="https://github.com/robustmq/robustmq/graphs/contributors">contributors</a>.</sub>
</div>
