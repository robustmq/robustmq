<div align="center">
  <img src="../../images/robustmq-logo.png" width="200"/>
</div>

## 什么是 RobustMQ

RobustMQ 是用 Rust 构建的、专门为 AI、IoT、大数据场景设计的下一代统一通信平台。

**愿景：下一代 AI、IoT、大数据统一的通信基础设施。**

通过 MQTT 和 Kafka 双协议、百万级 Topic、对象存储（S3/MinIO 等）数据源、多模式存储引擎（内存/混合/持久/分层）、智能数据缓存，为 AI 训练、Agent 通信、IoT 设备（边缘/云端）、大数据处理提供高性能、低成本、稳定的通信基础设施。

---

## 为什么需要 RobustMQ

现有的通信中间件都是为单一场景设计的：

- **Kafka** 为大数据日志流设计，Topic 数量受文件系统限制（万级上限），无法支撑百万级 Agent 通信和 AI 训练数据缓存需求
- **MQTT Broker** 为 IoT 设备连接设计，不具备大吞吐的数据流处理能力
- **两者都没有考虑** AI 时代的新场景：Agent 通信隔离、GPU 训练数据加速、边缘到云端统一数据链路

RobustMQ 从一开始就为这些场景设计，而不是在现有系统上打补丁。

---

## 架构

![architecture](../../images/robustmq-architecture.jpg)

RobustMQ 由三个组件构成，架构固定，边界清晰：

### Meta Service
负责集群元数据管理和协调。所有节点状态、Topic 配置、客户端会话信息都存储在 Meta Service 中，通过自研的 **Multi Raft** 机制保证一致性和高可用。支持多个独立的 Raft Group，不同类型元数据分组管理，避免单一 Raft 的性能瓶颈。

### Broker
负责协议处理和请求路由。Broker 是**无状态**的，只处理客户端连接、协议解析、消息路由，不持有任何持久化数据。存算分离的设计让 Broker 可以随时水平扩展，加节点不需要数据迁移。

### Storage Engine
负责数据持久化。支持三种存储引擎，可按 Topic 粒度独立配置：

| 引擎 | 延迟 | 适用场景 |
|------|------|---------|
| Memory | 微秒级 | 梯度同步、实时指标、临时通知 |
| RocksDB | 毫秒级 | 百万级 Topic、IoT 设备消息、离线存储 |
| File Segment | 毫秒级 | 大吞吐日志流、Kafka 场景 |

存储引擎是插件化接口，未来可扩展更多后端（HDFS、对象存储等）。

---

## 核心特性

### 🦀 Rust 高性能内核
零开销抽象、内存安全、无 GC 停顿。基于 Tokio 异步运行时，延迟稳定，P99 不受 GC 抖动影响。

### 🔌 多协议支持
- **MQTT 3.x / 5.0**：完整协议支持，适合 IoT 设备、边缘计算
- **Kafka 协议**（开发中）：现有 Kafka 客户端和工具链零改动接入

同一份数据可以通过 MQTT 写入、Kafka 消费，两个协议共享同一存储层，零拷贝协议转换。

RobustMQ 的架构设计天然支持多协议扩展。**AMQP、RocketMQ 等协议**在架构上已有规划，长期会逐步支持，但短期内专注于把 MQTT 和 Kafka 两个核心协议做好、做稳。

### 🤖 百万级轻量 Topic
基于 RocksDB 的统一 KV 存储，所有 Topic 共享同一存储实例，通过 Key 前缀区分。创建 Topic 只是增加一条元数据记录，不创建物理文件，支持百万级 Topic。每个 AI Agent 可以拥有独立的通信通道，完全隔离。

### 🔄 共享订阅
借鉴 MQTT 协议的共享订阅机制，打破 Kafka "并发度等于 Partition 数量"的限制。多个消费者可以并发消费同一个 Partition，消费并发度与存储分片完全解耦，支持 AI 训练弹性扩缩容。

### 🧠 对象存储数据源与智能缓存（开发中）
Topic 直接指向 S3/MinIO 中的文件路径，RobustMQ 作为智能缓存层，通过三层缓存（内存/SSD/对象存储）和预测式预加载，将训练数据访问延迟从 200ms 降低到 2ms 以下，大幅提升 GPU 利用率。

---

## 适用场景

| 场景 | 说明 |
|------|------|
| **IoT 设备连接** | MQTT 协议，支持百万级设备并发连接，QoS 0/1/2，离线消息，遗嘱消息 |
| **AI Agent 通信** | 百万级 Topic，每个 Agent 独立通道，精确监控和权限隔离 |
| **AI 训练数据加速** | 对象存储直连，智能缓存层降低 GPU 等待，提升训练吞吐 |
| **边缘到云端数据链路** | MQTT 边缘采集，Kafka 云端消费，双协议统一存储，无需桥接层 |
| **大数据流处理** | Kafka 协议兼容，Flink/Spark/Kafka Connect 零改动接入 |

---

## 当前状态

| 功能 | 状态 |
|------|------|
| MQTT 3.x / 5.0 核心协议 | ✅ 可用 |
| Session 持久化与恢复 | ✅ 可用 |
| 共享订阅 | ✅ 可用 |
| 认证与 ACL | ✅ 可用 |
| 规则引擎 | ✅ 基础可用 |
| Grafana + Prometheus 监控 | ✅ 可用 |
| Web 管理控制台 | ✅ 可用 |
| Kafka 协议 | 🚧 开发中 |
| AI 训练数据缓存 | 🚧 开发中 |

> **注意**：当前版本（0.3.0）仍处于早期阶段，暂不建议生产环境使用。预计 0.4.0（2025年5月左右）达到生产可用标准。

---

## 快速开始

```bash
# 安装并启动
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
broker-server start

# 验证 MQTT 连接
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"
mqttx sub -h localhost -p 1883 -t "test/topic"
```

访问 `http://localhost:8080` 打开 Web 管理控制台。

详细文档：[快速上手指南](../QuickGuide/Quick-Install.md)

---

## 项目理念

RobustMQ 是一个**非商业化的开源项目**，没有商业公司背景，没有付费版本，所有核心功能完全开源。

这是一个技术信仰驱动的项目——相信用 Rust 重新设计通信基础设施是正确的方向，相信 AI 时代需要一套真正为新场景设计的消息系统，相信优秀的基础软件应该属于整个社区。

项目的长期目标是成为 **Apache 顶级项目**，建立一个全球化的开发者社区，持续推动项目发展。

---

## 项目信息

- **语言**：Rust
- **协议**：Apache 2.0（完全开源，无商业版本）
- **GitHub**：https://github.com/robustmq/robustmq
- **官网**：https://robustmq.com
