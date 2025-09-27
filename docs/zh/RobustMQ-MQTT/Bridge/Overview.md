# RobustMQ 连接器概述

## 什么是连接器

RobustMQ 连接器是数据集成系统的核心组件，用于将 MQTT 消息桥接到外部数据系统。连接器作为消息队列与外部系统之间的桥梁，实现了数据的无缝传输和集成。

## 核心概念

### 连接器架构

RobustMQ 连接器采用插件化架构设计，主要包含以下组件：

- **连接器管理器（ConnectorManager）**：管理所有连接器的生命周期
- **桥接插件（BridgePlugin）**：实现具体的数据桥接逻辑
- **连接器配置（MQTTConnector）**：定义连接器的配置信息
- **心跳监控（Heartbeat）**：监控连接器运行状态

## 数据集成支持对比

基于 [EMQX 数据集成功能](https://docs.emqx.com/zh/emqx/latest/getting-started/feature-comparison.html#%E6%95%B0%E6%8D%AE%E9%9B%86%E6%88%90)，以下是 RobustMQ 与 EMQX 在数据集成方面的支持对比：

| 数据集成类型 | EMQX 支持 | RobustMQ 支持 | 备注 |
|-------------|-----------|---------------|------|
| **Webhook** | ✅ | ❌ | RobustMQ 暂不支持 HTTP Webhook |
| **Apache Kafka** | ✅ | ✅ | RobustMQ 支持 Kafka 连接器 |
| **Apache Pulsar** | ✅ | ✅ | RobustMQ 支持 Pulsar 连接器 |
| **Apache IoTDB** | ✅ | ❌ | RobustMQ 暂不支持 IoTDB |
| **Apache Doris** | ✅ | ❌ | RobustMQ 暂不支持 Doris |
| **AWS Kinesis** | ✅ | ❌ | RobustMQ 暂不支持 AWS Kinesis |
| **AWS S3** | ✅ | ❌ | RobustMQ 暂不支持 AWS S3 |
| **Azure Blob Storage** | ✅ | ❌ | RobustMQ 暂不支持 Azure Blob |
| **Azure Event Hubs** | ✅ | ❌ | RobustMQ 暂不支持 Azure Event Hubs |
| **Cassandra** | ✅ | ❌ | RobustMQ 暂不支持 Cassandra |
| **ClickHouse** | ✅ | ❌ | RobustMQ 暂不支持 ClickHouse |
| **Confluent** | ✅ | ❌ | RobustMQ 暂不支持 Confluent |
| **Couchbase** | ✅ | ❌ | RobustMQ 暂不支持 Couchbase |
| **DynamoDB** | ✅ | ❌ | RobustMQ 暂不支持 DynamoDB |
| **Elasticsearch** | ✅ | ❌ | RobustMQ 暂不支持 Elasticsearch |
| **GCP PubSub** | ✅ | ❌ | RobustMQ 暂不支持 GCP PubSub |
| **GreptimeDB** | ✅ | ✅ | RobustMQ 支持 GreptimeDB 连接器 |
| **HStreamDB** | ✅ | ❌ | RobustMQ 暂不支持 HStreamDB |
| **HTTP Server** | ✅ | ❌ | RobustMQ 暂不支持 HTTP Server |
| **InfluxDB** | ✅ | ❌ | RobustMQ 暂不支持 InfluxDB |
| **Lindorm** | ✅ | ❌ | RobustMQ 暂不支持 Lindorm |
| **Microsoft SQL Server** | ✅ | ❌ | RobustMQ 暂不支持 SQL Server |
| **MongoDB** | ✅ | ❌ | RobustMQ 暂不支持 MongoDB |
| **MQTT** | ✅ | ❌ | RobustMQ 暂不支持 MQTT 桥接 |
| **MySQL** | ✅ | ❌ | RobustMQ 暂不支持 MySQL |
| **OpenTSDB** | ✅ | ❌ | RobustMQ 暂不支持 OpenTSDB |
| **Oracle Database** | ✅ | ❌ | RobustMQ 暂不支持 Oracle |
| **PostgreSQL** | ✅ | ❌ | RobustMQ 暂不支持 PostgreSQL |
| **RabbitMQ** | ✅ | ❌ | RobustMQ 暂不支持 RabbitMQ |
| **Redis** | ✅ | ❌ | RobustMQ 暂不支持 Redis |
| **RocketMQ** | ✅ | ❌ | RobustMQ 暂不支持 RocketMQ |
| **Snowflake** | ✅ | ❌ | RobustMQ 暂不支持 Snowflake |
| **TDengine** | ✅ | ❌ | RobustMQ 暂不支持 TDengine |
| **TimescaleDB** | ✅ | ❌ | RobustMQ 暂不支持 TimescaleDB |
| **本地文件** | ✅ | ✅ | RobustMQ 支持本地文件连接器 |

### 支持情况总结

- **EMQX 支持**：30+ 种数据集成类型
- **RobustMQ 支持**：4 种数据集成类型
  - ✅ Apache Kafka
  - ✅ Apache Pulsar
  - ✅ GreptimeDB  
  - ✅ 本地文件

RobustMQ 目前专注于核心的数据集成场景，支持最常用的消息队列（Kafka、Pulsar）、时序数据库（GreptimeDB）和本地文件存储。未来版本将逐步扩展更多数据集成类型。


## 总结

RobustMQ 连接器采用插件化架构设计，为 MQTT 消息提供高效的数据集成能力。目前支持 Kafka、Pulsar、GreptimeDB 和本地文件四种核心连接器类型，覆盖了消息队列、时序数据库和文件存储的主要场景。

相比 EMQX 的 30+ 种数据集成类型，RobustMQ 专注于核心场景，通过 Rust 语言的内存安全和零成本抽象特性，实现了高性能、高可靠性的消息桥接。这种精简而高效的设计理念，为构建可靠的 IoT 数据管道提供了坚实的基础。
