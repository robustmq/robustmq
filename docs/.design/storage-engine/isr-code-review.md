# ISR 协议代码 Review 文档

> 本文档列出 ISR 协议的**核心链路**和**每条链路的 review 关注点**。
> 用于：① 对照实际代码验证协议正确性 ② 对照设计文档找文档-代码偏差。

---

## 核心链路一览

| 链路 | 入口 | 核心文件 | 关键不变式 |
|---|---|---|---|
| **L1 写入路径** | `batch_write` | core/write.rs / core/segment.rs / isr/follower.rs | active-segment leader check / I6(HW 单调) |
| **L2 follower 复制** | `fetch_round` | isr/fetcher.rs / isr/handle_fetch.rs | I9(KIP-101 截断) / I15(fetch fence) |
| **L3 leader 选举切换** | `apply_leader_and_isr` | isr/apply.rs / core/dynamic_cache.rs | I3(epoch 单调) / I11(epoch cache) |
| **L4 HW 推进** | `advance_hw` | isr/follower.rs / core/offset.rs | I6(HW 不降) |
| **L5 ISR 维护** | `start_isr_manager_thread` | isr/isr_manager.rs | ISR 收缩/扩张正确性 |
| **L6 reconcile 兜底** | `start_metadata_reconcile_thread` | reconcile.rs | 广播丢失后状态追平 |
| **L7 空 ISR 恢复** | `recover_unavailable_segments_on_node_join` | meta-service/core/isr_recovery.rs | 数据不丢，无双 leader |
| **L8 启动恢复** | `recover_local_segments` | isr/recover.rs | 崩溃后 epoch cache 一致 |

---

## L1：写入路径

### 调用链
```text
batch_write (write.rs)
  ├─ segment_validator (leader 身份 + segment 状态)
  ├─ write_{memory|rocksdb|segment}_to_local
  ├─ advance_hw (isr/follower.rs) ← 触发点 1
  └─ acks==-1 → wait_for_hw (isr/follower.rs)
```

### 关注点

#### R1.1 leader 身份判断
- `segment_validator` and `batch_write` use the cached active segment; local writes continue only when `broker_config().broker_id == active_segment.leader`.
- The cache is updated from metadata notifications and reconcile. There is no separate runtime `LeaderActive`/`LeaderInitializing` state to check.
- **Implementation limitation**: a notification/cache update race is therefore bounded by the cached segment view rather than by a role state machine.

#### R1.2 HW 推进时机
- leader 写完本地后立即调 `advance_hw`（write.rs:101-108）
- 单 ISR=[leader]：`committable_hw` = leader_leo，HW 立即推进 ✓
- 多 ISR：follower 无 progress → 跳过（不约束）→ HW 推进 ✓（修复 C2 后）
- **问题**：ISR 成员有 progress 且 leo=0 时，HW=0，acks=all 等待正确吗？（`acks_all_times_out` 测试覆盖）

#### R1.3 epoch 过滤
- 当前 `committable_hw` 只按 ISR 成员的 recorded LEO 计算，不保存或过滤 follower leader epoch。
- 这 differs from the original design proposal, which required an epoch match; the behavior should be treated as an implementation limitation, not as an implemented invariant.

#### R1.4 acks=all 超时参数
- `WriteReqBody` carries `timeout_ms`; `batch_write` uses it and falls back to `DEFAULT_WRITE_TIMEOUT_MS` when it is zero.

#### R1.5 min_in_sync_replicas 校验
- `batch_write` rejects `acks=-1` when `active_segment.isr.len() < shard.config.min_in_sync_replicas` (`core/write.rs:72-80`).
- The older review note claiming that this check was missing is stale.

---

## L2：follower 复制路径

### 调用链
```text
ReplicaFetcherThread::run (fetcher.rs)
  └─ fetch_round
       ├─ 按 leader 分组，构造 FetchReqBody{shards}
       ├─ transport.fetch(leader_node, req)
       └─ apply_shard_resp(resp)
            ├─ OffsetOutOfRange → clear(retention 落后) | truncate_after_fence
            ├─ FencedLeaderEpoch → truncate_after_fence
            │    ├─ transport.offsets_for_leader_epoch(leader)
            │    ├─ end_offset_epoch<0 → clear + cache.clear (memory 全量重拉)
            │    └─ else → truncate_to(end_offset-1) + truncate_from_end_by_epoch
            │         + 更新 state.current_leader_epoch (A2 bug 修复)
            └─ None → append_at, then assign a newer response epoch
```

### 关注点

#### R2.1 current_leader_epoch 更新（A2 修复验证）
- truncate_after_fence 成功后，`state.current_leader_epoch = resp.current_leader_epoch`（fetcher.rs:280,291）
- **需要验证**：下一轮 fetch_round 用的是新 epoch，不会再 Fenced

#### R2.2 truncate_to 的 exclusive 语义
- `truncate_to(end_offset - 1)` 保留到 end_offset-1，LEO=end_offset
- end_offset=0 时 checked_sub→None→clear（语义正确）
- **需要验证**：end_offset 是 exclusive 边界，`end_offset_for(epoch)` 返回的是下一 epoch 的 start（也是 exclusive）

#### R2.3 case 2 成为 follower 时的截断
- `apply_as_follower` sets `needs_truncation` when the leader epoch changes (`isr/apply.rs:156-166`).
- `fetch_round` runs `truncate_after_fence` for those segments before constructing normal fetch requests (`isr/fetcher.rs:122-149`).
- The old note that the follower always starts fetching first is stale; same-epoch assignments intentionally do not repeat the check.

#### R2.4 TOCTOU（contains_key → append_at 之间 segment 被 remove）
- apply_shard_resp 开头 contains_key，末尾 append_at，中间无锁
- role 切换（reconcile/parse_segment）可能 remove_segment
- 已有分析：低危，截断会修复脏尾部，非永久错误

#### R2.5 Memory 引擎全量重拉路径
- 当 `follower_leader_epoch > cache.latest_epoch()` 时，`handle_epoch.rs` 返回 `truncate_epoch=-1` 和当前 `leader_leo`。
- `fetcher.rs` 收到该 sentinel 后清空本地日志和 epoch cache；下一轮从本地 offset 0 开始 Fetch，直到追上该 LEO。
- 这是 memory 消息数据易失性的实现语义；文档 §9.5 已按此行为说明，不应再描述为从 `leader_log_start` 直接开始。

---

## L3：leader 切换（apply_leader_and_isr）

### 调用链（当前实现）
```text
parse_segment (dynamic_cache.rs)
  ├─ is_outdated_segment_notify [(segment_epoch, leader_epoch) 字典序]
  ├─ apply_leader_and_isr (isr/apply.rs)
  │    ├─ !segment.is_replica() → remove fetcher + replica state
  │    ├─ leader → remove fetcher, assign epoch cache, clear/seed progress
  │    └─ follower → load epoch cache, remove then assign fetch state
  └─ set_segment (cache 更新)
```

### 关注点

#### R3.1 stale 守卫扩展到 segment_epoch
- `core/dynamic_cache.rs::is_outdated_segment_notify` 丢弃 `segment_epoch` 更旧，或 segment epoch 相同且 `leader_epoch` 更旧的通知。
- `apply_leader_and_isr` 本身不再维护单独的 role/epoch state machine；同版本重复 apply 依靠 epoch-cache 的幂等 assign、progress 的 clear/seed 条件和 fetcher assignment 的覆盖语义保持安全。

#### R3.2 stale 过滤与 dynamic_cache.rs
- `parse_segment` 里有 `is_stale_segment_notification`
- `apply_leader_and_isr` is the application path; it does not implement a second role-state lock or independent role epoch.
- The notification filter compares `(segment_epoch, leader_epoch)` lexicographically; reconcile uses the same cached segment epoch before applying updates.

#### R3.3 case 2 截断时机
- `isr/apply.rs` 将 leader epoch 变化标记为 `needs_truncation`。
- `isr/fetcher.rs::fetch_round` 在构造普通 fetch 前执行 `truncate_after_fence`；同 epoch 的重复 assignment 不重复截断。

---

## L4：HW 推进

### 调用链
```text
advance_hw (isr/follower.rs:79-99)
  ├─ get_segment_replica → committable_hw(isr, leader_id, leader_leo)
  │    └─ 跳过无 follower_progress 记录的 ISR 成员（C2 修复）
  ├─ update_high_watermark_offset [只升不降]
  └─ send hw_watcher [唤醒 wait_for_hw]

触发点 1: batch_write 写完本地后（write.rs:101）
触发点 2: handle_fetch 更新 follower_progress 后（isr/handle_fetch.rs）
```

### 关注点

#### R4.1 两个触发点是否足够
- leader 写完 → 推 HW（如果 ISR 只有 leader，立即提交）
- follower fetch 后 → 推 HW（follower 追上后提交）
- **活性问题**：follower 滞后且停止 fetch 时，HW 不推进 → acks=all 超时
- ISR manager 会在 lag 超时后踢出 follower；下一次维护周期（默认 1s，可配置）会重新提议 ISR，随后 HW 可重新推进。

#### R4.2 leader_epoch 过滤缺失（已知实现限制）
- The current code does not keep `last_known_leader_epoch` in `FollowerProgress`; the design's epoch filter is therefore not part of HW computation.

#### R4.3 HW 持久化
- `advance_hw` persists a forward-only watermark through `ShardOffset::save_high_watermark_offset` and notifies the watcher.
- Startup recovery reads the shard offset state through `ShardOffset`; the older note that HW always restarts from zero is stale.

---

## L5：ISR 维护

### 调用链
```text
start_isr_manager_thread (isr/isr_manager.rs)
  └─ maintain_once (默认周期 1s，可配置)
       ├─ 遍历 cache_manager.leader_segments
       ├─ 检查 broker_config().broker_id == segment.leader
       ├─ compute_new_isr(state, current_isr, replicas, leader_id, leader_leo, lag_ms, now)
       │    ├─ leader 始终在 new_isr
       │    ├─ 每个非 leader replica：
       │    │    ├─ caught_up = (last_fetch_ts within lag_max)
       │    │    └─ caught_up 时加入 new_isr（expand）；否则不加（shrink）
       │    └─ new_isr == current_isr → None（无变化）
       └─ new_isr != None → propose_isr → update_segment_isr grpc
```

### 关注点

#### R5.1 lag 判定字段
- The current `FollowerProgress` uses `last_fetch_ts`; it is refreshed only when `fetch_offset >= leader_leo`.
- There is no `last_caught_up_ts` field in the implementation.

#### R5.2 expand 条件
- `compute_new_isr` ignores its `_leader_leo` argument and uses only the `last_fetch_ts` lag window.
- A follower with no progress entry is not added; a recent caught-up fetch is sufficient for both retaining and adding ISR membership.

#### R5.3 ISR 最小成员数保护
- `compute_new_isr` 可能返回 `new_isr=[leader]`（只剩 leader）。
- meta 侧校验 ISR 非空、包含 leader 且属于 replicas；broker 写入路径另外在 `acks=-1` 时校验 `min_in_sync_replicas`。
- 因此 ISR shrink 可以继续到单 leader，但 `acks=all` 会在 ISR 小于配置下限时拒绝。

---

## L6：reconcile 兜底

### 调用链
```text
start_metadata_reconcile_thread (reconcile.rs)
  └─ 每 metadata_reconcile_interval_ms 一轮（默认 30s）
       ├─ urgent：先处理 fetch 返回 UnknownLeaderEpoch 标记的 segment
       ├─ active：每个 shard 只拉 active segment 的 meta 元数据
       ├─ segment_epoch 变更或 follower fetcher 缺失 → apply_segment
       └─ apply_segment：set_segment + apply_leader_and_isr
```

### 关注点

#### R6.1 urgent reconcile
- `handle_fetch` marks a segment when it returns `UnknownLeaderEpoch`.
- `reconcile_urgent` consumes that marker before the regular active-segment scan; the old note that only the periodic path exists is stale.

#### R6.2 reconcile 与 parse_segment 并发
- reconcile 调 `set_segment` + `apply_leader_and_isr`。
- `parse_segment` 也调 `set_segment` + `apply_leader_and_isr`。
- 当前没有独立 `state_lock`；DashMap 提供并发容器安全，epoch 过滤负责丢弃旧通知。
- **已分析**：set_segment 和 apply 之间有 TOCTOU，reconcile 用旧 snapshot 可能重复 apply → P2 修复（segment_epoch 守卫）后影响降低

#### R6.3 ReconcileSegmentMetadata 是只读 grpc（验证）
- meta-service 端实现：读 `cache_manager.get_segment`，不走 raft
- ✓ 符合设计（纯读，无副作用）

---

## L7：空 ISR 恢复

### 调用链
```text
register_node_by_req (cluster.rs)
  └─ recover_unavailable_segments_on_node_join (meta-service/core/isr_recovery.rs)
       └─ 遍历 last_known_isr.contains(node_id) 的 Unavailable segments
            └─ try_recover_segment
                 ├─ per-segment mutex [防双 goroutine]
                 ├─ 重读 segment，Unavailable 才继续
                 ├─ 查 node_list 中 last_known_isr 成员
                 ├─ broker_query_replica_leo (QueryReplicaLeo grpc) 并发查
                 ├─ elect_recovery_leader (LEO 最大 + available)
                 ├─ 构造 new_segment (leader/isr/status=Write/last_known_isr 清空)
                 ├─ sync_save_segment_info (SetSegment raft op)
                 └─ send_notify_by_set_segment
```

### 关注点

#### R7.1 等待窗口缺失（规格与实现差异）
- 原始规格要求"等待 unavailable_recovery_wait_ms 收集更多副本回复"；该配置和等待窗口未实现
- 代码通过 broker gRPC `QueryReplicaLeo` 立即收集能响应的节点，超时的节点被跳过
- **影响**：可能遗漏 LEO 更大的节点 → 选出 LEO 次大的 leader → 数据不丢（已提交在 HW 内），但未提交数据被丢弃更多
- **建议**：加等待窗口，或多次重试后选

#### R7.2 SetSegment 无 CAS（已知 D1/E1 bug，已修复 per-segment mutex）
- per-segment mutex 防止同一 segment 的双 goroutine
- mutex 是进程内的，跨 meta 节点不共享
- **风险**：不同 meta 节点（raft follower forward）各自收到注册请求，各自 spawn goroutine
- raft 的串行提交是最终一致的保证

#### R7.3 last_known_isr 什么时候被写
- `compute_segment_after_leader_failure`（`segment_leader.rs`）：ISR 缩空时 `last_known_isr = segment.isr.clone()`
- 空 ISR 时 `last_known_isr` 非空才有意义——已验证

---

## L8：启动恢复

### 调用链
```text
StorageEngineServer::start (lib.rs)
  └─ recover_local_segments (isr/recover.rs) [async]
       └─ recover_one_segment (每个本地 replica)
            ├─ ShardOffset::get_shard_offsets（读取并钳制 persisted HW）
            ├─ LeaderEpochCache.load + recover_leader_epoch_cache(latest, earliest)
            │    ├─ truncate_from_end(latest)    [删超过 latest 的虚假 epoch]
            │    └─ truncate_from_start(earliest) [删低于 earliest 的旧 epoch]
            └─ 未找到 offset state 时写入默认 ShardOffsetState
```

### 关注点

#### R8.1 HW 持久化
- `advance_hw` 通过 `ShardOffset::save_high_watermark_offset` 将前进的 HW 写入 storage-engine RocksDB column family。
- 启动恢复从 `ShardOffset` 读取并将 HW 限制在 latest offset 以内；不存在 offset state 时才使用默认值。
- 这不是独立的异步 checkpoint 任务，也不使用文档中原先规划的 checkpoint 文件。

#### R8.2 startup/reconcile ordering
- `recover_local_segments` runs before daemon threads and the TCP server in `StorageEngineServer::start`.
- `recover.rs` does not use a role `state_lock`; concurrent notification ordering is handled by the existing cache/epoch checks.
- A live startup race still needs cluster validation because this audit does not run a multi-broker environment.

---

## 已知偏差汇总

| # | 偏差 | 位置 | 影响 | 优先级 |
|---|---|---|---|---|
| D1 | 写入路径无独立 `ReplicaRole` 校验 | `core/write.rs` | current leader/segment validation is cache-based; no role enum exists | 中 |
| D2 | HW 计算无 leader_epoch 过滤 | isr/follower.rs::committable_hw | 与原设计提案不同；当前结构无该字段，可能影响切换后的活性 | 低 |
| D3 | leader epoch 变化时首次 fetch 前执行 truncation | isr/apply.rs + isr/fetcher.rs | 当前实现已在普通 fetch 前完成检查 | — |
| D4 | ~~被动 reconcile 缺失~~ | reconcile.rs | ✅ 已修：fetch 收到 `UnknownLeaderEpoch` 时 `mark_reconcile_needed`，reconcile 任务在下次 tick 优先处理，限频 1s/segment | — |
| D5 | 空 ISR 恢复无等待窗口 | meta-service/core/isr_recovery.rs | 可能漏掉晚上线的高 LEO 副本 | 中 |
| D6 | `acks=all` 使用请求 `timeout_ms` | protocol.rs / core/write.rs | 已按请求超时，零值回退默认值 | — |
| D7 | WriteReqBody 已包含 timeout 和 leader epoch | protocol.rs | 旧 review finding 已解决 | — |
| D8 | `min_in_sync_replicas` 写入路径校验 | core/write.rs | `acks=-1` 低于最小 ISR 时拒绝 | — |
| D9 | HW 持久化 | isr/follower.rs / core/offset.rs | 已通过 storage-engine RocksDB column family 持久化 | — |
| D10 | memory 全量重拉返回 leader LEO | isr/handle_epoch.rs:86-100 | 当前实现行为，文档已说明 | — |
| D11 | shrink 使用 `last_fetch_ts` | isr/isr_manager.rs | 当前结构无 `last_caught_up_ts` | — |
| D12 | 空 ISR 恢复的跨 meta 节点竞态仍非原子 CAS | meta-service/core/isr_recovery.rs | 进程内锁不能覆盖不同 meta 节点的并发恢复；写后校验只能发现部分竞态 | 中 |

---

## Review 结论

**协议核心（复制、截断、epoch 管理）实现状态**：复制、截断和 epoch 处理已有进程内覆盖；但本文的早期 role/lock 设计并非现有 runtime API，跨节点故障恢复仍缺少集群级验证。

**主要剩余问题**：
1. **D1**：写入路径依据缓存中的 active segment leader 身份，未显式读取独立 role 状态；当前实现也没有这样的独立 role 状态。
2. **D2**：HW 计算没有原设计提案中的 leader-epoch 过滤，因为运行时 `FollowerProgress` 不保存该字段。
3. **D5/D12**：空 ISR 恢复窗口和跨节点恢复 CAS 仍需要单独验证。

**文档同步状态**：
- §5.3、§6.5、§7.2、§9.4 已按当前 `FollowerProgress`、HW、epoch-cache 和 ISR maintenance 实现更新；历史伪代码仍只用于设计背景。
