# RobustMQ Connector Overview

## What is a Connector

RobustMQ connectors are core components of the data integration system, used to bridge MQTT messages to external data systems. Connectors serve as bridges between message queues and external systems, enabling seamless data transmission and integration.

## Core Concepts

### Connector Architecture

RobustMQ connectors adopt a plugin-based architecture design, mainly including the following components:

- **Connector Manager (ConnectorManager)**: Manages the lifecycle of all connectors
- **Bridge Plugin (BridgePlugin)**: Implements specific data bridging logic
- **Connector Configuration (MQTTConnector)**: Defines connector configuration information
- **Heartbeat Monitoring (Heartbeat)**: Monitors connector running status

## Data Integration Support Comparison

Based on [EMQX Data Integration Features](https://docs.emqx.com/zh/emqx/latest/getting-started/feature-comparison.html#%E6%95%B0%E6%8D%AE%E9%9B%86%E6%88%90), the following is a comparison of data integration support between RobustMQ and EMQX.

### Priority Levels

| Priority | Meaning |
|----------|---------|
| P0 | Core component, supported, high-frequency use in IoT scenarios |
| P1 | Important component, high demand, planned for near-term support |
| P2 | General component, moderate demand, scheduled based on community feedback |
| P3 | Low priority, limited use cases or insufficient Rust ecosystem support |

### General Components

| Data Integration Type | EMQX Support | RobustMQ Support | Priority | Notes |
|----------------------|--------------|------------------|----------|-------|
| **Webhook** | ✅ | ✅ | P0 | |
| **HTTP Server** | ✅ | ✅ | P0 | Covered by the Webhook connector |
| **Apache Kafka** | ✅ | ✅ | P0 | |
| **MQTT** | ✅ | ✅ | P0 | MQTT bridging (Sink) |
| **MySQL** | ✅ | ✅ | P0 | |
| **PostgreSQL** | ✅ | ✅ | P0 | |
| **Redis** | ✅ | ✅ | P0 | |
| **MongoDB** | ✅ | ✅ | P0 | |
| **Elasticsearch** | ✅ | ✅ | P0 | |
| **ClickHouse** | ✅ | ✅ | P0 | |
| **InfluxDB** | ✅ | ✅ | P0 | Supports v1/v2, HTTP + Line Protocol |
| **Local File** | ✅ | ✅ | P0 | |
| **Apache Pulsar** | ✅ | ✅ | P1 | |
| **RabbitMQ** | ✅ | ✅ | P1 | |
| **Cassandra** | ✅ | ✅ | P1 | Based on scylla driver, ScyllaDB compatible |
| **GreptimeDB** | ✅ | ✅ | P1 | |
| **OpenTSDB** | ✅ | ✅ | P1 | |
| **TimescaleDB** | ✅ | ✅ | P2 | PostgreSQL extension, use PostgreSQL connector directly |
| **Apache Doris** | ✅ | ✅ | P2 | MySQL protocol compatible, use MySQL connector directly |
| **AlloyDB** | ✅ | ✅ | P2 | PostgreSQL protocol compatible, use PostgreSQL connector directly |
| **CockroachDB** | ✅ | ✅ | P2 | PostgreSQL protocol compatible, use PostgreSQL connector directly |
| **TDengine** | ✅ | ❌ | P2 | Chinese time-series DB, Rust client needs evaluation |
| **Microsoft SQL Server** | ✅ | ❌ | P2 | |
| **Datalayers** | ✅ | ❌ | P2 | Chinese IoT time-series DB |
| **RocketMQ** | ✅ | ❌ | P3 | Rust client requires nightly, not feasible yet |
| **Couchbase** | ✅ | ❌ | P3 | Semi-open-source (BSL), limited Rust ecosystem |
| **Oracle Database** | ✅ | ❌ | P3 | Commercial DB, limited Rust drivers |
| **HStreamDB** | ✅ | ❌ | P3 | Small user base |
| **Apache IoTDB** | ✅ | ❌ | P3 | Rust client immature |
| **SysKeeper** | ✅ | ❌ | P3 | Industrial security gateway, niche use case |
| **Disk Log** | ✅ | ❌ | P3 | Similar capability provided by Local File connector |

### Commercial / Cloud Vendor Components

| Data Integration Type | EMQX Support | RobustMQ Support | Priority | Cloud Vendor |
|----------------------|--------------|------------------|----------|-------------|
| **AWS S3** | ✅ | ❌ | P1 | AWS |
| **AWS Kinesis** | ✅ | ❌ | P2 | AWS |
| **AWS S3 Tables** | ✅ | ❌ | P2 | AWS |
| **AWS Redshift** | ✅ | ✅ | P2 | AWS (PostgreSQL protocol compatible, use PostgreSQL connector directly) |
| **AWS Timestream** | ✅ | ❌ | P2 | AWS |
| **DynamoDB** | ✅ | ❌ | P2 | AWS |
| **GCP PubSub** | ✅ | ❌ | P2 | Google Cloud |
| **BigQuery** | ✅ | ❌ | P2 | Google Cloud |
| **Azure Blob Storage** | ✅ | ❌ | P2 | Microsoft Azure |
| **Azure Event Hubs** | ✅ | ❌ | P2 | Microsoft Azure |
| **Confluent** | ✅ | ✅ | P2 | Confluent (Kafka protocol compatible, use Kafka connector directly) |
| **Snowflake** | ✅ | ❌ | P3 | Snowflake |
| **Lindorm** | ✅ | ❌ | P3 | Alibaba Cloud |
| **Tablestore** | ✅ | ❌ | P3 | Alibaba Cloud |

RobustMQ prioritizes general-purpose open-source components, covering HTTP push, message queues, time-series databases, relational databases, NoSQL databases, search engines, and file storage. Commercial / cloud vendor components will be expanded based on community demand.
