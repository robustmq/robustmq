# RobustMQ Overall Architecture

## Overview

RobustMQ is a next-generation, high-performance, multi-protocol message queue built in Rust, designed specifically for cloud-native and AI-native environments. This document provides a comprehensive overview of RobustMQ's overall architecture, design principles, and core components.

## Design Philosophy

RobustMQ is built on the following core design principles:

- **AI-Ready**: Optimized for AI workflows with microsecond-level latency
- **Cloud-Native**: Container-first design with Kubernetes support
- **Multi-Protocol Unification**: Single platform supporting MQTT, Kafka, and AMQP
- **Compute-Storage Separation**: Stateless compute layer for Serverless support
- **Pluggable Storage**: Flexible storage backends for different use cases

## High-Level Architecture

![RobustMQ Architecture](../../images/robustmq-architecture.png)

RobustMQ adopts a distributed, layered architecture with clear separation of concerns:

### 1. Protocol Layer (Multi-Protocol Computing Layer)
- **MQTT Broker**: Handles MQTT 3.x/4.x/5.x protocols
- **Kafka Broker**: Provides Kafka-compatible streaming capabilities
- **AMQP Broker**: Supports AMQP 0.9.1/1.0 for enterprise messaging
- **Protocol Isolation**: Each protocol uses dedicated ports for optimal performance

### 2. Broker Server Layer
- **Stateless Design**: All broker nodes are stateless for horizontal scaling
- **Load Balancing**: Automatic client distribution across broker nodes
- **Protocol Translation**: Converts different protocols to internal message format
- **Connection Management**: Handles millions of concurrent connections

### 3. Meta Service Layer (meta service)
- **Raft Consensus**: High-availability metadata management using Raft algorithm
- **Cluster Coordination**: Node discovery, health checks, and failover
- **Topic Management**: Topic routing, partitioning, and load balancing
- **Controller Threads**: Protocol-specific controllers for cluster scheduling

### 4. Storage Adapter Layer
- **Abstraction Layer**: Pluggable storage interface
- **Protocol Translation**: Converts internal messages to storage format
- **Batch Processing**: Optimized for high-throughput scenarios
- **Consistency Guarantees**: WAL (Write-Ahead Log) support

### 5. Storage Layer
- **Memory Storage**: Microsecond latency for real-time applications
- **SSD Storage**: Millisecond latency for high-frequency access
- **Object Storage**: Second-level latency for cost-effective long-term storage
- **Distributed Storage**: Support for S3, HDFS, and other distributed backends

## Core Components

### Broker Server (`src/broker-server/`)

The main entry point that coordinates all protocol brokers and provides unified service management.

**Key Responsibilities:**
- Service coordination and lifecycle management
- gRPC service interfaces for internal communication
- Cluster service management and health monitoring
- Performance metrics collection and reporting

**Core Files:**
- `cluster_service.rs`: Cluster service management
- `grpc.rs`: gRPC service interfaces
- `common.rs`: Shared utilities and configurations

### Meta Service (`src/meta-service/`)

The metadata management center responsible for cluster coordination and metadata storage.

**Key Responsibilities:**
- Cluster metadata storage using Raft consensus
- Node management and health monitoring
- Fault detection and automatic recovery
- Topic routing and load balancing decisions

**Core Modules:**
- `core/`: Core caching and controller logic
- `raft/`: Raft consensus algorithm implementation
- `storage/`: Metadata storage engine
- `controller/`: Protocol-specific controllers

### Journal Server (`src/journal-server/`)

The persistent storage layer responsible for message durability and retrieval.

**Key Responsibilities:**
- Message persistence and indexing
- Storage engine abstraction
- Data replication and consistency
- Performance optimization for different access patterns

**Core Features:**
- WAL (Write-Ahead Log) for consistency
- Pluggable storage backends
- Efficient indexing and querying
- Data compression and optimization

### Protocol Brokers

#### MQTT Broker (`src/mqtt-broker/`)
- **Protocol Support**: MQTT 3.1.1, 3.1, and 5.0
- **Transport**: TCP (1883), SSL/TLS (1885), WebSocket (8083), WebSocket SSL (8085)
- **Features**: QoS levels, retained messages, will messages, shared subscriptions
- **Performance**: Million-level concurrent connections

#### Kafka Broker (`src/kafka-broker/`)
- **Protocol Support**: Kafka 2.8+ compatible
- **Transport**: TCP (9092)
- **Features**: Topic partitioning, consumer groups, offset management
- **Use Cases**: Big data streaming, AI training pipelines

#### AMQP Broker (`src/amqp-broker/`)
- **Protocol Support**: AMQP 0.9.1 and 1.0
- **Transport**: TCP (5672)
- **Features**: Exchanges, queues, routing, acknowledgments
- **Use Cases**: Enterprise integration, microservices communication

### Storage Adapter (`src/storage-adapter/`)

Provides a unified interface for different storage backends.

**Supported Storage Types:**
- **Memory**: In-memory storage for ultra-low latency
- **Local Files**: File-based storage for development and small deployments
- **RocksDB**: Embedded key-value store for high performance
- **S3**: Object storage for cloud deployments
- **HDFS**: Distributed file system for big data scenarios

### Common Components

#### Network Server (`src/common/network-server/`)
- **Connection Management**: Efficient connection pooling and management
- **Protocol Parsing**: High-performance protocol parsing
- **Security**: TLS/SSL support and authentication
- **Monitoring**: Connection metrics and health checks

#### Metrics (`src/common/metrics/`)
- **Prometheus Integration**: Comprehensive metrics collection
- **Custom Metrics**: Application-specific performance indicators
- **Real-time Monitoring**: Live performance dashboards
- **Alerting**: Configurable alerting rules

#### Security (`src/common/security/`)
- **Authentication**: Multiple authentication mechanisms
- **Authorization**: Role-based access control
- **Encryption**: End-to-end encryption support
- **Audit Logging**: Security event logging

## Deployment Architecture

### All-in-One Deployment
- **Use Case**: Development, testing, and small-scale production
- **Components**: All services in a single process
- **Benefits**: Simple deployment and management
- **Limitations**: Single point of failure, limited scalability

### Microservices Deployment
- **Use Case**: Production environments requiring high availability
- **Components**: Independent services with dedicated resources
- **Benefits**: High availability, independent scaling, fault isolation
- **Complexity**: More complex deployment and management

### Cloud-Native Deployment
- **Use Case**: Kubernetes environments
- **Components**: Containerized services with K8s Operator
- **Benefits**: Auto-scaling, service discovery, rolling updates
- **Features**: Helm charts, CRDs, monitoring integration

## Data Flow Architecture

### Message Publishing Flow
1. **Client Connection**: Client connects to appropriate broker (MQTT/Kafka/AMQP)
2. **Protocol Parsing**: Broker parses protocol-specific message format
3. **Message Validation**: Validate message format and permissions
4. **Topic Resolution**: Meta service resolves topic routing information
5. **Storage Write**: Journal server persists message to storage
6. **Acknowledgment**: Send acknowledgment back to client

### Message Consumption Flow
1. **Subscription**: Client subscribes to topics/partitions
2. **Load Balancing**: Meta service distributes subscriptions across brokers
3. **Message Retrieval**: Journal server retrieves messages from storage
4. **Protocol Conversion**: Convert internal format to client protocol
5. **Message Delivery**: Deliver messages to subscribed clients
6. **Acknowledgment**: Handle client acknowledgments and offset management

## Performance Characteristics

### Latency
- **Memory Storage**: Microsecond-level latency
- **SSD Storage**: Millisecond-level latency
- **Object Storage**: Second-level latency

### Throughput
- **Concurrent Connections**: Million-level support
- **Message Rate**: High-throughput message processing
- **Bandwidth**: Optimized for high-bandwidth scenarios

### Scalability
- **Horizontal Scaling**: Add nodes to increase capacity
- **Auto-scaling**: Kubernetes-based auto-scaling
- **Load Distribution**: Automatic load balancing

## Security Architecture

### Authentication
- **Username/Password**: Traditional authentication
- **Certificate-based**: X.509 certificate authentication
- **OAuth 2.0**: Modern authentication protocols
- **LDAP/AD**: Enterprise directory integration

### Authorization
- **Topic-level**: Fine-grained topic access control
- **Operation-level**: Read/write permission control
- **Resource-based**: Resource-specific permissions
- **Time-based**: Temporary access grants

### Encryption
- **Transport Encryption**: TLS/SSL for data in transit
- **Storage Encryption**: Encryption at rest
- **End-to-End**: Application-level encryption
- **Key Management**: Secure key rotation and management

## Monitoring and Observability

### Metrics
- **System Metrics**: CPU, memory, disk, network usage
- **Application Metrics**: Message rates, latency, error rates
- **Business Metrics**: Topic usage, client connections
- **Custom Metrics**: Application-specific indicators

### Logging
- **Structured Logging**: JSON-formatted logs
- **Log Levels**: Configurable log levels
- **Log Aggregation**: Centralized log collection
- **Log Analysis**: Real-time log analysis

### Tracing
- **Distributed Tracing**: End-to-end request tracing
- **Performance Analysis**: Latency breakdown analysis
- **Dependency Mapping**: Service dependency visualization
- **Error Tracking**: Error propagation tracking

## Development and Operations

### Development
- **Rust Ecosystem**: Leveraging Rust's performance and safety
- **Async Runtime**: Tokio-based async programming
- **Testing**: Comprehensive unit and integration tests
- **Documentation**: Extensive API and user documentation

### Operations
- **Docker Support**: Containerized deployment
- **Kubernetes**: Native K8s support with operators
- **Monitoring**: Prometheus and Grafana integration
- **Backup/Restore**: Data backup and recovery procedures

## Future Roadmap

### Short-term (2025)
- **MQTT Production Ready**: Complete MQTT 5.0 support
- **Performance Optimization**: Further latency reduction
- **Cloud Integration**: Enhanced cloud provider integration

### Medium-term (2026)
- **Kafka Enhancement**: Full Kafka compatibility
- **AI Pipeline Support**: Optimized AI training data pipelines
- **Advanced Features**: Schema registry, message transformation

### Long-term
- **Apache Foundation**: Goal to become Apache top-level project
- **Ecosystem Growth**: Rich ecosystem of tools and integrations
- **Global Adoption**: Worldwide enterprise adoption

## Conclusion

RobustMQ represents a new generation of message queue systems, designed from the ground up for modern cloud-native and AI-native applications. Its layered architecture, multi-protocol support, and pluggable storage make it suitable for a wide range of use cases, from IoT devices to large-scale AI training pipelines.

The separation of compute and storage, combined with Rust's performance characteristics, enables RobustMQ to deliver microsecond-level latency while maintaining high availability and scalability. This makes it an ideal choice for applications requiring real-time performance and reliable message delivery.

---

*This document provides a comprehensive overview of RobustMQ's architecture. For more detailed information about specific components, please refer to the individual component documentation.*
