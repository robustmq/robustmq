# 兼容性与限制

本文诚实列出 RobustMQ Kafka **支持 / 部分支持 / 不支持**的能力,并给出**根因**。绝大多数限制都可以追溯到同一个设计取向:**Kafka 与 MQTT 共享同一份协议中立的存储**(见 [系统架构](./SystemArchitecture.md))。逐 API 的版本与状态见 [协议兼容矩阵](./Protocol.md)。

## 总览

| 能力 | 状态 |
|---|---|
| 生产 / 消费 / 位点 | ✅ 支持 |
| 幂等 Producer | ✅ 支持 |
| 经典消费组 / KIP-848 消费组 | ✅ 支持 |
| Topic / 分区 / 配置管理 | ✅ 支持 |
| SASL/SCRAM 认证 | ✅ 支持 |
| Metadata / DescribeCluster | ✅ 支持 |
| 委托令牌 | ✅ 支持(仅元数据) |
| Fetch 压缩 / 增量 fetch session / `read_committed` | 🟡 部分 |
| 配置强制 | 🟡 部分(可存不强制) |
| ACL / 配额 | 🟡 部分(可管理不强制) |
| 事务 | ❌ 不支持 |
| Share Group(KIP-932) | ❌ 不支持 |
| 副本重分配 / 日志目录 / 手动 leader 选举 | ⚪ 刻意不支持 |

## 完整支持 ✅

- **数据面**:`Produce`(含幂等)、`Fetch`(长轮询)、`ListOffsets`。
- **消费组**:经典协议(客户端分配)与 KIP-848(服务端分配)并存。
- **Topic 管理**:创建 / 删除 / 扩分区,默认开启自动创建。
- **配置管理**:`DescribeConfigs` / `AlterConfigs` / `IncrementalAlterConfigs`。
- **认证**:SASL/SCRAM(SCRAM-SHA-256 / SCRAM-SHA-512),SCRAM 用户凭据可通过 `kafka-configs.sh` 管理。
- **元数据**:`Metadata` / `DescribeCluster` / `DescribeTopicPartitions`。

## 部分支持 🟡

### Fetch 不回压缩、无增量 session

- 消费侧固定返回**未压缩**记录;`partition_leader_epoch` 恒为 `0`;不支持增量 fetch session;不支持 `read_committed` 隔离级别。
- **根因**:存储保存的是协议中立的已解码记录,而非 Kafka 原样压缩批次,因此无法做 zero-copy 与压缩透传,`Fetch` 时需重新组装 `RecordBatch`;没有事务也就没有 `read_committed`。

### 配置可存不强制

- 多数 topic / broker 配置**可以写入并读回**,但运行时**不一定强制生效**(例如某些保留 / 清理策略)。
- **根因**:存储与生命周期由 RobustMQ 统一内核按自身机制管理,并非逐条映射 Kafka 的配置语义。

### ACL 与配额可管理不强制

- `DescribeAcls` / `CreateAcls` / `DeleteAcls` 与 `DescribeClientQuotas` / `AlterClientQuotas` 均可用,规则会被存储和查询。
- 但 **ACL 不参与鉴权强制**,**配额不参与限流强制**(配额目前仅支持 `client-id` 维度)。
- **根因**:授权与限流的强制路径尚未接入,当前仅提供元数据管理能力。

### 委托令牌仅元数据

- `CreateDelegationToken` / `Renew` / `Expire` / `Describe` 可用于令牌的元数据管理,但**令牌本身不参与认证**。

### 客户端遥测为 no-op

- `GetTelemetrySubscriptions` / `PushTelemetry` 会被接受,但不下发订阅、不处理指标。

## 不支持 ❌

### 事务(Exactly-Once)

- 事务相关 API(`AddPartitionsToTxn` / `AddOffsetsToTxn` / `EndTxn` / `TxnOffsetCommit` / `DescribeTransactions` / `ListTransactions`)**不在 `ApiVersions` 中通告**。
- `InitProducerId` **仅支持幂等模式**;带 `transactional_id` 时返回 `TRANSACTIONAL_ID_AUTHORIZATION_FAILED`。
- `FindCoordinator` 会为 transaction 返回协调器,但随后的事务 API 立即失败。
- **影响**:请勿在客户端设置 `transactional.id`;`enable.idempotence=true`(幂等)是可用的。
- **根因**:事务需要事务日志、事务标记(`WriteTxnMarkers`)与 `read_committed` 读取路径,这套机制与协议中立存储的当前实现尚未打通。

### Share Group(KIP-932)

- 所有 Share Group API(`ShareGroupHeartbeat` / `ShareGroupDescribe` / `ShareFetch` / `ShareAcknowledge` / `*ShareGroupOffsets`)均不支持。
- **根因**:Share Group 是 Kafka 4.0 引入的新消费模型,依赖独立的共享状态存储,尚未实现。

## 刻意不支持 ⚪

以下**存储 / 副本运维类** API 会返回明确错误(而非崩溃),因为对应职责由 RobustMQ 存储层自动管理,不对外开放手动操作:

| API | 说明 |
|---|---|
| `AlterReplicaLogDirs` / `DescribeLogDirs` | 日志目录由存储层管理 |
| `ElectLeaders` | leader 选举由存储层 + Raft 自动完成 |
| `AlterPartitionReassignments` / `ListPartitionReassignments` | 副本分配自动管理,不支持手动重分配 |
| `UpdateFeatures` | 不提供 broker feature flag 更新 |
| `DescribeProducers` | 不通告 |

- **根因**:副本放置、leader 选举、日志目录都由存储引擎与 Raft 元数据层自动决策;暴露这些手动运维接口与 RobustMQ 的自治模型相冲突。

## 与原生 Kafka 的关键差异小结

| 维度 | 原生 Kafka | RobustMQ |
|---|---|---|
| 存储单元 | 原样压缩的 RecordBatch(zero-copy) | 协议中立的解码记录,Fetch 时重组 |
| Controller / Coordinator | KRaft / ZooKeeper | meta-service Raft Leader |
| 多协议 | 仅 Kafka | Kafka / MQTT 共享同一份数据 |
| 事务 | 支持 | 不支持 |
| ACL / 配额 | 强制 | 可管理,不强制 |
| 副本 / leader 运维 | 手动可控 | 存储层自动管理 |

## 延伸阅读

- [系统架构](./SystemArchitecture.md)
- [协议兼容矩阵](./Protocol.md)
- [核心概念](./KafkaCoreConcepts.md)
