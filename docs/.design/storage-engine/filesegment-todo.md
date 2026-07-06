# FileSegment 待完善任务列表

基于对 `filesegment`、`commitlog`、`handler`、ISR 四个模块的对比分析，整理出以下待完善任务。

---

## 零、Segment Scroll 重新设计 + Write 优化（已全部完成）

> 进度标记：`- [ ]` 待完成 / `- [x]` 已完成

### Scroll 修复任务

- [x] **S1** `scroll.rs:42` 触发条件 `last()` → `any()`
  - 文件：`src/storage-engine/src/filesegment/scroll.rs`
  - 改：`offsets.iter().any(|&o| o % SEGMENT_SCROLL_OFFSET_INTERVAL == 0)`
  - 测试：`filesegment::scroll::tests::is_trigger_scroll_test` ✓

- [x] **S2** Meta 服务新增 `update_active_segment_by_shard`，scroll 时更新 `active_segment_seq`
  - 文件：`src/meta-service/src/core/shard.rs`、`src/meta-service/src/server/services/engine/segment.rs`
  - 在 `create_segment_by_req` 创建 N+1 后追加调用，将 `active_segment_seq = N+1` 写入 raft 并广播
  - 测试：`core::dynamic_cache::tests::shard_update_notification_updates_active_segment_seq` ✓

- [x] **S3** Broker `parse_shard::Update` 实现 `set_shard()`，使 Shard Update 通知生效
  - 文件：`src/storage-engine/src/core/dynamic_cache.rs`
  - 改：`BrokerUpdateCacheActionType::Update => {}` → `cache_manager.set_shard(shard)`
  - 测试：`core::dynamic_cache::tests::shard_update_notification_updates_active_segment_seq` ✓

- [x] **S4** `CreateNextSegmentRequest` 传真实 LEO，不再估算 `+10000`
  - 文件：`src/storage-engine/src/filesegment/scroll.rs`
  - 改：直接用 `last_offset`（batch 最大 offset）作为 `current_segment_end_offset`，删除 `SEGMENT_SCROLL_OFFSET_BUFFER` 常量
  - 测试：`filesegment::scroll::tests` 全部通过 ✓

- [x] **S5** Meta 服务在创建新 segment 时主动 seal 旧 segment，删除 broker 侧基于 offset 匹配的触发逻辑
  - 文件：`src/meta-service/src/server/services/engine/segment.rs`、`src/storage-engine/src/filesegment/write.rs`
  - meta `create_segment_by_req` 末尾调用 `seal_up_segment(old_segment, now())`
  - `write.rs` 删除 `is_start_or_end_offset` / `trigger_update_start_or_end_info` 调用
  - 测试：`filesegment::write::tests` 全部通过 ✓

- [x] **S6** 新 segment 第一次写入时主动记录 `start_timestamp`
  - 文件：`src/storage-engine/src/filesegment/scroll.rs`（新增 `trigger_update_start_timestamp`）、`src/storage-engine/src/filesegment/write.rs`
  - `batch_write` 检测 `is_first_write`（`segment_file_writer` 里首次打开）时调用 `trigger_update_start_timestamp`
  - 测试：`filesegment::write::tests::write_manager_write_test` ✓

### Write 优化任务

- [x] **W1** 消除 IO 线程 idle 延迟：`try_recv + sleep(10ms)` → `recv().await + drain`
  - 文件：`src/storage-engine/src/filesegment/write.rs`
  - 改：先 `timeout(10ms, recv()).await` 等第一条，再 `try_recv` drain 至 100 条
  - 测试：`filesegment::write::tests::write_manager_write_test` ✓（功能回归通过）

- [x] **W2 / P0-4** mmap 写后不失效 — `file.rs::write()` flush 后调 `self.clear_cache()`，下次读时 `ensure_mmap()` 重建，新追加数据立即可见
  - 文件：`src/storage-engine/src/filesegment/file.rs:207`
  - 修复：`writer.flush().await?` 之后加 `self.clear_cache()`
  - 测试：`filesegment::file::tests` 全部通过 ✓

---

## 一、ISR / 复制 ✅ 已全部完成

### ~~P0-1：为 EngineSegment 实现 `ReplicaLog` trait~~ ✅ 已修复

**修复**：新建 `src/storage-engine/src/filesegment/replica.rs`，实现 `FileSegmentReplicaLog`，覆盖全部 7 个方法：
- `append_at`：校验 `base_offset == LEO`，写文件 + 建索引 + 推进 LEO
- `read_from`：调 `segment_read_by_offset`
- `latest_offset` / `log_start_offset`：读 `SegmentOffset`
- `truncate_to`：扫文件找截断字节位 → `set_len` → 清索引 → 重置 LEO
- `clear`：`set_len(0)` → 清索引 → 重置 LEO 到 start_offset
- `update_high_watermark`：写 `CommitLogOffset::save_high_watermark_offset`

同步修改：`FetchEngines` 新增 `segment` 字段；`EngineReplicaLog` 新增 EngineSegment 分支；`build_engine_fetcher_manager` 增加 `rocksdb_engine_handler` 参数；更新全部 7 处构造位置。

### ~~P0-2：EngineSegment 的 HW（高水位）追踪缺失~~ ✅ 已修复

**修复**：`FileSegmentReplicaLog::update_high_watermark` 调用 `CommitLogOffset::save_high_watermark_offset`，与 memory/rocksdb 路径统一。`FetchEngines` / `EngineReplicaLog` 均已路由 EngineSegment 到该实现。

### ~~P0-3：EngineSegment 缺少 epoch / truncation 支持~~ ✅ 已修复

**修复**：`isr/handle_epoch.rs` 中 `leo_for()` 和 `query_local_replica_state()` 均新增 `StorageType::EngineSegment` 分支，分别调用 `engines.segment.latest_offset` 和 `engines.segment.log_start_offset`。

---

## 二、Handler 入口 ✅ 已全部完成

### ~~P1-1：`shard_offset_req` 对 EngineSegment 返回 `(0, 0)`~~ ✅ 已修复

**修复**：`handler/data.rs::shard_offset_req` 新增 `StorageType::EngineSegment` 分支，通过 `SegmentOffset::get_earliest_offset` / `get_latest_offset` 返回正确的 `(start_offset, end_offset)`。

### ~~P1-2：`get_offset_by_timestamp` 对 EngineSegment 未接入 handler~~ ✅ 已修复

**修复**：`shard_offset_req` 的 `by_timestamp` 分支新增 `StorageType::EngineSegment` 分支，调用 `SegmentOffset::get_offset_by_timestamp`。

### ~~P1-3：tag / key 读取向全集群广播~~ ✅ 已确认行为正确

**结论**：`call_read_data_by_all_node` 内部调用 `get_segment_leader_nodes`，已限定为该 shard 所有 segment 的 leader 节点，并非全集群广播。对 EngineSegment 多 segment 场景，查询所有 segment leader 是必要的，行为正确。函数命名有误导性，但实现无误。

---

## 三、mmap 缓存正确性 ✅ 已修复

### ~~P0-4：写入后 mmap 缓存不失效，新记录不可见~~ ✅ 已修复

**修复**：`src/storage-engine/src/filesegment/file.rs:207`，`write()` 中 `writer.flush().await?` 之后加 `self.clear_cache()`，mmap 缓存在每次写入后立即失效，下次读取时 `ensure_mmap()` 重建映射，新追加数据立即可见。

---

## 四、Segment Meta 同步

### ~~P2-1：活跃 segment 的 end_offset 重启后覆盖问题~~ ✅ 已修复

**分析**：
- scroll 时 `create_segment_by_req` 调用 `update_last_offset_by_segment_metadata(current_segment_end_offset)`，meta 服务侧 end_offset 已被准确更新（S4 已修正为真实 `last_offset`）。
- 真正的 bug 是：broker 重启时 `parse_segment_meta` 用 meta 服务的旧值（上次 seal 时的值）覆盖本地 RocksDB 中保存的更高的 LEO，导致写入指针回退、消费者看到的 latest_offset 偏小。

**修复**：`core/dynamic_cache.rs::parse_segment_meta` 新增保护逻辑：先读取本地 RocksDB 的 `end_offset`，若本地值 > meta 服务值则保留本地值，不做覆盖。

```rust
let local_end = segment_index_manager.get_end_offset(&segment_iden).unwrap_or(-1);
let effective_end = if local_end > meta.end_offset { local_end } else { meta.end_offset };
```

meta 服务端的 `end_offset` 在下次 scroll 时会被正确更新，无需主动上报。

### ~~P2-2：segment 切换时 end_offset 精度~~ ✅ 已确认正确

**确认**：
- `create_segment_by_req` 在调用 `seal_up_segment` 之前，先调用 `update_last_offset_by_segment_metadata(shard, seg, req.current_segment_end_offset)` 精确设置 `end_offset`。
- `seal_up_segment` 本身只更新 `status = SealUp` 和 `end_timestamp`，不推算也不覆盖 `end_offset`。
- 新 segment 的 `start_offset = current_segment_end_offset + 1`（代码第 142 行），严格满足 `end_offset = next.start_offset - 1` 的语义。
- broker 侧 S4 已保证 `current_segment_end_offset` 是真实的最后写入 offset，而非估算值。

结论：meta seal 侧实现正确，无需额外修复。

---

## 五、过期清理

### ~~P2-3：删除 segment 后本地 `SegmentOffset` 元数据未清理~~ ✅ 已修复

**修复**：
- `filesegment/offset.rs` 新增 `SegmentOffset::delete_segment_metadata`，删除全部 5 个 RocksDB key（`offset_segment_start`、`offset_segment_end`、`offset_segment_high_watermark`、`timestamp_segment_start`、`timestamp_segment_end`）。
- `core/segment.rs::delete_local_segment` 在删除索引后，对 `EngineSegment` 类型的 shard 调用 `delete_segment_metadata`，与文件删除保持原子语义。
- 测试：`filesegment::offset::tests::delete_segment_metadata_cleans_all_keys` ✓（写入全部 5 个 key 后调用删除，验证各 key 返回 -1 哨兵值）。

### ~~P2-4：expire 仅在 leader 上触发，follower 本地文件无独立清理路径~~ ✅ 已修复

**修复**：
- `filesegment/expire.rs` 新增 `scan_and_clean_orphan_segments`：每轮循环从 meta 服务拉取全量 segment 列表，对比本地 cache，找出此节点作为 follower-replica 却不在 meta 中的孤儿 segment，逐一调用 `delete_local_segment` 清理。
- 提取了纯逻辑函数 `collect_follower_orphans(cache_manager, broker_id, meta_set) -> Vec<SegmentIdentity>`，便于单元测试。
- `start_segment_expire_thread` 新增 `rocksdb_engine_handler` 参数，`lib.rs` 调用处同步更新。
- 测试：
  - `orphan_detection_skips_leader_segments` — leader 持有的 segment 不会被误判为孤儿 ✓
  - `orphan_detection_returns_follower_not_in_meta` — follower segment 不在 meta 中时被正确识别 ✓
  - `orphan_detection_skips_non_replica_segments` — 非本节点 replica 的 segment 跳过 ✓

---

## 六、Offset / 接口统一

### ~~P3-1：`SegmentOffset` 与 `CommitLogOffset` 无统一接口~~ ✅ 已修复

**修复**：
- 新建 `src/storage-engine/src/core/offset_manager.rs`，定义 `ShardOffsetManager` trait：
  - `get_latest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError>`
  - `get_earliest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError>`
  - `get_offset_by_timestamp(...)` — 有默认实现（strategy 回退到 earliest/latest）
  - 额外提供 `Arc<T: ShardOffsetManager>` 的 blanket impl
- `CommitLogOffset` 实现该 trait（委托现有方法）
- `SegmentOffset` 实现该 trait（覆盖 `get_offset_by_timestamp`，使用真实 segment-level 时间戳索引）
- `handler/data.rs::shard_offset_req` 重构：3 分支 × 2 match → 1 match 构造 `Box<dyn ShardOffsetManager>`，统一调用 `get_latest_offset` / `get_earliest_offset`；Memory/RocksDB 的 by_timestamp 路径仍保留 async engine 方法以维持精度。
- 测试：`default_timestamp_fallback_earliest/latest`、`commit_log_offset_implements_trait`、`segment_offset_implements_trait` 共 4 个 ✓

### ~~P3-2：`SegmentOffset::get_high_watermark_offset` 读取了错误的 key~~ ✅ 已修复

`filesegment/offset.rs::get_high_watermark_offset` 现已使用正确的 `offset_segment_high_watermark` key，不再读 `offset_segment_end`。

---

## 七、读路径完整性

### ~~P3-3：`read_by_offset` 不支持跨 segment 连续读取~~ ✅ 已修复

**修复**：`core/read_offset.rs::read_by_segment` 重写为循环：读完当前 segment 后，若 `remaining_records > 0` 且 `remaining_size > 0` 且下一个 segment（`current_seq + 1`）在 cache 中存在，则继续以 `offset = 0` 读取下一个 segment，直至条件不满足或没有更多 segment。

测试：
- `reads_within_single_segment` — 单 segment 场景不退化 ✓
- `continues_into_next_segment_when_first_is_exhausted` — 跨两个 segment 读到全部 4 条记录 ✓
- `respects_max_record_num_across_segments` — `max_record_num=3` 限制跨 segment 后正确截断 ✓

---

## 优先级汇总

| 优先级 | 编号 | 状态 | 任务 |
|--------|------|------|------|
| P0 | W2/P0-4 | ✅ | mmap 写后不失效：`file.rs:207` 调 `clear_cache()`，新记录立即可见 |
| P0 | P0-1 | ✅ | EngineSegment 接入 ISR：新建 `filesegment/replica.rs`，`FetchEngines`/`EngineReplicaLog` 全部接入 |
| P0 | P0-2 | ✅ | EngineSegment HW 追踪：`update_high_watermark` 写 `CommitLogOffset`，路由已接入 |
| P0 | P0-3 | ✅ | EngineSegment epoch/truncation：`handle_epoch.rs` `leo_for` / `query_local_replica_state` 已加分支 |
| P1 | P1-1 | ✅ | `shard_offset_req` EngineSegment 分支：已接入 `SegmentOffset::get_earliest/latest_offset` |
| P1 | P1-2 | ✅ | `get_offset_by_timestamp` EngineSegment 分支：已接入 `SegmentOffset::get_offset_by_timestamp` |
| P1 | P1-3 | ✅ | tag/key 读取：`call_read_data_by_all_node` 已限定为 shard segment leader 节点，行为正确 |
| P2 | P2-1 | ✅ | `parse_segment_meta` 保留本地更高 LEO，不被 meta 旧值覆盖 |
| P2 | P2-2 | ✅ | meta `seal_up_segment` 不推算 end_offset；`update_last_offset_by_segment_metadata` 精确设置；new.start = end + 1 严格保证 |
| P2 | P2-3 | ✅ | `delete_local_segment` 清理 `SegmentOffset` RocksDB 元数据：新增 `delete_segment_metadata` 方法并在 delete_local_segment 中调用 |
| P2 | P2-4 | ✅ | follower 孤儿清理：`scan_and_clean_orphan_segments` 对比 meta 列表，`collect_follower_orphans` 逻辑已测试 |
| P3 | P3-1 | ✅ | `ShardOffsetManager` trait 已定义；`CommitLogOffset` / `SegmentOffset` 均实现；`shard_offset_req` 已统一调用 |
| P3 | P3-2 | ✅ | `get_high_watermark_offset` 读错 key — **已修复** |
| P3 | P3-3 | ✅ | `read_by_segment` 重写为循环，跨 segment 连续读取；3 个测试覆盖单/双 segment 及 max_record_num 截断 |
