<div align="center">
  <img src="../../images/robustmq-logo.png" width="200"/>
</div>

## 什么是 RobustMQ

**定位：下一代 AI、IoT、大数据统一的通信基础设施。**

**愿景：让数据在 AI 训练集群、百万 Agent、IoT 设备和云端之间，以最优路径、最低延迟、最小成本自由流动。**

RobustMQ 是用 Rust 构建的下一代统一消息平台，通过 Kafka 和 MQTT 双协议兼容、对象存储智能缓存、百万级轻量 Topic、共享订阅和多模式存储引擎，让 AI 训练集群、百万 Agent、IoT 设备和云端的数据以最优路径、最低延迟、最小成本自由流动。

完全兼容 Kafka 和 MQTT 3.1/3.1.1/5.0 协议，现有应用使用标准 Kafka SDK 即可无缝接入，零迁移成本即可获得 RobustMQ 的全部能力。

---

## 核心场景

### AI 场景：智能数据调度与缓存层

对象存储（S3/MinIO）直连加三层智能缓存，训练数据无需预导入，消除 I/O 瓶颈，极大提升 GPU 利用率；单集群支持百万级轻量 Topic，为大规模 AI Agent 网络提供独立通信通道，细粒度隔离与监控；共享订阅模式让 GPU 训练节点随时弹性伸缩，不受 Partition 数量限制。

### IoT 场景：MQTT in / Kafka out 统一链路

通过统一存储层实现协议互通——IoT 设备通过 MQTT 接入的数据，AI 和大数据系统可直接用 Kafka 协议消费，一套系统替代 MQTT + Kafka 双 Broker 架构。极小内存占用支持边缘部署，断网缓存加自动同步覆盖从边缘网关到云端集群的全链路。

### 大数据场景：兼容并增强 Kafka

兼容并增强 Kafka 协议，智能存储引擎提供内存/混合/持久/分层四种模式，Topic 级独立配置，热数据极速访问，冷数据自动分层到 S3，性能与成本兼顾。

---

## 核心特性

- 🚀 **极致性能**：基于 Rust 构建，微秒级延迟，无 GC 停顿，单节点百万级 QPS，极小内存占用支持边缘部署
- 🔌 **双协议统一**：完全兼容 MQTT 3.1/3.1.1/5.0 和 Kafka 协议，统一存储层实现 MQTT in / Kafka out，一套系统替代双架构
- 🎯 **AI 训练加速**：对象存储（S3/MinIO）直连，三层智能缓存（内存/SSD/S3），训练数据无需预导入，消除 I/O 瓶颈，极大提升 GPU 利用率
- 🤖 **Agent 通信**：单集群支持百万级轻量 Topic，每个 Agent 独立通道，细粒度隔离与监控，成本按 Agent 可追溯
- 🔄 **弹性消费**：共享订阅模式突破 Kafka "并发度 = Partition 数量"的限制，GPU 训练节点随时扩缩容，无需修改 Topic 配置
- 💾 **智能存储引擎**：内存/混合/持久/分层四模式，Topic 级独立配置，热数据极速访问，冷数据自动分层到 S3，性能与成本兼顾
- 🌐 **边缘到云端**：极小内存占用，从边缘网关到云端集群统一部署，断网缓存 + 自动同步覆盖 IoT 全链路
- 🛠️ **极简部署**：单二进制文件，零外部依赖，内置 Raft 共识，开箱即用，运维成本极低

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

## 阶段计划

**阶段一：基础架构（已完成）**

构建可扩展的技术架构，代码实现追求扎实、精简、易于抽象。为多协议适配、可插拔存储、可扩展性和弹性能力打下坚实基础。

**阶段二：MQTT Broker（初步完成）**

交付稳定、高性能的 MQTT Broker，支持 MQTT 3.x/5.0 协议，针对边缘部署优化，安装包控制在 20MB 以内。协议能力初步完成，将在后续版本中持续演进。

**阶段三：Kafka 协议与 AI 能力（启动中）**

在 MQTT Broker 初步完成的基础上，启动 Kafka 协议适配与 AI 能力建设。优先验证 AI 训练数据缓存加速和百万级轻量 Topic 的可行性，以 AI 场景驱动 Kafka 协议实现；在此基础上逐步补全标准 Kafka 协议能力。

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

> **注意**：当前版本（0.3.0）仍处于早期阶段，暂不建议生产环境使用。预计 0.4.0（2025 年 5 月左右）达到生产可用标准。

---

## 快速开始

```bash
# 一键安装
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# 启动服务
robust-server start

# 验证 MQTT 连接
mqttx sub -h localhost -p 1883 -t "test/topic"
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"
```

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
