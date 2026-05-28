# Storage Engine ISR (In-Sync Replica)

> **目标**:一套**稳定的 ISR 副本协议**,从 Kafka 十余年踩坑后的最终形态中提炼,直接作为 RobustMQ 的协议基线。**不背 Kafka 历史包袱**(早期 fetch 协议、ZK Controller、unclean 默认开等),**不留妥协路径**(没有"先 HW truncate,以后换 epoch"这种过渡状态)。
>
> 适用范围:`commitlog/memory`、`commitlog/rocksdb`、`filesegment` 三种存储引擎。
>
> **核心原则**:ISR 在 `EngineSegment` 维度实现,三引擎共享同一份 ISR 控制面,差异仅在 `ReplicaLog` trait 的本地存储实现。memory / rocksdb 的 `segment_seq` 恒为 0;filesegment 按大小自动切 segment——这是引擎特性,对 ISR 协议透明。

---

## 0. 协议总纲(不变式)

本协议的**正确性建立在以下 16 条不变式上**,任何实现都不得违反。后续章节是这些不变式的展开。

### 元数据权威与版本

- **(I1) 唯一权威**:`(shard, segment_seq)` 的 `leader / isr / leader_epoch / segment_epoch / replicas` 唯一权威值在 meta-service raft 状态机;broker 本地缓存仅作性能优化。
- **(I2) 三类 epoch 单调**:
  - `leader_epoch` 仅在 leader 切换时递增
  - `segment_epoch` 在 leader / isr / replicas 任一变化时递增
  - `broker_epoch` 在 broker 进程重启注册时递增(区分同一 broker_id 的不同进程实例,见 §3.5)
  - 三者一旦递增不可回退
- **(I3) ISR 变更需多重 fence**:meta-service 接受 ISR 变更前必须校验:
  - `req.leader_epoch == current.leader_epoch`(防 zombie leader)
  - `req.broker_epoch == current_known_broker_epoch(leader_id)`(防 zombie broker 进程)
  - `req.expected_segment_epoch == current.segment_epoch`(CAS,防并发 ISR 变更覆盖)
  - 任一失败即拒绝

### 写入与 Committed 语义

- **(I4) 写入原子性**:任何 append 必须通过 leader。leader 写入必须在同一 segment 锁内原子完成:
  1. 校验自身 role == Leader 且状态为 Active(非 Initializing,见 I11)
  2. 校验 `current_leader_epoch == meta.leader_epoch`
  3. 若 producer 携带 epoch(可选),校验 `producer.epoch == meta.leader_epoch`
  4. `append_at` 落本地
  5. 若是新 epoch 首批写入,同步更新并持久化 `LeaderEpochCache`
  6. 推进 `local_leo`
- **(I5) Committed 定义**:HW(High Watermark)是 **shard 维度**(对齐 memory/rocksdb 物理本质和现有 `ShardOffsetState.high_watermark_offset`)。一条记录 `offset < shard.HW` 即 committed。HW 由 active segment 的 leader 单方计算,见 I6。
- **(I6) HW 计算与单调性**:`new_hw_candidate = min(LEO over ISR of active segment)`,实际推进时强制 `shard.HW = max(current_shard.HW, new_hw_candidate)`,**HW 永不倒退**。HW 推进时只计入 ISR 中 `current_leader_epoch == self.leader_epoch` 的 follower(防止 epoch 不匹配的 follower 拉低 HW)。
- **(I7) HW 推进只由 fetch 触发**:follower 通过 fetch_offset 隐式上报 LEO,active segment leader 收到 fetch 时推进 shard.HW。写入路径不直接推 HW。
- **(I8) Committed 数据永不丢失**:任何已 committed(`offset < shard.HW`)的数据在任何故障后必须在新 leader 上保留。这是 ISR 协议的核心承诺。

### Truncation 与 Leader Epoch

- **(I9) Truncation 必须基于 Leader Epoch**:任何 follower 在加入 / 重启 / leader 切换 / 收到 FencedLeaderEpoch 后,必须先通过 `OffsetsForLeaderEpoch` 询问 leader 的 epoch 历史,再 truncate。**禁止使用本地 HW 或本地 LEO 作为 truncate 点**。
- **(I10) LeaderEpochCache 必须持久化**(rocksdb / filesegment);memory 引擎因无持久化,follower 重启等价于全新副本,必须从 leader_log_start 全量重拉(§9.5)。
- **(I11) Leader 上任原子性**:broker 收到 `SegmentLeaderAndIsr` 后,在转为 Active leader 之前必须完成:
  1. `LeaderEpochCache.assign(new_epoch, current_leo)` 并 fsync 持久化
  2. 重置 `follower_progress`
  3. 拒绝所有携带旧 `leader_epoch` 的在途 producer 写入和 follower fetch
  - 在第 1 步持久化完成前的 broker 状态为 `Initializing`,拒绝所有写入

### ISR 维护与可用性

- **(I12) ISR 扩展严格条件**:不在 ISR 的 follower 要重新加入,leader 端必须看到:
  - `follower.leo >= leader.local_leo`(追上当前 LEO,等价于已含全部 committed 数据)
  - `follower.current_leader_epoch == leader.epoch`
  - `follower.broker_epoch` 在 meta 中是已知存活值(未 fence)
- **(I13) min_in_sync_replicas**:`|ISR| < min_in_sync_replicas` 时,acks=all 写入拒绝(`NotEnoughReplicas`);leader 不得本地伪造 ISR 内容,ISR 是 meta 权威值的镜像。
- **(I14) 不做 unclean leader election**:ISR 空时拒写,segment 标记 `Unavailable`,绝不从非 ISR 副本选 leader。

### Fetch 协议

- **(I15) Fetch 是 long-poll**:无定时轮询路径。leader 收到 fetch 时:
  1. 校验 `req.current_leader_epoch == self.leader_epoch`(`<` 返回 `FencedLeaderEpoch`,`>` 返回 `UnknownLeaderEpoch`)
  2. 用 `req.fetch_offset` 反推 follower LEO,更新 follower_progress + 推进 HW(I6)
  3. 若有足够数据 → 立即返回;否则挂起最多 `max_wait_ms`
- **(I16) fetch_offset 即 LEO 上报**:follower 发 `fetch_offset = X` 隐式声明已持久化 `[0, X)` 全部数据。leader 据此推进 HW,**follower 不主动上报 LEO**。

---

## 1. 设计基础与历史教训

本协议从 Kafka 十余年 ISR 演进中提炼,具体来源:

- **KIP-101 (2017)**:Leader Epoch + OffsetsForLeaderEpoch,解决"本地 HW truncate 导致丢数据"
- **KIP-279 (2018)**:OffsetsForLeaderEpoch 协议细化,处理 fenced epoch
- **KIP-380 / KIP-497 (2019+)**:Broker Epoch + AlterPartition,防 zombie broker 进程发出陈旧元数据请求
- **KIP-320**:Consumer 路径的 leader epoch;思想推广到 ISR 变更带版本号 CAS
- **KIP-679**:ISR 扩展条件改为 `follower.leo >= leader.leo`(而非 `>= hw`)
- **KIP-237 / replica_lag_time_max_ms**:用时间窗口替代消息数判 lag,避免 flapping
- **long-poll fetch**:Kafka 早期就是长轮询,确保 HW 推进延迟可控
- **`min.insync.replicas`**:acks=all 的 committed 语义保证
- **HW 单调性**:`hw = max(current_hw, new_hw)`,防止 ISR 扩展时 HW 倒退使消费者看到的数据消失

本协议的 12 条不变式(§0)是上述 KIP 的提炼结果,**作为 RobustMQ 的初始协议直接落地**,不存在"先做简化版后续升级"的过渡形态。

协议固定行为:

- HW 推进由 leader 用 fetch_offset 反推,不接受 follower 主动上报 LEO
- Fetch 是 long-poll(携带 `max_wait_ms`)
- Truncation 由 `OffsetsForLeaderEpoch` RPC 驱动(KIP-101),禁止本地 HW 决定 truncate 点
- LeaderEpochCache 持久化(memory 引擎例外,见 §9.5)
- ISR 变更带 `segment_epoch` CAS
- Leader 切换后旧 leader 必须在下次写入被 fence

Kafka 的副本模型核心要素 → RobustMQ 映射：

| Kafka 概念 | RobustMQ 映射 | 说明 |
|---|---|---|
| Topic Partition | `EngineShard`(逻辑日志) | 仅用于路由 |
| Log Segment | `EngineSegment`(**物理副本单元**) | 副本身份在 segment |
| Replica / Leader / Follower | segment 维度。同一 shard 的不同 segment 可位于不同节点集 | |
| LEO (Log End Offset) | 已有 `commit_log_offset.latest_offset`,每个 `(shard, segment_seq)` 一个 | |
| HW (High Watermark) | 已有 `ShardOffsetState.high_watermark_offset`(shard 维度),由当前 active segment 的 leader 计算,单调 | shard 维度对齐 Kafka partition-level HW |
| ISR | 已有 `EngineSegment.isr: Vec<u64>` | |
| Leader Epoch | 已有 `EngineSegment.leader_epoch: u32` | KIP-101 |
| **Segment Epoch** | **新增** `EngineSegment.segment_epoch: u32`,每次 leader/ISR/replicas 变更递增 | KIP-320 思想,segment 维度 |
| **Broker Epoch** | **新增**,broker 注册到 meta 时拿到,进程级单调 | KIP-380 / KIP-497 |
| **LeaderEpochCache** | **新增** `Vec<(epoch, start_offset)>`,**持久化** | KIP-101 |
| **log_start_offset** | **新增** `EngineSegment.log_start_offset: u64`,retention 后该 segment 在 leader 上可读起点 | Kafka 同名 |
| Controller | meta-service(raft leader) | |
| Fetch RPC | follower → leader(long-poll) | 新增 |
| OffsetsForLeaderEpoch RPC | follower → leader(truncation 询问) | 新增 KIP-101 |
| AlterPartition (ISR 变更 RPC) | broker → meta-service raft op `UpdateSegmentIsr` | 新增 KIP-497 思想 |
| acks=0/1/all | producer 路径上 acks 语义 | |
| Producer leader_epoch | producer 写入可携带 epoch(可选),leader 校验 | KIP-320 思想 |

**为什么是 segment 而不是 shard**：
- 同一 shard 的不同 segment 在 filesegment 引擎下可能位于不同节点集（segment 创建时按当时的节点状态选副本），shard 维度无法表达这种"不同时段不同拓扑"
- segment 维度的副本元数据 `EngineSegment.{replicas, leader, leader_epoch, isr}` 在 meta-service 中已建模并由 `segment_leader_switch` 维护
- memory/rocksdb 永远单 segment（`segment_seq=0`），是 segment 模型的退化形态

非目标：
- 不引入新的共识算法，ISR 变更回写 meta-service raft
- 不做副本拓扑动态扩缩（reassign），segment 创建时副本拓扑固定
- 不实现 unclean leader election（§16 划出）
- 不实现 KIP-966 ELR / KIP-392 Observer（§16 划出）

---

## 2. 总体架构

![ISR 总体架构](./diagrams/01-architecture.png)

控制面 / 数据面分离：

- **控制面（meta-service）**：segment 创建时分配 `replicas / leader / isr`；segment leader 切换；广播 `SegmentLeaderAndIsr`。
- **数据面（storage-engine 各 broker）**：segment leader 接 producer 写、回答 follower fetch；follower 主动 fetch；leader 计算 HW；本地 `ReplicaState` 仅缓存。

---

## 3. 数据模型变更

### 3.1 `EngineSegment`(部分已存在,需扩展)

```rust
// metadata-struct/src/storage/segment.rs
pub struct EngineSegment {
    // === 已存在 ===
    pub shard_name: String,
    pub segment_seq: u32,
    pub replicas: Vec<Replica>,    // {replica_seq, node_id, fold}
    pub leader: u64,
    pub leader_epoch: u32,
    pub isr: Vec<u64>,
    pub status: SegmentStatus,     // Write / PreSealUp / SealUp / PreDelete / Deleting

    // === 新增 ===
    pub segment_epoch: u32,            // KIP-320 思想:ISR / leader / replicas 任一变更即递增
    pub leader_broker_epoch: u64,      // 当前 leader 上任时的 broker_epoch (见 §3.5)
                                       // meta 用它 fence 旧进程的 zombie ISR 请求
    pub log_start_offset: u64,         // retention 后该 segment 在 leader 上可读起点
}
```

**三类 epoch 的职责区别**:

| Epoch | 递增时机 | 作用 |
|---|---|---|
| `leader_epoch` | 仅 leader 切换 | KIP-101 truncation 协议核心;LeaderEpochCache 用它分段 |
| `segment_epoch` | leader / ISR / replicas 任一变更 | meta-service ISR 变更 CAS;广播通知排序 |
| `broker_epoch` | broker 进程注册到 meta | 区分同一 broker_id 的不同进程实例,防 zombie 进程 |

**为什么 `leader_broker_epoch` 要存在 segment 上**:
- ISR shrink 请求路径:leader broker → meta-service
- meta 收到请求要校验"发出请求的 broker 是当前 leader 的当前进程实例"
- 这就需要存 `(leader_node_id, leader_broker_epoch)` 二元组,只看 leader_id 不够(同一个 node_id 重启后是另一个进程)

所有副本身份信息都在 segment 上。**`EngineShard` 不加任何 ISR 相关字段**。

**`SegmentStatus` 扩展与读写权限语义修正(D4)**:

```rust
pub enum SegmentStatus {
    Write,         // active segment,可读可写
    PreSealUp,     // 准备封口,可读可写(写入路径会触发切到新 segment)
    SealUp,        // 已封口,可读不可写
    PreDelete,     // 准备删除,不可读不可写(retention 即将清除)
    Deleting,      // 删除中,不可读不可写
    Unavailable,   // === 新增 === ISR 空导致无法选 leader,不可写
                   //              仍可读历史数据(committed offset)
}

impl EngineSegment {
    /// 可读条件:Write / PreSealUp / SealUp / Unavailable
    /// 修正点:**SealUp 状态必须可读**(消费者读历史 + follower 追赶旧 segment)
    pub fn allow_read(&self) -> bool {
        matches!(self.status,
            SegmentStatus::Write | SegmentStatus::PreSealUp |
            SegmentStatus::SealUp | SegmentStatus::Unavailable)
    }

    /// 可写条件:Write / PreSealUp
    pub fn allow_write(&self) -> bool {
        matches!(self.status, SegmentStatus::Write | SegmentStatus::PreSealUp)
    }
}
```

**为什么 `allow_read` 必须包含 SealUp**(代码现状 bug):
- 当前 `allow_read()` 只允许 `Write`,导致 SealUp 后**完全不能读**
- 消费者读历史数据(filesegment 多 segment 场景):必须能读 SealUp 的旧 segment
- ISR follower 追赶:filesegment 切 segment 后 follower 仍需拉旧 segment 的尾巴
- 不修复这个,ISR 协议在 filesegment 上**无法工作**

写入路径用 `allow_write` 校验,读取路径(含 fetch handler)用 `allow_read` 校验。

### 3.2 LeaderEpochCache（持久化，关键）

**这是 KIP-101 的核心数据结构，协议强制要求**（不变式 #7、#8）。

```rust
// storage-engine/src/isr/leader_epoch.rs（新建）
pub struct LeaderEpochCache {
    // 单调递增 (epoch, start_offset)
    // start_offset 是该 epoch 产生的第一条记录的 offset
    entries: Vec<LeaderEpochEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct LeaderEpochEntry {
    pub epoch: u32,
    pub start_offset: u64,
}

impl LeaderEpochCache {
    /// 收到新的 leader epoch 时调用(包括第一次成为副本)
    pub fn assign(&mut self, epoch: u32, start_offset: u64);

    /// follower 询问"我有到 my_epoch 的日志,该 epoch 的 end_offset 是多少"
    /// 返回:my_epoch 在本地的下一个 epoch 的 start_offset - 1,
    ///       如果 my_epoch 是当前最大,返回当前 LEO
    pub fn end_offset_for(&self, my_epoch: u32) -> u64;

    /// 用于 truncate 后修剪
    pub fn truncate_from_end(&mut self, end_offset: u64);
    pub fn truncate_from_start(&mut self, start_offset: u64);
}
```

**持久化策略**（三引擎差异）：
- memory：**无法持久化**（进程重启就丢），因此 memory 引擎的 follower 在 leader 重启后必须**整段重新同步**。这是 memory 引擎的固有限制（数据本身就不持久化）。
- rocksdb：单独 column family 或 key 前缀 `/leader_epoch/{shard}/{segment_seq}/`
- filesegment：sidecar 文件 `{segment_file}.leader-epoch-checkpoint`（同 Kafka）

每次 leader 切换、本地 append 新数据时检查并写入。**未持久化的 LeaderEpochCache 等于没有 ISR 安全性**。

### 3.3 `EngineShardConfig` 扩展

```rust
// metadata-struct/src/storage/shard.rs
pub struct EngineShardConfig {
    pub replica_num: u32,                  // 已存在
    pub storage_type: StorageType,
    pub max_segment_size: Option<u64>,
    pub max_record_num: Option<u64>,
    pub retention_sec: u64,

    // === 新增 ISR 相关配置(命名对齐 Kafka) ===
    pub min_in_sync_replicas: u32,         // acks=all 最小 ISR,默认 1
    pub replica_lag_time_max_ms: u64,      // follower 多久没追上被踢,默认 30_000(同 Kafka)
    pub replica_fetch_max_bytes: u64,      // 单次 fetch 最大字节数,默认 1 MiB
    pub replica_fetch_wait_max_ms: u64,    // long-poll 最大等待,默认 500
    pub replica_fetch_min_bytes: u64,      // long-poll 累积多少字节就立即返回,默认 1

    // unclean leader election 协议禁用,字段保留供运维 override 时强制错误
    pub unclean_leader_election_enable: bool,  // 协议要求始终 false
}
```

> 兼容性：新字段均带 `#[serde(default)]`，老编码解码后填默认值。

### 3.4 `ReplicaState`(broker 进程内运行时)

**两层结构**:
- **ShardReplicaState**:每个 shard 一份,持有 shard 维度的 HW / LEO / hw_watcher(对齐代码现状 `ShardOffsetState`)
- **SegmentReplicaState**:每个 segment 一份,持有副本身份(role / leader_epoch / isr / follower_progress)

```rust
// storage-engine/src/isr/state.rs(新建)
pub struct ReplicaStateRegistry {
    // 每 shard 一份
    pub shard_states: DashMap<String, Arc<ShardReplicaState>>,
    // 每 (shard, segment_seq) 一份
    pub segment_states: DashMap<(String, u32), Arc<SegmentReplicaState>>,
}

/// shard 维度状态:HW / LEO 跨 segment 连续,因此放在 shard 上
/// (对应代码现状 `ShardOffsetState`,本结构最终替换之)
pub struct ShardReplicaState {
    pub shard_name: String,
    pub local_leo: AtomicU64,        // shard 全局 LEO(对应 latest_offset)
    pub local_hw: AtomicU64,         // shard 全局 HW (单调,只增不减)
    pub log_start_offset: AtomicU64, // retention 起点
    pub hw_watcher: watch::Sender<u64>,  // 广播给 acks=all 等待者
}

/// segment 维度状态:副本身份
pub struct SegmentReplicaState {
    pub shard_name: String,
    pub segment_seq: u32,
    pub leader_epoch: u32,
    pub segment_epoch: u32,
    pub role: ReplicaRole,
    // leader 视角:每个 follower 的 LEO + 进度时间戳 + broker_epoch
    pub follower_progress: DashMap<u64 /*node_id*/, FollowerProgress>,
}

pub enum ReplicaRole {
    /// 收到 LeaderAndIsr 但尚未完成 LeaderEpochCache 持久化等准备工作。
    /// 拒绝所有 producer 写入和 follower fetch(I11)。
    LeaderInitializing,
    /// 完全就绪,接受写入和 fetch。
    LeaderActive,
    /// 跟随某个 leader,启动 fetcher 拉取。
    /// 跟随前必须先走 OffsetsForLeaderEpoch truncation(I9)。
    FollowerInitializing,
    FollowerActive,
}

pub struct FollowerProgress {
    pub broker_epoch: u64,         // 上次 fetch 时 follower 自报的 broker_epoch
    pub leo: u64,                  // = req.fetch_offset
    pub last_fetch_ts: u64,
    pub last_caught_up_ts: u64,    // 上次"追上"时刻,见 §6.4
}
```

**持久化策略**:
- `ShardReplicaState`:`local_leo` / `local_hw` / `log_start_offset` 持久化到 commitlog 已有的 shard offset checkpoint(对齐代码现状 `ShardOffsetState`)
- `SegmentReplicaState`:**不持久化**。进程重启时从 meta-service 拉取 `EngineSegment` + 本地持久化 `LeaderEpochCache` 重建

**为什么 HW 在 shard 维度而非 segment 维度**:
- memory/rocksdb 的物理本质是"shard = 一根连续 offset 轴",没有"per-segment HW"概念
- filesegment 切 segment 后 HW 跨 segment 连续,消费者从 segment N 读到 N+1 不需要切换 HW
- 代码现状 `ShardOffsetState.high_watermark_offset` 已经是 shard 维度,对齐
- 对应 Kafka:HW 在 partition 维度(我们的 partition = shard),不是在 log segment 维度

**状态转换**:
- `LeaderInitializing → LeaderActive`:LeaderEpochCache 持久化完成
- `FollowerInitializing → FollowerActive`:`OffsetsForLeaderEpoch` + truncate 完成
- `LeaderActive → FollowerInitializing`:收到 LeaderAndIsr 通知自己不再是 leader
- `FollowerActive → LeaderInitializing`:收到 LeaderAndIsr 通知自己当选为新 leader

### 3.5 Broker Epoch 注册机制

broker 进程的生命周期标识。每次进程启动到 meta-service 注册时获取新值。

**protobuf 改动(D5,必须做)**:

```protobuf
// 当前 RegisterNodeReply 是空 struct,需扩展
message RegisterNodeReply {
  uint64 broker_epoch = 1;   // === 新增 === meta 分配的当前进程 epoch
}

// 当前 RegisterNodeRequest 不变(broker 不提议自己的 epoch,由 meta 分配)
```

**meta-service raft 状态机改动**:

```rust
// MetaCacheManager 内部新增
pub struct NodeRegistry {
    // node_id → last assigned broker_epoch (持久化到 raft)
    epochs: HashMap<u64, u64>,
}

// register_node_by_req 处理:
fn register_node(req: RegisterNodeRequest) -> RegisterNodeReply {
    let new_epoch = node_registry.epochs.entry(node.node_id).or_insert(0);
    *new_epoch += 1;
    // 通过 raft op 持久化
    raft_write(NodeRegistryUpdate { node_id: node.node_id, broker_epoch: *new_epoch });
    RegisterNodeReply { broker_epoch: *new_epoch }
}
```

**broker 进程缓存与使用**:

- broker 启动时调 register_node → 拿到 broker_epoch → 缓存到内存
- 所有向 meta 发起的写请求(`UpdateSegmentIsr` 等)必须携带这个 epoch
- broker 不持久化 broker_epoch 到本地(进程重启就是新进程,自然该重新注册拿新值)

**fence 流程示例**:

1. broker B1 启动 → register_node → 拿到 `broker_epoch=7`
2. B1 进程崩溃,但 meta-service 心跳还没超时
3. B1 重启 → register_node → 拿到 `broker_epoch=8`
4. 此时若 B1 旧进程残留的网络包还在路上,带着 `broker_epoch=7` 到达 meta
5. meta 用 `node_registry[B1]==8` 校验,7 < 8 → 拒绝 `StaleBrokerEpoch`

> **broker_epoch 与 leader_broker_epoch 的关系**:
> - `broker_epoch` 是 broker 进程级版本,跟具体 segment 无关
> - `leader_broker_epoch` 是某个 segment 当选 leader 时该 broker 的 broker_epoch 快照,存在 `EngineSegment` 上
> - ISR 变更校验时 meta 比对:`req.leader_broker_epoch == segment.leader_broker_epoch && req.broker_epoch == node_registry[req.node_id]`

---

## 4. ReplicaLog 抽象

memory / rocksdb / filesegment 三个引擎共享 ISR 控制面，差异仅在本地存储读写。抽出 trait：

```rust
// storage-engine/src/isr/log.rs（新建）
#[async_trait]
pub trait ReplicaLog: Send + Sync {
    /// follower 收到 leader 的 records 后落本地。要求 base_offset == latest_offset(...)。
    /// 不连续返回 OutOfOrder,触发 truncate 流程。
    async fn append_at(
        &self,
        shard: &str,
        segment_seq: u32,
        base_offset: u64,
        records: Vec<StorageRecord>,
    ) -> Result<(), StorageEngineError>;

    /// 用于 follower fetch、消费者 read 都走这个接口。
    async fn read_from(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
        max_bytes: u64,
    ) -> Result<Vec<StorageRecord>, StorageEngineError>;

    /// 当前 segment 已写入的最大 offset。
    fn latest_offset(
        &self,
        shard: &str,
        segment_seq: u32,
    ) -> Result<u64, StorageEngineError>;

    /// 截断到指定 offset(含),用于 follower 在 leader 切换时丢弃未提交日志。
    async fn truncate_to(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
    ) -> Result<(), StorageEngineError>;
}
```

**三个引擎的实现要点**：

- **memory**：`segment_seq` 参数对当前实现来说恒为 0，存储仍然是 `DashMap<offset, Record>`，`segment_seq` 仅传入接口供未来扩展。`truncate_to`：遍历 `>offset` 的 key 删除。
- **rocksdb**：key 编码加入 `segment_seq` 段：`/record/{namespace}/{shard}/{segment_seq:08}/record/{offset:20}`（兼容现有 key 设计，老数据按 `segment_seq=0` 解读）。`truncate_to`：range delete 前缀 `(shard, segment_seq, offset+1)`。
- **filesegment**：`segment_seq` 直接对应文件名；`append_at` 写当前 active segment 文件；`truncate_to` 截断文件尾或删除尾部 segment。

> 关键：`segment_seq` 在签名里是显式参数。memory/rocksdb 传 0；filesegment 传当前 active 或 follower 正在追的 segment。**ISR 控制面只调 trait，不知道也不关心引擎类型**。

---

## 5. 写入路径(segment leader 侧)

### 5.0 写入路由模型(broker 转发)

**本协议的"客户端"是 broker**(主要来自 storage-adapter / mqtt-broker / kafka-broker / nats-broker 等),不是面向终端 producer 的接口。因此采用 **broker 转发模型**:

```text
非 leader broker A 收到写入请求:
  1. 从本地 metadata 缓存找到 shard 的 active_segment.leader = B
  2. 通过 storage-engine RPC 通道转发给 B
  3. 等 B 应答后返回给上游

leader broker B 收到 (本地或转发):
  按 §5.2 原子流程处理
```

**代码现状对齐**:`storage-engine/src/core/write.rs::batch_write` 已实现此路由,本协议沿用。

**与 Kafka client 重试模型的差异**:

| 维度 | broker 转发(本协议) | Kafka client 重试 |
|---|---|---|
| 客户端复杂度 | 简单(broker 端 broker 转发) | 复杂(client 拉 metadata + 重试) |
| 网络跳数 | 多一跳(非 leader broker → leader broker) | 直接到 leader |
| Zombie leader fence | 转发链路上的每个 broker 都要校验 epoch | client 收到 NotLeader 立即重试,不堆 zombie |
| 路由表延迟 | broker 之间通过 SegmentLeaderAndIsr 同步 metadata,延迟低 | client metadata 缓存延迟较高 |

**Zombie 防御要点**(对应不变式 I4):
- 中转 broker(转发方)也必须用 §5.2 的 epoch 校验逻辑:在转发之前校验自己缓存的 leader 是当前 leader(epoch 一致)
- 否则转到旧 leader → 旧 leader 校验 epoch 失败 → 返回 FencedLeaderEpoch → 中转 broker 拉新 meta 重试
- 等价于"client 重试",只是重试逻辑在 broker 内部

### 5.1 ProduceRequest 协议字段

```protobuf
message StorageEngineProduceRequest {
  string shard_name = 1;
  bytes records = 2;
  uint32 acks = 3;                          // 0 / 1 / -1(=all)
  uint64 timeout_ms = 4;                    // acks=all 等 HW 推进的最大时间
  // 转发方(上游 broker)写入时携带其认知的 leader_epoch
  // 用于防 zombie:转发方 metadata 过时时被 leader 直接拒
  optional uint32 current_leader_epoch = 5;
  // === §18.1 扩展点(默认不带,leader 忽略) ===
  optional uint64 producer_id = 6;
  optional uint32 producer_epoch = 7;
  optional int32  base_sequence = 8;
}
```

> 注:不传 `segment_seq`,因为路由是按 shard,leader 自己用 `shard.active_segment_seq` 决定写入到哪个 segment。这避免"上游缓存的 active_segment 已过期"导致写入老 segment 的问题。

### 5.2 原子写入路径(I4)

**整段必须在同一把 segment 锁内完成**,中途不得释放锁(否则 LeaderAndIsr 通知到达可能让 role 切换发生在 append 之后):

```text
acquire(shard_lock):

  1. 路由:active_segment_seq = shard.active_segment_seq
     active_segment = local_cache.get_segment(shard, active_segment_seq)
  2. 校验自身状态(对 active_segment):
     - role == LeaderActive → 否则:
       LeaderInitializing → 返回 NotReady(让上游退避重试)
       FollowerActive/Initializing → 返回 NotLeaderForPartition(让上游转发到真 leader)
     - self.leader_epoch == meta.leader_epoch → 否则 FencedLeaderEpoch
  3. 若 req.current_leader_epoch 存在:
     - == self.leader_epoch → 通过
     - < self.leader_epoch → FencedLeaderEpoch(上游持有旧 metadata)
     - > self.leader_epoch → UnknownLeaderEpoch(上游比 leader 还新,极罕见)
  4. acks=all 校验:|ISR| >= min_in_sync_replicas → 否则 NotEnoughReplicas
  5. ReplicaLog::append_at(shard, active_segment_seq, shard.local_leo, records) 落本地
  6. 若是当前 epoch 的首批写入:
       LeaderEpochCache.assign(current_leader_epoch, shard.local_leo)
       并 fsync 持久化(rocksdb 单条 put 或 filesegment 单 fsync)
  7. shard.local_leo += records.len()
  8. [hook §18.1] on_append_with_pid(req.producer_id, req.producer_epoch,
                                     req.base_sequence, base_offset = shard.local_leo - N)
       默认 no-op;未来 idempotent producer 在此更新 ProducerStateEntry

release(shard_lock)

9. 按 acks 语义应答(锁外):
   acks=0:  立即返回 OK(fire-and-forget)
   acks=1:  本地落盘成功即返回 OK
   acks=all: select on (shard.hw_watcher.changed(), timeout_ms)
            条件:shard.local_hw >= records.last_offset
            timeout → RequestTimedOut,**不回滚已写入**(对齐 Kafka)
```

> **不变式 I4 的关键体现**:步骤 2-7 在锁内原子。LeaderAndIsr 通知改 role 也需要拿同一把锁,这样不会插入到"epoch 校验通过"与"append" 之间。
>
> 锁的粒度是 **shard 级别**(而非 segment 级别),因为 active_segment_seq 是 shard 上的状态,而且写入只会进 active segment。filesegment 跨 segment 切换时也持此锁。

### 5.3 HW 推进与单调性(I6, I7)

HW 是 **shard 维度**(`ShardReplicaState.local_hw`),不在 segment 上。HW 推进**只发生在 fetch handler 里**(§6),写入路径只更新 LEO。这是 Kafka 的关键设计——分离"写入"和"确认"。

```text
// active segment leader 收到 fetch 后(详见 §6):
let isr = active_segment.isr;
// I6:只计入 epoch 匹配的 follower,防止陈旧 epoch 的 follower 拉低 HW
let eligible_followers = isr.iter().filter(|f| f.current_leader_epoch == self.leader_epoch);
new_hw_candidate = min(shard.local_leo, min(p.leo for p in eligible_followers))
shard.local_hw = max(shard.local_hw, new_hw_candidate)   // I6 强制单调
if shard.local_hw 推进了:
    shard.hw_watcher.send(shard.local_hw)                // 唤醒 acks=all 等待者
```

**为什么 HW 必须单调**(避免 ISR 扩展时的 HW 倒退):
- t1:ISR={A,B},LEO_A=100,LEO_B=100,HW=100
- t2:C 追上加入 ISR,ISR={A,B,C},LEO_C=99(C 刚追完到 99)
- 若 `new_hw = min(...) = 99` → HW 倒退,消费者已读到 offset=99 的数据"消失"
- 强制 `HW = max(100, 99) = 100`,等 C 也追到 100 才正常推进

**为什么 HW 在 shard 维度不在 segment 维度**:
- memory/rocksdb 是 shard 全局 offset 轴,segment 永远 0,HW 是 shard 概念
- filesegment 切 segment 后 HW 跨 segment 连续(消费者从 segment N 读到 N+1 不需要换 HW)
- 代码现状 `ShardOffsetState.high_watermark_offset` 已是 shard 维度
- 对齐 Kafka:HW 在 partition 维度而非 log segment 维度

### 5.4 min_in_sync_replicas 保护

- acks=all 时:`|ISR| < min_in_sync_replicas` → 立即 `NotEnoughReplicas`
- acks=1 时:不受此约束(只要 leader 在就接受写入,用户自选语义)
- **Committed 语义**:一条记录被视为 committed 当且仅当 `offset < HW` 且 leader 计算 HW 时 ISR 满足 `min_in_sync_replicas`
- ISR 由 meta 权威定义(I13),leader 不得本地伪造

### 5.5 写入期间 ISR 缩小的处理(K6)

acks=all 写入是异步等待 HW 推进的(§5.2 step 9)。**等待期间 ISR 可能缩小**,处理规则:

- **已 append 但还在等 HW 的写入**:**不主动失败**,继续等。可能两种结局:
  - HW 推到 records.last_offset → 正常返回 OK
  - 等到 timeout_ms 超时 → 返回 RequestTimedOut(不回滚已写入数据,与 Kafka 一致)
- **ISR 缩小后到达的新 acks=all 写入**:在 §5.2 step 4 校验 `|ISR| < min_in_sync_replicas` → 立即 `NotEnoughReplicas` 拒绝
- **若 HW 因 ISR 缩小后变得能推进**(原本卡在某个慢 follower):立即推进 HW,唤醒已等待的 acks=all 请求

> **注意**:ISR 缩小后,剩余 ISR 副本的 HW 推进**反而可能加快**(因为不需等被踢出的慢副本)。这是 Kafka 的预期行为——已 ack 的语义没破坏,新 ack 的语义也没破坏。

### 写入时序（acks=all）

![写入时序（acks=all）](./diagrams/02-write-sequence.png)

---

## 6. 复制路径（follower 侧）

**核心设计:long-poll fetch + leader 用 fetch_offset 反推 LEO**(严格对齐 Kafka)。

### 6.1 fetch_offset 的隐式含义(关键)

follower 发送 `fetch_offset = X` 给 leader 时,**隐式声明:"我已经持久化了 [0, X) 的全部数据"**。  
leader 收到后:
1. 把 `follower_progress[replica_id].leo = X` 更新进 ReplicaState
2. **leader 检查这是否能推进 HW**:`new_hw = min(leader_leo, min over isr.leo)`,若推进则唤醒 acks=all 等待者
3. 然后才开始准备返回 records

**这意味着 follower 自己的 LEO 推进永远比 leader 视角的 LEO 滞后一个 fetch round**:
```text
T1: leader_hw=5, leader_leo=8, follower_local_leo=5
T2: follower 发 fetch(fetch_offset=5)
T3: leader 看到 fetch_offset=5,更新 follower.leo=5,HW=min(8,5)=5(不变)
    返回 records=[5,6,7], leader_hw=5
T4: follower 落盘 5,6,7, follower_local_leo=8
T5: follower 发 fetch(fetch_offset=8)
T6: leader 看到 fetch_offset=8,更新 follower.leo=8,HW=min(8,8)=8
    唤醒等 offset<8 的 acks=all 等待者
    返回 records=[], leader_hw=8
T7: follower 知道 HW=8,推进 local_hw=8
```

→ **follower 本地的 HW 比 leader 慢一个 RTT**。本协议消费者只读 leader(§16),所以消费者拿到的 HW 是实时的,这个滞后不影响消费者可见性。但 follower 本地的 HW 仍要参与 §11 checkpoint 持久化,以便 follower 升任 leader 时 HW 不倒退(见 §8.1 case 1 + K7 注解)。

### 6.2 long-poll fetch

follower fetch 是长轮询,不是定时轮询。fetcher 任务:

```text
loop {
    let local_leo = replica_log.latest_offset(shard, segment_seq)?;
    let req = FetchRequest {
        shard_name, segment_seq,
        fetch_offset: local_leo,
        current_leader_epoch: my_leader_epoch,  // follower 当前认知的 leader epoch
        replica_id: self.node_id,
        replica_broker_epoch: self.broker_epoch, // 防 zombie follower 进程
        max_bytes: replica_fetch_max_bytes,
        min_bytes: replica_fetch_min_bytes,
        max_wait_ms: replica_fetch_wait_max_ms,
    };
    let resp = leader_client.fetch(req).await?;
    // ... 处理 resp(无 sleep,立刻进入下一轮)
}
```

**leader 端的精确校验顺序**(I15):

```text
acquire(segment_lock):
  // 1) 角色校验
  if self.role != LeaderActive:
      release lock; return NotLeaderForPartition  // 含 LeaderInitializing

  // 2) leader_epoch 三态校验
  if req.current_leader_epoch < self.leader_epoch:
      return FencedLeaderEpoch   { current_leader_epoch: self.leader_epoch }
  if req.current_leader_epoch > self.leader_epoch:
      return UnknownLeaderEpoch  // follower 比 leader 新,极罕见,follower 退避

  // 3) fetch_offset 范围校验
  if req.fetch_offset < self.log_start_offset:
      return OffsetOutOfRange { leader_log_start, leader_leo }  // 被 retention 抛在后面
  if req.fetch_offset > self.local_leo:
      return OffsetOutOfRange { leader_log_start, leader_leo }  // 脑裂残余,follower 比 leader 远

  // 4) 更新 follower_progress (I7, §6.4 详细规则)
  progress = follower_progress.entry(req.replica_id).or_default()
  if req.replica_broker_epoch < progress.broker_epoch:
      return StaleBrokerEpoch  // zombie follower 进程
  progress.broker_epoch = req.replica_broker_epoch
  progress.leo          = req.fetch_offset
  progress.last_fetch_ts = now
  if req.fetch_offset >= leader_leo_at_request_arrival:
      progress.last_caught_up_ts = now

  // 5) 用 progress 推进 shard HW (I6, 单调)
  //    注意:HW 是 shard 维度,但 ISR 是当前 active segment 的 ISR
  //    本 segment 不是 active 时只更新 follower_progress 不推 HW
  if self.is_active_segment && req.replica_id in self.isr:
      // 只计入 epoch 匹配的 follower(I6 防陈旧 follower 拉低 HW)
      let eligible_followers = self.isr
          .iter()
          .filter(|f_id| self.follower_progress[f_id].current_leader_epoch == self.leader_epoch);
      new_hw_candidate = min(shard.local_leo, min(p.leo for p in eligible_followers));
      shard.local_hw = max(shard.local_hw, new_hw_candidate)
      if shard.local_hw 推进了:
          shard.hw_watcher.send(shard.local_hw)  // 唤醒 acks=all 等待者
release lock

// 6) long-poll 数据返回(锁外)
data_available = shard.local_leo - req.fetch_offset
if data_available >= req.min_bytes:
    return Ok(records, leader_hw = shard.local_hw, leader_log_start, leader_leo = shard.local_leo, leader_epoch)
挂起 wait_for_either(append_signal, timeout(max_wait_ms))
唤醒后:
    重读 shard.local_leo,返回当前可读 records (可能为空)
```

**为什么不能用定时轮询**:
- 轮询间隔短 → 空闲时 CPU/网络浪费
- 轮询间隔长 → HW 推进延迟、acks=all 延迟升高
- long-poll 同时解决两个问题,且与 Kafka 一致

### 6.3 fetch 错误分支(follower 侧)

```text
match resp {
    Ok { records, leader_hw, leader_log_start, leader_leo, leader_epoch } => {
        replica_log.append_at(shard, segment_seq, shard.local_leo, records).await?;
        // 若 records 跨 epoch(batch header 含 partition_leader_epoch),
        // 用 leader_epoch_cache.assign(new_epoch, base_offset_of_batch) 并 fsync
        // 推进 follower 本地 shard HW(单调):
        shard.local_hw = max(shard.local_hw, min(shard.local_leo + records.len(), leader_hw));
        // 同时校对 leader_epoch:若 != self.leader_epoch 说明本地缓存陈旧,刷新
    }

    Err(NotLeaderForPartition) | Err(NotReady) => {
        // 拉新 metadata,切换 fetcher target
        // (NotReady 表示新 leader 处于 LeaderInitializing,稍后重试同一目标)
    }

    Err(FencedLeaderEpoch { current_leader_epoch }) => {
        // 本地 epoch 比 leader 旧 → 必须先做 truncation,不能直接继续 fetch
        // 1) 刷新本地 leader_epoch = current_leader_epoch
        // 2) 走 §9 OffsetsForLeaderEpoch 流程
        // 3) truncate 完成后才能 fetch
    }

    Err(UnknownLeaderEpoch) => {
        // follower 比 leader 还新(leader 刚启动尚未拉到最新 meta)
        // 退避等待,不 truncate。等 LeaderAndIsr 推到 leader 再继续
    }

    Err(StaleBrokerEpoch) => {
        // follower 自己的 broker_epoch 居然被 leader 拒了
        // 这意味着 meta 已经看到 follower 的新 epoch,但 follower 用了旧值
        // → 重新到 meta 注册取最新 broker_epoch
    }

    Err(OffsetOutOfRange { leader_log_start, leader_leo }) => {
        // leader 返回时同时给出 [leader_log_start, leader_leo],follower 据此区分:
        if local_leo < leader_log_start:
            // (a) follower 太落后,被 retention 抛后
            // → 清空本地 + 从 leader_log_start 开始全量重拉
            replica_log.truncate_to(shard, segment_seq, 0).await?
            leader_epoch_cache.clear()
            fetch_offset = leader_log_start
        else if local_leo > leader_leo:
            // (b) follower 比 leader 还远(脑裂残余,且 epoch 校验已通过)
            // → 走 §9 OffsetsForLeaderEpoch 流程
            // 注意:理论上 (b) 应在 epoch 校验时就被 Fenced 拦下,
            // 此分支是兜底保险
    }

    Err(SegmentSealedUp) => {
        // filesegment 专属:segment 已封口
        // follower 拉完最后一批后 fetcher 退出
        // 新 segment 的 fetcher 由 SegmentLeaderAndIsr 通知触发
    }
}
```

**memory/rocksdb 不会遇到 `SegmentSealedUp`**(segment 永远 = 0,不会封口)。

### 6.4 last_caught_up_ts 的精确语义(避免 flapping)

leader 维护 `follower_progress[node_id]`:

```rust
pub struct FollowerProgress {
    pub leo: u64,                       // 上次 fetch 时 follower 的 LEO
    pub last_fetch_ts: u64,             // 上次 fetch 到达时刻
    pub last_caught_up_ts: u64,         // 上次"追上"时刻,见下
}
```

更新规则(收到 fetch 时):
```text
// P2-3: 关键!leader_leo 必须在 fetch 请求 *进入处理* 的瞬间取值,
//       不能用"当前最新值"。否则高并发写入下 follower 永远追不上。
//       这与 Kafka 一致(Kafka 用请求到达时的 leader leo 作为追上判定基准)
let leader_leo_at_request_arrival = shard.local_leo.load();
follower_progress[req.replica_id].leo = req.fetch_offset;
follower_progress[req.replica_id].last_fetch_ts = now;

if req.fetch_offset >= leader_leo_at_request_arrival {
    // follower 已经追上 *本次请求到达时* 的 LEO
    follower_progress[req.replica_id].last_caught_up_ts = now;
}
// 关键:即使没追上,只要还在 fetch,**不更新 last_caught_up_ts** 而非 last_fetch_ts
// 这样 lag 检查看的是"距上次追上多久",不是"距上次 fetch 多久"
// 长 poll 等待中也算"在追",不会误踢
```

ISR shrink 判定(见 §7):
```text
lag_ms = now - last_caught_up_ts
in_isr && lag_ms > replica_lag_time_max_ms → 踢出
```

→ **不会因为 long-poll 阻塞误判**,也**不会因为短时大量写入误判**(只要 follower 能追上某一时刻的 leader_leo)。

### 6.5 Follower fetch 流程图

![Follower fetch 循环](./diagrams/03-fetch-flow.png)

### 6.6 FetchRequest / FetchResponse

```protobuf
message StorageEngineFetchRequest {
  string shard_name = 1;
  uint32 segment_seq = 2;
  uint64 fetch_offset = 3;            // follower 期待的下一个 offset(隐式上报 LEO,I16)
  uint32 current_leader_epoch = 4;    // follower 当前认知的 leader epoch
  uint64 replica_id = 5;              // follower 的 node_id
  uint64 replica_broker_epoch = 6;    // follower 进程的 broker_epoch(防 zombie follower)
  uint64 max_bytes = 7;
  uint64 min_bytes = 8;               // long-poll: 累积多少字节即返回
  uint64 max_wait_ms = 9;             // long-poll: 最大等待
  // K1 / §18.4 扩展点:Fetch Session(KIP-227 incremental fetch)
  // 当前实现:session_id=0 / session_epoch=0 表示 full request
  int32  session_id = 10;             // 默认 0
  int32  session_epoch = 11;          // 默认 0
}

message StorageEngineFetchResponse {
  bytes records = 1;                  // 编码后的 StorageRecord 列表(可能含 partition_leader_epoch header)
  uint64 leader_hw = 2;               // leader 当前 HW(follower 用 min(local_leo+N, leader_hw) 更新本地 hw)
  uint64 leader_log_start = 3;        // 该 segment 在 leader 上的可读起点(retention 后)
  uint64 leader_leo = 4;              // leader 当前 LEO(用于 follower 判断是否真追上,见 §6.4)
  uint32 leader_epoch = 5;            // 当前 leader epoch,follower 用于校对本地缓存
  uint32 error_code = 6;
}
```

**协议要点**:
- request 携带 `replica_broker_epoch`,leader 据此拒绝同一 broker 旧进程的残留 fetch
- response 同时返回 `leader_log_start` 和 `leader_leo`,follower 处理 `OffsetOutOfRange` 时据此区分原因(§6.3)
- response 携带 `leader_epoch`,follower 用于侧校对(若与本地 cache 不一致 → 触发 metadata 刷新)
- `records` 中可能跨 epoch 边界(filesegment 跨 segment 后 epoch 可能不同,memory/rocksdb 不会),follower 用 batch header 解析后更新 LeaderEpochCache

复用 storage-engine 现有的 `StorageEnginePacket` RPC 通道(参考 `core/read_key.rs`)。

**与 consumer read 路径的边界(K8)**:

| 路径 | 协议 | 调用者 | 数据范围 |
|---|---|---|---|
| Follower fetch(本协议) | `IsrFetchReq/Resp`(新增) | broker 内的 follower fetcher | 含未 committed 数据(LEO 之前) |
| Consumer read | 现有 `ReadReq/Resp` | storage-adapter / 上层 broker | 只到 HW(committed 之前) |

**两条路径完全分离**:
- 协议格式不同(FetchReq 携带 replica_id/broker_epoch/leader_epoch,ReadReq 不带)
- 处理逻辑不同(fetch 推进 HW,read 不推 HW)
- handler 不同(`isr/fetch.rs` 处理 follower fetch,`core/read_*.rs` 处理 consumer read)
- consumer read 路径**不读 follower**(§16 协议规定),只能从 leader 读

**为什么分离而不合并**:Kafka 通过 `replica_id == -1` 区分,本协议从一开始就分开两条 RPC,消费者不背 ISR 协议复杂度,handler 逻辑也更清晰。

---

## 7. ISR 维护

### 7.1 leader 侧判定

segment leader 每秒(可配)扫一次 `follower_progress`,只看自己是 `LeaderActive` 的 segment:

```text
for (node_id, prog) in self.follower_progress {
    let in_isr = self.isr_cache.contains(node_id);

    // ---- shrink 判定 ----
    if in_isr && node_id != self.node_id {
        let lag_ms = now - prog.last_caught_up_ts;
        if lag_ms > replica_lag_time_max_ms {
            meta_call(UpdateSegmentIsr {
                shard_name, segment_seq,
                new_isr: isr - {node_id},
                requester_node_id:     self.node_id,
                requester_broker_epoch: self.broker_epoch,
                leader_epoch:          self.leader_epoch,
                expected_segment_epoch: self.segment_epoch,
            })
        }
    }

    // ---- expand 判定 ----
    if !in_isr && expand_eligible(prog, leader_state) {  // 见 §7.2
        meta_call(UpdateSegmentIsr {
            shard_name, segment_seq,
            new_isr: isr + {node_id},
            requester_node_id:     self.node_id,
            requester_broker_epoch: self.broker_epoch,
            leader_epoch:          self.leader_epoch,
            expected_segment_epoch: self.segment_epoch,
        })
    }
}
```

只有 leader 发起 ISR 变更,meta-service 不主动检测(对齐 Kafka KIP-497 AlterPartition 模型)。

### 7.2 重新加入 ISR 的精确条件(KIP-679 对齐)

不在 ISR 的 follower 要重新加入,leader 端必须看到 (I12):

```text
fn expand_eligible(prog: &FollowerProgress, leader: &State) -> bool {
    // 1) 追上当前 LEO
    //    注意:不是 "leo >= hw"。Kafka KIP-679 修正过:
    //    若用 >= hw,因 hw 滞后 leo,follower 满足时实际可能还没追上 leo,
    //    一旦加入 ISR 立即被算入 HW 计算 → 拉低 HW 或丢未复制数据
    if prog.leo < leader.local_leo { return false; }

    // 2) follower 当前认知的 leader_epoch 与 leader 一致
    //    (来自上次 fetch 请求时的 current_leader_epoch)
    if prog.last_known_leader_epoch != leader.leader_epoch { return false; }

    // 3) broker_epoch 未被 fence
    //    (meta 推送的节点状态里包含每个 node 的存活 broker_epoch)
    if !leader.unfenced_brokers.contains(prog.node_id, prog.broker_epoch) { return false; }

    // 4) 可选 flapping 抑制:持续追上窗口
    if now - prog.first_caught_up_after_oos < replica_lag_time_max_ms / 2 {
        return false;  // 才追上不到半个 lag 窗口,等一等
    }
    true
}
```

> **关于"为什么不是 `leo >= hw`"**:这正是 Kafka 早期 bug(KIP-679 之前)。`hw` 永远 ≤ `leo`,所以 `leo >= leo` 是更严格的条件,自动蕴含 `leo >= hw`。

### 7.3 meta-service raft 路由

`raft/route/engine.rs` 新增 op:

```rust
pub enum EngineDataType {
    // 已有...
    UpdateSegmentIsr,
    // payload: {
    //   shard_name, segment_seq,
    //   new_isr: Vec<u64>,
    //   requester_node_id: u64,        // 发起请求的 leader broker
    //   requester_broker_epoch: u64,   // 该 broker 的当前进程 epoch
    //   leader_epoch: u32,             // 发起时 leader 自报
    //   expected_segment_epoch: u32,   // CAS 语义
    // }
}
```

**raft 状态机校验逻辑(I3,三重 fence)**:

```text
let current = state.get_segment(shard, segment_seq);
let known_broker_epoch = state.node_registry.get(req.requester_node_id);

// fence 1: 发起者必须是当前 leader
if req.requester_node_id != current.leader:
    return Err(NotLeaderForPartition)

// fence 2: leader_epoch 匹配(防 zombie leader epoch)
if req.leader_epoch != current.leader_epoch:
    return Err(FencedLeaderEpoch)

// fence 3: broker_epoch 匹配(防 zombie leader 进程实例)
if req.requester_broker_epoch != known_broker_epoch:
    return Err(StaleBrokerEpoch)

// fence 4: segment_epoch CAS(防并发 ISR 变更覆盖)
if req.expected_segment_epoch != current.segment_epoch:
    return Err(InvalidUpdateVersion)

// 全部通过,应用变更
current.isr = req.new_isr
current.segment_epoch += 1
state.write(current)

// 触发广播 SegmentLeaderAndIsr (§7.4)
```

**三类 fence 的职责区分**:
- `leader_epoch`:防同一 node_id 不同 leader 任期的请求(任期间崩溃重启)
- `broker_epoch`:防同一 node_id 同一 leader 任期但不同进程实例的请求(进程崩溃极快重启)
- `segment_epoch`:防同一 leader 同一进程同一任期内,并发的两个 ISR 变更请求互相覆盖

### 7.4 SegmentLeaderAndIsr 广播

meta-service 已有 `core/notify.rs::send_notify_by_segment_*` 系列调用。新增 ISR 变更通知(leader 切换通知复用现有路径):

```rust
pub async fn send_notify_by_segment_isr_change(
    call_manager: &Arc<...>,
    segment: EngineSegment,
) -> Result<(), MetaServiceError>;
```

各 broker 收到通知后:
1. 校验 `segment_epoch > local_segment_epoch`,否则丢弃通知(可能乱序到达)
2. 更新本地 `SegmentReplicaState.isr_cache / leader / leader_epoch / segment_epoch`
3. 若 leader 变更:执行 §9 truncation 流程

---

## 8. Leader 切换

### 8.0 现有实现重写说明(D3)

**代码现状有严重缺陷**(`core/leader_switch.rs::segment_leader_switch`):

```rust
// 现有错误实现(摘自代码):
let new_leader = segment.replicas
    .iter()
    .find(|rep| rep.node_id != remove_id)  // ← 从 replicas 选,不是 isr!
    .map(|rep| rep.node_id)
```

这是 **unclean leader election**:从 `replicas`(可能含已被踢出 ISR 的滞后副本)选新 leader,且不更新 ISR、不 bump segment_epoch。**会丢已 committed 数据,违反 I8 和 I14**。

**本协议要求重写为**:

```text
segment_leader_switch(failed_node_id):
  for segment in segments where leader == failed_node_id:
    // 1. 候选必须来自 ISR(I14:不做 unclean leader election)
    let candidates = segment.isr.iter().filter(|id| *id != failed_node_id);
    let new_leader = candidates.next();  // ISR 顺序优先(后续可加策略)

    match new_leader {
      Some(new_leader) => {
        let mut new_segment = segment.clone();
        new_segment.leader = new_leader;
        new_segment.leader_epoch += 1;
        new_segment.segment_epoch += 1;
        new_segment.leader_broker_epoch = node_registry[new_leader];  // 新 leader 当前的 broker_epoch
        new_segment.isr.retain(|id| *id != failed_node_id);  // 从 ISR 移除故障节点
        // 不变:replicas 保持不变(故障节点恢复后可重新追回 ISR)

        sync_save_segment_info(raft_manager, &new_segment).await?;
        send_notify_by_set_segment(call_manager, new_segment).await?;
      }
      None => {
        // ISR 空(去掉故障节点后),拒绝选 leader(I14)
        let mut new_segment = segment.clone();
        new_segment.status = SegmentStatus::Unavailable;
        new_segment.segment_epoch += 1;
        // 不动 leader / leader_epoch,等运维介入或等 ISR 恢复

        sync_save_segment_info(raft_manager, &new_segment).await?;
        send_notify_by_set_segment(call_manager, new_segment).await?;
        log::warn!("segment {}/{} ISR empty, marked Unavailable",
                   segment.shard_name, segment.segment_seq);
      }
    }
```

**`SegmentStatus` 需要新增 `Unavailable` 状态**:

```rust
pub enum SegmentStatus {
    Write,
    PreSealUp,
    SealUp,
    PreDelete,
    Deleting,
    Unavailable,  // === 新增 === ISR 为空,无法选 leader,等运维或 ISR 恢复
}
```

- `Unavailable` segment 不接受写入(返回 `SegmentUnavailable`)
- 不接受 fetch(follower 收到 SegmentUnavailable 后退避等下次 LeaderAndIsr)
- 但仍**允许读历史数据**(消费者可读 `< log_start_offset` 之前已 committed 的数据)
- 当原 ISR 节点恢复且本地 LEO 仍是当时的 HW 时,可重新选举(meta-service 通过节点心跳感知到恢复后触发)

### 8.1 broker 接收 SegmentLeaderAndIsr 的状态转换

控制面流程见 §8.0。本节定义**数据面收到通知后的精确状态机**(I11)。

```text
on_receive_leader_and_isr(notification):
    // 校验通知本身
    if notification.segment_epoch <= local.segment_epoch:
        ignore  // 乱序到达,旧通知

    case notification.leader == self.node_id:
        // === 我成为 leader ===
        self.role = LeaderInitializing
        // 拒绝所有当前进行中的 producer 请求(返回 NotReady 让 client 重试)
        cancel_inflight_producer_requests()
        // 停掉 fetcher(如果之前是 follower)
        stop_fetcher_if_any()

        // 关键 (I11):必须先持久化新 epoch 才能接受写入
        let current_leo = replica_log.latest_offset(shard, segment_seq)?
        leader_epoch_cache.assign(notification.leader_epoch, current_leo)
        leader_epoch_cache.fsync()                  // 必须 fsync!

        // 准备 leader 端状态
        self.leader_epoch = notification.leader_epoch
        self.segment_epoch = notification.segment_epoch
        self.isr_cache = notification.isr
        // K7: 新 leader **不能** 把 shard.local_hw 设为 shard.local_leo!
        //     原因:之前 ISR 中可能有 follower LEO < leader LEO,这部分数据未 commit,
        //          把 LEO 当 HW 等于错误承诺 commit 了未复制的数据,违反 I8。
        //     正确做法:保持 checkpoint 中的旧 HW(本地 follower 期就维护好的),
        //              等 follower 重新追上后通过 fetch 推进。
        //     这是 shard 维度的 HW(I5/I6),已由 commitlog checkpoint 持久化。
        shard.local_hw = max(shard.local_hw, persisted_hw_from_checkpoint)
        reset_follower_progress()
        // (此时尚未有 follower fetch 过来,follower_progress 为空)

        // [hook §18.1] rebuild_producer_state(epoch_cache, replica_log)
        // 默认 no-op;未来 idempotent producer 实现时在此重建 PID state

        // 转 Active,开始接受写入和 fetch
        self.role = LeaderActive

    case notification.leader != self.node_id and self.node_id in notification.replicas:
        // === 我成为(继续)follower,可能换了 leader ===
        self.role = FollowerInitializing
        // 停掉旧 fetcher(可能在追旧 leader)
        stop_fetcher_if_any()
        // 取消等待中的 acks=all 请求(如果上一刻还是 leader)
        cancel_inflight_producer_requests()

        self.leader_epoch = notification.leader_epoch
        self.segment_epoch = notification.segment_epoch
        self.isr_cache = notification.isr

        // 关键 (I9):必须先走 OffsetsForLeaderEpoch truncation 才能 fetch
        let (end_offset_leader_epoch, end_offset) = offsets_for_leader_epoch_query(
            target_leader = notification.leader,
            follower_leader_epoch = leader_epoch_cache.latest_epoch(),
            current_leader_epoch = notification.leader_epoch,
        )?
        // K4:用 leader 返回的 end_offset_leader_epoch(可能 < follower 请求的 epoch)
        //     来修剪本地 LeaderEpochCache,而不是用 follower 自己的 epoch。
        //     truncate_point 用 min(local_leo, end_offset)。
        let truncate_point = min(shard.local_leo, end_offset);
        replica_log.truncate_to(shard, segment_seq, truncate_point)
        // 修剪本地 LeaderEpochCache:删 epoch > end_offset_leader_epoch 的所有 entries
        leader_epoch_cache.truncate_from_end_by_epoch(end_offset_leader_epoch, end_offset)
        leader_epoch_cache.fsync()

        // 启动 fetcher
        start_fetcher(target = notification.leader, fetch_offset = truncate_point)
        self.role = FollowerActive

    case self.node_id not in notification.replicas:
        // === 我被从 replicas 移除 ===
        stop_fetcher_if_any()
        cancel_inflight_producer_requests()
        // 可保留本地数据用于运维诊断,但不再参与协议
        unregister_replica_state(shard, segment_seq)
```

**关键 invariant 体现**:
- 不变式 I11 在 case 1 的 `leader_epoch_cache.fsync()` 之前 role 一直是 `LeaderInitializing`,期间所有写入返回 `NotReady`
- 不变式 I9 在 case 2 的 truncation 完成前 fetcher 不启动
- I3 的 broker_epoch 校验在 meta 端做(§7.3),数据面 broker 只用本地缓存的 epoch 校验自己发出的请求

### 8.2 Leader 切换时序

![Leader 切换时序](./diagrams/04-leader-switch.png)

---

## 9. Log Truncation（KIP-101，协议强制路径）

> ⚠️ **协议禁止用"truncate 到本地 HW"**。这是 Kafka 2017 年前的设计缺陷,会丢已 committed 数据(见 §13.1)。  
> 唯一允许的 truncate 路径:**`OffsetsForLeaderEpoch`** RPC + 持久化 `LeaderEpochCache`。

### 9.1 触发时机

follower 必须在以下两种情况执行 truncation,**在执行完之前不许 fetch**:

1. **本地启动**:进程拉起后,在第一次 fetch 前
2. **leader 切换**:收到 `SegmentLeaderAndIsr` 发现 leader 变了
3. **遇到 `FencedLeaderEpoch` 错误**:本地 epoch 比 leader 旧

### 9.2 协议

follower 维护本地 `LeaderEpochCache`(已持久化,见 §3.2),`local_leo` 及本地 `latest_epoch`。

```text
1. follower 拿本地最后一个 epoch 条目 (latest_epoch, _)
   (空 cache 表示无历史 → 走 §9.5 特殊路径,例如 memory follower 重启)

2. follower → leader: OffsetsForLeaderEpoch {
       shard, segment_seq,
       follower_leader_epoch: latest_epoch,
       current_leader_epoch:  follower 当前认知的 leader_epoch,  // 防陈旧 leader
       replica_id, replica_broker_epoch,
   }

3. leader 收到后(精确语义,对齐 KIP-101):
   a. 校验 current_leader_epoch == leader.epoch:
      <  → FencedLeaderEpoch  { current_leader_epoch: leader.epoch }
      >  → UnknownLeaderEpoch (让 follower 等 meta 同步)

   b. 查自己的 LeaderEpochCache:
      情况 1: follower_leader_epoch > leader.cache.latest_epoch
         → 返回 { end_offset_leader_epoch: -1, end_offset: -1, error: UnknownLeaderEpoch }
         (follower 在 leader 看来的"未来" epoch,可能 leader 也没拉到最新 meta)

      情况 2: follower_leader_epoch < leader.cache.earliest_epoch
         → 返回 { end_offset_leader_epoch: -1, end_offset: leader.log_start_offset, error: 0 }
         (follower 历史太旧,被 retention,需要从 leader_log_start 全量重拉)

      情况 3: follower_leader_epoch 恰好是 leader.cache.latest_epoch
         → 返回 { end_offset_leader_epoch: follower_leader_epoch,
                  end_offset: leader.local_leo, error: 0 }

      情况 4: leader.cache 中存在 next_epoch = follower_leader_epoch 的下一个 epoch
         → end_offset = next_epoch.start_offset
         → 返回 { end_offset_leader_epoch: follower_leader_epoch, end_offset, error: 0 }

      情况 5: leader.cache 中没有 follower_leader_epoch 本身(可能因 cache 修剪),
              但 follower_leader_epoch < latest_epoch 且 >= earliest_epoch
         → 返回 leader.cache 中 < follower_leader_epoch 的最大 epoch 的 end_offset
         → end_offset_leader_epoch = 那个较小的 epoch(而非 follower 请求的)
         (这是 Kafka 的精确处理:让 follower 知道 leader 上对应的实际 epoch)

4. follower 收到 response 后:
   if error == UnknownLeaderEpoch:
       退避等待 metadata 刷新,不 truncate
   else:
       // 使用 response.end_offset_leader_epoch(可能与 follower 请求的不同!)和 end_offset
       truncate_point = min(local_leo, end_offset)
       replica_log.truncate_to(shard, segment_seq, truncate_point)
       // 修剪 LeaderEpochCache:删 epoch > end_offset_leader_epoch 的所有条目
       leader_epoch_cache.truncate_from_end_by_epoch(end_offset_leader_epoch, end_offset)
       leader_epoch_cache.fsync()

5. 然后才能开始 fetch,fetch_offset = truncate_point
```

> **细节**:第 5 种情况是 Kafka 的精确语义。response 的 `end_offset_leader_epoch` 不一定等于 request 的 `follower_leader_epoch`——它是 leader 实际找到的最近匹配 epoch。follower 据此修剪本地 cache 时也按这个 epoch 切。

**举例**(完全对齐 Kafka KIP-101 文档示例):

```text
事件序列:
- t0:epoch=1,records offsets [0,1,2,3,4],leader=A,follower=B
  A,B 本地都有 [0..5),epoch_cache: [(1,0)]
- t1:A crash,B 选为新 leader,epoch=2
  B 写入 records [5,6],epoch=2
  B.epoch_cache: [(1,0),(2,5)]
- t2:A 恢复,但 A 不知道发生了什么,A.local_leo=5, A.epoch_cache=[(1,0)]

A 作为 follower 重新加入:
- A 发 OffsetsForLeaderEpoch { follower_leader_epoch=1 }
- B 查到 epoch=1 的 end_offset = (epoch=2 的 start_offset) = 5
- A truncate_to(5)(其实没动,因为 A.local_leo 本来就是 5)
- A fetch_offset=5,正常追上 B 的 [5,6]
```

```text
另一例:
- A 在 t1 之前还多写了一条 epoch=1 的本地日志(没复制给 B):
  A.local = [0,1,2,3,4,5],epoch_cache=[(1,0)],local_leo=6
- B 没收到这条,B.local_leo=5(t1 时)
- B 当选 leader,写 epoch=2 的 [5,6]
- A 恢复后查 OffsetsForLeaderEpoch:
  B 返回 epoch=1 的 end_offset=5
- A truncate_to(5)→ 丢掉本地 offset=5 那条**未 committed** 数据(✓ 正确)
- A fetch_offset=5,拿到 B 写的 [5,6]
```

→ **A 主动丢弃了一条永远不会 committed 的脏数据**。如果改用"truncate 到本地 HW=5"(同一结果),没问题;**但若 A 在 commit 之前 HW 推到了 7**(场景:f1,f2 都已 ack epoch=2 数据,但 A 还没收到 HW 更新),用本地 HW truncate 就会**保留 offset=5 的脏数据**,跟 B 的 offset=5 内容不同——分歧出现。这就是 KIP-101 解决的问题。

### 9.3 OffsetsForLeaderEpoch RPC

```protobuf
message OffsetsForLeaderEpochRequest {
  string shard_name = 1;
  uint32 segment_seq = 2;
  uint32 follower_leader_epoch = 3;     // follower 本地最后一个 epoch
  uint32 current_leader_epoch = 4;      // follower 认知的 leader epoch
  uint64 replica_id = 5;
  uint64 replica_broker_epoch = 6;
}

message OffsetsForLeaderEpochResponse {
  // leader 实际找到的 epoch(可能与 request.follower_leader_epoch 不同!
  // 例如 follower 请求 epoch=3 但 leader 只有 epoch=5,会返回 5 之前的某个)
  // -1 表示 UnknownLeaderEpoch
  int32 end_offset_leader_epoch = 1;

  // 该 epoch 在 leader 上的 end_offset
  // (= 下一个 epoch 的 start_offset,或 leader_leo,或 leader_log_start)
  uint64 end_offset = 2;

  uint32 error_code = 3;                // FencedLeaderEpoch / UnknownLeaderEpoch / 0
}
```

### 9.4 三引擎差异

| 引擎 | LeaderEpochCache 持久化 | truncate_to 实现 | 备注 |
|---|---|---|---|
| memory | 不持久化(进程重启即丢) | 删 `>offset` 的 DashMap entries | **memory follower 重启等价于全新副本**,必须从 leader_log_start 全量重拉。这是 memory 引擎的固有特性,不算 ISR 缺陷。 |
| rocksdb | 单独 key 前缀 `/leader_epoch/{shard}/{segment_seq}/` | range delete 前缀 `(shard,segment_seq,offset+1..)` | 持久化等同 Kafka |
| filesegment | sidecar 文件 `*.leader-epoch-checkpoint`(同 Kafka) | 截断文件尾或删尾部 segment | 完全等同 Kafka |

### 9.5 memory 引擎的特殊处理

memory 引擎 follower 重启后,本地 LeaderEpochCache 必然为空。处理:
1. follower 报告 `follower_leader_epoch=0`(表示无历史)
2. leader 返回 `end_offset = leader_log_start`(让 follower 从头拉)
3. follower 不用 truncate(本地本来就是空的)
4. fetch_offset = leader_log_start

→ memory 引擎下 follower 重启 = 全量重拉。这是引擎特性,不是 ISR 协议缺陷。

---

## 10. 节点变更、副本分配与 retention

### 10.1 segment 副本分配

**完全复用现有流程**(`meta-service/core/segment.rs::create_segment_by_shard`):

- shard 创建时同步创建 `segment_seq=0` 的 segment,按 `replica_num` 选节点(memory/rocksdb 之后永远不再切 segment)
- filesegment 写满 `max_segment_size` 时触发 `seal_up_segment`,创建下一个 segment(按当时节点状态重新选副本,可能选到与上一段不同的节点集)
- 新 segment 创建时:
  - `leader_epoch = 0`
  - `segment_epoch = 0`
  - `leader_broker_epoch = node_registry[leader].broker_epoch`
  - `log_start_offset = 0`
  - `isr = replicas`

**节点选择策略**(已存在,本方案不改):按 `node_id` 排序后轮询。

**对 memory/rocksdb 的隐含约束**:因为永远只有 segment 0,其副本拓扑在 shard 创建时即固定,**没法做副本重平衡**。要重平衡 memory/rocksdb 的副本只能整 shard 重建。本方案明确接受这个约束(见 §16)。

### 10.2 log_start_offset 推进(retention)

`log_start_offset` 是该 segment 在 leader 上**可读的最小 offset**。retention 推进它。

**推进时机与责任**:
- 各引擎自身的 retention 机制(memory 的 GC、rocksdb 的 TTL、filesegment 的 segment 删除)
- 引擎在 retention 删数据后,**主动调用** `update_log_start_offset(shard, segment_seq, new_log_start)` 接口
- 该接口:
  1. 更新本地 `SegmentReplicaState.log_start_offset`
  2. 持久化到本地 checkpoint(防进程重启回退)
  3. 异步通过 `UpdateSegmentLogStart` raft op 同步给 meta(非关键路径,delay ok)

**meta-service 端**:
- raft op `UpdateSegmentLogStart` 校验:`req.requester_node_id == current.leader`、`req.leader_epoch == current.leader_epoch`、`req.new_log_start > current.log_start_offset`
- 通过后写 raft,**不**广播到 follower(follower 自然通过下一次 fetch 收到 `OffsetOutOfRange` + `leader_log_start` 得知)

**follower 端行为**:
- follower 不主动推进自己的 log_start_offset
- 当 fetch 拿到 `OffsetOutOfRange { leader_log_start }` 且 `local_leo < leader_log_start`,清空本地从 `leader_log_start` 开始全量重拉(§6.3)

**为什么 log_start_offset 推进不需要严格通知 follower**:
- follower 自己的 retention 也按自己的节奏走(对齐 Kafka)
- leader 与 follower 的 log_start_offset 短期可以不一致,只要 leader 的更激进(更大)即可
- follower 上有 leader 没有的旧数据 → 下次 follower fetch 时,leader 不会知道 follower 有旧数据,follower 自己用本地 retention 清

---

## 11. RPC / 模块分布

### 11.1 协议层(protocol crate)新增 packet 类型(M6)

`src/protocol/src/storage/codec.rs::StorageEnginePacket` **必须新增**:

```rust
pub enum StorageEnginePacket {
    // 现有
    WriteReq(WriteReq),
    WriteResp(WriteResp),
    ReadReq(ReadReq),
    ReadResp(ReadResp),

    // === ISR 新增 ===
    IsrFetchReq(IsrFetchReq),
    IsrFetchResp(IsrFetchResp),
    OffsetsForLeaderEpochReq(OffsetsForLeaderEpochReq),
    OffsetsForLeaderEpochResp(OffsetsForLeaderEpochResp),
}
```

注:`AlterPartition` 不走 storage-engine 通道,走 broker → meta-service 的 grpc(对应 §7.3 的 raft op `UpdateSegmentIsr`)。

### 11.2 meta-service 端(M8)broker → meta 的请求路径

**不是直接写 raft**(broker 没有 raft handle)。流程:

```
broker leader 想做 ISR shrink/expand:
  1. broker 通过 grpc-clients 调 meta-service 的 grpc API(新增 AlterPartition RPC)
  2. meta-service grpc handler 接收
  3. handler 内部走 meta-service 的 raft op `UpdateSegmentIsr`
  4. raft 状态机校验 + 应用 (见 §7.3)
  5. 触发 SegmentLeaderAndIsr 广播 (见 §7.4)
```

对应的 grpc 定义新增:

```protobuf
// src/protocol/src/meta/storage.proto(已有此文件,扩展)
rpc AlterPartition(AlterPartitionRequest) returns (AlterPartitionReply);

message AlterPartitionRequest {
  string shard_name = 1;
  uint32 segment_seq = 2;
  repeated uint64 new_isr = 3;
  uint64 requester_node_id = 4;
  uint64 requester_broker_epoch = 5;
  uint32 leader_epoch = 6;
  uint32 expected_segment_epoch = 7;
}

message AlterPartitionReply {
  uint32 error_code = 1;     // 0 / FencedLeaderEpoch / StaleBrokerEpoch /
                             // InvalidUpdateVersion / NotLeaderForPartition
  uint32 new_segment_epoch = 2;  // 成功时返回新值,失败时为当前值
}
```

### 11.3 模块布局

```
src/protocol/src/storage/codec.rs              // 扩展 StorageEnginePacket(M6)
src/protocol/src/meta/storage.proto            // 新增 AlterPartition RPC(M8)
src/protocol/src/meta/common.proto             // RegisterNodeReply 加 broker_epoch(D5)

src/storage-engine/src/isr/
├── mod.rs            // pub mod log; pub mod state; pub mod fetch; pub mod manager; ...
├── log.rs            // ReplicaLog trait + 三个 impl 的封装
├── state.rs          // ReplicaStateRegistry / ShardReplicaState / SegmentReplicaState
├── leader_epoch.rs   // LeaderEpochCache + 持久化(rocksdb 持久层)
├── fetch.rs          // follower fetcher + leader fetch handler
├── offsets_for_leader_epoch.rs  // KIP-101 truncation handler + client
├── append.rs         // leader 写入路径插桩、acks=all 等待
└── manager.rs        // ISR 维护后台 + segment leader change 响应

src/storage-engine/src/commitlog/memory/
└── impl ReplicaLog for MemoryStorageEngine    // segment_seq 参数忽略(恒为 0)

src/storage-engine/src/commitlog/rocksdb/
└── impl ReplicaLog for RocksDBStorageEngine   // key 编码加 segment_seq 段

src/storage-engine/src/filesegment/
└── impl ReplicaLog for FileSegment            // segment_seq 直接对应文件

src/storage-engine/src/server/                 // 现有 RPC 入口
└── inner.rs:dispatch                          // 新增 IsrFetch / OffsetsForLeaderEpoch handler

src/meta-service/src/raft/route/engine.rs      // 新增 UpdateSegmentIsr op
src/meta-service/src/core/cluster.rs           // register_node 返回 broker_epoch(D5)
src/meta-service/src/core/leader_switch.rs     // **重写** segment_leader_switch(D3)
src/meta-service/src/core/notify.rs            // 复用 send_notify_by_set_segment(已存在)
src/meta-service/src/server/                   // 新增 AlterPartition grpc handler
```

### 11.4 需要重写的现有代码

| 文件 | 重写原因 |
|---|---|
| `src/meta-service/src/core/leader_switch.rs::segment_leader_switch` | D3:当前从 replicas 选 leader 是 unclean,必须改为从 ISR 选 |
| `src/common/metadata-struct/src/storage/segment.rs::allow_read` | D4:当前只允许 Write,必须扩到 Write/PreSealUp/SealUp/Unavailable |
| `src/storage-engine/src/core/segment.rs::segment_validator` | D4:跟随 allow_read 修正 |
| `src/common/metadata-struct/src/storage/segment.rs::SegmentStatus` | D3:新增 Unavailable |
| `src/meta-service/src/core/cluster.rs::register_node_by_req` | D5:返回 broker_epoch |

### 11.5 不改动

- `EngineShard` 结构(ISR 元数据全在 segment 上)
- `meta-service/core/segment.rs::create_segment_by_shard`(副本分配策略保留,但初始化时增加 segment_epoch=0/leader_broker_epoch 字段)
- `storage-engine/src/core/write.rs::batch_write` 的转发路由(D2 保留)

---

## 12. 异常场景详解

每个场景给出**触发条件 / 风险 / 本方案的避免机制**三段式。

### 12.1 旧 Leader 复活产生 zombie 写入

**触发**:
- B1 是 segment leader,网络抖动导致 meta 判其 dead,选 B2 为新 leader,epoch E+1
- B1 网络恢复,自己尚未感知到下线,producer 仍连着 B1 写

**风险**:
- B1 接受写入,本地数据进展超过 B2,日志分歧
- 客户端收到 ack 但数据后续被 truncate(丢确认数据)

**避免**:
1. **每次写入校验 leader_epoch**:写入路径(§5)开头必须查本地缓存的 epoch,不等则 `FencedLeaderEpoch` 拒写。本地缓存由 meta-service `SegmentLeaderAndIsr` 推送更新(秒级);**推送到达前的窗口内可能错写**,见 #2 / #3 兜底。
2. **acks=all 等的是 HW 推进**:HW 推进必须经过 ISR 中其他 follower(B2/B3),B1 即使本地写成功也推不动 HW(因为 B2/B3 已经切换到 B2 为 leader,不再 fetch B1),producer 会 timeout 重试 → 拉新 metadata → 切到 B2。
3. **B1 在收到下次 fetch 时被发现**:当 B2/B3 之一被切换成 follower 后,B1 不会再有 fetch 进来,leader 的 `follower_progress` 全部 stale。B1 自己也可以基于"长时间没有 fetch"自动降级(可选优化)。

→ 最差情况(可接受):在 producer 拿到新 metadata 之前,B1 用 acks=1 错写了一批数据,这些数据在 truncation 时被 B1 自己丢掉(§9 的 OffsetsForLeaderEpoch)。**acks=all 场景永不丢已 ack 数据**(HW 推不动)。这是 acks=1 用户自选语义的代价。

### 12.2 Follower 截断错误导致已 committed 数据丢失(KIP-101 经典)

**触发**:
- 旧 leader A,follower B,records [0..5),HW=5,epoch=1
- A 又写了 records [5,6](epoch=1),已 ack=1 但未 commit(HW 还是 5)
- A 挂,B 当选,epoch=2,B.local_leo=5,B 开始接受新写入 records [5'](与 A 的 5 不同内容)
- A 重启,A.local_leo=7,A.epoch_cache=[(1,0)]

**风险(若用本地 HW 做 truncate 点)**:
- A 看到本地 HW=5,truncate_to(5) → 丢掉 A 的 offset 5,6
- 这本来正确(它们没 committed)。**但若另一个 follower C 在 A 挂前刚 fetch 到 7,C.local_leo=7,C.local_hw=5**
- C 重启,truncate_to(local_hw=5) → 没丢数据
- C 作为 follower 加入 B,fetch_offset=5 → B 推送 B 的 offset=5 内容
- **但 C.local 在 offset 5,6 是 A 的旧内容,不是 B 的新内容!分歧出现且未被发现**(因为 fetch 不会校验已有数据)

**避免(KIP-101,本方案 §9)**:
- 所有 truncation 必须走 `OffsetsForLeaderEpoch`,不许用本地 HW
- C 启动后查 B:`OffsetsForLeaderEpoch(follower_leader_epoch=1)`,B 答 `end_offset_of_epoch_1 = 5`
- C truncate_to(5),丢掉本地 offset=5,6 的脏数据
- C 重新 fetch 5,6 拿 B 的正确内容

→ 这是为什么 §9 必须用 `OffsetsForLeaderEpoch`(不变式 #7)。

### 12.3 ISR 缩到 0

**触发**:
- 3 副本,min_in_sync_replicas=2
- 网络抖动,follower 1、follower 2 都被踢出 ISR
- ISR = {leader 自己}

**风险**:
- 若仍接 acks=all:实际只有 leader 一份,leader 挂则数据全丢
- 若仍接 acks=1:数据"committed"语义破坏(committed 应当代表 ISR 多数持有)

**避免**:
- acks=all 写入直接拒绝 `NotEnoughReplicas`(§5)
- acks=1 仍接受(用户自选语义,接受单点风险)
- 关键:**leader 不能自我替换 ISR**,即使只剩自己也不能假装"我自己就是 ISR=1"。ISR 收缩必须经 meta-service raft 校验(§7.3)

### 12.4 ISR 收缩与 leader 切换的竞态(KIP-320)

**触发**:
- leader A 发起 `ShrinkSegmentIsr(remove=B)`,raft 处理中
- 同时 meta 判定 A 死了,触发 leader switch,选 B 为新 leader,epoch E+1
- A 的 shrink 请求晚到 raft

**风险(无 segment_epoch 防护)**:
- A 的 shrink 应用,把 B 踢出 ISR
- B 已经是新 leader,但 ISR 中没了 B → 写不动 HW,系统僵死

**避免(本方案 §7.3)**:
- A 的 shrink 请求带 `leader_epoch=E`,raft 校验时发现当前 `leader_epoch=E+1`,直接 `FencedLeaderEpoch` 拒绝
- 再加 `expected_segment_epoch` CAS 防同 leader 内的并发 shrink/expand 互相覆盖

### 12.5 Follower 长时间 GC 暂停被误踢

**触发**:
- follower JVM/runtime 卡顿 5 秒,无法发 fetch
- `replica_lag_time_max_ms = 10s`,临界值
- 恢复后立即追上 LEO

**风险(早期 Kafka)**:
- 用"messages lag"判断:大量写入时 follower 看似落后 → 误踢
- 用"距离上次 fetch 时间":恰好超阈值即误踢,刚恢复又加回 → flapping

**避免(本方案 §6.4)**:
- 用 `last_caught_up_ts`(上次追上 leader_leo 的时刻),不是 `last_fetch_ts`
- 短时大量写入时,只要 follower 能追上"某个时间点的"LEO,就一直被视为在追
- 5 秒 GC 期间没 fetch,但 last_caught_up_ts 还在 10s 窗口内 → 不踢

### 12.6 网络分区(脑裂)

**触发**:
- 3 节点:A(leader)/B/C,A 与 meta-service 分区,B/C 与 meta-service 连通
- meta 判 A 死,选 B 为新 leader,epoch E+1
- A 这边的 producer 仍连 A 写入

**避免(组合多重防御)**:
1. A 写入路径每次校验 epoch(§5):一旦 A 拉到新 meta 就停写
2. A 拉不到新 meta(因为它也分区了):
   - acks=all 路径写不动 HW(B/C 不再 fetch A),producer timeout 重试
   - acks=1 路径短窗内错写,但 A 重新连上后走 §9 truncate 丢弃
3. **关键**:已经被 B 承认 committed 的数据(B 推进 HW 之后)永远不会被 truncate 丢掉,因为 B 是真理源

→ "短窗内 acks=1 错写"是可接受代价。acks=all 严格无丢。

### 12.7 Producer 重试导致重复消息

**触发**:
- producer 写入超时,但 leader 实际已成功
- producer 重试 → 同一条消息写入两次

**避免**:
- 这是 producer 端问题,不在 ISR 范围
- 项目层面可在 producer 端做幂等 producer(类似 Kafka 的 idempotent producer,PID + sequence)
- 不在 ISR 协议范围,见 §16

### 12.8 LeaderEpochCache 损坏 / 丢失

**触发**:
- rocksdb / filesegment 的 LeaderEpochCache 文件损坏
- 或 memory 引擎进程重启

**避免**:
- memory:见 §9.5,follower 启动后从 leader_log_start 全量重拉
- rocksdb/filesegment:启动时校验 cache 合法性,若损坏则**拒绝以 follower 加入**,等运维介入。
  - 不能自动从 0 开始(会和 §12.2 同理产生分歧)
  - 不能自动从本地 HW 开始(同 §12.2)
  - 正确做法:把这个副本视为 lost,从 ISR 移除,从头(leader_log_start)重建

### 12.9 跨 segment 切换时 fetcher 抖动(filesegment 专属)

**触发**:
- 旧 segment N 刚被 SealUp,新 segment N+1 创建
- follower 已收到 SegmentSealedUp,fetcher 退出
- SegmentLeaderAndIsr 通知 N+1 尚未到达,中间窗口 follower 没有 fetcher

**风险**:
- 短窗内 follower 不 fetch,leader 收不到它的 fetch_offset
- leader 端 `last_caught_up_ts` 不更新,达到 lag 阈值会踢出 ISR

**避免**:
- N+1 的通知由 meta-service 在 seal up N 时**原子广播**(seal + new segment + LeaderAndIsr 一个 raft 提案)
- follower 在收到 SegmentSealedUp 后保留旧 fetcher state 直到 N+1 通知到达,避免重复创建
- 调大 `replica_lag_time_max_ms` 给跨 segment 切换留余量
- memory/rocksdb 永远不会遇到此场景(单 segment)

### 12.10 meta-service Raft Leader 切换期间

**触发**:
- meta-service raft leader 切换中,几百毫秒不能处理写
- 此时若有 segment leader 想做 ISR shrink/expand

**避免**:
- meta-service raft 重试由调用方处理(已有机制)
- ISR shrink 不是高频操作,延迟几百毫秒可接受
- segment leader 在 meta 不可用期间继续按"上次已知的 ISR"工作,只是 ISR 状态短窗内不能变化(等效"冻结")

### 12.11 配置变更冲突

**触发**:
- 运维改 `min_in_sync_replicas` 从 1 → 2,正好赶上 ISR=1
- 立即所有 acks=all 写失败

**避免**:
- 配置变更通过 meta-service raft 广播,broker 接收后立即生效
- 设计上接受这个语义:运维责任,改之前确认 ISR 足够
- 文档明确:"修改 min_in_sync_replicas 不会回溯已 ack 的数据,但影响后续写入"

### 12.12 ISR 扩展导致 HW 倒退

**触发**:
- ISR={A,B},LEO_A=100,LEO_B=100,HW=100
- C 追上加入 ISR,ISR={A,B,C},LEO_C=99(刚追到 99 的边界值)
- 若 leader 直接 `new_hw = min(LEO over ISR) = 99` → **HW 倒退**
- 消费者已读到 offset=99 的数据"消失"(因为读 follower 时 follower 收到的 leader_hw 倒退)

**避免(I6)**:
- HW 推进强制单调:`local_hw = max(local_hw, new_hw_candidate)`
- 实际场景:ISR={A,B,C},HW 仍是 100,等 C 也 fetch 到 100 时再正常推进(若有新写入)

### 12.13 新 Leader 上任未完成持久化即崩溃

**触发**:
- meta 选 B2 为新 leader,epoch=E+1
- LeaderAndIsr 通知到达 B2
- B2 尚未 fsync LeaderEpochCache 就崩溃(电源、kill -9)
- B2 重启,本地 LeaderEpochCache 没有 (E+1, X) 条目

**风险(无 LeaderInitializing 状态)**:
- B2 重启后直接接受写入,以 leader_epoch=E+1 写入但本地 cache 显示最高 epoch=E
- 别的 follower 来 `OffsetsForLeaderEpoch` 查询 epoch=E 时 B2 答的 end_offset 是错的(本应是 E+1 的 start_offset,但 B2 不知道有 E+1)
- → 日志分歧

**避免(I11)**:
- LeaderInitializing 状态下拒写,直到 `LeaderEpochCache.assign(E+1, leo) + fsync` 完成
- 崩溃恢复后:本地 LeaderEpochCache 仍是旧状态 → B2 重启后看到自己 leader_epoch=E+1(从 meta 拉)但 cache 最大是 E → 这是冲突信号 → B2 走 fence 路径(拒绝以 leader 提供服务,等 meta 重新 LeaderAndIsr 推送或选别人)

### 12.14 Zombie broker 进程的 ISR 变更

**触发**:
- B1 是 leader,broker_epoch=7
- B1 进程崩溃,操作系统调度太快,B1 新进程注册到 meta 拿到 broker_epoch=8
- B1 旧进程崩溃前发出的 `UpdateSegmentIsr` 请求恰好这时到达 raft

**风险(只有 leader_epoch 校验)**:
- 旧请求带 leader_epoch=E(对的),通过 leader_epoch 校验
- 旧请求覆盖新进程已经发起的某个变更 → ISR 状态错乱

**避免(I3,broker_epoch fence)**:
- 旧请求带 broker_epoch=7,raft 状态机比对 `node_registry[B1].broker_epoch == 8`
- 7 < 8 → `StaleBrokerEpoch` 拒绝
- 新进程后续请求带 broker_epoch=8,正常通过

### 12.15 Producer 持有的 metadata 过时导致写错 leader

**触发**:
- Producer 拉 metadata 时 leader=B1,epoch=E
- 此后 leader 切换:B1→B2,epoch=E+1
- Producer 不知道切换,仍向 B1 写入
- B1 已是 follower

**避免(producer 带 epoch,I4 步骤 3)**:
- Producer 在 ProduceRequest 携带 `current_leader_epoch=E`
- B1 收到请求:`self.role != LeaderActive` 直接 `NotLeaderForPartition`(B1 已经处理过 LeaderAndIsr)
- 即使 B1 没及时处理 LeaderAndIsr(role 还显示 Leader),`req.current_leader_epoch=E < self.leader_epoch=E+1` → `FencedLeaderEpoch`
- Producer 重拉 metadata → 找到 B2

> 若 producer 不带 epoch(简单 client),则只能靠 B1 自己已经感知 leader 切换。B1 处理 LeaderAndIsr 的延迟 = zombie 写入窗口。**带 epoch 是协议推荐做法**。

### 12.16 跨 segment 写入和 leader 切换重叠(filesegment)

**触发**:
- filesegment 引擎,segment N 写满,正在 seal up + 创建 segment N+1
- 同时 N 的 leader 故障,meta 触发 leader switch

**风险**:
- N+1 还没建好就开始选 leader,N+1 的 leader 选择基于 N 时刻的 replicas 还是新 replicas?
- 多个 raft op 并发:seal_up_N、create_N+1、leader_switch_N 会不会乱序?

**避免**:
- meta-service 把"seal up N + 选 N+1 副本 + 创建 N+1 + 广播 LeaderAndIsr"作为**单个 raft proposal** 原子提交
- 此时若同时有 leader switch 请求,raft 串行执行
- segment_epoch CAS 保证 N 的 ISR 变更不会乱序覆盖 N+1 的状态(不同 segment_seq 有独立 epoch)

---

## 13. Kafka 历史教训与本方案对应

Kafka 自 2011 年开始,ISR 协议经过多次重大修正。本方案选择 **2017 年之后 KIP-101 修正后的形态**作为基线,并明确避开以下早期坑。

| Kafka 历史问题 | 解决 KIP | 早期错误做法 | 本方案对应 |
|---|---|---|---|
| Truncate 用本地 HW 导致丢已 committed 数据 | KIP-101 (2017) | `truncate_to(local_hw)` | 协议禁止 HW 路径;唯一路径 `OffsetsForLeaderEpoch`(§9) |
| 日志分歧无法检测 | KIP-101 | 没有 leader epoch,offset 一样以为内容一样 | §3.1 leader_epoch + §3.2 LeaderEpochCache 持久化 |
| Controller 故障切换后用陈旧 ISR 选 leader | KIP-320 | ISR 无版本号 | §7.3 segment_epoch CAS(不变式 I3) |
| Zombie broker 进程发出陈旧元数据请求 | KIP-380 / KIP-497 | 只有 node_id,无法区分同 id 的不同进程实例 | §3.5 broker_epoch + §7.3 broker_epoch fence |
| ISR 扩展条件用 `leo>=hw` 导致刚加入即被算入 HW 计算反而拉低 HW | KIP-679 | `follower.leo >= leader.hw` | §7.2 严格用 `follower.leo >= leader.leo`(不变式 I12) |
| HW 倒退导致消费者已读数据"消失"(ISR 扩展场景) | HW 单调性 | `hw = min(LEO over ISR)`直接赋值 | §5.3 `hw = max(current_hw, new_hw_candidate)`(不变式 I6) |
| 用 `replica.lag.max.messages` 判 follower lag → 大流量误踢 flapping | KIP-237 | 消息数差距阈值 | §6.4 用 `last_caught_up_ts`(时间维度) |
| `LeaderEpochCache` 不持久化,重启丢 → §12.2 攻击面回归 | KIP-101 持久化要求 | 内存缓存 | §3.2 强制持久化(rocksdb/filesegment);memory 引擎特殊处理 §9.5 |
| 新 leader 上任未完成持久化即崩溃,以新 epoch 写但本地 cache 无对应条目 | I11 leader 上任原子性 | 收到 LeaderAndIsr 立即接写 | §3.4 LeaderInitializing 状态 + §8.1 持久化完成才转 Active |
| Unclean leader election 默认开启 → 默默丢数据 | Kafka 2.0 默认 false | 默认开启 | 协议禁用(不变式 I14) |
| ZK 心跳判 broker 死太敏感 | KRaft(KIP-500) | ZK session timeout | meta-service raft 已是类似 KRaft 模型 |
| Fetch 用定时轮询 → CPU 浪费或延迟高 | (Kafka 一直是 long-poll) | - | §6.2 long-poll(不变式 I15) |
| 写入路径不校验 epoch → zombie leader 错写 | Leader Epoch fence | - | §5.2 每次写入校验 epoch(I4) |
| Producer 持有旧 metadata 误写 zombie leader | KIP-320 思想推广 | producer 不带 epoch | §5.1 ProduceRequest 可选携带 `current_leader_epoch`,leader 校验 |
| `min.insync.replicas` 与 acks=all 语义不一致 | 文档/语义澄清 | 早期默认 1 | §3.3 默认 1 但生产环境鼓励 ≥ 2,§5.4 明确语义 |
| 大量 follower 同时 fetch 同一 leader 导致 leader CPU 打满 | KIP-227 (incremental fetch) | 全量元数据每次都传 | 协议外性能优化,不属于 ISR 正确性范围;见 §16 |
| 副本 fetcher 数量过多压垮 follower | KIP-219 (throttling) | 无限制 | 协议固定:单 fetcher / segment;吞吐控制不在 ISR 范围 |

**本方案对 KIP-966 ELR、KIP-405 Tiered Storage、KIP-392 Observer 等较新特性都明确不实现(见 §16)**。但已做的部分(尤其 KIP-101 / KIP-320 / KIP-497 思想 / KIP-679)是 ISR 正确性的最低线,**不可降级,不留妥协路径**。

---

## 14. 实施顺序（工程落地节奏）

> 本节描述的是**工程实施顺序**,不是协议版本演进。**协议是单一稳定版本(§0 不变式),所有 step 都必须满足同一套不变式**;早期 step 没覆盖到的部分由后续 step 补齐,不是"先用弱版本以后再升级"。

| Step | 范围 | 验证手段 |
|---|---|---|
| S1 | 数据模型:`EngineSegment` 加 `segment_epoch / log_start_offset`,`EngineShardConfig` 加 ISR 配置,raft op + CAS 校验 | meta-service 单测覆盖陈旧 epoch 拒绝 |
| S2 | `ReplicaLog` trait + memory/rocksdb 实现 + `LeaderEpochCache` rocksdb 持久化 | 单元测试覆盖 append_at / truncate_to / 重启重建 |
| S3 | 写入路径校验 epoch + long-poll fetch handler + `OffsetsForLeaderEpoch` handler | 单 leader + 2 follower,follower 能追上 |
| S4 | follower fetcher 循环(含 truncation-before-fetch) + `last_caught_up_ts` 维护 | 注入 GC 暂停,验证不被误踢 |
| S5 | acks=all 写入 + HW 推进 + min_in_sync_replicas 拒写 | producer 超时与重试演练 |
| S6 | ISR shrink/expand + `segment_epoch` CAS + SegmentLeaderAndIsr 广播 | 注入 follower lag,观察 ISR 收敛;并发 shrink 验证 CAS |
| S7 | 数据面响应 LeaderAndIsr + 完整 KIP-101 truncation | **故障演练:验证 §12.2 不丢数据**(核心回归用例) |
| S8 | filesegment 接入 `ReplicaLog` + LeaderEpochCache sidecar 文件 + segment seal 时 fetcher 切换 | filesegment 全场景演练 |

> 排序考虑:把 memory/rocksdb 排在 filesegment 前,因为单 segment 退化模型最简单,先在它身上把完整协议跑通(含 KIP-101 truncation);filesegment 接入只是多一个 `ReplicaLog` 实现 + segment 切换路径。
>
> **任何 Step 都不得放宽 §0 不变式**。例如 S3 上线时如果还没做 S6,允许 ISR 始终 = replicas(不收缩),但已经做的部分必须严格走 epoch 路径,不允许临时 HW truncate。

---

## 15. 三引擎一致性视角

从 ISR 控制面看，三个引擎完全一致：

| 维度 | memory | rocksdb | filesegment |
|---|---|---|---|
| `EngineSegment` 元数据 | 同 | 同 | 同 |
| 副本分配流程 | 同(创建 shard 时建 segment 0) | 同 | 同(切 segment 时建新 segment) |
| `segment_leader_switch` | 同 | 同 | 同 |
| ISR shrink/expand | 同 | 同 | 同 |
| FetchRequest 路由 | 同 | 同 | 同 |
| Truncation 流程 | 同 | 同 | 同 |
| `ReplicaLog` trait 调用 | 同 | 同 | 同 |
| **差异:`segment_seq` 取值** | 恒为 0 | 恒为 0 | 写满后递增 |
| **差异:本地存储** | DashMap | RocksDB KV | 文件 |
| **差异:fetcher 数量** | 1 / shard | 1 / shard | 1 / 活跃 segment + 待追平的旧 segment |
| **差异:retention 实现** | 各自现有 GC 机制 | 各自现有 GC 机制 | 删旧 segment 文件 |

**结论**：ISR 模块代码与引擎类型解耦。引擎只需实现 `ReplicaLog` trait 即可接入 ISR。

---

## 16. 不实现的事项（本方案明确划出）

| 项 | 决策 | 备注 |
|---|---|---|
| Reassign replicas | 不做 | segment 创建后副本拓扑固定;memory/rocksdb 由于不切 segment,等价于 shard 创建后拓扑固定。要重平衡 = 整 shard 重建。 |
| Rack awareness | 不做 | 协议外的调度优化,不影响 ISR 正确性。本协议固定为按 node_id 轮询选副本。 |
| Cross-region replication | 不做 | 单集群内 ISR。 |
| Idempotent / Exactly-once producer | 实现不做,接口预留 | producer 重试导致的重复消息由 producer 端方案处理(对应 Kafka PID + sequence)。本协议**在接口层预留扩展点**:LeaderAndIsr 响应路径(§8.1)、写入路径(§5.2)、follower fetch(§6.2)都暴露 hook,以便未来加 idempotent producer 时**不破坏协议结构**,仅需补 hook 实现。详见 §18。 |
| Quota / throttling | 不做 | 流控不在 ISR 协议范围。 |
| Observer replicas (KIP-392) | 不做 | 协议固定:所有 replicas 都是 ISR 候选,无静默副本概念。 |
| Tiered Storage (KIP-405) | 不做 | 不在本方案范围。 |
| ELR (KIP-966 Eligible Leader Replicas) | 不做 | Kafka 较新特性,ISR 协议本身不依赖。 |
| Unclean leader election | 不做 | ISR 空时拒写,标记 Unavailable。`unclean_leader_election_enable` 配置项保留但硬编 false。 |
| Incremental Fetch (KIP-227) | 不做 | 协议固定:每次 fetch 是 full request,leader 不缓存 fetch session。 |
| Consumer 从 follower 读 (KIP-392) | 不做 | 协议固定:consumer 只读 leader,follower 不暴露读接口。简化 ISR 边界。 |
| 旧版本 fetch 协议兼容 | 不做 | **本项目是从头实现,FetchRequest 协议从一开始就携带 `current_leader_epoch`/`min_bytes`/`max_wait_ms`**。不需要 Kafka 那种 v0/v1/v2... 多版本兼容路径。 |
| 旧版本 LeaderEpochCache 兼容 | 不做 | **从第一行代码就持久化**。不存在"老 segment 没有 epoch cache"的情形,不需要回退逻辑。 |

---

## 17. 与 Kafka 协议的差异(刻意为之)

本方案 90% 对齐 Kafka,但有几处刻意简化/改进:

| 差异 | 本方案 | Kafka | 原因 |
|---|---|---|---|
| 副本单元 | segment 维度(`EngineSegment`) | partition 维度(`Log`) | segment 自然边界 + 已有 segment 副本元数据 |
| 三类 epoch 显式拆分 | `leader_epoch` / `segment_epoch` / `broker_epoch` 一开始就明确 | partition_epoch / broker_epoch 在 KRaft 之后才显式化(KIP-380/497) | 从一开始就语义干净 |
| `leader_broker_epoch` 存在 segment 上 | 显式存,方便 fence | Kafka 用 broker registration 隐式关联 | 简化 fence 校验逻辑 |
| ISR 变更 RPC | broker → meta raft op `UpdateSegmentIsr` | KIP-497 broker → controller `AlterPartition` RPC | 模型一致,RPC 复用 meta-service raft |
| RPC 通道 | 复用 storage-engine 现有 `StorageEnginePacket` | 单独的 Replication 协议 | 减少协议碎片 |
| Controller | meta-service raft(已有) | KRaft / ZK | 用现有基础设施 |
| Fetch session | 无,每次 full request | KIP-227 incremental | 协议外性能优化 |
| Consumer fetch 路径 | 与 follower fetch 完全分离 | 同一 RPC 通过 `replica_id=-1` 区分 | 协议干净,消费者不带 ISR 复杂度 |
| 协议版本协商 | 无 | API version negotiation | 单一版本,简化客户端 |
| Producer 携带 leader_epoch | 可选,但鼓励 | 推荐(KIP-320 思想) | 与 Kafka 行为一致 |
| 跨 segment 切换 fetcher | filesegment 引擎独有,memory/rocksdb 永远不切 | Kafka segment 切换在同一 Log 内,不影响副本协议 | 引擎特性差异 |

**项目是绿地实现,不背 Kafka 历史包袱**(早期 fetch v0 没 epoch、ZK 协议、unclean leader 默认开等)。所有特性要么按 KIP-101+ 标准做,要么明确不做。**不存在中间状态**(例如"现在用本地 HW truncate,以后换成 epoch")。

---

## 18. 扩展点(接口预留,实现可后做)

本协议核心闭环(§0 16 条不变式)是稳定的、不留妥协路径。但有些**协议外的高级特性**未来可能要加,本节列出**接口层必须预留的扩展点**,以避免届时回头改协议核心。

**预留原则**:
- 接口签名 / 数据结构里预留字段(可选 / Option 类型),实现可暂时不填
- 关键路径上预留 hook 调用点,默认 no-op
- **不影响当前协议正确性**,但加扩展时只需补 hook 实现,不破坏 §0 不变式

### 18.1 Idempotent Producer(PID + Sequence,对应 Kafka KIP-98)

未来要做 idempotent producer 时,需要 leader 维护 `(PID, segment_seq) → ProducerStateEntry` 状态:

```rust
pub struct ProducerStateEntry {
    pub producer_id: u64,           // PID
    pub producer_epoch: u32,        // 区分同一 PID 的不同 producer 实例
    pub last_sequence: i32,         // 最后一个写入的 sequence
    pub last_offset: u64,
    pub last_timestamp: u64,
}
```

**接口层预留**(本协议必须先把这些 hook 暴露,实现可空):

| 位置 | 预留 hook | 默认行为 | 未来实现 |
|---|---|---|---|
| `ProduceRequest`(§5.1) | 可选字段 `producer_id / producer_epoch / base_sequence` | leader 收到忽略 | leader 校验 sequence 单调,重复请求返回上次的 offset |
| 写入路径(§5.2) | hook `on_append_with_pid(pid_state)` | no-op | 更新 `(PID, segment) → ProducerStateEntry` |
| LeaderEpochCache append 时 | 同时 snapshot `ProducerState` 到本地 | no-op | snapshot 落盘到 `<segment>.producer-snapshot` 文件 |
| Leader 上任(§8.1 case 1) | hook `rebuild_producer_state(epoch_cache, replica_log)` | no-op | 从最近的 snapshot 重放最近的 records,重建内存 `ProducerStateEntry` |
| Follower fetch handler | response 可携带 `producer_state_snapshot` | 不携带 | follower 收到后同步本地 snapshot |

**为什么必须接口预留而不是直接加字段**:
- 如果协议不预留,未来加 idempotent 时需要改 ProduceRequest / FetchResponse / LeaderAndIsr 通知三个 RPC + 增加新的 snapshot 文件格式 + 改 Leader 上任逻辑 → 触及核心路径
- 预留后:只需补 hook 实现,核心路径不变

### 18.2 Transactional Messaging(对应 Kafka KIP-98 transaction)

依赖 18.1,基于 PID 增加 transaction marker 和 transactional state coordinator。本协议**只预留 records 携带 control batch 的能力**,不预留更多。

### 18.3 Consumer 从 follower 读(KIP-392)

需要 consumer 端 fetch 也走类似 follower fetch 的协议,但只读 HW 以下数据。**预留**:

| 预留 | 当前 |
|---|---|
| FetchRequest 增加 `replica_id == -1`(consumer)语义 | 当前 follower fetch 用正 node_id,-1 保留 |
| follower 暴露 fetch 接口供 consumer 调用 | 当前 follower 不暴露,但接口签名兼容 |

### 18.4 Incremental Fetch Session(KIP-227)

协议外性能优化,**协议层预留 fetch session id 字段(默认 0 = full request)**,未来加增量协议时不破坏现有 client。

### 18.5 Tiered Storage(KIP-405)

需要扩展 `EngineSegment.status` 和 `log_start_offset` 的语义。**预留**:`SegmentStatus` 已是 enum,加新状态不破坏现有持久化格式。

### 18.6 Observer Replicas(KIP-392 Observer)

非 ISR 候选的纯观察者副本。**预留**:`EngineSegment.replicas` 加 `is_observer: bool` 字段(默认 false),`isr / replicas` 选举逻辑跳过 observer。

### 18.7 ELR(Eligible Leader Replicas,KIP-966)

ISR 空时的可选 leader 候选。**预留**:`EngineSegment` 加 `eligible_leader_replicas: Vec<u64>` 字段(默认空),meta 选 leader 时优先 ISR,空时再考虑 ELR(若实现)。

### 18.8 Producer 携带 `current_leader_epoch`

**已在 §5.1 预留**(`optional current_leader_epoch`),producer 不带时 leader 跳过该校验。这与 Kafka KIP-320 一致。

---

### 扩展点的协议约束

**任何扩展实现都必须遵守**:
- 不破坏 §0 任一不变式
- 不引入新的 truncation 路径(必须经 KIP-101)
- 不绕过 broker_epoch / leader_epoch / segment_epoch 任一 fence
- 与现有客户端版本兼容(可选字段,旧 client 默认行为不变)

**扩展点不是协议组成部分**,它们是"如果以后做了 X,在哪里挂 hook 不破坏现有协议"的指导。当前实现完全可以跳过 §18 的全部内容,§0~§17 描述的闭环已经是完整可用的 ISR 协议。
