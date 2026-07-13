# 核心概念

本文解释 RobustMQ Kafka 中的核心概念,以及它们在 RobustMQ 统一内核上的具体落地方式。如果你熟悉原生 Kafka,这些概念完全一致;差异之处会明确标注。

![RobustMQ Kafka 核心概念](../../images/kafka-concepts.svg)

## Topic 与 Partition

**Topic** 是消息的逻辑分类,**Partition(分区)** 是 topic 的并行与顺序单元。

- 一个 topic 由一个或多个 partition 组成;顺序只在**单个 partition 内**保证,跨 partition 不保证全局有序。
- Partition 是并行度的基本单位:分区越多,可并行的生产 / 消费越多。
- 在 RobustMQ 中,每个 partition 对应存储层的一个 shard(段序列),由 File Segment 引擎持久化。

## Offset(位点)

**Offset** 是一条记录在某个 partition 内的位置,从 0 单调递增。

- Offset 由 broker 在写入时分配,连续且不可变。
- 消费进度即"已提交的 offset",由消费组通过 `OffsetCommit` 保存、`OffsetFetch` 读取。
- `ListOffsets` 用于查询 earliest / latest / 按时间戳定位的 offset。

> RobustMQ 的 offset 由存储层在写入时分配,保证同一 partition 内连续递增。

## Record 与 RecordBatch

**Record(记录)** 是最小的消息单元,包含 key、value、headers 和 timestamp。**RecordBatch** 是生产者为提升吞吐把多条 record 打包压缩后的批次。

RobustMQ 与原生 Kafka 在这里有一个**根本差异**:

| | 原生 Kafka | RobustMQ |
|---|---|---|
| 存储形态 | 原样保存压缩后的 RecordBatch | 保存**协议中立的、已解码的记录** |
| Fetch | zero-copy 直接回传原批次 | 读取记录后**重组**为 RecordBatch |
| 压缩 | 保留生产端压缩编码 | 消费侧固定返回**未压缩**记录 |

这样做的原因是:Kafka 与 MQTT 共享同一份 topic 数据,存储层必须使用一种不绑定任何单一协议的中立格式。代价是放弃了 Kafka 的 zero-copy 与压缩透传(见 [兼容性与限制](./Compatibility-and-Limitations.md))。

## Producer(生产者)

**Producer** 向 topic 的 partition 写入记录。

- 通过 key 的哈希或显式分区器决定目标 partition。
- **幂等 Producer**:RobustMQ 支持幂等生产。`InitProducerId` 分配 Producer ID,broker 用"序列号 + epoch fencing"去重——每个 `(producer, partition)` 维护一个 last-5 的滑动窗口,重复批次被安全丢弃。
- **事务 Producer**:暂不支持。带 `transactional_id` 的 `InitProducerId` 会返回 `TRANSACTIONAL_ID_AUTHORIZATION_FAILED`。

## Consumer 与 Consumer Group

**Consumer(消费者)** 从 partition 拉取记录;**Consumer Group(消费组)** 是一组协同消费同一批 topic 的消费者。

- 组内每个 partition 只会分配给**一个**成员,以此实现负载均衡与水平扩展。
- 成员加入 / 退出会触发 **rebalance(再均衡)**,重新分配 partition。
- 消费进度以"组 + topic + partition → offset"为粒度提交。

RobustMQ 同时支持两代消费组协议:

| 协议 | 相关 API | 分配方 |
|---|---|---|
| 经典协议 | `FindCoordinator` / `JoinGroup` / `SyncGroup` / `Heartbeat` / `LeaveGroup` | 客户端(group leader)分配 |
| KIP-848 | `ConsumerGroupHeartbeat` / `ConsumerGroupDescribe` | 服务端(协调器)分配 |

> KIP-848 暂不支持 `subscribed_topic_regex`(正则订阅)。

## Broker、Controller 与 Coordinator

**Broker** 是对外提供 Kafka 协议服务的节点:处理 `ApiVersions` / `Metadata`,serving 其负责的 partition 的读写。

**Controller** 与 **Coordinator** 在 RobustMQ 中都**不是**独立组件,而是 **meta-service 的 Raft Leader**:

- **Controller**:`Metadata` / `DescribeCluster` 返回的 controller id 指向当前 Raft Leader。
- **Group Coordinator**:消费组的成员管理与位点提交由协调器负责,协调器同样是 Raft Leader。`FindCoordinator` 返回它的地址。
- 只有当前节点是 Raft Leader 时才承担协调职责;否则返回 `NOT_COORDINATOR`,客户端据此重定向。
- 协调器地址经约 3 秒 TTL 缓存的 gRPC 查询获得,避免每次请求都打到元数据层。

> 原生 Kafka 用 KRaft / ZooKeeper 承担 Controller 职责;RobustMQ 统一用 Raft,省去了独立的协调组件。

## Segment(存储段)

**Segment(段)** 是 File Segment 存储引擎的物理存储单元,是 partition 在磁盘上的落地形态:

- **append / seal / scroll**:记录追加写入当前段,达到阈值后 seal(封存)并 scroll(滚动)到新段。
- **`offset → position` 索引 + mmap 读**:按 offset 快速定位物理位置并读取。
- **ISR 多副本**:段按多副本复制,`leader_epoch` 做 fencing 防止过期 leader 写入。

## 概念映射一览

| Kafka 概念 | 在 RobustMQ 的落地 |
|---|---|
| Topic / Partition | 统一内核的 topic 与 shard |
| Offset | 存储层写入时分配的连续序号 |
| RecordBatch | 写入时解码入库,Fetch 时重组 |
| Controller | meta-service Raft Leader |
| Group Coordinator | meta-service Raft Leader |
| 日志段(Log Segment) | File Segment 引擎的 Segment |

## 延伸阅读

- [系统架构](./SystemArchitecture.md)
- [协议兼容矩阵](./Protocol.md)
- [兼容性与限制](./Compatibility-and-Limitations.md)
