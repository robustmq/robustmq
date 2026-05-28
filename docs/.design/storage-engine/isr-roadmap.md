# Storage Engine ISR — 开发任务拆分

> 配套文档:[isr.md](./isr.md)。本文把 ISR 协议拆成**独立可交付的小单元**,每个单元有自己的代码、单测、验收标准,**不依赖后续任务才能上线**。
>
> 原则:
> - 每个 task 是一个 PR 的体量(几百到 ~1500 行代码 + 测试)
> - Task 之间只有"前置依赖",没有"必须一起合"的强耦合
> - **每个 task 合并后,isr.md §0 的 16 条不变式必须全部成立**;不允许"先上 task X 做半截,等 task Y 来补"
> - 若某 task 单独上线会违反不变式,必须等组合 task 一起合(见 §"原子合并组")
>
> ⚠️ Task 编号(T1, T2, ...)是依赖关系排序,不是版本号。多个 task 可并行做。

## 原子合并组

下列 task 集合**必须一次合并**,因为单独合任一会破坏不变式:

| 原子组 | 包含 task | 一起合的理由 |
|---|---|---|
| 元数据基础 | T1 + T2 | T1 单独合后 segment_epoch 字段在但 raft 不校验 → I3 不成立 |
| 控制面闭环 | T3 + T13a + T13b | broker 端不响应 LeaderAndIsr 等于不感知 leader 切换 → I4 (zombie write fence) 不成立 |
| 写入闭环 | T11a + T11b + T11c | 单独的"写入路径校验 epoch"没有 HW 推进配套 → acks=all 永远超时;单独的 HW 推进没校验 → I4 不成立 |
| KIP-101 闭环 | T7 + T10 + T13c | 单独有 LeaderEpochCache 但没有 OffsetsForLeaderEpoch 流程,follower 重启走错 truncate 路径 → I9 不成立 |

单个 task 仍可独立开发、独立 review,但**合并到主分支必须是原子组一起合**。

---

## 依赖图概览

![ISR 开发任务依赖图(T1–T15)](./diagrams/05-roadmap-dependencies.png)

> 注:图为 T1-T12 旧版本,具体任务列表以下方文字版为准(已扩展到 T15 + 拆 T11/T13)。

**分组**:
- **A 组(元数据控制面)**:T1, T2, T3 — 改 meta-service,不碰数据面
- **B 组(本地存储抽象)**:T4, T5, T6, T7 — 改 storage-engine 本地,不联网
- **C 组(数据面 RPC + 复制)**:T8, T9, T10 — 副本同步真正跑起来
- **D 组(写入闭环)**:T11a, T11b, T11c — acks 语义 + HW 推进 + min_isr 拒写
- **E 组(控制面响应)**:T12, T13a, T13b, T13c — broker 端响应 LeaderAndIsr 并做 truncation

A 组和 B 组可完全并行。C 组需要 B 组先有 trait,可以与 A 组并行。D 组与 E 组的部分子项可并行,最终收口在 T13c(KIP-101 truncation 全流程)。

---

## A 组:元数据控制面

### T1:`EngineSegment` 扩字段 + `EngineShardConfig` 扩 ISR 配置 + broker_epoch 注册

**目标**:为 ISR 协议提供元数据载体。**纯结构扩展 + 节点注册返回 broker_epoch**。

**前置**:无

**改动**:
- `metadata-struct/src/storage/segment.rs::EngineSegment`:
  - 新增 `segment_epoch: u32`
  - 新增 `leader_broker_epoch: u64`(当前 leader 上任时的 broker_epoch 快照)
  - 新增 `log_start_offset: u64`
  - 所有新字段加 `#[serde(default)]`
- `metadata-struct/src/storage/shard.rs::EngineShardConfig`:
  - 新增 `min_in_sync_replicas: u32`(默认 1)
  - 新增 `replica_lag_time_max_ms: u64`(默认 30_000,同 Kafka)
  - 新增 `replica_fetch_max_bytes: u64`(默认 1 MiB)
  - 新增 `replica_fetch_wait_max_ms: u64`(默认 500)
  - 新增 `replica_fetch_min_bytes: u64`(默认 1)
  - 新增 `unclean_leader_election_enable: bool`(协议要求 false,字段保留供运维报错)
- meta-service 节点注册:
  - `RegisterNodeRequest`:节点继续无变化
  - `RegisterNodeResponse` 新增 `broker_epoch: u64`
  - meta-service raft 状态机新增 `node_registry: HashMap<u64 /*node_id*/, u64 /*last_broker_epoch*/>`,每次 register 时该 node 的值 +1 并返回
- broker 进程:启动时缓存自己的 `broker_epoch`,供 §3.5 用

**不做**:
- 不改 ISR / leader 切换 raft op(留 T2)
- 不在创建 segment 时填新字段(留 T2 顺手做)
- broker 端缓存的 `broker_epoch` 暂时不被任何 RPC 使用(留 T2、T11 用)

**验收**:
- 老编码反序列化为带默认新字段的对象;新字段序列化往返一致
- 单测:同一 broker 二次 register 拿到的 broker_epoch 严格递增
- 单测:meta-service 重启后 node_registry 从 raft 日志恢复

**预估**:中(~350 行,含 raft op + 测试)

---

### T2:meta-service raft op `UpdateSegmentIsr` + 三重 fence(I3)

**目标**:让 meta-service 状态机支持 ISR 变更,严格按 §7.3 的三重 fence 校验。

**前置**:T1

**与 T1 一起原子合并**。单独合 T1 后字段在但无逻辑,反而误导。

**改动**:
- `meta-service/src/raft/route/engine.rs`:
  - 新增 `EngineDataType::UpdateSegmentIsr`
  - payload: `{ shard_name, segment_seq, new_isr, requester_node_id, requester_broker_epoch, leader_epoch, expected_segment_epoch }`
- 状态机应用逻辑(按 §7.3 顺序):
  1. `req.requester_node_id != current.leader` → `NotLeaderForPartition`
  2. `req.leader_epoch != current.leader_epoch` → `FencedLeaderEpoch`
  3. `req.requester_broker_epoch != node_registry[req.requester_node_id]` → `StaleBrokerEpoch`
  4. `req.expected_segment_epoch != current.segment_epoch` → `InvalidUpdateVersion`
  5. 全部通过:`current.isr = req.new_isr`,`current.segment_epoch += 1`
- `meta-service/src/core/segment.rs::create_segment_by_shard`:
  - 初始化新 segment:`segment_epoch=0`、`log_start_offset=0`、`leader_broker_epoch = node_registry[leader]`
- `meta-service/src/core/leader_switch.rs::segment_leader_switch` **完全重写**(D3):
  - **从 ISR 选 leader**(不是从 replicas) — 修复 unclean leader election bug
  - ISR 空 → segment 标 `SegmentStatus::Unavailable`,不选 leader,等运维
  - 从 ISR 移除故障节点
  - `leader_epoch += 1`
  - `segment_epoch += 1`
  - `leader_broker_epoch = node_registry[new_leader]`
- `metadata-struct/src/storage/segment.rs::SegmentStatus` 新增 `Unavailable` 枚举值
- `metadata-struct/src/storage/segment.rs::EngineSegment` 同时新增方法:
  - `allow_read()`:`Write | PreSealUp | SealUp | Unavailable` — 修复 SealUp 不能读 bug(D4)
  - `allow_write()`:`Write | PreSealUp`(新增)
- `storage-engine/src/core/segment.rs::segment_validator` 跟随 allow_read 修正(D4)

**不做**:
- 不广播变更(留 T3)
- 不接受 broker 端真正发的请求(broker 端 ISR 触发在 T12)
- 此时 ISR 永远等于 replicas(因为没人发 shrink/expand)

**验收**:
- 单测:陈旧 leader_epoch 拒绝
- 单测:陈旧 broker_epoch 拒绝(zombie broker fence,§12.14)
- 单测:`segment_leader_switch` 在 ISR 空时不选 leader 而是标 Unavailable
- 单测:`segment_leader_switch` 从 ISR 选 leader 后 ISR 不含故障节点
- 单测:SealUp 状态的 segment `allow_read()=true` `allow_write()=false`
- 单测:陈旧 segment_epoch CAS 拒绝
- 单测:非 leader 节点的 ISR 变更请求拒绝
- 单测:并发两个 ISR 变更,只有 expected_segment_epoch 匹配的能成功
- segment create / leader switch 后所有三个 epoch 正确

**预估**:中(~500 行)

---

### T3:`SegmentLeaderAndIsr` 广播 + broker 端 epoch 缓存更新

**目标**:meta-service 在 segment leader / ISR 变更后推送通知给相关 broker。**broker 端必须真正处理通知**(更新 epoch 缓存 + 切换 role),否则违反 I4(zombie leader 写入 fence 失效)。

**前置**:T1

**与 T13a (角色切换) 一起原子合并**。本 task 单独上线会让 broker 收到通知不切换 role → 旧 leader 继续接写 → 违反 I4。

**改动**:
- `meta-service/src/core/notify.rs`:
  - 新增 `send_notify_by_segment_isr_change(call_manager, segment) -> ...`
  - 复用 / 扩展现有 leader 切换通知路径
- 广播 payload 包含完整 `EngineSegment`(broker 端用 `segment_epoch` 判定是否为最新)
- broker 端 handler 必须实现:
  - 校验 `notification.segment_epoch > local.segment_epoch`,否则丢弃
  - 更新 `SegmentReplicaState.leader_epoch / segment_epoch / isr_cache / role`
  - role 状态机基本骨架(完整角色切换逻辑见 T13a)

**不做**:
- 不实现完整的 LeaderInitializing / FollowerInitializing 状态转换(留 T13a)
- 不实现 OffsetsForLeaderEpoch truncation(留 T10/T13c)
- 不实现 ISR 自动 shrink/expand(留 T12)

**验收**:
- meta-service 测试:模拟 ISR 变更,验证通知发出
- broker 测试:能解析通知并更新本地缓存的 `(leader_epoch, segment_epoch, isr_cache)`
- 通知乱序到达时,旧 segment_epoch 通知被丢弃
- broker 收到自己变成 follower 的通知,停止接受 producer 写入

**预估**:中(~450 行)

---

## B 组:本地存储抽象

### T4:`ReplicaLog` trait 定义

**目标**:抽出三引擎统一接口。**只定义,不实现**。

**前置**:无(纯新代码)

**改动**:
- 新建 `storage-engine/src/isr/log.rs`:
  - `pub trait ReplicaLog`(签名见 isr.md §4)
- 新建 `storage-engine/src/isr/mod.rs` 注册子模块(`log` 暴露,其他模块占位)
- 错误类型补全 `StorageEngineError`:`OutOfOrder`、`OffsetOutOfRange`、`SegmentSealedUp` 等

**不做**:
- 不实现任何引擎的 trait
- 不接 RPC

**验收**:
- `cargo check -p storage-engine` 通过
- trait 文档注释完整

**预估**:小(~100 行)

---

### T5:memory 引擎实现 `ReplicaLog`

**目标**:memory commitlog 实现 trait,作为最小工作模型。

**前置**:T4

**改动**:
- `storage-engine/src/commitlog/memory/`:
  - `impl ReplicaLog for MemoryStorageEngine`:
    - `append_at`:校验 base_offset == latest_offset,落 DashMap,更新 `latest_offset`
    - `read_from`:DashMap range scan,受 max_bytes 限制
    - `latest_offset`:已存在
    - `truncate_to`:DashMap retain offset <= target

**不做**:
- 不实现 LeaderEpochCache(memory 无持久化,见 isr.md §9.5,留 T9/T10 时按"全量重拉"处理)

**验收**:
- 单元测试:append → read 往返
- 单元测试:append 不连续 offset 报错
- 单元测试:truncate_to 后 latest_offset 正确

**预估**:小(~250 行,含测试)

---

### T6:rocksdb 引擎实现 `ReplicaLog`

**目标**:rocksdb commitlog 实现 trait。

**前置**:T4

**改动**:
- `storage-engine/src/commitlog/rocksdb/`:
  - key 编码改为 `/record/{namespace}/{shard}/{segment_seq:08}/record/{offset:20}`
    - 兼容:`segment_seq` 不存在时按 0 处理(旧数据自动识别为 segment_seq=0)
  - `impl ReplicaLog for RocksDBStorageEngine`:
    - `append_at`:批量 put,更新 latest_offset 元数据 key
    - `read_from`:prefix scan
    - `truncate_to`:range delete `(shard, segment_seq, target+1..)`

**不做**:
- 不实现 LeaderEpochCache 持久化(留 T7)

**验收**:
- 单元测试同 T5
- 兼容性测试:用 segment_seq=0 写入,旧 key 路径仍可读

**预估**:中(~400 行)

---

### T7:`LeaderEpochCache` 数据结构 + rocksdb 持久化

**目标**:实现 KIP-101 的 epoch 缓存,这是后续 truncation 协议的基础。

**前置**:T6(需要 rocksdb 接口)

**改动**:
- 新建 `storage-engine/src/isr/leader_epoch.rs`:
  - `pub struct LeaderEpochCache { entries: Vec<LeaderEpochEntry> }`
  - 方法:`assign / end_offset_for / truncate_from_end / truncate_from_start`
- rocksdb 持久化:
  - key 前缀 `/leader_epoch/{shard}/{segment_seq}/`
  - 每个 entry 一个 key:`/leader_epoch/{shard}/{segment_seq}/{epoch:10}` → value=start_offset
  - 启动时全量加载到内存,运行时双写(内存 + 落盘)
- 单测:
  - `assign + end_offset_for` 各种边界(epoch 最早、中间、最新)
  - 重启后能从 rocksdb 恢复

**不做**:
- 不接 filesegment(留待 filesegment 接入时,sidecar 文件实现)
- 不接 memory(memory 不持久化,follower 重启全量重拉)
- 不被 fetch/truncation 流程调用(留 T9/T10)

**验收**:
- 数据结构单测覆盖 KIP-101 文档里的所有 case
- 重启重建测试
- 写性能基准:append 每条消息能否承受同步更新 leader_epoch(预期是 append batch 才触发更新,不是每条)

**预估**:中(~500 行,含测试)

---

## C 组:数据面 RPC + 复制

### T8:long-poll fetch RPC + 完整 epoch 校验(I15)

**目标**:实现 follower → leader 的 fetch 协议,follower 能拉到数据。**严格按 §6.2 顺序做完整校验**,不允许"暂时不校验 epoch"。

**前置**:T3(需要 SegmentReplicaState 的 role / leader_epoch 缓存,见 T3 的改动)、T4、T5/T6

**改动**:
- 新建 `storage-engine/src/isr/fetch.rs`:
  - `pub struct FetchHandler`(leader 端)
  - 处理 `FetchRequest`,按 §6.2 校验顺序:
    1. role == LeaderActive(否则 NotLeaderForPartition / NotReady)
    2. leader_epoch 三态校验(Fenced / Unknown / 通过)
    3. fetch_offset 范围校验(返回 OffsetOutOfRange 带 leader_log_start + leader_leo)
    4. broker_epoch 校验(StaleBrokerEpoch)
    5. 更新 follower_progress(无 last_caught_up_ts 精确语义,留 T9)
    6. HW 推进逻辑预留接口,实际推进留 T11b
  - long-poll(`tokio::time::timeout` + `Notify` 或 `watch::channel`)
- protobuf 定义 `StorageEngineFetchRequest / FetchResponse`(见 isr.md §6.6)
- broker RPC router:挂载 `handle_isr_fetch`
- client wrapper:`fetch_client.fetch(req) -> resp`

**不做**:
- 不做 `last_caught_up_ts` 的精确语义(留 T9)
- 不真正推进 HW(留 T11b,本 task 里 HW 更新接口可以是 no-op)
- 不真正起 fetcher 循环(留 T9)
- 不处理 `OffsetsForLeaderEpoch`(留 T10)

**验收**:
- 单测:陈旧 leader_epoch 返回 FencedLeaderEpoch
- 单测:陈旧 broker_epoch 返回 StaleBrokerEpoch
- 集成测试:两 broker 一 leader 一 follower(手动构造 ReplicaState),follower 发 fetch,leader 返回 records
- long-poll 超时返回空 records
- min_bytes 达到立即返回

**预估**:大(~750 行,含完整 epoch 校验)

---

### T9:follower fetcher 循环 + `last_caught_up_ts` 维护

**目标**:follower 自动拉取数据,leader 维护 follower 进度。

**前置**:T7, T8

**改动**:
- 新建 `storage-engine/src/isr/state.rs`:
  - `ReplicaStateRegistry / SegmentReplicaState / FollowerProgress`(见 isr.md §3.4)
  - 不含 `hw_watcher`(留 T11)
- 新建 `storage-engine/src/isr/fetcher.rs`(或扩 `fetch.rs`):
  - per `(shard, segment_seq)` fetcher 任务
  - 循环:`latest_offset → fetch → append_at → update LeaderEpochCache`
  - 错误分支:NotLeader / FencedEpoch / OffsetOutOfRange(`FencedEpoch` 暂时只重连,T10 加 truncation;`OutOfRange` 同理)
- leader 端 fetch handler 扩展:
  - 收到 fetch 时更新 `follower_progress[replica_id]`
  - `last_caught_up_ts` 按 §6.4 规则更新

**不做**:
- 不做 truncation(留 T10,此时 `FencedEpoch` 只是退避重试)
- 不做 HW 推进(留 T11)
- 不做 ISR shrink/expand(留 T12)

**验收**:
- 集成测试:三 broker,follower 自动拉到 leader 全部数据
- leader 端 `last_caught_up_ts` 在 follower 追上时更新
- 注入 follower 短暂离线,恢复后能继续拉
- **限制**:没有 truncation,leader 切换场景会 fail(预期,留 T10)

**预估**:中大(~800 行)

---

### T10:`OffsetsForLeaderEpoch` RPC + truncation 协议

**目标**:实现 KIP-101 truncation 完整流程。这是协议正确性的关键。

**前置**:T7, T8, T9

**改动**:
- protobuf:`OffsetsForLeaderEpochRequest / Response`(见 isr.md §9.3)
- broker RPC router:挂载 `handle_offsets_for_leader_epoch`
- leader 端 handler:
  - 查 `LeaderEpochCache::end_offset_for(req.follower_leader_epoch)`
  - 校验 `current_leader_epoch`
- follower 端 truncation 流程:
  - fetcher 启动前 / 收到 `FencedLeaderEpoch` 后,先发 `OffsetsForLeaderEpoch`
  - 拿到 `end_offset_of_epoch` 后 `replica_log.truncate_to`
  - 同步 `LeaderEpochCache.truncate_from_end`
- memory 引擎特殊路径(isr.md §9.5):无本地 epoch,从 leader_log_start 全量重拉

**不做**:
- 不依赖 §12 异常场景全部覆盖(那是 §12.x 的回归用例,T11/T12 完成后再做)

**验收**:
- 单测:模拟 isr.md §9.2 的两个 KIP-101 经典 case
- 集成测试:三 broker,kill leader,新 leader 起来,旧 leader 重启,验证 truncate 正确(**§12.2 回归用例**)
- memory 引擎的全量重拉路径

**预估**:大(~900 行,含集成测试)

---

## D 组:写入闭环

> D 组三个子项 **T11a + T11b + T11c 必须一起合并**(原子组,见顶部"原子合并组")。单独合任一会让 acks=all 永远 timeout 或 epoch 校验缺失,违反 I4/I6。

### T11a:写入路径完整 epoch 校验 + 原子性(I4)

**目标**:写入路径按 §5.2 严格执行,所有校验和 append + LeaderEpochCache 更新在同一锁内原子完成。

**前置**:T3、T6(或 T5)、T7

**改动**:
- `SegmentReplicaState` 加 segment_lock(`tokio::sync::Mutex` 或 RwLock)
- 写入路径(`storage-engine/src/handler/adapter.rs` 等)严格按 §5.2:
  1. role == LeaderActive 校验
  2. self.leader_epoch == meta.leader_epoch 校验
  3. producer.current_leader_epoch 校验(若携带)
  4. `|ISR| >= min_in_sync_replicas` 校验(acks=all)
  5. `ReplicaLog::append_at` 落本地
  6. 若新 epoch 首批:`LeaderEpochCache.assign(epoch, leo) + fsync`
  7. local_leo += records.len()
  上述 1-7 在同一 segment_lock 内
- ProduceRequest protobuf 加 `optional current_leader_epoch`

**不做**:
- 不做 HW 推进 / acks=all 等待(留 T11b)
- 不做 NotEnoughReplicas 之外的 ISR 状态变化响应(留 T12)

**验收**:
- 单测:role=Follower 时写入返回 NotLeaderForPartition
- 单测:role=LeaderInitializing 时写入返回 NotReady
- 单测:producer 旧 epoch 写入返回 FencedLeaderEpoch
- 单测:|ISR|<min_isr 且 acks=all 返回 NotEnoughReplicas
- 单测:append 期间收到 LeaderAndIsr 通知不会插入到 append 中间(锁串行)
- 单测:崩溃恢复:写入完成但 LeaderEpochCache 未 fsync 完成 → 重启后副本拒绝以 leader 提供服务

**预估**:大(~900 行)

---

### T11b:HW 推进(单调 I6)+ acks=all 等待

**目标**:fetch handler 推进 HW,acks=all producer 阻塞等待 HW 跨过其 last_offset。

**前置**:T11a

**改动**:
- `SegmentReplicaState`:
  - `local_hw: AtomicU64`(单调)
  - `hw_watcher: tokio::sync::watch::Sender<u64>`
- leader fetch handler(T8 已挖钩子):
  - 收到 fetch 后:`new_hw_candidate = min(local_leo, min(p.leo for p in isr))`
  - **强制单调**:`local_hw = max(local_hw, new_hw_candidate)`
  - 若推进:`hw_watcher.send(local_hw)`
- 写入路径锁外段:
  - acks=all 监听 `hw_watcher.subscribe()`,直到 `hw >= records.last_offset`
  - 带 `req.timeout_ms` 超时

**不做**:
- 不做 ISR shrink/expand(留 T12),所以 |ISR|=replicas 时 follower 必须全员追上 HW 才推进
- 不做角色切换 fix(留 T13a):若过程中 self 不再是 leader,acks=all 请求由 T13a 取消

**验收**:
- 集成测试:三 broker,所有 follower 都健康追上 → acks=all 写入成功
- 单测:`max(current_hw, new_hw_candidate)` 单调性(HW 倒退场景):
  - ISR={A,B},LEO 都 100,HW=100
  - C 追到 99 加入 ISR → HW 仍 100(不倒退)→ 验证 I6
- 单测:|ISR|=replicas 全员未追上 → acks=all 阻塞至 timeout

**预估**:中(~600 行)

---

### T11c:HW checkpoint 持久化(防 HW 倒退跨重启)

**目标**:`local_hw` 持久化到 checkpoint,进程重启不丢。

**前置**:T11b

**改动**:
- broker 后台周期 fsync `local_hw` 到 `replication-offset-checkpoint`(对齐 Kafka)
- broker 重启时加载 checkpoint 初始化 `local_hw`(不读则起为 0)
- broker 收到 LeaderAndIsr 时:`local_hw = max(local_hw, persisted)`
- rocksdb / filesegment 各自实现 checkpoint

**不做**:
- memory 引擎不实现(memory 数据本身不持久,follower 重启等价于全新副本)

**验收**:
- 集成测试:三 broker,HW 推到 100 → kill follower → 重启 → 本地 HW 仍是 100
- 单测:HW checkpoint 写入失败时,broker 不接受写入(防止 HW 倒退)

**预估**:中(~400 行)

---

## E 组:控制面响应

> E 组中 **T12 + T13a + T13b + T13c 是关键路径**。T13a 与 T3 必须一起合并(原子组,见顶部说明)。

### T12:ISR 维护后台(shrink/expand 触发)

**目标**:leader 后台周期检查 follower_progress,触发 ISR shrink/expand。

**前置**:T2、T9、T11b

**改动**:
- 新建 `storage-engine/src/isr/manager.rs`:
  - leader 后台扫 `follower_progress`,按 §7.1 / §7.2 判定:
    - shrink:`lag_ms > replica_lag_time_max_ms` → 调 `UpdateSegmentIsr(new_isr = isr - {node_id})`
    - expand:满足 §7.2 全部条件 → 调 `UpdateSegmentIsr(new_isr = isr + {node_id})`
  - 调用时携带 `leader_epoch / requester_broker_epoch / expected_segment_epoch`
- T9 的 `last_caught_up_ts` 必须严格按 §6.4 维护(本 task 顺手补强)
- ISR 变更后 hw_watcher 触发(因为 ISR 缩小后 HW 可能能推进)

**不做**:
- broker 端 LeaderAndIsr 响应只更新 isr_cache(由 T3 的 broker handler 处理),不切 role(留 T13a)

**验收**:
- 集成测试:杀 follower → 30s 后(`replica_lag_time_max_ms`)被踢出 ISR
- 集成测试:follower 恢复 → 追上后自动 expand 回 ISR
- 单测:并发的 shrink + expand 通过 segment_epoch CAS 串行化,不丢请求
- 单测:meta 拒绝陈旧 epoch 请求后,manager 拉新 meta 重试

**预估**:中(~500 行)

---

### T13a:数据面响应 LeaderAndIsr(role 状态机)

**目标**:broker 端实现完整的 LeaderInitializing / LeaderActive / FollowerInitializing / FollowerActive 状态转换(§8.1)。

**前置**:T3、T7、T11a

**与 T3 一起原子合并**(否则 T3 上线后 broker 不切 role,违反 I4)。

**改动**:
- `SegmentReplicaState.role` 用 `Mutex<ReplicaRole>` 或 `ArcSwap`
- LeaderAndIsr handler 完整实现 §8.1 三个 case:
  - case 1(成为 leader):进入 LeaderInitializing → 取消在途 producer 请求 → 停 fetcher → `LeaderEpochCache.assign + fsync` → 重置 follower_progress → 转 LeaderActive
  - case 2(成为/继续 follower):进入 FollowerInitializing → 停旧 fetcher → 取消在途 producer 请求 → **预留 truncation 钩子**(具体实现 T13c) → 转 FollowerActive
  - case 3(从 replicas 移除):停所有 → 卸载 state

**不做**:
- 真正的 OffsetsForLeaderEpoch truncation 在 T13c
- 暂时:case 2 用占位的"truncate 到 0 后全量重拉"路径(明确标 TODO,T13c 替换)

**验收**:
- 集成测试:杀 leader → 新 leader 走完 LeaderInitializing 才接写
- 单测:`LeaderEpochCache.fsync` 失败时不转 LeaderActive
- 单测:case 2 占位路径:follower 切换到新 leader 后能继续工作

**预估**:大(~800 行)

---

### T13b:Fetcher 管理与角色切换协作

**目标**:fetcher 任务的启停生命周期与 role 切换协调。

**前置**:T9、T13a

**改动**:
- `storage-engine/src/isr/fetcher_manager.rs`:
  - per `(shard, segment_seq)` 启动/停止 fetcher
  - fetcher task 优雅退出(在 fetch round-trip 完成边界退,避免落盘到一半)
  - 与 T13a 的 role 切换协作:role → Follower 时启 fetcher,role → Leader 时停 fetcher

**不做**:
- 跨 segment seal 时的 fetcher 切换(filesegment 专属,留 T15)

**验收**:
- 集成测试:role 反复切换(L→F→L→F),fetcher 不泄漏不僵死
- 单测:fetcher 任务取消时不丢未持久化的 append(在 append + fsync 边界退)

**预估**:中(~400 行)

---

### T13c:OffsetsForLeaderEpoch 替换占位 truncation(I9 完整闭环)

**目标**:把 T13a case 2 里的占位 truncation 替换为真正的 KIP-101 协议。

**前置**:T10、T13a、T13b

**改动**:
- T13a case 2 的 truncation 钩子调用 §9.2 完整流程:
  - 发 OffsetsForLeaderEpoch 给新 leader
  - 处理所有 5 种情况(§9.2 step 3)
  - truncate + 修剪 LeaderEpochCache + fsync
  - 然后启动 fetcher

**不做**:
- 仍不实现 KIP-227 incremental fetch / KIP-219 throttling
- 仍不实现 unclean leader

**验收**:
- **§12.2 KIP-101 经典丢数据场景回归用例**:三 broker,leader 切换 + 旧 leader 重启,验证旧 leader 的脏数据被正确 truncate,**不会污染新 leader 的日志**
- §12.13 新 leader 上任未完成持久化即崩溃:重启后 fence
- §12.14 zombie broker:新进程发出 ISR 请求,旧进程的请求被 StaleBrokerEpoch 拒
- §12.12 HW 倒退场景(I6 单调性)

**预估**:大(~700 行,含演练)

---

### T14:故障演练用例集

**目标**:把 isr.md §12 的全部场景写成自动化回归测试。

**前置**:T11c, T13c

**改动**:
- `tests/isr/` 新建:
  - 每个场景一个测试文件,模拟触发 + 验证避免机制有效
  - 用 `tokio::time` mock 时间,网络分区用 mocked transport
- 至少覆盖 §12.1 ~ §12.16 全部 16 个场景

**预估**:大(~1500 行测试代码)

---

### T15(可选):filesegment 引擎接入

**目标**:filesegment 引擎实现 ReplicaLog + LeaderEpochCache 持久化(sidecar 文件)。

**前置**:T10 完成后,memory/rocksdb 全协议跑通

**改动**:
- `impl ReplicaLog for FileSegment`
- LeaderEpochCache 用 `*.leader-epoch-checkpoint` 文件(对齐 Kafka)
- 跨 segment seal 时 fetcher 切换逻辑(§12.16)
- segment seal up 原子提案(meta-service)

**预估**:大(~1200 行)

---

## 任务总览表

| Task | 名称 | 组 | 前置 | 预估 | 原子合并组 |
|---|---|---|---|---|---|
| T1 | EngineSegment / Config 字段扩展 + broker_epoch 注册 | A | — | 中 | T1+T2 |
| T2 | raft op `UpdateSegmentIsr` + 三重 fence(I3) | A | T1 | 中 | T1+T2 |
| T3 | SegmentLeaderAndIsr 广播 + broker epoch 缓存 | A | T1 | 中 | T3+T13a |
| T4 | ReplicaLog trait | B | — | 小 | — |
| T5 | memory ReplicaLog 实现 | B | T4 | 小 | — |
| T6 | rocksdb ReplicaLog 实现 | B | T4 | 中 | — |
| T7 | LeaderEpochCache 持久化 | B | T6 | 中 | T7+T10+T13c |
| T8 | long-poll fetch RPC + 完整 epoch 校验(I15) | C | T3, T4, T5/T6 | 大 | — |
| T9 | follower fetcher 循环 + last_caught_up_ts(§6.4) | C | T7, T8 | 中大 | — |
| T10 | OffsetsForLeaderEpoch RPC + handler | C | T7, T8 | 大 | T7+T10+T13c |
| T11a | 写入路径 epoch 校验 + 原子性(I4) | D | T3, T6, T7 | 大 | T11a+b+c |
| T11b | HW 推进(I6 单调) + acks=all 等待 | D | T11a | 中 | T11a+b+c |
| T11c | HW checkpoint 持久化 | D | T11b | 中 | T11a+b+c |
| T12 | ISR 维护后台(shrink/expand) | E | T2, T9, T11b | 中 | — |
| T13a | LeaderAndIsr role 状态机(§8.1) | E | T3, T7, T11a | 大 | T3+T13a |
| T13b | Fetcher 管理与 role 切换协作 | E | T9, T13a | 中 | — |
| T13c | OffsetsForLeaderEpoch 替换占位 truncation(I9 闭环) | E | T10, T13a, T13b | 大 | T7+T10+T13c |
| T14 | §12 全场景故障演练用例集 | — | T11c, T13c | 大 | — |
| T15 | filesegment 引擎接入(可选) | — | T10, T13c | 大 | — |

### 里程碑

| 里程碑 | 完成 Task | 能做什么 | 还不能做什么 |
|---|---|---|---|
| M1:元数据就位 | T1+T2 | meta-service 已能接受 ISR 变更请求(虽然还没人发) | 数据面什么都做不了 |
| M2:本地存储就位 | T4+T5+T6+T7 | 单进程能读写本地副本日志 + 持久化 LeaderEpochCache | 没有跨节点复制 |
| M3:首次能见副本同步 | M1+M2+T3+T8+T9+T13a+T13b | 三节点 follower 能追上 leader,leader 切换 role 切换正确 | 没 truncation(脏日志会停留),没 acks=all,ISR 永远=replicas |
| M4:协议正确性闭环 | M3+T10+T11(全)+T12+T13c | **完整协议**:KIP-101 truncation、acks=all、ISR 自动收缩、segment_epoch CAS 全部生效 | 没 §12 全套故障演练验证 |
| M5:production-ready | M4+T14 | §12.x 16 个异常场景全部回归通过 | filesegment 未接入 |
| M6:全引擎 | M5+T15 | filesegment 也走 ISR 协议 | — |

**关键路径**(决定最早能跑到 M4 的依赖链):
- 控制面链:T1 → T2 → T12
- 数据面链:T4 → T6 → T7 → T8 → T9 → T10 → T13c
- 收口:T3+T13a → T11a → T11b/c → T13c

**最强建议**:M3 → M4 之间不要试图"部分上线"。M3 在测试环境跑得通是因为 ISR 永远=replicas,没人故障;一旦上生产 ISR 收缩或 leader 切换发生 → 立刻进入未定义行为。M3 → M4 必须一次性切到 M4。

---

## 不在本拆分中的事项

下列内容**不在 task 拆分中**,因为它们不是协议本身,或属于协议明确不实现:

- filesegment 引擎接入(协议外的引擎适配,可独立做,等 T4-T10 跑通后单独立项)
- Tiered Storage、Observer、ELR 等(isr.md §16 划出)
- 监控指标 / 告警 / 运维工具(独立工作流,不在 ISR 协议范围)
- producer 端幂等性 / exactly-once(isr.md §16 划出)
- consumer 从 follower 读(isr.md §16 划出)
