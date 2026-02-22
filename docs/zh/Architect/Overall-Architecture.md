# RobustMQ 整体架构概述

RobustMQ 是用 Rust 构建的下一代统一消息平台，专为 AI、IoT、大数据场景设计。架构目标是：**计算、存储、调度三层完全分离，各组件无状态、可独立扩展，支持多协议接入和插件化存储，同时保持零外部依赖。**

---

## 整体架构

![architecture overview](../../images/robustmq-architecture-overview.jpg)

整体由三个核心组件构成：

| 组件 | 职责 |
|------|------|
| **Meta Service** | 集群元数据管理、节点协调、集群控制器 |
| **Broker** | 多协议解析与消息逻辑处理（MQTT、Kafka 等） |
| **Storage Engine** | 内置存储引擎，提供 Memory / RocksDB / File Segment 三种存储后端 |

三个组件通过单一二进制文件交付，由 `roles` 配置决定启用哪些角色：

```toml
roles = ["meta", "broker", "engine"]
```

可按需组合部署，单机全启或分节点独立部署均支持。

---

## 集群部署视图

![architecture](../../images/robustmq-architecture.jpg)

典型三节点集群中，每个 Node 包含以下四大模块：

### 1. 通用 Server 层

为所有上层模块提供公共基础服务：

| 组件 | 说明 |
|------|------|
| Inner gRPC Server | 节点间内部通信，供 Meta Service 和 Broker 使用 |
| Admin HTTP Server | 对外暴露 REST 运维接口 |
| Prometheus Server | 指标采集接口，供监控系统拉取 |

### 2. Meta Service

集群元数据管理与控制器，技术核心为 **gRPC + Multi Raft + RocksDB**。

**职责：**
- **集群协调**：节点发现、上下线管理、节点间数据分发
- **元数据存储**：Broker 信息、Topic 配置、Connector 配置等集群元数据
- **KV 型业务数据**：MQTT Session、保留消息、遗嘱消息等运行时数据
- **控制器**：故障切换、Connector 任务调度等集群级调度

**Multi Raft 设计：** Meta Service 运行多个独立的 Raft 状态机（Metadata / Offset / Data），多 Leader 并行处理，避免单 Raft 写入瓶颈。数据通过 RocksDB 持久化，严格控制内存使用，可支撑百万级 Topic、亿级 Session 的大规模场景。

> Meta Service 的定位类似于 ZooKeeper 之于 Kafka、NameServer 之于 RocketMQ，但同时承担了元数据协调、KV 存储和集群控制器三个角色，是 RobustMQ 零外部依赖架构的核心。

### 3. Broker

无状态协议处理层，采用分层架构设计：

```
┌─────────────────────────────────────┐
│           网络层                     │  TCP / TLS / WebSocket / WSS / QUIC
├─────────────────────────────────────┤
│           协议层                     │  MQTT / Kafka / AMQP / RocketMQ
├─────────────────────────────────────┤
│         协议逻辑层                   │  mqtt-broker / kafka-broker / ...
├─────────────────────────────────────┤
│        消息通用逻辑层                 │  收发 / 过期 / 延迟 / 安全 / Schema / 监控
├─────────────────────────────────────┤
│         Storage Adapter              │  Shard 抽象 + 存储引擎路由
└─────────────────────────────────────┘
```

- **网络层**：支持 TCP、TLS、WebSocket、WebSockets、QUIC 五种接入方式
- **协议层**：当前已支持 MQTT（完整），Kafka 开发中，AMQP/RocketMQ 长期规划
- **协议逻辑层**：每种协议有独立的业务实现模块
- **消息通用逻辑层**：消息收发、过期、延迟发布、安全认证、Schema 校验、监控指标等跨协议公共能力
- **Storage Adapter**：将 MQTT Topic、Kafka Partition、AMQP Queue 统一抽象为 **Shard**，根据配置路由到对应存储后端

### 4. Storage Engine

内置存储引擎，提供三种存储类型，配置粒度为 **Topic 级别**：

| 存储类型 | 配置值 | 延迟 | 特点 | 适用场景 |
|---------|--------|------|------|---------|
| Memory | `EngineMemory` | 微秒级 | 纯内存，重启数据丢失 | 实时数据、临时消息 |
| RocksDB | `EngineRocksDB` | 毫秒级 | 本地 KV 持久化 | IoT 设备消息、离线消息 |
| File Segment | `EngineSegment` | 毫秒级 | 分段式日志，高吞吐 | 大数据流处理、高吞吐写入 |

此外，Storage Adapter 同样支持对接外部存储（MinIO、S3、MySQL 等），由配置决定路由目标。

---

## 启动流程

节点启动时各模块按以下顺序初始化：

```
通用 Server → Meta Service → Storage Engine → Broker
```

1. **通用 Server**：最先启动，建立节点间通信通道
2. **Meta Service**：通过 Raft 协议在多节点间完成选举，Leader 节点同时启动控制器线程
3. **Storage Engine**：依赖 Meta Service 完成集群组建和元数据注册
4. **Broker**：最后启动，依赖 Meta Service 做集群协调，依赖 Storage Engine 做数据写入

---

## 混合存储

RobustMQ 支持在同一集群内混合使用多种存储，粒度为 Topic 级别：

| 存储选择 | 适用场景 |
|---------|---------|
| Memory | 高吞吐、极低延迟，可容忍少量数据丢失 |
| RocksDB | 低延迟持久化，不允许数据丢失 |
| File Segment | 高吞吐持久化，大数据 / Kafka 场景 |
| MinIO / S3 | 大数据量，成本敏感，对延迟要求不高 |

> 大多数场景只需配置单一存储类型。混合存储主要用于 AI 训练等需要分层缓存的高级场景。

---

## 设计原则

| 原则 | 实现方式 |
|------|---------|
| **零外部依赖** | Meta Service 内置替代 ZooKeeper/etcd；Storage Engine 内置替代外部存储 |
| **存算分离** | Broker 无状态，Storage Engine 独立扩展 |
| **多协议扩展** | 协议层与存储层解耦，通用逻辑复用，新增协议只需实现协议逻辑层 |
| **插件化存储** | Storage Adapter 统一 Shard 抽象，存储后端可按需切换 |
| **Topic 级配置** | 存储类型、QoS、过期策略均可按 Topic 独立配置 |
