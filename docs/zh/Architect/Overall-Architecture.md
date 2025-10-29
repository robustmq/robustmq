# RobustMQ 整体架构概述
## 架构概述

RobustMQ 整体架构如下图所示：

![image](../../images/robustmq-architecture-overview.jpg)

如上图所示，RobustMQ 整体由 Meta Service、Broker Server、Storage Adapter、Journal Server、数据存储层五个核心模块组成：

**1. Meta Service（元数据与调度层）**

负责集群的元数据存储与调度管理，主要职责包括：

- Broker 和 Journal 集群相关元数据（Topic、Group、Queue、Partition 等）的存储与分发
- Broker 和 Journal 集群的控制与调度，包括集群节点的上下线管理、配置存储与分发等

**2. Broker Server（消息队列逻辑处理层）**

负责不同消息队列协议的解析与逻辑处理，主要职责包括：

- 解析 MQTT、Kafka、AMQP、RocketMQ 等协议
- 整合与抽象不同协议的通用逻辑，处理各协议的特定业务逻辑，实现对多种消息队列协议的兼容适配

**3. Storage Adapter（存储适配层）**

将多种 MQ 协议中的 Topic/Queue/Partition 统一抽象为 Shard 概念，同时适配不同的底层存储引擎（如本地文件存储 Journal Server、远程 HDFS、对象存储、自研存储组件等），实现存储层的插件化与可插拔特性。Storage Adapter 根据配置将数据路由至相应的存储引擎。

**4. Journal Server（持久化存储引擎）**

RobustMQ 内置的持久化存储引擎，采用类似 Apache BookKeeper 的本地多副本、分段式持久化存储架构。其设计目标是构建高性能、高吞吐、高可靠的持久化存储引擎。从 Storage Adapter 的视角来看，Journal Server 是其支持的存储引擎之一。该组件的实现旨在满足 RobustMQ 高内聚、无外部依赖的架构特性。

**5. 数据存储层（本地/远程存储）**

指实际的数据存储介质，可以是本地内存，也可以是远程存储服务（如 HDFS、MinIO、AWS S3 等）。该层由 Storage Adapter 负责对接。

## 详细架构

RobustMQ 详细架构如下图所示：

![image](../../images/robustmq-architecture.png)

如上图所示，这是一个由三个 RobustMQ Node 组成的集群。当选择使用内置的持久化存储引擎 Journal Server 时，无需依赖任何外部组件，可通过 `./bin/robust-server start` 命令一键启动节点。

从单节点视角来看，主要由通用 Server、Meta Service、Message Broker、存储层四大部分组成：

### 通用 Server

由 Inner gRPC Server、Admin HTTP Server、Prometheus Server 三个组件构成，为 Meta Service、Message Broker、Journal Server 提供公共服务：

- **Inner gRPC Server**：用于多个 RobustMQ Node 之间的内部通信
- **Admin HTTP Server**：提供统一的对外 HTTP 协议运维接口
- **Prometheus Server**：对外暴露指标采集接口，用于监控数据的收集

### Meta Service

节点内的元数据服务模块。当 Inner gRPC Server 启动成功后，Meta Service 读取配置中的 `meta_addrs` 参数获取所有 Meta Server Node 信息，通过 gRPC 协议与所有节点通信，基于 Raft 协议选举出 Meta Master 节点。选举完成后，Meta Service 即可对外提供服务。

### Message Broker

节点中负责消息逻辑处理的核心模块，用于适配多种消息协议并完成相应的逻辑处理。该模块采用分层架构设计：

**1. 网络层**

支持 TCP、TLS、WebSocket、WebSockets、QUIC 五种网络协议的解析与处理。

**2. 协议层**

在网络层之上，负责解析不同协议的请求包内容。长期规划支持 MQTT、Kafka、AMQP、RocketMQ 等多种协议，当前已完成 MQTT、Kafka 协议的支持。

**3. 协议逻辑层**

由于不同协议具有各自的处理逻辑，该层为每个协议提供独立的实现，如 mqtt-broker、kafka-broker、amqp-broker 等，负责处理各协议的特定业务逻辑。

**4. 消息通用逻辑层**

消息队列作为垂直领域，其核心为 Pub/Sub 模型，不同协议间存在大量可复用的通用逻辑，如消息收发、消息过期、延时消息、监控、日志、安全、Schema 等。这些通用代码被抽离为独立模块，供各协议复用。基于这种设计，随着核心基础设施的完善，新增协议支持的开发成本将显著降低。

**5. 存储适配层**

负责适配不同的存储引擎，主要完成两项工作：

- 将 MQTT Topic、AMQP Queue、Kafka Partition 等概念统一抽象为 Shard
- 对接不同的存储引擎，完成数据的持久化存储

### 存储层

消息的实际存储层，由两部分组成：

- **内置存储引擎**：分段式分布式存储引擎 Journal Server
- **第三方存储引擎**：支持对接外部分布式存储系统

## 单机启动流程

![image](../../images/04.jpg)

在单机模式下，节点启动时各组件的启动顺序如下：通用 Server → Meta Service → Journal Server → Message Broker。具体流程说明：

**1. 启动通用 Server 层**

首先启动 Server 层，建立多节点间的通信能力。

**2. 启动元数据协调服务**

启动 Meta Service。在三节点集群场景下，多个 Node 间通过 Raft 协议进行选举，选出 Meta Service Master。选举完成后，集群元数据层即可对外提供服务。

**3. 启动内置存储层**

启动 Journal Server。Journal Server 采用集群架构，依赖 Meta Service 完成选举、集群构建、元数据存储等工作，因此必须等待 Meta Service 就绪后才能启动。

**4. 启动消息代理层**

启动 Message Broker。Message Broker 依托 Meta Service 完成集群构建、选举、协调等工作。若配置了内置存储，还需依赖 Journal Server 完成数据的持久化存储，因此必须最后启动。

## 混合存储架构

RobustMQ 支持混合存储架构，存储引擎的选择粒度为 Topic 级别而非集群级别。在集群启动或运行过程中，可以配置集群默认的存储引擎，也支持为不同 Topic 配置不同的存储引擎。根据业务特性可选择合适的存储方案：

**1. 本地持久化存储引擎**

适用于数据量小、对延迟敏感、不允许数据丢失的 Topic。

**2. 内存存储引擎**

适用于数据量大、对延迟敏感、极端情况下可容忍少量数据丢失的 Topic。

**3. 远程存储引擎**

适用于数据量大、对延迟不敏感、不允许数据丢失、对成本敏感的 Topic。

**4. 内置 Journal Server**

适用于数据量大、对延迟敏感、不允许数据丢失、对成本不敏感的 Topic。

根据实际业务观察，大多数业务场景不需要混合存储架构，即使存在混合需求，通常也会在部署层面进行隔离。因此在实际部署过程中，通常只需配置单一存储引擎即可满足需求。
