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

## 🌟 Introduction & Vision
RobustMQ is a next-generation unified messaging infrastructure built with Rust for AI, IoT, and data-intensive systems. It is designed to deliver high throughput, predictable latency, and low operational complexity from edge devices to cloud clusters.

![RobustMQ Architecture](docs/images/robustmq-architecture.jpg)

### 🎯 Why RobustMQ
- ⚡ **High-performance core**: Rust-native implementation with low latency and low memory overhead.
- 🔁 **Unified protocol access**: MQTT + Kafka compatibility in one system, reducing architecture duplication.
- 🧠 **AI-ready data path**: Object storage integration and multi-tier cache to reduce data loading bottlenecks.
- 🌍 **Edge-to-cloud consistency**: One architecture for edge gateways, regional clusters, and central cloud.

### 🧭 Vision
Enable data to move freely and efficiently across AI agents, training clusters, IoT devices, and analytics platforms through one unified messaging layer.

### 🏗️ Workload Fit
- 🤖 **AI workloads**: Lightweight topics for agent communication, shared subscription for elastic training consumers.
- 📡 **IoT workloads**: MQTT ingestion with Kafka consumption on the same data plane (MQTT in / Kafka out).
- 📊 **Data workloads**: Flexible storage modes for balancing throughput, durability, and cost.


## 🗺️ RobustMQ Development Roadmap

**🚀 Long-term Vision**

Enable data to flow freely across AI training clusters, millions of Agents, IoT devices, and the cloud — via the optimal path, at the lowest latency, and with minimal cost.

**✨ Roadmap**
- **Phase 1**: Foundation (Completed) — Built a scalable technical architecture with solid, streamlined, and abstraction-friendly code implementation. Established a robust foundation for multi-protocol adaptation, pluggable storage, extensibility, and elasticity.

- **Phase 2**: MQTT Broker (Initial Release) — Delivered a stable, high-performance MQTT Broker with MQTT 3.x/5.0 protocol support, optimized for edge deployment with package size under 20MB. Core protocol capabilities are in place and will continue to evolve in future releases.

- **Phase 3**: Kafka Protocol & AI Capabilities (Starting) — With the MQTT Broker initially complete, now launching Kafka protocol adaptation and AI capability development. Prioritizing validation of AI training data caching acceleration and million-level lightweight topic feasibility, using AI workloads to drive Kafka protocol implementation; progressively building out full standard Kafka protocol compatibility on this foundation.

## ✨ Features

- ⚙️ **Unified Messaging Layer**: MQTT 3.1/3.1.1/5.0 + Kafka compatibility, enabling MQTT in / Kafka out in one platform.
- 🚀 **Performance by Design**: Rust implementation, low memory usage, low latency, and no GC pause behavior.
- 🧠 **AI Data Acceleration**: S3/MinIO integration with multi-tier caching (memory/SSD/object storage) to improve data path efficiency.
- 🤖 **Agent-scale Topics**: Support for massive lightweight topic counts with isolation and observability per workload.
- 🔄 **Elastic Consumption Model**: Shared subscription to scale consumers beyond rigid partition-concurrency coupling.
- 💾 **Flexible Storage Modes**: Memory, hybrid, persistent, and tiered storage strategies configurable per topic.
- 🌐 **Edge-to-Cloud Deployment**: Consistent runtime model for edge nodes and cloud clusters, with offline buffering + sync.
- 🛡️ **Ops-friendly Architecture**: Single-binary deployment, built-in Raft consensus, and simplified operations.

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
