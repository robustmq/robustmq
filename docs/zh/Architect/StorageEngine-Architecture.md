# Storage Engine 架构

RobustMQ 提供三种存储引擎，通过 Storage Adapter 对上层 Broker 屏蔽差异，按 Topic 粒度独立配置。

---

## 三种存储引擎

| 引擎 | 配置值 | 延迟 | 持久化 | 适用场景 |
|------|--------|------|--------|---------|
| Memory | `EngineMemory` | 微秒级 | 否 | 实时数据、允许丢失 |
| RocksDB | `EngineRocksDB` | 毫秒级 | 是 | 单机持久化、边缘部署 |
| File Segment | `EngineSegment` | 毫秒级 | 是 | 生产集群、高吞吐 |

![三种存储引擎对比](../../images/arch_engine_overview.png)

---

## Memory 引擎

基于 `DashMap` 的纯内存存储，支持四种索引：

| 索引 | 结构 | 用途 |
|------|------|------|
| 主数据 | `DashMap<shard, DashMap<offset, Record>>` | 按 Offset 读取 |
| Tag 索引 | `DashMap<shard, DashMap<tag, Vec<offset>>>` | 按 Tag 查询 |
| Key 索引 | `DashMap<shard, DashMap<key, offset>>` | 按 Key 查询（Key 唯一） |
| 时间戳索引 | `DashMap<shard, DashMap<timestamp, offset>>` | 按时间查 Offset |

进程重启后数据丢失。在副本 Shard 中，Leader 先本地写入，Follower 通过 Pull 异步拉取；生产者的 `acks` 值决定写入是否等待副本确认。

---

## RocksDB 引擎

使用专用 Column Family（`DB_COLUMN_FAMILY_BROKER`）存储消息，内存中维护写锁避免并发冲突。

数据持久化在本地，RocksDB 不在节点间共享数据。集群中的副本同步由 storage-engine 的 ISR 协议提供，而不是共享 RocksDB；该引擎适合本地持久化，不承担 File Segment 引擎面向生产集群的主要场景。

---

## File Segment 引擎

生产级存储引擎，集群化部署，支持多副本、高吞吐、分层存储。

### I/O Pool

用固定数量的 I/O Worker（默认 16 个）管理所有 Partition，通过 `partition_id % worker_count` 固定映射，同一 Partition 的请求总是路由到同一个 Worker，保证写入顺序。

Worker 批量处理：阻塞等待第一个请求，然后非阻塞收集后续请求，一次批量处理可聚合数百至数千条消息，同一次 fsync 持久化。

消息从网络接收到磁盘写入使用 `Bytes`（Arc 引用计数）实现零拷贝，数据只有一份，不同地方持有引用。

### 索引

使用 RocksDB 存储四种索引：offset 索引、时间索引、key 索引、tag 索引。

索引与数据同步构建：Worker 批量处理 N 条记录时，同时构建这 N 条的所有索引，通过 RocksDB WriteBatch 一次性写入。数据文件一次 I/O，索引一次 I/O。

offset 索引采用稀疏索引策略：每 1000 条建一个索引点，记录该 Offset 对应的文件位置。查询时定位最近索引点，再顺序扫描最多 1000 条。1000 万条记录约占 240KB 索引空间，查询延迟约 2ms。

### 一致性协议：ISR

每个 Active Segment 有一个 Leader，维护 ISR（In-Sync Replicas）列表，Follower 从 Leader 拉取记录。持久性取决于请求的 `acks` 值：`acks=1` 在本地写入后返回，`acks=all` 则要求 ISR 数量满足配置的最小值，并等待高水位（HW）达到写入位置。

Follower 通过 Pull 模式主动批量拉取，高 QPS 场景下网络请求从百万级降到百级。

acks 配置：

| acks | 语义 |
|------|------|
| `0` | 不等待生产者响应 |
| `1` | 只等待 Leader 本地写入 |
| `all` / `-1` | 要求 `min_in_sync_replicas`，然后等待基于 ISR 计算的 HW |

当前 Kafka Produce 协议不支持 `quorum`。协议层只接受 `0`、`1`、`-1`；storage-engine 对除 `-1` 以外的值都走非 `acks=all` 写入路径。

### Active Segment 与 Sealed Segment

**Active Segment**：正在写入的活跃段，有 Leader 和 ISR 机制，Follower 持续 Pull 复制。

**Sealed Segment**：Segment 达到配置的结束条件后，storage-engine 异步向 Meta Service 请求封存。Meta Service 将其从可写状态切换出去；封存后的历史 Segment 只读，不参与 Active Segment 的 Leader 轮换。当前封存请求路径不会同步等待所有 ISR Follower 追上，因此不能把封存理解为 `acks=all` 的确认屏障。读取由仍保留该封存 Segment 的副本提供。

结果：Leader 数量 = Shard 数量（而非 Segment 总数），1000 个 Shard 只需 1000 个 Leader。历史数据读取压力分散到所有 Storage Node。

![Active/Sealed Segment 与 ISR 机制](../../images/arch_engine_segment.png)

### 扩容

新增 Storage Node 时不迁移任何历史数据。当前 Active Segment 写满后，新 Segment 自动分配到新节点。

### 分层存储

Sealed Segment 不可变，可直接从任意副本上传 S3，更新元数据即完成迁移。

| 数据层级 | 存储位置 | 延迟 |
|---------|---------|------|
| 热数据（Active Segment）| 本地 SSD | 毫秒级 |
| 温数据（近期 Sealed）| 本地 SSD/HDD | 毫秒级 |
| 冷数据（历史 Sealed）| S3 / MinIO / HDFS | ~50ms |

冷数据迁移到 S3 时可转换为 Parquet 格式，Spark、Hive 等分析工具可直接查询。

![分层存储](../../images/arch_engine_tiered.png)

---

## 两种文件模型

| 模型 | 描述 | 适用场景 |
|------|------|---------|
| Partition 独立文件 | 每个 Partition 独占文件，类似 Kafka | 低延迟高吞吐，Topic 数量不多 |
| Partition 共享文件 | 多个 Partition 共享文件，类似 RocketMQ | 海量 Topic/Partition |
