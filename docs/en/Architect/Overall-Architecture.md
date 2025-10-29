# RobustMQ Overall Architecture Overview

## Architecture Overview

The overall architecture of RobustMQ is shown in the following diagram:

![image](../../images/robustmq-architecture-overview.jpg)

As shown above, RobustMQ consists of five core modules: Meta Service, Broker Server, Storage Adapter, Journal Server, and Data Storage Layer:

**1. Meta Service (Metadata and Scheduling Layer)**

Responsible for cluster metadata storage and scheduling management, with primary responsibilities including:

- Storage and distribution of Broker and Journal cluster-related metadata (Topic, Group, Queue, Partition, etc.)
- Control and scheduling of Broker and Journal clusters, including cluster node online/offline management, configuration storage and distribution, etc.

**2. Broker Server (Message Queue Logic Processing Layer)**

Responsible for parsing and logic processing of different message queue protocols, with primary responsibilities including:

- Parsing MQTT, Kafka, AMQP, RocketMQ and other protocols
- Integrating and abstracting common logic across different protocols, handling protocol-specific business logic, and implementing compatibility adaptation for multiple message queue protocols

**3. Storage Adapter (Storage Adaptation Layer)**

Unifies Topic/Queue/Partition concepts from various MQ protocols into the Shard abstraction, while adapting to different underlying storage engines (such as local file storage Journal Server, remote HDFS, object storage, self-developed storage components, etc.), achieving pluggable storage layer characteristics. Storage Adapter routes data to appropriate storage engines based on configuration.

**4. Journal Server (Persistent Storage Engine)**

RobustMQ's built-in persistent storage engine, adopting a local multi-replica, segmented persistent storage architecture similar to Apache BookKeeper. Its design goal is to build a high-performance, high-throughput, highly reliable persistent storage engine. From the Storage Adapter's perspective, Journal Server is one of the supported storage engines. This component is designed to meet RobustMQ's high-cohesion, no-external-dependency architectural characteristics.

**5. Data Storage Layer (Local/Remote Storage)**

Refers to the actual data storage medium, which can be local memory or remote storage services (such as HDFS, MinIO, AWS S3, etc.). This layer is interfaced through the Storage Adapter.

## Detailed Architecture

The detailed architecture of RobustMQ is shown in the following diagram:

![image](../../images/robustmq-architecture.png)

As shown above, this is a cluster composed of three RobustMQ Nodes. When choosing to use the built-in persistent storage engine Journal Server, no external components are required, and nodes can be started with a single command: `./bin/robust-server start`.

From a single-node perspective, it consists of four main parts: Common Server, Meta Service, Message Broker, and Storage Layer:

### Common Server

Composed of three components: Inner gRPC Server, Admin HTTP Server, and Prometheus Server, providing common services for Meta Service, Message Broker, and Journal Server:

- **Inner gRPC Server**: Used for internal communication between multiple RobustMQ Nodes
- **Admin HTTP Server**: Provides unified external HTTP protocol operations interface
- **Prometheus Server**: Exposes metrics collection interface for monitoring data collection

### Meta Service

The metadata service module within the node. After the Inner gRPC Server starts successfully, Meta Service reads the `meta_addrs` parameter from configuration to obtain all Meta Server Node information, communicates with all nodes through gRPC protocol, and elects the Meta Master node based on the Raft protocol. After election completion, Meta Service can provide services externally.

### Message Broker

The core module in the node responsible for message logic processing, used to adapt multiple message protocols and complete corresponding logic processing. This module adopts a layered architecture design:

**1. Network Layer**

Supports parsing and processing of five network protocols: TCP, TLS, WebSocket, WebSockets, and QUIC.

**2. Protocol Layer**

Above the network layer, responsible for parsing request packet content of different protocols. Long-term plans support MQTT, Kafka, AMQP, RocketMQ and other protocols, with current completion of MQTT and Kafka protocol support.

**3. Protocol Logic Layer**

Since different protocols have their own processing logic, this layer provides independent implementations for each protocol, such as mqtt-broker, kafka-broker, amqp-broker, etc., responsible for handling protocol-specific business logic.

**4. Message Common Logic Layer**

As a vertical domain, message queues are centered on the Pub/Sub model, with extensive reusable common logic across different protocols, such as message sending/receiving, message expiration, delayed messages, monitoring, logging, security, Schema, etc. These common codes are extracted as independent modules for protocol reuse. Based on this design, as core infrastructure improves, the development cost of adding new protocol support will be significantly reduced.

**5. Storage Adaptation Layer**

Responsible for adapting different storage engines, mainly completing two tasks:

- Unifying MQTT Topic, AMQP Queue, Kafka Partition and other concepts into Shard abstraction
- Interfacing with different storage engines to complete data persistent storage

### Storage Layer

The actual storage layer for messages, composed of two parts:

- **Built-in Storage Engine**: Segmented distributed storage engine Journal Server
- **Third-party Storage Engine**: Supports interfacing with external distributed storage systems

## Single Node Startup Process

![image](../../images/04.jpg)

In single-node mode, the component startup sequence when a node starts is: Common Server → Meta Service → Journal Server → Message Broker. Specific process description:

**1. Start Common Server Layer**

First start the Server layer to establish inter-node communication capabilities.

**2. Start Metadata Coordination Service**

Start Meta Service. In a three-node cluster scenario, multiple Nodes elect the Meta Service Master through the Raft protocol. After election completion, the cluster metadata layer can provide services externally.

**3. Start Built-in Storage Layer**

Start Journal Server. Journal Server adopts a cluster architecture and depends on Meta Service to complete election, cluster construction, metadata storage, etc., so it must wait for Meta Service to be ready before starting.

**4. Start Message Broker Layer**

Start Message Broker. Message Broker relies on Meta Service to complete cluster construction, election, coordination, etc. If built-in storage is configured, it also depends on Journal Server to complete data persistent storage, so it must start last.

## Hybrid Storage Architecture

RobustMQ supports hybrid storage architecture, with storage engine selection granularity at the Topic level rather than the cluster level. During cluster startup or operation, you can configure the cluster's default storage engine, and also support configuring different storage engines for different Topics. According to business characteristics, you can choose appropriate storage solutions:

**1. Local Persistent Storage Engine**

Suitable for Topics with small data volume, latency-sensitive, and no data loss tolerance.

**2. Memory Storage Engine**

Suitable for Topics with large data volume, latency-sensitive, and can tolerate small amounts of data loss in extreme cases.

**3. Remote Storage Engine**

Suitable for Topics with large data volume, not latency-sensitive, no data loss tolerance, and cost-sensitive.

**4. Built-in Journal Server**

Suitable for Topics with large data volume, latency-sensitive, no data loss tolerance, and not cost-sensitive.

According to actual business observations, most business scenarios do not require hybrid storage architecture. Even when hybrid needs exist, isolation is typically performed at the deployment level. Therefore, in actual deployment processes, configuring a single storage engine usually meets requirements.
