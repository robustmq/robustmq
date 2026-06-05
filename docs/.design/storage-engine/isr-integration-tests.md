# ISR 协议集成测试设计

> 集成测试放在 `tests/tests/engine/isr_replication.rs`，独立于单元测试。
> 测试分两层：**进程内（in-proc）** 不依赖外部服务，直接运行；**集群级（cluster）** 需要起多 broker，标 `#[ignore]` 手动运行。

---

## 测试分层

| 层级 | 运行方式 | 依赖 | 覆盖范围 |
|---|---|---|---|
| **in-proc** | `cargo test` 直接运行 | 仅 in-proc mock | 复制流程、截断逻辑、ISR 选举 |
| **cluster** | `cargo test -- --ignored` | 起真实 3-broker 集群 | leader failover、网络分区、崩溃恢复 |

---

## In-proc 测试场景（现在实现）

### T1: 复制正常路径（replication_happy_path）

**验证**：leader 写入 → follower fetch 拉取 → HW 推进 → follower LEO 追上 leader

**环境构造**：
- leader: `MemoryStorageEngine`，role=LeaderActive，leader_epoch=1，写 3 条记录（offset 0/1/2）
- follower: 空 `MemoryStorageEngine`
- transport: in-proc，`fetch_one_shard` 直接调 leader

**步骤**：
1. leader 写 records(0,1,2)，LEO=3
2. advance_hw（单副本 ISR=[leader]，HW 立即=3）
3. 构造 `ReplicaFetcherThread`，`SegmentFetchState{current_leader_epoch=1}`
4. 跑 `fetch_round`
5. 断言：follower LEO=3

**期望**：follower 完整追上，no truncation

---

### T2: leader 切换触发截断（leader_switch_triggers_truncation）

**验证**：follower 有分叉数据 → fetch 收到 FencedLeaderEpoch → `truncate_after_fence` → OffsetsForLeaderEpoch 查截断点 → 截到正确位置

**环境构造**：
- 新 leader epoch=2，epoch cache: [(1,0),(2,3)]，LEO=3
- follower 本地 LEO=5（offset 0-4，其中 3/4 是旧 leader epoch 的分叉数据）
- follower seg_state: `current_leader_epoch=1`（旧）
- transport mock：`fetch` 返回 `FencedLeaderEpoch`；`offsets_for_leader_epoch` 真实查 leader epoch cache

**步骤**：
1. follower append 5 条记录（offset 0-4）
2. fetch_round → FencedLeaderEpoch → truncate_after_fence
3. leader 的 `end_offset_for(epoch=1)` = epoch 2 的 start = 3
4. follower truncate_to(2)，新 LEO=3
5. `SegmentFetchState.current_leader_epoch` 更新为 2（修复 A2 bug 的验证）

**期望**：follower LEO=3（分叉尾部被截掉），下一轮 fetch 用新 epoch=2，不再 Fenced

---

### T3: ISR shrink/expand 逻辑（isr_maintain_logic）

**验证**：`compute_new_isr` 按时间 lag 正确踢出/加回 follower

**环境构造**：进程内，`SegmentReplicaState`，构造 follower_progress

**步骤**：
1. ISR=[1,2]，follower 2 的 `last_caught_up_ts` 超过 lag 阈值 → compute_new_isr 返回 [1]（shrink）
2. follower 2 追上（update_follower_progress leo=leader_leo）→ compute_new_isr 返回 [1,2]（expand）

**期望**：shrink/expand 逻辑正确，不会误踢 leader

---

### T4: 空 ISR 选举逻辑（recovery_election_logic）

**验证**：`elect_recovery_leader` 选 LEO 最大且 available 的副本

**步骤**：
1. 多种组合：LEO 最大但 available=false → 选 LEO 次大的
2. LEO 相同，比 leader_epoch
3. 全部 unavailable → None

**期望**：见 isr_recovery.rs 已有 5 个单测

---

### T5: HW 推进与 acks=all 提交（acks_all_commit）

**验证**：单副本 leader，acks=all 写入立即提交；双副本 ISR follower 滞后时超时

**步骤**：
1. ISR=[leader]：写 3 条 acks=all → 立即返回成功，HW=3
2. ISR=[leader,follower]，follower progress leo=0：写 acks=all → 超时返错

**期望**：acks=all 语义正确（已有 write.rs 测试，集成测试作 smoke test）

---

### T6: reconcile 幂等性（reconcile_idempotent）

**验证**：reconcile 重复 apply 同一 segment_epoch 不触发 reset_follower_progress

**步骤**：
1. apply_leader_and_isr(leader_epoch=1, segment_epoch=2)，建立 follower progress
2. 再次 apply 同一 segment（segment_epoch=2）
3. 断言：follower_progress 未被清空（leader_epoch 未变）

**期望**：幂等，progress 保留（修复 P2 bug 的验证）

---

## Cluster 测试场景（标 `#[ignore]`，后续实现）

### C1: kill leader 触发 failover（kill_leader_triggers_failover）

**需要**：3 broker，meta-service 集群

**步骤**：
1. 起 3-broker 集群，创建 3 副本 shard
2. leader 写入 100 条 acks=all
3. kill leader 进程
4. 等待新 leader 选出（心跳超时 → remove_node → leader_switch → LeaderAndIsr 广播）
5. 继续写入，验证 HW 推进、follower truncation 对齐
6. 重启被 kill 的节点，验证它作为 follower 追上

**期望**：无数据丢失（已提交的），新 leader 的 epoch cache 正确

---

### C2: reconcile 兜底广播丢失（reconcile_recovers_missed_notification）

**步骤**：
1. 人为 drop 一次 `update_cache` 通知（网络层拦截）
2. broker 的 role 停留在旧状态
3. 等待 `metadata_reconcile_interval_ms`（默认 30s，测试可配短）
4. 验证 broker 通过 ReconcileSegmentMetadata 追上正确状态

**期望**：在 reconcile_interval 内收口

---

### C3: 全集群重启触发空 ISR 恢复（all_crash_isr_recovery）

**步骤**：
1. 3 副本 shard，写入数据，模拟所有节点宕机（segment 标 Unavailable，last_known_isr 记录）
2. 节点陆续重启，register_node 触发 `on_node_online`
3. per-segment mutex 防双选举
4. 选 LEO 最大节点为新 leader，广播 LeaderAndIsr
5. 其他节点作为 follower，通过 T13c truncation 对齐

**期望**：LEO 最大副本当 leader，无 committed 数据丢失

---

## 文件组织

```
tests/
└── tests/
    └── engine/
        ├── mod.rs                 # 注册 isr_replication
        └── isr_replication.rs    # 本文档对应的测试文件
```

## 依赖说明

in-proc 测试依赖：`storage-engine`（已有）+ `meta-service`（选举函数）+ `rocksdb-engine::test`

cluster 测试额外依赖：需要 broker/meta grpc 地址配置（通过环境变量 `BROKER_ADDR`）
