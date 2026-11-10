# Storage Engine ISR 概览

> **一句话**：给 storage-engine 的三种引擎（memory / rocksdb / filesegment）加一套副本同步协议，让已 ack 的数据在节点故障时不丢失，故障切换在秒级完成。协议形态对齐 Kafka KIP-101+ 稳定版本。

---

## 1. 核心模块

![架构图](diagrams/isr-01-arch.png)

| 模块 | 职责 |
|---|---|
| `core/dynamic_cache.rs` + `isr/apply.rs` | 接收 LeaderAndIsr 元数据更新，按 `(segment_epoch, leader_epoch)` 过滤旧通知，并切换 leader/follower fetcher |
| `core/cache.rs` + `isr/follower.rs` | 运行时副本进度：`SegmentReplicaState` 是 `DashMap`，`FollowerProgress` 保存 broker epoch、LEO 和 `last_fetch_ts` |
| `handle_fetch.rs` | Leader 端 fetch handler：epoch/broker fence + 更新 follower progress + 推进 HW |
| `fetcher.rs` | Follower 端拉取线程：按 leader 批量 fetch → append → 更新 local HW |
| `fetcher_manager.rs` | 线程池管理：`leader_node % N` 路由，负责 start / stop fetcher |
| `leader_epoch.rs` | `LeaderEpochCache`：`epoch → start_offset` 映射，持久化到 RocksDB，truncation 的基准 |
| `handle_epoch.rs` | KIP-101 `OffsetsForLeaderEpoch` RPC 处理端：返回某 epoch 在本地的 end_offset |
| `follower.rs` | HW 推进：按 ISR follower LEO 计算，持久化并唤醒 `acks=all` 等待者 |
| `isr_manager.rs` | ISR 维护后台任务：按配置周期扫描 follower progress，收缩失去近期 caught-up fetch 的副本，加入近期 caught-up 的副本 |
| `reconcile.rs` | 元数据兜底：定时 + urgent 两路，批量拉 meta 对比 segment_epoch，补齐本地遗漏的 LeaderAndIsr |
| `recover.rs` | 启动恢复：加载 offset state，按本地 LEO/log start 修剪 `LeaderEpochCache` |
| `log.rs` | `ReplicaLog` trait：三引擎共享的本地存储抽象（append / truncate / latest_offset） |

---

## 2. 核心设计

### 2.1 三层 Epoch — 防止 Zombie 写入

ISR 协议围绕三个独立的计数器运转，每类请求必须携带对应 epoch，接收方校验：

| Epoch | 递增时机 | 作用 |
|---|---|---|
| `leader_epoch` | leader 切换 | KIP-101 truncation 基准；follower 对齐日志用 |
| `segment_epoch` | ISR / leader / replicas 任一变化 | UpdateSegmentIsr 的 CAS 防止并发覆盖 |
| `broker_epoch` | broker 进程注册 | Fence 同 node_id 的旧进程残留请求 |

### 2.2 High Watermark — 定义"已提交"

```
HW = max(old_HW, min(leader_LEO, recorded follower LEOs in ISR))
```

- `offset < HW` 的数据**永不丢失**；仅有 progress 记录的 ISR followers 约束本次 min，缺失记录不会阻塞 HW
- HW 推进发生在 leader 本地写入完成后，也发生在 leader 处理 follower fetch 后；每次推进都会持久化 shard 的 high watermark 并唤醒等待者
- 当前 `FollowerProgress` 只记录 `follower_broker_epoch`、`leo` 和 `last_fetch_ts`；HW 计算不额外过滤 leader epoch
- **单调性**：外套 `max(old, new)`，扩 ISR 时新成员 LEO 低也不会让 HW 倒退

### 2.3 KIP-101 Truncation — 日志对齐

**禁止用本地 HW 截断**（Kafka 2017 前经典丢数据 bug）。唯一合法路径：

1. Follower 拿本地 LeaderEpochCache 的最新 epoch
2. 向新 leader 请求：`OffsetsForLeaderEpoch(my_epoch)` → 返回 `end_offset`
3. Follower `truncate_to(min(local_LEO, end_offset))`，修剪本地 cache
4. 从 `end_offset` 开始正常 fetch

`LeaderEpochCache` 持久化在 storage-engine 的 RocksDB column family。成为 leader 时，`isr/apply.rs::apply_as_leader` 幂等写入当前 epoch；代码没有独立的 `LeaderActive` runtime state。

---

## 3. 主要链路

### 3.1 写入 + 副本同步

![写入与复制](diagrams/isr-02-write-replicate.png)

当前实现：`core/write.rs::batch_write` 校验缓存中的 active segment，执行本地引擎写入，再调用 `advance_hw`；代码没有设计文档草案中的 shard/segment 双锁原子区。

### 3.2 Leader 切换

![Leader 切换](diagrams/isr-03-leader-switch.png)

新 leader 处理通知时由 `apply_as_leader` 持久化当前 epoch cache，并清理或 seed follower progress；follower 在 leader epoch 变化时由 `needs_truncation` 标记，`fetch_round` 会先执行 `OffsetsForLeaderEpoch`，然后才发普通 Fetch。

### 3.3 ISR 维护 + Reconcile 兜底

![ISR 维护与 Reconcile](diagrams/isr-04-maintain-reconcile.png)

- **isr_manager**（默认 1s，仅 leader）：只有在最近一次 fetch 已追上 leader LEO 的 follower 才会刷新 `last_fetch_ts`；超过 `replica_lag_time_max_ms` 的副本被移出 ISR，近期 caught-up 的副本可被加入
- **reconcile**（30s 间隔 + urgent 触发）：每 shard 只比对 active segment，批量一次 RPC，segment_epoch 变化才 apply，把遗漏的 LeaderAndIsr 通知补齐

---

## 4. 边界场景

| 场景 | 处理方式 |
|---|---|
| **ISR 缩为空**（所有副本全挂） | segment 标为 `Unavailable`，停止写入；节点上线后 `IsrRecovery` 按 LEO 择优恢复 |
| **Follower 日志比 leader 多**（旧 leader 遗留写） | fetch 返回 `OffsetOutOfRange` → 触发 `OffsetsForLeaderEpoch` + truncate，裁掉多余尾部 |
| **Leader 收到旧 leader_epoch 的 fetch** | 返回 `FencedLeaderEpoch`，follower 触发 truncation 流程 |
| **acks=all 等待超时** | 写入**保留**在 leader 日志，返回 `RequestTimedOut`；数据靠正常 fetch 流程自然消化，不丢失 |
| **LeaderAndIsr 通知丢失** | reconcile 定时兜底：每 30s 批量拉 meta active segment，发现 segment_epoch 变化则补做角色切换 |
| **同一节点进程重启（broker_epoch 变化）** | meta UpdateSegmentIsr 校验 broker_epoch，拒绝旧进程残留的 ISR 变更请求 |
| **扩 ISR 时新成员 LEO 低于 HW** | HW 外套 `max(old, new)` 保证单调，新成员不会拉低已提交水位 |
| **Memory 引擎重启** | 消息数据在进程重启后丢失；LeaderEpochCache 仍由 storage-engine 的 RocksDB column family 保存，空 epoch cache 的 follower 会清空本地数据并从 offset 0 重新拉取 |

---

## 5. 三引擎差异（控制面无感知）

| | Memory | RocksDB | FileSegment |
|---|---|---|---|
| segment 数 | 恒为 1 | 恒为 1 | 写满后递增 |
| LeaderEpochCache | storage-engine RocksDB column family | storage-engine RocksDB column family | storage-engine RocksDB column family |
| 重启等价语义 | 消息数据全新副本，全量重拉 | EpochCache truncation 对齐 | EpochCache truncation 对齐 |

协议代码与引擎解耦：ISR 控制面只调 `ReplicaLog` trait，引擎差异封闭在 trait 实现内。

---

## 6. 相关文档

- [isr.md](./isr.md) — 协议精确规格（16 条不变式 + 完整边界场景）
- [isr-roadmap.md](./isr-roadmap.md) — 开发里程碑与 task 拆分
