# ISR 协议代码 Review 文档

> 本文档列出 ISR 协议的**核心链路**和**每条链路的 review 关注点**。
> 用于：① 对照实际代码验证协议正确性 ② 对照设计文档找文档-代码偏差。

---

## 核心链路一览

| 链路 | 入口 | 核心文件 | 关键不变式 |
|---|---|---|---|
| **L1 写入路径** | `batch_write` | write.rs / hw.rs / state.rs | I4(epoch fence) / I6(HW 单调) |
| **L2 follower 复制** | `fetch_round` | fetcher.rs / fetch.rs | I9(KIP-101 截断) / I15(fetch fence) |
| **L3 leader 选举切换** | `apply_leader_and_isr` | role.rs / dynamic_cache.rs | I3(epoch 单调) / I11(epoch cache) |
| **L4 HW 推进** | `advance_hw` | hw.rs / state.rs | I6(HW 不降) |
| **L5 ISR 维护** | `start_isr_maintain_thread` | isr_maintain.rs | ISR 收缩/扩张正确性 |
| **L6 reconcile 兜底** | `start_metadata_reconcile_thread` | reconcile.rs | 广播丢失后状态追平 |
| **L7 空 ISR 恢复** | `on_node_online` | isr_recovery.rs | 数据不丢，无双 leader |
| **L8 启动恢复** | `recover_local_segments` | startup.rs | 崩溃后 epoch cache 一致 |

---

## L1：写入路径

### 调用链
```text
batch_write (write.rs)
  ├─ segment_validator (leader 身份 + segment 状态)
  ├─ write_{memory|rocksdb|segment}_to_local
  ├─ advance_hw (hw.rs) ← 触发点 1
  └─ acks==-1 → wait_for_hw (hw.rs)
```

### 关注点

#### R1.1 leader 身份判断
- `conf.broker_id == active_segment.leader`（write.rs:63）
- 权威来源：`active_segment` 从 cache 读，由 meta 推送更新
- **问题**：role 状态机（LeaderActive/LeaderInitializing）和 meta segment.leader 之间有窗口——role 还是 LeaderInitializing 但已经能写？
- **期望**：写入路径是否应该加 `role == LeaderActive` 检查？现在没有，靠 meta segment.leader 判断

#### R1.2 HW 推进时机
- leader 写完本地后立即调 `advance_hw`（write.rs:101-108）
- 单 ISR=[leader]：`committable_hw` = leader_leo，HW 立即推进 ✓
- 多 ISR：follower 无 progress → 跳过（不约束）→ HW 推进 ✓（修复 C2 后）
- **问题**：ISR 成员有 progress 且 leo=0 时，HW=0，acks=all 等待正确吗？（`acks_all_times_out` 测试覆盖）

#### R1.3 epoch 过滤
- 文档 §5.3 伪代码要求"只计入 `last_known_leader_epoch == self.leader_epoch` 的 follower"
- 代码 `committable_hw` 不过滤 epoch——**已知偏差**，epoch 陈旧的 follower 仍参与 HW
- 实际影响：leader 切换后，旧 epoch follower 的 leo 可能偏低，拉低 HW → acks=all 慢
- 不影响正确性（HW 只升不降保证），但影响活性

#### R1.4 acks=all 超时参数
- 代码用 `replica_fetch_max_wait_ms`（全局配置）
- 文档说用请求的 `timeout_ms`（WriteReqBody 里没有此字段）
- **已知偏差**：WriteReqBody 只有 `acks: i8`，无 `timeout_ms`

#### R1.5 min_in_sync_replicas 校验
- 代码无此检查（write.rs 无 `min_isr` 判断）
- 文档 §5.2 step 4 要求 acks=all 时先校验 `|ISR| >= min_in_sync_replicas`
- **已知偏差**：min_isr 字段存在（EngineShardConfig），但写入路径不用

---

## L2：follower 复制路径

### 调用链
```text
ReplicaFetcherThread::run (fetcher.rs)
  └─ fetch_round
       ├─ 按 leader 分组，构造 FetchReqBody{shards}
       ├─ transport.fetch(leader_node, req)
       └─ apply_shard_resp(resp)
            ├─ OffsetOutOfRange → clear(retention 落后) | 退避
            ├─ FencedLeaderEpoch → truncate_after_fence
            │    ├─ transport.offsets_for_leader_epoch(leader)
            │    ├─ end_offset_epoch<0 → clear + cache.clear (memory 全量重拉)
            │    └─ else → truncate_to(end_offset-1) + truncate_from_end_by_epoch
            │         + 更新 state.current_leader_epoch (A2 bug 修复)
            └─ None → assign cache epoch + append_at
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
- **已知偏差**：role.rs 的 follower 分支直接 `assign_segment`，未先做 OffsetsForLeaderEpoch
- 文档 §8.1 case 2 要求先 truncation 再启动 fetcher（I9 不变式）
- 实际效果：follower 启动后第一次 fetch，若有分叉数据，收到 FencedLeaderEpoch 后会 `truncate_after_fence`——**功能上等效，只是延迟一轮 fetch**
- 严格来说 I9 要求启动 fetch 前就 truncation，避免那一轮的 append_at 尝试

#### R2.4 TOCTOU（contains_key → append_at 之间 segment 被 remove）
- apply_shard_resp 开头 contains_key，末尾 append_at，中间无锁
- role 切换（reconcile/parse_segment）可能 remove_segment
- 已有分析：低危，截断会修复脏尾部，非永久错误

#### R2.5 Memory 引擎全量重拉路径
- follower_leader_epoch > leader.latest_epoch 时，leader 返回 `end_offset_epoch=-1, end_offset=leader_leo`
- 文档 §9.5 说应返回 `leader_log_start`，代码返回 `leader_leo`
- **区别**：若 leader_log_start < leader_leo（有数据），follower clear 后从 leader_leo fetch → **拉不到 log_start 到 leader_leo 之间的历史数据**
- 对 memory 引擎：重启后 clear 从头拉，从 0 开始 fetch，leader_leo 是正确目标 → 实际无问题
- 但文档描述的语义（返回 log_start）和代码（返回 leo）不一致，应修文档或说明

---

## L3：leader 切换（apply_leader_and_isr）

### 调用链
```text
parse_segment (dynamic_cache.rs)
  ├─ is_stale_segment_notification [(segment_epoch, leader_epoch) 字典序]
  ├─ set_segment (cache 更新)
  └─ apply_leader_and_isr (role.rs)
       ├─ 守卫：(leader_epoch, segment_epoch) 字典序 stale 检查
       ├─ !segment.is_replica() → Initializing + remove_segment
       ├─ segment.leader == broker_id (case 1 → leader)
       │    ├─ role = LeaderInitializing
       │    ├─ remove_segment
       │    ├─ LeaderEpochCache.load + assign(epoch, leo) [失败→回滚 role]
       │    ├─ leader_epoch_changed → reset_follower_progress (P2 修复)
       │    ├─ set_leader_epoch / set_segment_epoch
       │    └─ role = LeaderActive
       └─ else (case 2 → follower)
            ├─ role = FollowerInitializing
            ├─ LeaderEpochCache.load [失败→回滚 role]
            ├─ set_leader_epoch / set_segment_epoch
            ├─ remove_segment + assign_segment (直接启动 fetcher)
            └─ role = FollowerActive
```

### 关注点

#### R3.1 stale 守卫扩展到 segment_epoch（P2 修复验证）
- `(leader_epoch < local) OR (== AND segment_epoch < local)`
- 同 leader_epoch、同 segment_epoch 的重复通知不被守卫，会重入 apply
- case 1 leader 分支：LeaderEpochCache.assign 是幂等的（重复 assign 同 epoch 是 no-op），reset_follower_progress 有 `leader_epoch_changed` 保护
- case 2 follower 分支：set_leader_epoch、assign_segment 重复也是幂等的
- **结论**：重复 apply 安全

#### R3.2 stale 过滤与 dynamic_cache.rs 的双重检查
- `parse_segment` 里有 `is_stale_segment_notification`
- `apply_leader_and_isr` 里也有守卫
- 两层都用字典序——是否冗余？是否一致？
- 需要确认两处用的比较逻辑相同

#### R3.3 case 2 截断缺失（已知 TODO）
- 见 R2.3
- **建议**：在 role.rs 的 follower 分支加注释说明"truncation 在首次 fetch 的 FencedLeaderEpoch 处理中完成"

---

## L4：HW 推进

### 调用链
```text
advance_hw (hw.rs:20-36)
  ├─ get_segment_replica → committable_hw(isr, leader_id, leader_leo)
  │    └─ 跳过无 follower_progress 记录的 ISR 成员（C2 修复）
  ├─ update_high_watermark_offset [只升不降]
  └─ send hw_watcher [唤醒 wait_for_hw]

触发点 1: batch_write 写完本地后（write.rs:101）
触发点 2: fetch_one_shard 更新 follower_progress 后（fetch.rs）
```

### 关注点

#### R4.1 两个触发点是否足够
- leader 写完 → 推 HW（如果 ISR 只有 leader，立即提交）
- follower fetch 后 → 推 HW（follower 追上后提交）
- **活性问题**：follower 滞后且停止 fetch 时，HW 不推进 → acks=all 超时
- ISR maintain 会在 lag 超时后踢出 follower → HW 重新推进（5s 内）

#### R4.2 leader_epoch 过滤缺失（文档偏差）
- 文档 §5.3 要求过滤陈旧 epoch 的 follower
- 代码不过滤——已知偏差，影响活性非正确性

#### R4.3 HW 持久化缺失（T11c 未做）
- `update_high_watermark_offset` 只更新内存，不写 rocksdb
- 重启后 HW 从 0 重爬（安全，只是 acks=all 在追上前超时）
- T11c "HW 异步 checkpoint" 是已知未做项

---

## L5：ISR 维护

### 调用链
```text
start_isr_maintain_thread (isr_maintain.rs)
  └─ maintain_once (周期 5s)
       ├─ 遍历 segment_replica_states [LeaderActive]
       ├─ 检查 conf.broker_id == segment.leader
       ├─ compute_new_isr(state, current_isr, replicas, leader_id, leader_leo, lag_ms, now)
       │    ├─ leader 始终在 new_isr
       │    ├─ 每个非 leader replica：
       │    │    ├─ caught_up = (leo >= leader_leo) OR (lag <= lag_max)
       │    │    └─ caught_up 时加入 new_isr（expand）；否则不加（shrink）
       │    └─ new_isr == current_isr → None（无变化）
       └─ new_isr != None → propose_isr → update_segment_isr grpc
```

### 关注点

#### R5.1 lag 判定字段
- 代码用 `last_caught_up_ts`（follower 最后一次追上 leader LEO 的时间）
- 文档 §6.3 说"last_fetch_ts 超过阈值"——两者有区别
- `last_caught_up_ts` 更严格（必须真正追上，而非只是发了 fetch 请求）
- **建议**：文档更新为 `last_caught_up_ts`，语义更精确

#### R5.2 expand 条件
- 代码：`p.leo >= leader_leo`（leader 当前 LEO）
- 文档：follower LEO 追上 leader HW
- leader_leo >= leader_hw，所以代码条件更严（`leo >= leo` 比 `leo >= hw` 难）
- **实际效果**：follower 必须追到 leader 写入的最新位置才加 ISR（比文档说的更严格）

#### R5.3 ISR 最小成员数保护
- compute_new_isr 可能返回 new_isr=[leader]（只剩 leader）
- meta 侧 UpdateSegmentIsr fence 校验"ISR 非空且含 leader 且是 replicas 子集"
- `min_in_sync_replicas` 保护在 meta 侧不在 broker 侧
- **文档是否说明**：broker 侧 shrink 到什么程度由 meta fence 兜底

---

## L6：reconcile 兜底

### 调用链
```text
start_metadata_reconcile_thread (reconcile.rs)
  └─ reconcile_once (周期 30s)
       ├─ 遍历 cache.segments，收集 (shard, segment_seq, known_epoch)
       ├─ ReconcileSegmentMetadata grpc → meta 读 cache 比对 epoch
       ├─ has_update=true → decode EngineSegment
       ├─ cache_manager.set_segment
       └─ apply_leader_and_isr (幂等)
```

### 关注点

#### R6.1 被动 reconcile 缺失（已知偏差）
- 规格要求：fetch handler 收到 `UnknownLeaderEpoch` 时主动触发 reconcile
- 代码：只有周期任务，无被动触发
- **影响**：follower 在 UnknownLeaderEpoch 期间会等最多 30s 才能追上

#### R6.2 reconcile 与 parse_segment 并发
- reconcile 调 `set_segment` + `apply_leader_and_isr`（持 state_lock）
- parse_segment 也调 `set_segment` + `apply_leader_and_isr`（持 state_lock）
- state_lock 串行化 apply，`set_segment` 无锁但 DashMap 安全
- **已分析**：set_segment 和 apply 之间有 TOCTOU，reconcile 用旧 snapshot 可能重复 apply → P2 修复（segment_epoch 守卫）后影响降低

#### R6.3 ReconcileSegmentMetadata 是只读 grpc（验证）
- meta-service 端实现：读 `cache_manager.get_segment`，不走 raft
- ✓ 符合设计（纯读，无副作用）

---

## L7：空 ISR 恢复

### 调用链
```text
register_node_by_req (cluster.rs)
  └─ on_node_online (isr_recovery.rs) [异步 spawn]
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

#### R7.1 等待窗口缺失（已知偏差）
- 规格要求"等待 unavailable_recovery_wait_ms 收集更多副本回复"
- 代码立即收集能响应的节点，超时的节点被跳过
- **影响**：可能遗漏 LEO 更大的节点 → 选出 LEO 次大的 leader → 数据不丢（已提交在 HW 内），但未提交数据被丢弃更多
- **建议**：加等待窗口，或多次重试后选

#### R7.2 SetSegment 无 CAS（已知 D1/E1 bug，已修复 per-segment mutex）
- per-segment mutex 防止同一 segment 的双 goroutine
- mutex 是进程内的，跨 meta 节点不共享
- **风险**：不同 meta 节点（raft follower forward）各自收到注册请求，各自 spawn goroutine
- raft 的串行提交是最终一致的保证

#### R7.3 last_known_isr 什么时候被写
- `compute_segment_after_leader_failure`（leader_switch.rs:182-185）：ISR 缩空时 `last_known_isr = segment.isr.clone()`
- 空 ISR 时 `last_known_isr` 非空才有意义——已验证

---

## L8：启动恢复

### 调用链
```text
StorageEngineServer::start (lib.rs)
  └─ recover_local_segments (startup.rs) [async, 持 state_lock]
       └─ recover_one_segment (每个 memory/rocksdb segment)
            ├─ state.lock_state().await [P3 修复，防与 apply 并发]
            ├─ leo = log.latest_offset
            ├─ log_start = log.log_start_offset
            ├─ LeaderEpochCache.load + recover_leader_epoch_cache(leo, log_start)
            │    ├─ truncate_from_end(leo)    [删超过 leo 的虚假 epoch]
            │    └─ truncate_from_start(log_start) [删低于 log_start 的旧 epoch]
            └─ recover_hw(persisted_hw=0, leo) = min(0, leo) = 0 [HW 从 0 重爬]
```

### 关注点

#### R8.1 HW 从 0 重爬（T11c 未做）
- 重启后 HW=0，acks=all 在追上前全超时
- 无 persisted_hw（T11c 未实现持久化）
- **影响范围**：仅 acks=all 的活性，不影响正确性

#### R8.2 state_lock 保护（P3 修复验证）
- recover_one_segment 持 state_lock，防与 parse_segment 的 apply_leader_and_isr 并发写 epoch cache
- **需要验证**：broker 启动时 meta 推送是否在 recover 完成前就到达（gRPC 服务和 storage server 的启动顺序）

---

## 已知偏差汇总

| # | 偏差 | 位置 | 影响 | 优先级 |
|---|---|---|---|---|
| D1 | 写入路径无 `role==LeaderActive` 校验 | write.rs:63 | role 中间态窗口可写 | 中 |
| D2 | HW 计算无 leader_epoch 过滤 | state.rs:committable_hw | 陈旧 epoch follower 拉低活性 | 低 |
| D3 | case 2 follower 未先 truncation（I9 偏差）| role.rs:follower 分支 | 延迟一轮 fetch 完成截断 | 低 |
| D4 | ~~被动 reconcile 缺失~~ | reconcile.rs | ✅ 已修：fetch 收到 `UnknownLeaderEpoch` 时 `mark_reconcile_needed`，reconcile 任务在下次 tick 优先处理，限频 1s/segment | — |
| D5 | 空 ISR 恢复无等待窗口 | isr_recovery.rs | 可能漏掉晚上线的高 LEO 副本 | 中 |
| D6 | acks=all 超时用全局配置 | write.rs:111 | 无法按请求调整超时 | 低 |
| D7 | WriteReqBody 无 timeout_ms / leader_epoch | protocol.rs | zombie leader 防御缺失 | 中 |
| D8 | min_in_sync_replicas 写入路径不校验 | write.rs | 语义保证不完整 | 中 |
| D9 | HW 无持久化（T11c 未做）| cache.rs / startup.rs | 重启后 HW 从 0 重爬 | 低 |
| D10 | memory 全量重拉返回 leo 而非 log_start | offsets_for_leader_epoch.rs:82 | 文档不一致，功能上 OK | 低（文档） |
| D11 | shrink 用 last_caught_up_ts 非 last_fetch_ts | isr_maintain.rs | 文档描述不精确 | 低（文档） |
| D12 | SetSegment 无跨节点 CAS（空 ISR 恢复）| isr_recovery.rs | 集群并发恢复时 segment_epoch 可被覆盖 | 中 |

---

## Review 结论

**协议核心（复制、截断、epoch 管理）正确性**：经过多轮修复，ISR 的核心不变式（I3/I6/I9/I11/I15）基本满足，数据不丢失。

**主要剩余问题**：
1. **D3（case 2 截断偏差）**：follower 切换时未先做 OffsetsForLeaderEpoch truncation，延迟一轮 fetch——逻辑上最终等效，但违反文档 §8.1 case 2 的步骤定义
2. **D4（被动 reconcile）**：fetch 收到更高 epoch 时的快速恢复路径缺失
3. **D7/D8（WriteReqBody 残缺）**：zombie leader 防御和 min_isr 校验是协议完整性的重要组成

**文档需要更新的地方**：
- §5.3 HW 计算说明：明确"无 progress 记录的 ISR 成员跳过"
- §9.5 memory 重拉：澄清 leader 返回 leader_leo 而非 leader_log_start 的原因
- §6.3 ISR shrink：`last_caught_up_ts` 替代 `last_fetch_ts`
