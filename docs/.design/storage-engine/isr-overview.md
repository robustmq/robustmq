# Storage Engine ISR — 实现指南

> 本文写给**第一次接手实现这个协议的工程师**。读完应该能回答:
>
> 1. 我在做什么(目标 + 边界)
> 2. 我要写哪些代码(模块清单 + 现有代码改动)
> 3. 哪些地方一不小心就写错(Kafka 踩过的坑)
> 4. 按什么顺序写(里程碑)
>
> 协议精确规格见 [isr.md](./isr.md)(2400 行,实现时再查);任务拆分见 [isr-roadmap.md](./isr-roadmap.md)。

---

## 1. 我在做什么

给 storage-engine 加一套**副本同步协议**(ISR, In-Sync Replicas),让 memory / rocksdb / filesegment 三种引擎都能做到:

- 节点挂了,**秒级**切到另一个副本继续服务
- 用户选 `acks=all` + `min.insync.replicas >= 2`,**已 ack 的数据永不丢失**
- 用户选 `acks=1`,接受边缘场景丢数据,换性能
- 三种引擎共用同一套副本控制面,**只在本地存储读写部分有差异**

模型直接对齐 Kafka 2017 年 KIP-101 之后的稳定形态,**不要发明新协议**,不要"先做简化版以后再升级"。Kafka 自己花了 6 年才稳定下来,我们没有重新踩一遍坑的预算。

**不做的事**(明确划出,见 §10):unclean leader election、副本重平衡、idempotent producer、consumer 从 follower 读、tiered storage。

---

## 2. 核心模型(看懂这三个维度,后面就好懂了)

副本协议的所有状态都按这三个维度组织:

```
┌──────────────────────────────────────────────────────────────────────┐
│ Shard(逻辑日志,一根连续 offset 轴)                                │
│   ├── HW           — committed 水位,消费者只能读 < HW              │
│   ├── LEO          — 本地写入水位                                    │
│   └── log_start    — retention 后的可读起点                          │
│                                                                      │
│   一个 shard 包含 1..N 个 segment(memory/rocksdb 永远只有 1 个,    │
│   filesegment 写满后切下一个)                                       │
└──────────────────────────────────────────────────────────────────────┘
              │
              v
┌──────────────────────────────────────────────────────────────────────┐
│ Segment(副本身份单元)                                              │
│   ├── replicas              — 这个 segment 复制到哪些 broker        │
│   ├── leader / isr          — 当前 leader,当前 in-sync 集合        │
│   ├── leader_epoch          — leader 切换计数,KIP-101 关键          │
│   ├── segment_epoch         — ISR 变更计数,CAS 用                   │
│   ├── leader_broker_epoch   — 当前 leader 进程的 broker_epoch       │
│   └── log_start_offset      — leader 端可读起点                     │
│                                                                      │
│   每个 segment 是独立副本单元:跨 segment 时副本拓扑可以变,       │
│   leader_epoch 重置 0,LeaderEpochCache 独立                         │
└──────────────────────────────────────────────────────────────────────┘
              │
              v
┌──────────────────────────────────────────────────────────────────────┐
│ Broker(物理进程)                                                   │
│   └── broker_epoch  — 每次进程重启,meta 分配新值                    │
│                      用来 fence 同 node_id 不同进程实例的残留请求    │
└──────────────────────────────────────────────────────────────────────┘
```

**关键决定**:HW/LEO 在 shard 维度(对齐 Kafka partition-level HW,也对齐现有 `ShardOffsetState.high_watermark_offset`),副本身份在 segment 维度(对齐现有 `EngineSegment`)。两个维度耦合的地方就一处:**HW 推进时取的是"当前 active segment 的 ISR"**。

---

## 3. 协议核心:三个问题三个答案

ISR 协议本质上在回答三个问题。所有复杂度都围绕这三件事。

### 3.1 「谁是 leader,谁有权写」 → Epoch 三件套

只看 `node_id` 不够,因为同一个 node 可能:
- 跨任期(同一个 broker 在历史上当过 leader,然后被替换,然后又被选回来)
- 跨进程实例(crash + 立刻重启,旧进程还有残留请求在飞)
- 跨 ISR 版本(meta 端 ISR 变了,leader 自己缓存还是旧的)

所以引入三个独立的 epoch:

| Epoch | 谁递增 | 谁存 | 用途 |
|---|---|---|---|
| `leader_epoch` | leader 切换 | meta raft,segment 上 | KIP-101 truncation 基准 |
| `segment_epoch` | ISR / leader / replicas 任一变化 | meta raft,segment 上 | ISR 变更 CAS |
| `broker_epoch` | broker 进程注册 | meta raft,node_registry 上 | fence zombie 进程 |

每一类请求要带相关 epoch,接收方校验。任何不带 epoch 的路径都是 zombie 攻击面。

### 3.2 「什么数据算 committed」 → HW(High Watermark)

```
HW = min(LEO over ISR of active segment)
```

`offset < HW` 的数据就是 committed,**永不丢失**。`acks=all` 写入阻塞等 HW 跨过 `records.last_offset`。

关键约束(写错就出大问题):

- **HW 单调**:`new_HW = max(old_HW, min(...))`,扩 ISR 时 HW 不能倒退(否则消费者已读到的数据"消失")
- **HW 推进只在 fetch handler**:follower 通过 fetch_offset 隐式上报自己的 LEO,leader 收到 fetch 后再算 HW。**写入路径只更新 LEO 不动 HW**
- **HW 推进只算 epoch 匹配的 follower**:`progress.last_known_leader_epoch == leader.leader_epoch` 才计入,防止 leader 切换后陈旧 follower 拉低 HW

### 3.3 「故障后日志怎么对齐」 → KIP-101 OffsetsForLeaderEpoch

**禁止用本地 HW 截断**(这是 Kafka 2017 前的经典丢数据 bug)。

唯一允许的路径:

```
follower 启动 / leader 切换 / 收到 FencedLeaderEpoch:
  1. 拿本地 LeaderEpochCache 的最新 epoch
  2. 问 leader: "epoch=X 在你这边的 end_offset 是多少?"
  3. leader 查自己的 LeaderEpochCache 答复
  4. follower truncate_to(end_offset) + 修剪本地 cache
  5. 然后才能开始 fetch
```

`LeaderEpochCache` 是这条路径的核心,**必须持久化**(rocksdb/filesegment),memory 引擎天然丢失等价于"重启即全新副本"。

---

## 4. 怎么实现(模块清单 + 代码改动)

### 4.1 新增模块(全部在 `storage-engine/src/isr/`)

```
storage-engine/src/isr/
├── log.rs                       # ReplicaLog trait + 三引擎实现适配
├── leader_epoch.rs              # LeaderEpochCache + 持久化
├── state.rs                     # ShardReplicaState / SegmentReplicaState
├── append.rs                    # 写入路径(epoch 校验 + append + acks=all 等 HW)
├── fetch.rs                     # follower fetcher 任务 + leader fetch handler
├── offsets_for_leader_epoch.rs  # KIP-101 truncation RPC handler + caller
├── alter_partition.rs           # ISR shrink/expand 后台 + AlterPartition RPC client
└── manager.rs                   # LeaderAndIsr 通知响应、启动恢复、生命周期管理
```

**模块依赖关系**:

```
                manager (LeaderAndIsr 路由 + 状态机)
                │
                ├── append      → state, log, leader_epoch
                ├── fetch       → state, log, leader_epoch
                ├── offsets_..  → state, log, leader_epoch
                └── alter_part  → state(读 follower_progress)
```

### 4.2 现有代码改动(分类清楚)

**复用,不动**:
- `EngineSegment` 已有字段 `replicas / leader / leader_epoch / isr / status`
- `EngineShard` 全部字段(ISR 元数据都加在 segment 上,不动 shard)
- `core/segment.rs::create_segment_by_shard` 选副本逻辑
- `core/notify.rs::send_notify_by_set_segment` 广播通道
- `core/write.rs::batch_write` broker 间转发路由
- `commitlog::offset::ShardOffsetState` 已有 `high_watermark_offset`

**扩展(加字段,不破坏老结构)**:

| 文件 | 改动 |
|---|---|
| `metadata-struct/src/storage/segment.rs` | `EngineSegment` 加 `segment_epoch / leader_broker_epoch / log_start_offset`;`SegmentStatus` 加 `Unavailable` |
| `metadata-struct/src/storage/shard.rs` | `EngineShardConfig` 加 ISR 配置(`min_in_sync_replicas / replica_lag_time_max_ms / replica_fetch_*` / `replica_hw_checkpoint_interval_ms`) |
| `protocol/src/meta/common.proto` | `RegisterNodeReply` 加 `broker_epoch` |
| `protocol/src/storage/codec.rs` | `StorageEnginePacket` 加 `FetchReq/Resp` + `OffsetsForLeaderEpochReq/Resp` |
| `protocol/src/meta/storage.proto` | 新增 `AlterPartition` gRPC |

**重写(现有实现有 bug)**:

| 文件 | 现有问题 | 重写后 |
|---|---|---|
| `meta-service/src/core/leader_switch.rs::segment_leader_switch` | **从 `replicas` 选 leader 是 unclean,会丢数据** | 从 `isr` 选,ISR 空标 `Unavailable`(I14) |
| `metadata-struct/src/storage/segment.rs::EngineSegment::allow_read` | **只允许 `Write`,SealUp 后完全不能读** | 允许 `Write / PreSealUp / SealUp / Unavailable` |
| `storage-engine/src/core/segment.rs::segment_validator` | 跟随上面改 | 同步 |
| `meta-service/src/core/cluster.rs::register_node_by_req` | 不返回 broker_epoch | 维护 `node_registry`,分配并返回 broker_epoch |

### 4.3 关键数据结构(看 isr.md §3 详细定义)

```rust
// ReplicaLog trait — 三引擎共享的本地存储抽象(isr.md §4)
#[async_trait]
pub trait ReplicaLog: Send + Sync {
    async fn append_at(&self, shard, segment_seq, base_offset, records);    // 必须 fsync
    async fn read_from(&self, shard, segment_seq, offset, max_bytes);
    fn latest_offset(&self, shard, segment_seq) -> u64;
    async fn truncate_to(&self, shard, segment_seq, offset);                // 必须 fsync
    async fn clear(&self, shard, segment_seq);                              // retention 后强制全量重拉用
    fn log_start_offset(&self, shard, segment_seq) -> u64;
}

// 三层运行时状态(isr.md §3.4)
pub struct ShardReplicaState {       // shard 级,持有 HW / LEO / log_start
    pub local_leo:    AtomicU64,
    pub local_hw:     AtomicU64,
    pub log_start:    AtomicU64,
    pub hw_watcher:   watch::Sender<u64>,  // 唤醒 acks=all 等待者
    pub write_lock:   AsyncMutex<()>,      // shard 级写入锁
}

pub struct SegmentReplicaState {     // segment 级,持有副本身份
    pub role:              ReplicaRole,
    pub leader_epoch:      u32,
    pub segment_epoch:     u32,
    pub isr_cache:         Vec<u64>,
    pub follower_progress: DashMap<NodeId, FollowerProgress>,
    pub state_lock:        AsyncMutex<()>, // segment 级状态锁
}

pub struct FollowerProgress {
    pub broker_epoch:               u64,
    pub last_known_leader_epoch:    u32,
    pub leo:                        u64,
    pub last_fetch_ts:              u64,
    pub last_caught_up_ts:          u64,
    pub first_caught_up_after_oos:  Option<u64>,
}
```

### 4.4 锁结构(写代码前必须看懂,否则一定死锁)

两层锁,**任何路径都先 write_lock 再 state_lock**(顺序固定):

| 锁 | 粒度 | 谁持有 | 持有期内能做的事 |
|---|---|---|---|
| `ShardReplicaState.write_lock` | shard 级 | leader 写入路径(§5.2) | 选 active segment + 校验 epoch + append + LEO 推进 |
| `SegmentReplicaState.state_lock` | segment 级 | LeaderAndIsr 处理、fetch handler、fetcher 落盘 | role 转换 + isr_cache 更新 + HW 推进 |

**禁止在锁内做 long-poll wait / RPC**:fetcher 的 long-poll wait 在锁外,response 回来后再拿锁二次校验 role/epoch。

### 4.5 fetcher 与 LeaderAndIsr 的并发协调(Kafka `partitionMapLock` 模型)

最容易写错的地方之一。规则:

```
fetcher loop {
    state_lock {
        if role != FollowerActive: exit;
        snapshot = (target_leader, leader_epoch, local_leo);
    }                                       // 锁释放
    resp = leader.fetch(snapshot).await;    // 锁外 long-poll
    state_lock {
        if role != FollowerActive: discard; // 通知已切换了,丢
        if leader_epoch != snapshot.leader_epoch: discard;
        process(resp);                      // append + 推 local_hw
    }
}
```

`stop_fetcher_if_any()` 的语义是:**不取消网络请求,只在 state_lock 内改 role**。下一轮 fetcher 自己看到 role 变了退出。这跟 Kafka `AbstractFetcherManager.removeFetcherForPartitions` 同模型,代价只是一次浪费的网络 RTT,远好过处理"取消半完成的 stream"的复杂度。

---

## 5. 关键不变式(写错就出大事)

完整 16 条见 [isr.md §0](./isr.md)。这里列实现时最容易踩的 6 条:

| ID | 不变式 | 写错后果 |
|---|---|---|
| **I4** | 写入路径**在同一锁内**校验 epoch + append,中间不让出锁 | LeaderAndIsr 插队 → zombie leader 漏网 → 已 ack 数据被 truncate |
| **I6** | HW 单调,`new_hw = max(old, min(...))` | 消费者已读数据"消失" |
| **I8** | Committed 数据永不丢 | 客户端的根本承诺破坏 |
| **I9** | Truncation 只能走 `OffsetsForLeaderEpoch`,**禁止用本地 HW** | KIP-101 经典丢数据 |
| **I11** | Leader 上任先 fsync LeaderEpochCache 才转 Active | 上任崩溃 → 日志分歧 |
| **I12** | ISR 扩展条件是 `leo >= leader.leo`(不是 `>= hw`)(KIP-679) | 刚加入 ISR 就拉低 HW |

---

## 6. 端到端流程(两张图说清楚)

### 6.1 写入 + 复制(acks=all)

```
producer / 上层 broker
   │ write(shard, records, acks=all)
   v
broker A (任意,从 metadata 缓存查 active_segment.leader=B):
   if self == B: 走下面写入流程
   else:         转发给 B,带 current_leader_epoch
   ↓
broker B (leader),在 shard.write_lock 内:
   ├── 校验 role==LeaderActive  否则 NotReady / NotLeader
   ├── 校验 self.leader_epoch == meta.leader_epoch
   ├── 若请求带 epoch:校验 req.epoch == self.leader_epoch
   ├── acks=all:校验 |ISR| >= min_in_sync_replicas
   ├── ReplicaLog::append_at  (内部 fsync)
   ├── 兜底校验 LeaderEpochCache.latest_epoch() >= self.leader_epoch
   ├── shard.local_leo += N
   └── (释放锁)
   ↓ acks=all: 在 hw_watcher 上等 shard.local_hw >= records.last_offset
                超时 → RequestTimedOut(数据保留,靠 KIP-101 自然消化)
                成功 → ack 给上游

并行: follower C 持续 long-poll fetch B:
   C → B: fetch(shard, fetch_offset=C.local_leo, current_leader_epoch=E, broker_epoch)
   ↓ B 在 segment.state_lock 内:
     校验 epoch 三态(<返 Fenced,>返 Unknown)
     校验 fetch_offset 范围(< log_start 或 > leader_leo → OffsetOutOfRange)
     更新 follower_progress[C].leo = req.fetch_offset
     更新 last_known_leader_epoch / last_fetch_ts
     若 fetch_offset >= leader_leo_at_request_arrival:
        last_caught_up_ts = now
     HW 推进(只算 last_known_leader_epoch == self.leader_epoch 的 follower):
        new_hw = min(B.local_leo, min(eligible.leo))
        B.shard.local_hw = max(B.shard.local_hw, new_hw)
        若推进 → hw_watcher.send → 唤醒 acks=all 等待者
   ↓ B → C: records + leader_hw + leader_log_start + leader_leo + leader_epoch
   C: 检测跨 epoch → LeaderEpochCache.assign + fsync(先 cache 后 log!)
      → ReplicaLog::append_at(records)
      → C.shard.local_hw = max(local_hw, min(local_leo, leader_hw))
```

### 6.2 Leader 切换

```
heartbeat 发现 B1 down
   ↓
meta-service: segment_leader_switch(failed=B1)
   for each segment where leader == B1:
     candidates = isr - {B1}
     if candidates.empty():
        segment.status = Unavailable  (I14)
        segment_epoch += 1
     else:
        new_leader = candidates.next()
        segment.leader              = new_leader
        segment.leader_epoch        += 1
        segment.segment_epoch       += 1
        segment.leader_broker_epoch = node_registry[new_leader]
        segment.isr                 -= {B1}
   raft 写入 + 广播 SegmentLeaderAndIsr
   ↓
broker B2 (新 leader),在 state_lock 内:
   ├── role = LeaderInitializing
   ├── cancel_inflight_producer_requests → 返 NotLeaderForPartition
   ├── stop_fetcher_if_any                (改 role,下一轮 fetcher 自己退)
   ├── current_leo = ReplicaLog::latest_offset
   ├── LeaderEpochCache.assign(new_epoch, current_leo) + fsync ← I11 关键
   ├── isr_cache = notification.isr
   ├── shard.local_hw = max(current, persisted_hw_from_checkpoint)   ← 不要设成 LEO
   ├── reset_follower_progress
   └── role = LeaderActive

broker B3 (继续 follower,但 leader 变了):
   ├── role = FollowerInitializing
   ├── stop_fetcher_if_any
   ├── cancel_inflight_producer_requests(如果之前是 leader 现在降级)
   ├── (resp_epoch, resp_offset) = OffsetsForLeaderEpoch(target=B2, my_epoch=E_old, current=E_new)
   ├── match resp_epoch:
   │     -1 (整段被 retention):
   │       ReplicaLog::clear + LeaderEpochCache::clear,fetch_offset = resp_offset
   │     epoch:
   │       truncate_point = min(local_leo, resp_offset)
   │       ReplicaLog::truncate_to(truncate_point)
   │       LeaderEpochCache::truncate_from_end_by_epoch(epoch, resp_offset)
   │       fetch_offset = truncate_point
   ├── LeaderEpochCache::fsync
   ├── start_fetcher(target=B2, fetch_offset)
   └── role = FollowerActive

上游 broker 写入仍走老路由到 B1 → B1 已降级返回 NotLeader → 拉新 metadata → 转 B2
```

---

## 7. 持久化策略(三个不同 cadence,别搞混)

| 数据 | 持久化时机 | fsync? | 崩溃后果 |
|---|---|---|---|
| **LEO**(通过 ReplicaLog 数据) | 每次 `append_at` 返回前 | **是** | LEO 回退 = 已 ack 数据丢失,不允许 |
| **LeaderEpochCache** | 每次 `assign` / `truncate_*` 后 | **是** | cache 与 log 不一致 → OffsetsForLeaderEpoch 答错 → 丢数据 |
| **HW** | 后台每 5 秒 checkpoint(对齐 Kafka) | 异步 | 允许回退一个 interval,KIP-101 路径兜底 |
| **log_start_offset** | retention 推进前先 checkpoint 再删数据 | **是** | 先删后写 → 启动后读到 hole |

关键认知:**HW 可以异步**是 KIP-101 的核心红利 — 不用每次 HW 推进都 fsync,因为 LeaderEpochCache 路径会兜底安全性。

启动恢复时(`§8.-1`):

1. 扫 ReplicaLog 拿真实 LEO(不信 checkpoint,checkpoint 必滞后)
2. 读 HW checkpoint,若 `hw > leo` 修正为 `hw = leo`
3. **修复 LeaderEpochCache 与 log 的一致性**:
   - 删 `entry.start_offset > local_leo` 的条目(虚假声明的未来 epoch)
   - 删 `entry.start_offset < log_start_offset` 的条目(已被 retention 但 cache 没修剪)
4. 所有 segment 进 `Initializing`,等 meta 推 LeaderAndIsr 才转角色

---

## 8. ISR 维护(后台任务)

只有 leader 触发 ISR 变更(对齐 Kafka KIP-497 `AlterPartition`)。

```
leader 每 N 秒扫 follower_progress (只看自己 LeaderActive 的 segment):

  for (node, prog) in follower_progress:
     in_isr = isr_cache.contains(node)

     # shrink
     if in_isr and node != self:
        lag_ms = now - prog.last_caught_up_ts
        if lag_ms > replica_lag_time_max_ms:
           AlterPartition(new_isr = isr - {node})

     # expand
     if not in_isr and expand_eligible(prog):  # I12: leo>=leader.leo + epoch 匹配 + broker_epoch 未 fence + 反 flapping
        AlterPartition(new_isr = isr + {node})

AlterPartition 路径:
  broker → meta gRPC → meta raft op UpdateSegmentIsr → 多重 fence:
     - leader_epoch 匹配
     - broker_epoch 匹配(防 zombie 进程)
     - segment_epoch CAS(防并发 ISR 变更覆盖)
     - new_isr 合法性(必须含 leader,必须 ⊆ replicas,非空)
   通过 → segment_epoch += 1,广播 SegmentLeaderAndIsr
```

**节流**:单 segment 同时只能一个 in-flight AlterPartition,500ms 内最多一次。

---

## 9. 三引擎差异(代码上要分开处理的地方)

| 维度 | memory | rocksdb | filesegment |
|---|---|---|---|
| `segment_seq` 取值 | 恒为 0 | 恒为 0 | 写满后递增 |
| 本地存储 | DashMap<offset, Record> | RocksDB KV(key 加 segment_seq 段) | 文件 |
| `LeaderEpochCache` 持久化 | **不持久化**(进程重启即丢) | rocksdb key 前缀 | sidecar 文件 |
| follower 重启等价 | 全新副本(必须从 leader_log_start 全量重拉) | 用 epoch cache 走 truncation | 同 rocksdb |
| 跨 segment 切换 | 永不发生 | 永不发生 | 新 segment 独立副本拓扑 |
| 副本重平衡 | 不支持(整 shard 重建) | 不支持 | seal up 后切新 segment 时可换节点 |

**协议代码与引擎解耦**:所有差异封闭在 `ReplicaLog` trait 实现里。ISR 控制面(`fetch.rs / append.rs / manager.rs`)只调 trait,**不感知**引擎类型。

---

## 10. 不做的事(明确划出)

| 项 | 原因 |
|---|---|
| Unclean leader election | I14 协议禁用,ISR 空直接 Unavailable |
| Reassign replicas | segment 创建后副本拓扑固定 |
| Idempotent / EOS producer | 接口层预留 hook(`§18.1`),实现可后做 |
| Tiered Storage / KIP-405 | 不在范围 |
| Consumer 从 follower 读 / KIP-392 | 简化协议边界,consumer 只读 leader |
| Incremental Fetch / KIP-227 | 字段预留(session_id=0 表 full),实现不做 |
| Rack awareness | 协议外调度优化 |
| Observer replicas / KIP-392 | 不在范围 |
| ELR / KIP-966 | ISR 空直接 Unavailable,不引入"次优 leader 池" |

完整清单见 [isr.md §16](./isr.md)。

---

## 11. 实施顺序(里程碑)

详细 task 拆分见 [isr-roadmap.md](./isr-roadmap.md)。里程碑节奏:

| 里程碑 | 内容 | 完成判据 |
|---|---|---|
| **M1 元数据就位** | `EngineSegment` 加字段 + raft `UpdateSegmentIsr` op + `register_node` 返回 broker_epoch | 单测覆盖陈旧 epoch 拒绝 + segment_epoch CAS |
| **M2 本地存储就位** | `ReplicaLog` trait + memory/rocksdb 实现 + `LeaderEpochCache` 持久化 | append/truncate/重启重建单测 |
| **M3 副本同步跑通** | M1+M2 + 写入 epoch 校验 + long-poll fetch + OffsetsForLeaderEpoch | 单 leader + 2 follower,follower 能追上;注入 GC 暂停不被误踢 |
| **M4 协议闭环** | M3 + ISR shrink/expand + AlterPartition + SegmentLeaderAndIsr 响应 + KIP-101 truncation 完整路径 | **故障演练:验证 §12.2 不丢数据**(核心回归用例) |
| **M5 故障演练** | M4 + §12 各场景的混沌测试 | 全部 17 个场景在测试环境下行为符合预期 |
| **M6 filesegment 接入** | M5 + filesegment 实现 ReplicaLog + segment seal 时 fetcher 切换 | filesegment 全场景演练 |

**关键约束**:任何里程碑都**不得放宽 §0 不变式**。例如 M3 上线时如果 M4 的 ISR 维护还没做,允许 ISR 始终 = replicas(不收缩),但已实现部分必须严格走 epoch 路径,**不允许临时用本地 HW truncate**。

---

## 12. 实现者必看 — 最容易写错的 7 个点

每一条都是 Kafka 早期(2011-2017)踩过的真实坑。

1. **`truncate_to(local_hw)` 是 bug**
   一定要走 `OffsetsForLeaderEpoch`(I9)。任何"先用 HW 凑合,以后再换 epoch"的想法都是回到 Kafka 2017 前 bug。

2. **HW 一定要 `max(old, new)`**
   `min(LEO over ISR)` 不是 final 值,要套一层 `max` 保证单调(I6)。扩 ISR 时新成员 LEO 可能比 HW 还低,直接赋值会让消费者已读数据"消失"。

3. **HW 推进只算 epoch 匹配的 follower**
   leader 切换后,旧 follower 还没切换到新 epoch 就 fetch 进来,**不算它**。否则它的旧 LEO 会拉低 HW(I6 补强)。

4. **Leader 上任要 fsync LeaderEpochCache 才能接写**
   `LeaderInitializing` 状态期间所有写都拒(I11)。否则上任后立刻崩溃,新 epoch 没持久化,follower 用 OffsetsForLeaderEpoch 来问,leader 答不出来 → 日志分歧。

5. **ISR 扩展条件是 `leo >= leader.leo`,不是 `>= hw`**(KIP-679 / I12)
   `hw` 永远 ≤ `leo`,follower 满足 `leo>=hw` 时实际可能还没追上,加入 ISR 立即被算入 HW 计算反而拉低 HW。

6. **写入路径的 epoch 校验和 append 必须在同一把锁内**
   否则 LeaderAndIsr 通知插进来会让 zombie leader 漏网。锁内顺序:`校验 role → 校验 epoch → append → 更新 LEO`(I4)。

7. **跨 epoch records 的处理顺序:先 cache.assign + fsync,后 log.append_at**
   反过来会让本地 log 比 epoch cache 长,崩溃后 OffsetsForLeaderEpoch 答错。同理 truncate:先 log truncate,后 cache truncate。

---

## 13. 相关文档

- **[isr.md](./isr.md)** — 协议精确规格(2400 行,16 条不变式 + 18 个章节)
- **[isr-roadmap.md](./isr-roadmap.md)** — 15 个开发 task 拆分 + 原子合并组
- **[diagrams/](./diagrams/)** — 架构图、写入时序、fetch 流程、leader 切换时序

---

## 14. 一句话总结

> **协议照抄 Kafka KIP-101+ 的稳定形态**(不发明新协议),**控制面与引擎解耦**(三引擎共享 ISR 模块,差异封在 `ReplicaLog` trait),**严格遵循 16 条不变式**(任何降级都是回到 Kafka 早期 bug)。
