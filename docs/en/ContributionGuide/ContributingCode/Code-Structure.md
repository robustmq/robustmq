# RobustMQ Code Structure Guide

## Overview

RobustMQ is a high-performance, multi-protocol message queue system built with Rust. This document provides a comprehensive guide to the project's code organization structure, helping developers quickly understand and contribute to the project.

## Project Architecture

RobustMQ adopts a distributed layered architecture consisting of four main layers:

- **Metadata and Scheduling Layer (Placement Center)**: Manages cluster metadata storage and scheduling
- **Multi-protocol Computing Layer**: Supports adaptation of various message protocols
- **Storage Adapter Layer**: Provides pluggable storage abstraction
- **Storage Layer**: Concrete storage engine implementations

![RobustMQ Architecture](../../../images/robustmq-architecture.png)

## Source Code Directory Structure

### Root Directory Structure

```
robustmq/
├── src/                    # Source code directory
├── docs/                   # Project documentation
├── config/                 # Configuration file templates
├── docker/                 # Docker-related files
├── example/                # Examples and test scripts
├── scripts/                # Build and deployment scripts
├── tests/                  # Integration tests
├── Cargo.toml             # Rust workspace configuration
└── README.md              # Project description
```

## Core Module Details

### 1. Server Components

#### `src/broker-server/`
**Purpose**: Main Broker server implementation
- **Responsibility**: Coordinates various protocol brokers, provides unified service entry
- **Key Files**:
  - `cluster_service.rs` - Cluster service management
  - `grpc.rs` - gRPC service interface
  - `metrics.rs` - Performance metrics collection

#### `src/meta-service/`
**Purpose**: Metadata service (Placement Center)
- **Responsibility**: Cluster metadata storage, node management, fault recovery
- **Core Modules**:
  - `core/` - Core cache and controllers
  - `raft/` - Raft consensus algorithm implementation
  - `storage/` - Metadata storage engine
  - `controller/` - Protocol-specific controllers

#### `src/journal-server/`
**Purpose**: Log storage server
- **Responsibility**: Message persistence storage, index management
- **Core Modules**:
  - `segment/` - Storage segment management
  - `index/` - Index management
  - `core/` - Core storage logic

### 2. Protocol Implementations (Protocol Brokers)

#### `src/mqtt-broker/`
**Purpose**: MQTT protocol implementation
- **Responsibility**: Supports MQTT 3.x/4.x/5.x protocols
- **Core Modules**:
  - `handler/` - MQTT message handlers
  - `subscribe/` - Subscription management
  - `security/` - Authentication and authorization
  - `bridge/` - Message bridging functionality
  - `observability/` - Observability support

#### `src/kafka-broker/`
**Purpose**: Kafka protocol implementation
- **Responsibility**: Kafka-compatible message processing
- **Status**: Under development

#### `src/amqp-broker/`
**Purpose**: AMQP protocol implementation
- **Responsibility**: AMQP protocol support
- **Status**: Under development

### 3. Management & Tools

#### `src/admin-server/`
**Purpose**: Web management console backend
- **Responsibility**: Provides HTTP API for cluster management
- **Core Modules**:
  - `mqtt/` - MQTT-related management interfaces
  - `cluster/` - Cluster management interfaces
  - `server.rs` - HTTP server implementation

#### `src/cli-command/`
**Purpose**: Command-line tools
- **Responsibility**: Provides cluster management and operations commands
- **Supported Commands**:
  - MQTT management commands
  - Cluster management commands
  - Journal management commands

#### `src/cli-bench/`
**Purpose**: Performance testing tools
- **Responsibility**: Provides various performance testing scenarios
- **Test Types**:
  - MQTT publish/subscribe tests
  - KV storage tests
  - Raft consistency tests

### 4. Core Libraries & Utilities

#### `src/common/`
**Purpose**: Common component library
- **base/** - Basic utilities and type definitions
- **config/** - Configuration management
- **metadata-struct/** - Metadata structure definitions
- **network-server/** - Network server framework
- **rocksdb-engine/** - RocksDB storage engine wrapper
- **metrics/** - Metrics collection framework
- **security/** - Security-related components

#### `src/protocol/`
**Purpose**: Protocol definitions and codecs
- **Responsibility**: Defines data structures and encoding/decoding logic for various protocols
- **Supported Protocols**:
  - MQTT (3.x/4.x/5.x)
  - Kafka
  - AMQP
  - RobustMQ internal protocol

#### `src/storage-adapter/`
**Purpose**: Storage adapter
- **Responsibility**: Provides unified storage interface supporting multiple storage backends
- **Supported Storage**:
  - Memory storage
  - RocksDB
  - MySQL
  - S3
  - Journal Server

### 5. Specialized Modules

#### `src/delay-message/`
**Purpose**: Delayed message processing
- **Responsibility**: Implements message delayed delivery functionality

#### `src/schema-register/`
**Purpose**: Message schema registry
- **Responsibility**: Message format validation and schema management
- **Supported Formats**: JSON, Avro, Protobuf

#### `src/message-expire/`
**Purpose**: Message expiration handling
- **Responsibility**: Cleans up expired messages

#### `src/grpc-clients/`
**Purpose**: gRPC client library
- **Responsibility**: Provides client implementations for various gRPC services
- **Connection Pool Management**: Optimizes connection reuse

#### `src/journal-client/`
**Purpose**: Journal client
- **Responsibility**: Provides client interface for Journal Server

## Module Dependencies

### Dependency Hierarchy

```
Application Layer:  broker-server, admin-server, cli-*
                   ↓
Protocol Layer:    mqtt-broker, kafka-broker, amqp-broker
                   ↓
Service Layer:     meta-service, journal-server
                   ↓
Adapter Layer:     storage-adapter, grpc-clients
                   ↓
Foundation Layer:  common/*, protocol
```

### Core Dependency Relationships

1. **broker-server** depends on all protocol brokers
2. **Protocol brokers** depend on meta-service and storage-adapter
3. **meta-service** uses Raft algorithm for consistency guarantees
4. **storage-adapter** provides unified storage abstraction
5. **common** modules are depended upon by all other modules

## Configuration and Deployment

### Configuration File Structure

```
config/
├── server.toml              # Main configuration file
├── server.toml.template     # Configuration template
├── server-tracing.toml      # Tracing configuration
├── version.ini              # Version information
└── certs/                   # TLS certificates
```

### Build Artifacts

```
target/
├── debug/                   # Debug builds
├── release/                 # Release builds
└── bin/                     # Executables
    ├── robust-server        # Main server
    ├── robust-ctl          # Command-line tool
    └── robust-bench        # Performance testing tool
```

## Development Guide

### Adding New Protocol Support

1. Create a new protocol directory under `src/` (e.g., `new-protocol-broker/`)
2. Define protocol structures in `src/protocol/`
3. Implement protocol encoding/decoding logic
4. Integrate the new protocol in `src/broker-server/`
5. Add corresponding tests and documentation

### Extending Storage Backends

1. Implement a new storage adapter in `src/storage-adapter/`
2. Implement the `StorageAdapter` trait
3. Add configuration options for the new storage type in config files
4. Add integration tests

### Adding Management Features

1. Add new HTTP interfaces in `src/admin-server/`
2. Add corresponding command-line commands in `src/cli-command/`
3. Update relevant documentation

## Testing Structure

### Unit Tests
Each module contains a `tests/` directory for module-level unit tests.

### Integration Tests
The `tests/` directory contains end-to-end integration tests:
- Cluster functionality tests
- Protocol compatibility tests
- Performance tests

### Performance Tests
Use `src/cli-bench/` tools for performance testing and benchmarking.

## Monitoring and Observability

### Metrics Collection
- `src/common/metrics/` - Metrics collection framework
- Prometheus metrics export
- Custom metrics support

### Distributed Tracing
- OpenTelemetry integration
- Distributed tracing
- Performance analysis support

### Log Management
- Structured logging
- Multi-level log output
- Log rotation support

## Summary

RobustMQ adopts a modular design with clear component responsibilities, making it easy to develop and maintain. By understanding this code structure, developers can:

1. Quickly locate relevant functional code
2. Understand inter-module dependencies
3. Add new features following established patterns
4. Effectively troubleshoot issues and optimize performance

New contributors are recommended to start by understanding `src/common/` and `src/protocol/`, then dive into specific protocol implementations.

## Contributing Guidelines

When contributing to RobustMQ:

1. **Follow Rust best practices** - Use idiomatic Rust code
2. **Maintain module boundaries** - Respect the layered architecture
3. **Add comprehensive tests** - Include unit and integration tests
4. **Document your changes** - Update relevant documentation
5. **Consider performance** - Profile and optimize critical paths
6. **Handle errors gracefully** - Use proper error handling patterns

For more detailed contribution guidelines, see the [Contributing Guide](../README.md).