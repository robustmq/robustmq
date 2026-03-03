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

基于 [EMQX 数据集成功能](https://docs.emqx.com/zh/emqx/latest/getting-started/feature-comparison.html#%E6%95%B0%E6%8D%AE%E9%9B%86%E6%88%90)，以下是 RobustMQ 与 EMQX 在数据集成方面的支持对比。

### 优先级说明

| 优先级 | 含义 |
|--------|------|
| P0 | 核心组件，已支持，IoT 场景高频使用 |
| P1 | 重要组件，用户需求较多，计划近期支持 |
| P2 | 一般组件，有一定需求，按社区反馈排期 |
| P3 | 低优先级，使用场景较少或 Rust 生态支持有限 |

### 通用组件

| 数据集成类型 | EMQX 支持 | RobustMQ 支持 | 优先级 | 备注 |
|-------------|-----------|---------------|--------|------|
| **Webhook** | ✅ | ✅ | P0 | |
| **HTTP Server** | ✅ | ✅ | P0 | 由 Webhook 连接器提供同等能力 |
| **Apache Kafka** | ✅ | ✅ | P0 | |
| **MQTT** | ✅ | ✅ | P0 | MQTT 桥接（Sink） |
| **MySQL** | ✅ | ✅ | P0 | |
| **PostgreSQL** | ✅ | ✅ | P0 | |
| **Redis** | ✅ | ✅ | P0 | |
| **MongoDB** | ✅ | ✅ | P0 | |
| **Elasticsearch** | ✅ | ✅ | P0 | |
| **ClickHouse** | ✅ | ✅ | P0 | |
| **InfluxDB** | ✅ | ✅ | P0 | 支持 v1/v2，HTTP + Line Protocol |
| **本地文件** | ✅ | ✅ | P0 | |
| **Apache Pulsar** | ✅ | ✅ | P1 | |
| **RabbitMQ** | ✅ | ✅ | P1 | |
| **Cassandra** | ✅ | ✅ | P1 | 基于 scylla 驱动，兼容 ScyllaDB |
| **GreptimeDB** | ✅ | ✅ | P1 | |
| **OpenTSDB** | ✅ | ✅ | P1 | |
| **TDengine** | ✅ | ❌ | P2 | 国产时序数据库，需评估 Rust 客户端 |
| **TimescaleDB** | ✅ | ✅ | P2 | 基于 PostgreSQL 扩展，直接使用 PostgreSQL 连接器 |
| **Apache Doris** | ✅ | ✅ | P2 | 兼容 MySQL 协议，直接使用 MySQL 连接器 |
| **Microsoft SQL Server** | ✅ | ❌ | P2 | |
| **RocketMQ** | ✅ | ❌ | P3 | Rust 客户端依赖 nightly，暂无法支持 |
| **Couchbase** | ✅ | ❌ | P3 | 半开源（BSL），Rust 生态支持有限 |
| **Oracle Database** | ✅ | ❌ | P3 | 商业数据库，Rust 驱动有限 |
| **HStreamDB** | ✅ | ❌ | P3 | 用户量较少 |
| **Apache IoTDB** | ✅ | ❌ | P3 | Rust 客户端不成熟 |

### 商业/云厂商组件

| 数据集成类型 | EMQX 支持 | RobustMQ 支持 | 优先级 | 云服务厂商 |
|-------------|-----------|---------------|--------|-----------|
| **AWS S3** | ✅ | ❌ | P1 | AWS |
| **AWS Kinesis** | ✅ | ❌ | P2 | AWS |
| **DynamoDB** | ✅ | ❌ | P2 | AWS |
| **GCP PubSub** | ✅ | ❌ | P2 | Google Cloud |
| **Azure Blob Storage** | ✅ | ❌ | P2 | Microsoft Azure |
| **Azure Event Hubs** | ✅ | ❌ | P2 | Microsoft Azure |
| **Confluent** | ✅ | ❌ | P2 | Confluent |
| **Snowflake** | ✅ | ❌ | P3 | Snowflake |
| **Lindorm** | ✅ | ❌ | P3 | 阿里云 |

RobustMQ 优先支持通用开源组件，覆盖 HTTP 推送、消息队列、时序数据库、关系型数据库、NoSQL 数据库、搜索引擎和文件存储等核心场景。商业/云厂商组件将根据社区需求逐步扩展。
