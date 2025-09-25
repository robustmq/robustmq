# RobustMQ MQTT Overview

## Introduction

RobustMQ MQTT is a complete implementation of the MQTT protocol by RobustMQ, providing high-performance and highly available message delivery services. As an enterprise-grade MQTT message broker, RobustMQ MQTT is fully compatible with MQTT 3.1, 3.1.1, and 5.0 protocol standards, ensuring seamless integration with existing MQTT ecosystems.

## Core Advantages

- **100% Protocol Compatibility**: Fully supports standard MQTT protocols, compatible with all MQTT clients, SDKs, and tools
- **High-Performance Architecture**: Supports millions of single-machine connections and billions of cluster connections
- **Enterprise Features**: Provides enterprise-grade features such as cluster deployment, high availability, and security authentication
- **Complete Functionality**: Covers all core MQTT protocol features, aligned with mainstream MQTT broker functionality
- **Cloud-Native Support**: Supports Kubernetes deployment and cloud-native architecture

## Core Feature Set

### Protocol Support

| Feature | Description | Support Status |
|---------|-------------|----------------|
| MQTT Protocol | Complete support for all MQTT 3.1/3.1.1/5.0 features | ✅ Fully Supported |
| Network Protocols | Supports TCP, SSL, WebSocket, WebSockets, QUIC protocol access | ✅ Fully Supported |
| Publish/Subscribe | Standard MQTT publish-subscribe pattern | ✅ Fully Supported |
| QoS Levels | Supports QoS 0, 1, 2 levels | ✅ Fully Supported |
| Wildcard Subscription | Supports single-level wildcard `+` and multi-level wildcard `#` | ✅ Fully Supported |

### MQTT 5.0 Features

| Feature | Description | Support Status |
|---------|-------------|----------------|
| User Properties | User Properties support | ✅ Fully Supported |
| Topic Alias | Topic Alias support | ✅ Fully Supported |
| Payload Format Indicator | Payload Format Indicator & Content Type | ✅ Fully Supported |
| Shared Subscriptions | Shared Subscriptions | ✅ Fully Supported |
| Subscription Options | Subscription Options | ✅ Fully Supported |
| Subscription Identifier | Subscription Identifier | ✅ Fully Supported |
| Message Expiry Interval | Message Expiry Interval | ✅ Fully Supported |
| Maximum Packet Size | Maximum Packet Size | ✅ Fully Supported |
| Request/Response | Request/Response pattern | ✅ Fully Supported |

### Advanced Features

| Feature | Description | Support Status |
|---------|-------------|----------------|
| Session Management | Supports Session persistence and expiration | ✅ Fully Supported |
| Retained Messages | Retained Messages | ✅ Fully Supported |
| Will Messages | Will Messages | ✅ Fully Supported |
| Exclusive Subscriptions | Exclusive Subscriptions | ✅ Fully Supported |
| Delayed Publishing | Delayed Publishing | ✅ Fully Supported |
| Auto Subscription | Auto Subscription | ✅ Fully Supported |
| Topic Rewrite | Topic Rewrite | ✅ Fully Supported |
| System Topics | System Topics | ✅ Fully Supported |
| Slow Subscription Statistics | Slow Subscription Statistics | ✅ Fully Supported |

### Security & Authentication

| Feature | Description | Support Status |
|---------|-------------|----------------|
| Password Authentication | Supports built-in database, MySQL, Redis password authentication | ✅ Fully Supported |
| Access Control | Supports built-in database, MySQL, Redis authorization implementation | ✅ Fully Supported |
| Blacklist | Client blacklist management | ✅ Fully Supported |
| Connection Jitter Detection | Connection jitter detection and protection | ✅ Fully Supported |

### Data Integration & Storage

| Feature | Description | Support Status |
|---------|-------------|----------------|
| Data Integration | Supports File, Kafka bridge connectors | ✅ Fully Supported |
| Schema Support | Supports JSON, Protobuf, AVRO formats | ✅ Fully Supported |
| Offline Messages | Supports storage engines based on Memory, RocksDB, MySQL, Journal Engine, S3, Minio | ✅ Fully Supported |
| Message Storage Strategy | Messages are automatically discarded when Topic has no subscribers | ✅ Fully Supported |

### Monitoring & Operations

| Feature | Description | Support Status |
|---------|-------------|----------------|
| Metrics Monitoring | Supports cluster/Topic dimension monitoring metrics | ✅ Fully Supported |
| Prometheus Integration | Supports Prometheus metrics export | ✅ Fully Supported |
| Distributed Tracing | Supports distributed tracing | ✅ Fully Supported |

### Deployment & Scaling

| Feature | Description | Support Status |
|---------|-------------|----------------|
| Cluster Deployment | Stateless Broker node deployment, single cluster supports hundreds to thousands of Broker nodes | ✅ Fully Supported |
| Single Machine Max Connections | Single machine can handle millions of connections | ✅ Fully Supported |
| Cluster Max Connections | Cluster can handle billions of connections | ✅ Fully Supported |
| Cloud-Native Support | Supports Kubernetes deployment and cloud-native architecture | ✅ Fully Supported |

## Feature Comparison with EMQX

### Core Capability Comparison

| Feature Category | Feature | EMQX | RobustMQ MQTT | Notes |
|------------------|---------|------|---------------|-------|
| **MQTT Protocol** | MQTT 5.0 Broker | ✅ Supported | ✅ Supported | Fully Compatible |
| | MQTT over QUIC | ✅ Supported | ✅ Supported | Fully Compatible |
| | MQTT Extensions | ✅ Supported | ✅ Supported | Fully Compatible |
| **Advanced Features** | Multi-Protocol Gateway | ✅ Supported | ❌ Not Supported | Planned Support |
| | Multi-Tenancy | ✅ Supported | ❌ Not Supported | Planned Support |
| | Cluster Connection | ✅ Supported | ❌ Not Supported | Planned Support |
| | Rule Engine | ✅ Supported | ❌ Not Supported | Planned Support |
| | Flow Designer | ✅ Supported | ❌ Not Supported | Planned Support |
| | File Transfer | ✅ Supported | ❌ Not Supported | Planned Support |
| | Edge Computing | ✅ Supported | ❌ Not Supported | Planned Support |
| **Data Management** | Event History | ✅ Supported | ✅ Supported | Fully Compatible |
| | Data Persistence | ✅ Supported | ✅ Supported | Fully Compatible |
| | Schema Registry | ✅ Supported | ✅ Supported | Fully Compatible |
| | Message Encoding/Decoding | ✅ Supported | ✅ Supported | Fully Compatible |
| | Message Validation | ✅ Supported | ✅ Supported | Fully Compatible |
| **Data Integration** | Kafka Integration | ✅ Supported | ✅ Supported | Fully Compatible |
| | Enterprise Data Integration | ✅ Supported (40+) | ✅ Supported (2+) | Continuously Expanding |
| **Operations & Monitoring** | Troubleshooting | ✅ Supported | ✅ Supported | Fully Compatible |
| | Cloud-Native & K8s | ✅ Supported | ✅ Supported | Fully Compatible |

### Feature Coverage

- **MQTT Core Features**: 100% coverage
- **Enterprise Features**: 90% coverage
- **Data Integration**: Basic functionality covered, continuously expanding
- **Advanced Features**: Partial coverage, planned gradual support

## Management Tools

### Dashboard

RobustMQ Dashboard has completed support for RobustMQ MQTT protocol features, providing an intuitive web interface for managing and monitoring MQTT services.

**Access URL**: [http://117.72.92.117:8080/](http://117.72.92.117:8080/)

**Main Features**:

- Client connection management
- Topic subscription monitoring
- Message traffic statistics
- System performance monitoring
- Configuration management

### Command Line Tools

RobustMQ MQTT supports the `robust-ctl mqtt` command line tool, providing complete MQTT management functionality.

**Main Features**:

- Client management
- Topic management
- Message publishing/subscribing
- System status queries
- Configuration management

For detailed documentation, please refer to: [robustmq-ctl mqtt](../RobustMQ-Command/CLI_COMMON.md)

## Quick Start

1. **Installation & Deployment**: Refer to [Installation & Deployment Guide](../InstallationDeployment/Docker-Deployment)
2. **Configuration Management**: Refer to [Configuration Documentation](../Configuration/COMMON)
3. **Client Connection**: Refer to [Client SDKs](SDK/c-sdk)
4. **Feature Usage**: Refer to detailed documentation for each feature module
