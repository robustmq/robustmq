# 概览

RobustMQ Kafka 是构建在 RobustMQ 统一内核之上的 **Kafka 协议兼容层**——它不是一个独立的 Kafka 发行版,而是让标准 Kafka 生态直接连接 RobustMQ 的一层协议实现。原生 Kafka 客户端(Java `kafka-clients`、`librdkafka`)与官方命令行工具(`kafka-*.sh`)都可直连,默认端口 `9092`。

## 设计取向:一份数据、多协议视图

RobustMQ Kafka 最核心的设计取向是 **"一份数据、多协议视图"**:Kafka 与 MQTT 共享同一份 topic 存储和元数据。因此存储层保存的是**协议中立的、已解码的消息记录**,而不是 Kafka 私有的压缩批次格式。这一取向决定了许多实现选择(例如为什么不采用 Kafka 的"原样存批次 + zero-copy")。

几个关键事实:

- **Controller / 协调器 = meta-service 的 Raft Leader**:不使用 ZooKeeper 或 KRaft。Kafka 语义中的 controller、消费组协调器、事务协调器都定位到当前 Raft Leader,经约 3 秒 TTL 缓存的 gRPC 查询获得。
- **存储引擎 = File Segment**:段的 append / seal / scroll、`offset → position` 索引 + mmap 读、ISR 多副本 + `leader_epoch` fencing。
- **写入即解码**:`Produce` 写入时把 RecordBatch 解开入库,`Fetch` 时再重组为 `RecordBatch`,因此同一 topic 也能被其它协议读写。

详见 [系统架构](./SystemArchitecture.md)。

## 能力总览

| 能力 | 状态 | 说明 |
|---|---|---|
| 生产 / 消费 / 位点 | ✅ | `Produce` / `Fetch` / `ListOffsets` / `OffsetCommit` / `OffsetFetch` |
| 幂等 Producer | ✅ | 分配 Producer ID + 序列号去重(滑动窗口 last-5 + epoch fencing) |
| 经典消费组 | ✅ | `FindCoordinator` / `JoinGroup` / `SyncGroup` / `Heartbeat` / `LeaveGroup` |
| KIP-848 消费组 | ✅ | `ConsumerGroupHeartbeat`(服务端分配),不支持 `subscribed_topic_regex` |
| Topic 管理 | ✅ | 创建 / 删除 / 扩分区;默认开启自动创建 |
| 配置管理 | ✅ | `DescribeConfigs` / `AlterConfigs` / `IncrementalAlterConfigs` |
| SASL / SCRAM 认证 | ✅ | SCRAM-SHA-256 / SCRAM-SHA-512 |
| ACL / 配额 | 🟡 | 可增删查,但**不参与鉴权与限流强制** |
| 委托令牌 | ✅ | 元数据管理(令牌本身不参与认证) |
| Metadata / DescribeCluster | ✅ | 集群拓扑、broker、topic / partition 信息 |
| Fetch 压缩 | 🟡 | 消费侧固定返回**未压缩**记录 |
| 事务 | ❌ | 不通告、不支持(见下方原因) |
| Share Group(KIP-932) | ❌ | 不支持 |

> 逐 API 的支持版本与差异见 [协议兼容矩阵](./Protocol.md);"支持 / 部分 / 不支持"的完整清单与原因见 [兼容性与限制](./Compatibility-and-Limitations.md)。

## 快速上手

启动单节点后,用官方 CLI 建 topic、生产、消费:

```bash
# 创建 topic
kafka-topics.sh --bootstrap-server localhost:9092 \
  --create --topic quickstart --partitions 3

# 生产(输入几行后 Ctrl-C)
kafka-console-producer.sh --bootstrap-server localhost:9092 --topic quickstart

# 从头消费
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic quickstart --from-beginning
```

完整步骤(含 Java `kafka-clients` 示例、SASL 连接)见 [快速开始](./QuickStart.md)。

## 文档导航

| 文档 | 内容 |
|---|---|
| [系统架构](./SystemArchitecture.md) | 五层架构、请求走向、与原生 Kafka 的关键差异 |
| [核心概念](./KafkaCoreConcepts.md) | Topic / Partition / Offset、Record、消费组、Coordinator、Segment |
| [协议兼容矩阵](./Protocol.md) | 逐 API 的支持版本、状态与差异 |
| [快速开始](./QuickStart.md) | 单节点启动、CLI 与 Java 客户端最小示例 |
| [CLI 操作指南](./CLI-Guide.md) | 官方 kafka 命令行工具对 RobustMQ 的操作大全 |
| [兼容性与限制](./Compatibility-and-Limitations.md) | 支持 / 部分 / 不支持清单及根因 |
