<p align="center">
  <picture>
    <img alt="RobustMQ Logo" src="docs/images/robustmq-logo.png" width="300">
  </picture>
</p>

<p align="center">
  <a href="https://deepwiki.com/robustmq/robustmq"><img src="https://deepwiki.com/badge.svg" alt="Ask DeepWiki"></a>
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
    New generation of cloud-native and AI-native messaging infrastructure
</h3>

<p align="center">
  <a href="#-introduction">Introduction</a> •
  <a href="#-features">Features</a> •
  <a href="#-architecture">Architecture</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-build-script">Build Script</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-contributing">Contributing</a> •
  <a href="#-community">Community</a>
</p>

---

> **⚠️ Development Status**
> This project is currently in its early preview stage and is undergoing rapid iteration and testing. A stable release is expected in the second half of 2025. We are actively working towards making RobustMQ production-ready and aim to become a top-level Apache project in the message queue ecosystem.

## 🚀 Introduction

RobustMQ is a next-generation, high-performance, multi-protocol message queue built in Rust. Our vision is to create a unified messaging infrastructure tailored for modern cloud-native and AI systems.

## ✨ Features

- **🚀 High Performance**: Built with Rust, ensuring memory safety, zero-cost abstractions, and blazing-fast performance
- **🏗️ Distributed Architecture**: Separation of compute, storage, and scheduling for optimal scalability and resource utilization
- **🔌 Multi-Protocol Support**: Native support for MQTT (3.x/4.x/5.x), AMQP, Kafka, and RocketMQ protocols
- **💾 Pluggable Storage**: Modular storage layer supporting local files, S3, HDFS, and other storage backends
- **☁️ Cloud-Native**: Kubernetes-ready with auto-scaling, service discovery, and observability built-in
- **🏢 Multi-Tenancy**: Support for virtual clusters within a single physical deployment
- **🔐 Security First**: Built-in authentication, authorization, and encryption support
- **📊 Observability**: Comprehensive metrics, tracing, and logging with Prometheus and OpenTelemetry integration
- **🎯 User-Friendly**: Simple deployment, intuitive management console, and extensive documentation

## 🏗️ Architecture

![RobustMQ Architecture](docs/images/robustmq-architecture.png)

### Core Components

- **Broker Server**: High-performance message handling with multi-protocol support
- **Meta Service**: Metadata management and cluster coordination using Raft consensus
- **Journal Server**: Persistent storage layer with pluggable backends
- **Web Console**: Management interface for monitoring and administration

### Key Design Principles

- **One Binary, One Process**: Simplified deployment and operations
- **Protocol Isolation**: Different protocols use dedicated ports (MQTT: 1883/1884/8083/8084, Kafka: 9092, gRPC: 1228)
- **Fault Tolerance**: Built-in replication and automatic failover
- **Horizontal Scaling**: Add capacity by simply adding more nodes

## 🚀 Quick Start

### One-Line Installation (Recommended)

```bash
# Install latest version automatically
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# Start the server
broker-server start
```

### Other Installation Methods

- **📦 [Pre-built Binaries](https://github.com/robustmq/robustmq/releases)** - Download for your platform
- **🔨 [Build from Source](https://robustmq.com/QuickGuide/Build-and-Package.html)** - Compile from source code
- **🐳 [Docker](https://robustmq.com/InstallationDeployment/Docker.html)** - Container deployment (Coming Soon)

### Quick Verification

```bash
# Check installation
broker-server --version
cli-command status

# Connect with MQTT client to localhost:1883
```

📚 **For detailed installation instructions, see our [Installation Guide](https://robustmq.com/InstallationDeployment/Overview.html)**

🇨🇳 **中文用户请查看 [快速安装指南](docs/zh/QuickGuide/Quick-Install.md)**

## 🔧 Development

### Build from Source

```bash
# Clone and build
git clone https://github.com/robustmq/robustmq.git
cd robustmq
cargo run --package cmd --bin broker-server
```

### Build Scripts

```bash
# Build package for current platform
./scripts/build.sh

# Build with frontend
./scripts/build.sh --with-frontend
```

📚 **For advanced build options and cross-platform compilation:**
- **🇺🇸 [Build Guide (English)](https://robustmq.com/en/QuickGuide/Build-and-Package.html)**
- **🇨🇳 [构建指南 (中文)](https://robustmq.com/zh/QuickGuide/Build-and-Package.html)**

## 📚 Documentation

- **📖 [Official Documentation](https://robustmq.com/)** - Comprehensive guides and API references
- **🚀 [Quick Start Guide](https://robustmq.com/QuickGuide/Overview.html)** - Get up and running in minutes
- **🔧 [MQTT Documentation](https://robustmq.com/RobustMQ-MQTT/Overview.html)** - MQTT-specific features and configuration
- **💻 [Command Reference](https://robustmq.com/RobustMQ-Command/Mqtt-Broker.html)** - CLI commands and usage
- **🎛️ [Web Console](https://github.com/robustmq/robustmq-copilot)** - Management interface

<div align="center">
  <img src="docs/images/web-ui.jpg" alt="Web UI" width="45%" style="margin-right: 2%;">
  <img src="docs/images/web-ui-cluster.jpg" alt="Web UI Cluster" width="45%">
</div>

## 💻 Development

### Fast CI/CD Builds

RobustMQ uses pre-built dependency cache images to speed up CI/CD pipelines:

- **⚡ 5-10x faster** build times (2-3 min vs 15-18 min)
- **📦 Cache Image:** `ghcr.io/socutes/robustmq/rust-deps:latest`
- **🔄 Auto-updated** when dependencies change

📚 **For CI/CD optimization details, see [Build Documentation](scripts/README.md)**

## 🤝 Contributing

We welcome contributions! Please see our [Contribution Guide](https://robustmq.com/ContributionGuide/GitHub-Contribution-Guide.html) for details.

**Quick Start:**
1. **🔍 Check [Good First Issues](https://github.com/robustmq/robustmq/labels/good%20first%20issue)**
2. **🍴 Fork and create a feature branch**
3. **✅ Make changes with tests**
4. **📤 Submit a pull request**


## 🌐 Community

- **🎮 [Discord](https://discord.gg/sygeGRh5)** - Real-time chat and collaboration
- **🐛 [GitHub Issues](https://github.com/robustmq/robustmq/issues)** - Bug reports and feature requests
- **💡 [GitHub Discussions](https://github.com/robustmq/robustmq/discussions)** - General discussions

### 🇨🇳 Chinese Community

- **微信群**: Join our WeChat group for Chinese-speaking users

  <div align="center">
    <img src="docs/images/wechat-group.jpg" alt="WeChat Group QR Code" width="200" />
  </div>

- **微信公众号**: If the group QR code has expired, welcome to follow our official WeChat account!

  <div align="center">
    <img src="docs/images/WeChat-Official-Account.jpg" alt="WeChat Official Account QR Code" width="200" />
  </div>

## 📄 License

RobustMQ is licensed under the [Apache License 2.0](LICENSE), which strikes a balance between open collaboration and allowing you to use the software in your projects, whether open source or proprietary.

---

<div align="center">
  <sub>Built with ❤️ by the RobustMQ team and <a href="https://github.com/robustmq/robustmq/graphs/contributors">contributors</a>.</sub>
</div>
