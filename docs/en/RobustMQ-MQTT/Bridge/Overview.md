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

Based on [EMQX Data Integration Features](https://docs.emqx.com/zh/emqx/latest/getting-started/feature-comparison.html#%E6%95%B0%E6%8D%AE%E9%9B%86%E6%88%90), the following is a comparison of data integration support between RobustMQ and EMQX:

| Data Integration Type | EMQX Support | RobustMQ Support | Notes |
|----------------------|--------------|------------------|-------|
| **Webhook** | ✅ | ❌ | RobustMQ does not support HTTP Webhook |
| **Apache Kafka** | ✅ | ✅ | RobustMQ supports Kafka connector |
| **Apache Pulsar** | ✅ | ❌ | RobustMQ does not support Pulsar |
| **Apache IoTDB** | ✅ | ❌ | RobustMQ does not support IoTDB |
| **Apache Doris** | ✅ | ❌ | RobustMQ does not support Doris |
| **AWS Kinesis** | ✅ | ❌ | RobustMQ does not support AWS Kinesis |
| **AWS S3** | ✅ | ❌ | RobustMQ does not support AWS S3 |
| **Azure Blob Storage** | ✅ | ❌ | RobustMQ does not support Azure Blob |
| **Azure Event Hubs** | ✅ | ❌ | RobustMQ does not support Azure Event Hubs |
| **Cassandra** | ✅ | ❌ | RobustMQ does not support Cassandra |
| **ClickHouse** | ✅ | ❌ | RobustMQ does not support ClickHouse |
| **Confluent** | ✅ | ❌ | RobustMQ does not support Confluent |
| **Couchbase** | ✅ | ❌ | RobustMQ does not support Couchbase |
| **DynamoDB** | ✅ | ❌ | RobustMQ does not support DynamoDB |
| **Elasticsearch** | ✅ | ❌ | RobustMQ does not support Elasticsearch |
| **GCP PubSub** | ✅ | ❌ | RobustMQ does not support GCP PubSub |
| **GreptimeDB** | ✅ | ✅ | RobustMQ supports GreptimeDB connector |
| **HStreamDB** | ✅ | ❌ | RobustMQ does not support HStreamDB |
| **HTTP Server** | ✅ | ❌ | RobustMQ does not support HTTP Server |
| **InfluxDB** | ✅ | ❌ | RobustMQ does not support InfluxDB |
| **Lindorm** | ✅ | ❌ | RobustMQ does not support Lindorm |
| **Microsoft SQL Server** | ✅ | ❌ | RobustMQ does not support SQL Server |
| **MongoDB** | ✅ | ❌ | RobustMQ does not support MongoDB |
| **MQTT** | ✅ | ❌ | RobustMQ does not support MQTT bridging |
| **MySQL** | ✅ | ❌ | RobustMQ does not support MySQL |
| **OpenTSDB** | ✅ | ❌ | RobustMQ does not support OpenTSDB |
| **Oracle Database** | ✅ | ❌ | RobustMQ does not support Oracle |
| **PostgreSQL** | ✅ | ✅ | RobustMQ supports PostgreSQL connector |
| **RabbitMQ** | ✅ | ❌ | RobustMQ does not support RabbitMQ |
| **Redis** | ✅ | ❌ | RobustMQ does not support Redis |
| **RocketMQ** | ✅ | ❌ | RobustMQ does not support RocketMQ |
| **Snowflake** | ✅ | ❌ | RobustMQ does not support Snowflake |
| **TDengine** | ✅ | ❌ | RobustMQ does not support TDengine |
| **TimescaleDB** | ✅ | ❌ | RobustMQ does not support TimescaleDB |
| **Local File** | ✅ | ✅ | RobustMQ supports local file connector |

### Support Summary

- **EMQX Support**: 30+ data integration types
- **RobustMQ Support**: 4 data integration types
  - ✅ Apache Kafka
  - ✅ GreptimeDB  
  - ✅ PostgreSQL
  - ✅ Local File

RobustMQ currently focuses on core data integration scenarios, supporting the most commonly used message queues (Kafka), time-series databases (GreptimeDB), relational databases (PostgreSQL), and local file storage. Future versions will gradually expand more data integration types.

## Summary

RobustMQ connectors adopt a plugin-based architecture design, providing efficient data integration capabilities for MQTT messages. Currently supporting four core connector types: Kafka, GreptimeDB, PostgreSQL, and local file, covering the main scenarios of message queues, time-series databases, relational databases, and file storage.

Compared to EMQX's 30+ data integration types, RobustMQ focuses on core scenarios, achieving high-performance and high-reliability message bridging through Rust's memory safety and zero-cost abstraction features. This streamlined and efficient design philosophy provides a solid foundation for building reliable IoT data pipelines.
