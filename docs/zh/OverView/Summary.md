# RobustMQ 和现有 MQ 的对比

## 背景

现有的消息队列大多诞生于特定的历史场景：Kafka 服务大数据日志流，MQTT Broker 服务 IoT 设备连接，RabbitMQ 服务企业级消息路由。它们各自在所属场景中表现出色，但随着业务边界的模糊，企业往往需要同时维护多套系统，带来运维复杂度和数据孤岛问题。

RobustMQ 的出发点是：能否用一套统一的架构，同时支撑 IoT、大数据、AI 等多种场景？本文从架构设计和核心能力两个维度，对 RobustMQ 与主流 MQ 进行客观对比。

---

## 整体对比

| 维度 | Kafka | Pulsar | NATS | RabbitMQ | RobustMQ |
|------|-------|--------|------|----------|----------|
| **主要语言** | Java/Scala | Java | Go | Erlang | Rust |
| **协议支持** | Kafka | Kafka / Pulsar | NATS | AMQP | MQTT（已支持）/ Kafka（开发中）/ AMQP、RocketMQ 等（长期规划）|
| **架构依赖** | ZooKeeper（旧）/ KRaft（新）| Broker + BookKeeper + ZooKeeper | 无外部依赖 | 无外部依赖 | Meta Service + Broker + Storage Engine |
| **存算分离** | ❌（耦合）| ✅ | ✅ | ❌ | ✅ |
| **Topic 规模** | 万级（受文件系统限制）| 百万级 | 百万级 | 万级 | 百万级（RocksDB KV）|
| **GC 停顿** | 有（JVM）| 有（JVM）| 极低（Go GC）| 无 | 无（Rust，零 GC）|
| **IoT 场景** | 不适合 | 有限支持 | 基础支持 | 有限支持 | ✅ 原生 MQTT |
| **大数据场景** | ✅ 主场 | ✅ 支持 | ❌ | ❌ | 开发中（Kafka 兼容）|
| **运维复杂度** | 中（KRaft 后降低）| 高（三种进程）| 低 | 低 | 低（单二进制，无外部依赖）|

---

## 架构差异

### Kafka

Kafka 的核心是 Partition + Replication 模型，数据持久化到本地磁盘，计算与存储紧耦合。每个 Topic 对应文件系统上的目录和文件，因此 Topic 数量受文件描述符和磁盘 IO 限制，生产环境通常控制在万级以内。

KRaft 模式移除了对 ZooKeeper 的依赖，简化了运维，但存储与计算耦合的根本问题未变。

### Pulsar

Pulsar 采用存算分离设计，Broker 无状态，数据持久化到 BookKeeper。架构上更灵活，支持百万级 Topic，并支持多租户和跨地域复制。但三层架构（Broker / BookKeeper / ZooKeeper）导致部署和运维成本较高。

### NATS

NATS 轻量、低延迟，适合微服务场景下的消息传递。JetStream 增加了持久化能力。Topic 规模大，但缺乏对 Kafka/MQTT 协议的原生支持，与现有生态集成需额外工作。

### RabbitMQ

RabbitMQ 基于 AMQP 协议，提供灵活的路由模型（exchange / queue / binding），适合企业消息场景。但吞吐相对有限，IoT 和大数据场景非其设计目标。

### RobustMQ

RobustMQ 的核心设计选择：

**三组件固定架构**：Meta Service 负责元数据（Multi Raft 保一致性），无状态 Broker 负责协议处理，Storage Engine 负责数据持久化。架构边界清晰，各组件独立扩展。

**百万级 Topic**：基于 RocksDB 的 KV 存储，所有 Topic 共享存储实例，创建 Topic 仅需写入一条元数据记录，不创建物理文件，从根本上解除了文件系统对 Topic 数量的限制。

**插件化存储引擎**：同一套 Broker 可对接内存、RocksDB、File Segment 三种存储后端，按 Topic 粒度独立配置，适配不同延迟和成本需求。

**Rust 实现**：零 GC 停顿，内存安全，延迟稳定可预期。

**多协议扩展架构**：Broker 层的协议处理是插件化设计，天然支持接入新协议。短期专注于把 MQTT 和 Kafka 做好；AMQP、RocketMQ 等协议在架构上已有规划，长期会逐步支持。

---

## 各场景的选择建议

### IoT 设备连接

**首选 RobustMQ 或专用 MQTT Broker**（如 EMQX）。Kafka 和 Pulsar 不支持 MQTT 协议，需要额外的协议网关层。RobustMQ 原生 MQTT 支持，可省去中间层。

### 大数据 / 日志流处理

**现阶段仍建议 Kafka**。Kafka 生态成熟（Kafka Connect、Kafka Streams、大量连接器），Flink/Spark 原生集成稳定。RobustMQ Kafka 协议支持仍在开发中，2026 年逐步可用后可作为候选。

### IoT + 大数据融合场景

**RobustMQ 目标场景**。IoT 设备通过 MQTT 上报数据，分析平台通过 Kafka 协议消费，两端共享同一存储，无需部署独立的协议桥。这是 RobustMQ 与现有方案相比差异化最明显的场景。

### 微服务通信

**NATS 或 RabbitMQ 更合适**。RobustMQ 当前未针对这类场景优化。

### 百万级轻量 Topic

**RobustMQ 或 Pulsar**。Kafka 不适合，文件系统开销随 Topic 数量线性增长。RobustMQ 基于 KV 的实现在资源开销上优于 Pulsar 的 BookKeeper 模型。

---

## RobustMQ 的局限性

客观来讲，当前 RobustMQ 仍处于早期阶段，存在以下明显局限：

- **Kafka 协议尚未完成**：Consumer Group、Rebalance、事务等复杂特性仍在开发中，不能用于替换生产环境的 Kafka
- **生态不成熟**：连接器数量（8+）远少于 Kafka（300+），与 Flink/Spark 的深度集成仍需时间
- **生产案例缺失**：尚无大规模生产环境验证，当前版本（0.3.0）不建议生产使用
- **社区规模小**：贡献者和用户数量与成熟项目差距明显

预计 v0.4.0（2026 年 5 月）后 MQTT 场景可用于生产，Kafka 协议在 v0.5.0（2026 年 9 月）后逐步成熟。

---

## 详细对比文档

- [RobustMQ vs Kafka](./Diff-kafka.md)
- [RobustMQ vs Pulsar](./Diff-pulsar.md)
- [RobustMQ vs NATS](./Diff-nats.md)
- [RobustMQ vs Redpanda](./Diff-redpanda.md)
- [RobustMQ vs Iggy](./Diff-iggy.md)
- [综合对比](./Diff-MQ.md)
