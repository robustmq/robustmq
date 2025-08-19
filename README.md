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
    A high-performance distributed message queue built with Rust.
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

### Prerequisites

- **Rust**: 1.70 or later
- **Operating System**: Linux, macOS, or Windows
- **Memory**: Minimum 2GB RAM
- **Storage**: At least 1GB available disk space

### Installation Options

#### Option 1: Build from Source

```bash
# Clone the repository
git clone https://github.com/robustmq/robustmq.git
cd robustmq

# Build and run
cargo run --package cmd --bin broker-server
```

#### Option 2: Pre-built Binaries

**Method 1: Manual Download**

Visit the [releases page](https://github.com/robustmq/robustmq/releases) and download the appropriate package for your platform:

```bash
# Example for Linux x86_64 (replace with your platform)
wget https://github.com/robustmq/robustmq/releases/latest/download/robustmq-v0.1.30-linux-amd64.tar.gz

# Extract the package
tar -xzf robustmq-v0.1.30-linux-amd64.tar.gz
cd robustmq-v0.1.30-linux-amd64

# Run the server
./bin/robust-server start
```

**Available platforms**: `linux-amd64`, `linux-arm64`, `darwin-amd64`, `darwin-arm64`, `windows-amd64`

**Method 2: Automated Install Script** (Recommended)

```bash
# Download and install automatically
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# Or download the script first to review it
wget https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh
chmod +x install.sh
./install.sh --help  # See available options
```

#### Option 3: Docker (Coming Soon)

```bash
docker run -p 1883:1883 -p 9092:9092 robustmq/robustmq:latest
```

### Verify Installation

Once RobustMQ is running, you should see output similar to:

![Console Start](docs/images/console-start.png)

You can verify the installation by connecting with any MQTT client to `localhost:1883` or using the web console.

## 🔧 Build Script

RobustMQ provides a powerful build script (`scripts/build.sh`) for creating distribution packages:

### Quick Usage

```bash
# Build for current platform (default: server component only)
./scripts/build.sh

# Build for specific platform
./scripts/build.sh --platform linux-amd64

# Build for all platforms
./scripts/build.sh --platform all

# Build specific component
./scripts/build.sh --component operator

# Build with custom version
./scripts/build.sh --version v1.0.0

# Show all options
./scripts/build.sh --help
```

### Available Options

| Option | Description | Default |
|--------|-------------|---------|
| `-p, --platform` | Target platform | auto-detect |
| `-c, --component` | Component to build | server |
| `-v, --version` | Build version | git describe |
| `-t, --build-type` | Build type (release/debug) | release |
| `-o, --output` | Output directory | build/ |
| `--all-platforms` | Build for all supported platforms | - |

### Supported Platforms

- **Linux**: `linux-amd64`, `linux-arm64`, `linux-386`, `linux-armv7`
- **macOS**: `darwin-amd64`, `darwin-arm64`
- **Windows**: `windows-amd64`, `windows-386`
- **FreeBSD**: `freebsd-amd64`

### Output

Built packages are saved to `build/` directory with format:
- **Server**: `robustmq-{version}-{platform}.tar.gz`
- **Operator**: `robustmq-operator-{version}-{platform}.tar.gz`

## 📚 Documentation

- **📖 [Official Documentation](https://robustmq.com/)** - Comprehensive guides and API references
- **🚀 [Quick Start Guide](https://robustmq.com/QuickGuide/Overview.html)** - Get up and running in minutes
- **🔧 [MQTT Documentation](https://robustmq.com/RobustMQ-MQTT/Overview.html)** - MQTT-specific features and configuration
- **💻 [Command Reference](https://robustmq.com/RobustMQ-Command/Mqtt-Broker.html)** - CLI commands and usage
- **🎛️ [Web Console](https://github.com/robustmq/robustmq-copilot)** - Management interface

![Web UI](docs/images/web-ui.png)

## 🤝 Contributing

We welcome contributions from the community! RobustMQ is an open-source project, and we're excited to collaborate with developers interested in Rust, distributed systems, and message queues.

### How to Contribute

1. **📋 Read our [Contribution Guide](https://robustmq.com/ContributionGuide/GitHub-Contribution-Guide.html)**
2. **🔍 Check [Good First Issues](https://github.com/robustmq/robustmq/labels/good%20first%20issue)**
3. **🍴 Fork the repository**
4. **🌿 Create a feature branch**
5. **✅ Make your changes with tests**
6. **📤 Submit a pull request**

### Development Setup

```bash
# Clone and setup
git clone https://github.com/robustmq/robustmq.git
cd robustmq

# Run tests
cargo test

# Check code style
cargo clippy
cargo fmt
```

## 🌐 Community

Join our growing community of developers, users, and contributors:

### 💬 Discussion & Support

- **🎮 [Discord Server](https://discord.gg/sygeGRh5)** - Real-time chat, questions, and collaboration
- **🐛 [GitHub Issues](https://github.com/robustmq/robustmq/issues)** - Bug reports and feature requests
- **💡 [GitHub Discussions](https://github.com/robustmq/robustmq/discussions)** - General discussions and ideas

### 🇨🇳 Chinese Community

- **微信群**: Join our WeChat group for Chinese-speaking users

  <div align="center">
    <img src="docs/images/wechat-group.png" alt="WeChat Group QR Code" width="200" />
  </div>

- **个人微信**: If the group QR code has expired, add the developer's personal WeChat:

  <div align="center">
    <img src="docs/images/wechat.jpg" alt="Personal WeChat QR Code" width="200" />
  </div>

## 📄 License

RobustMQ is licensed under the [Apache License 2.0](LICENSE), which strikes a balance between open collaboration and allowing you to use the software in your projects, whether open source or proprietary.

---

<div align="center">
  <sub>Built with ❤️ by the RobustMQ team and <a href="https://github.com/robustmq/robustmq/graphs/contributors">contributors</a>.</sub>
</div>
