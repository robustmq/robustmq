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

### P2-1：活跃 segment 的 end_offset 不实时同步到 meta 服务 ❌

**现状**：`SegmentOffset::save_latest_offset` 只写本地 RocksDB，不通知 meta 服务。

**影响**：broker 崩溃重启后，meta 服务侧的 `end_offset` 是上次 seal（滚动）时的值，可能远落后于实际写入进度；重启后消费者通过 meta 服务查到的 latest_offset 偏小。

**修复方向**：重启时在本地恢复完成后，将实际 LEO 上报给 meta 服务；或 seal 时写入准确的 end_offset。

### P2-2：segment 切换时 end_offset 精度 ⚠️ 部分修复

**现状**：S4 已修复 broker 侧（`scroll.rs` 传真实 `last_offset` 而非 `last_offset + 10000`），meta 服务收到 `create_segment` 请求时 seal 旧 segment 使用的 `end_offset` 已是真实值。但 seal 的精确语义（`end_offset = next_segment.start_offset - 1`）待确认 meta 侧实现是否严格保证。

**修复方向**：确认 meta `seal_up_segment` 使用传入的 `current_segment_end_offset` 字段，而非自行推算。

---

## 五、过期清理

### P2-3：删除 segment 后本地 `SegmentOffset` 元数据未清理 ❌

**现状**：`core/segment.rs::delete_local_segment` 删除了 `.msg` 文件和 RocksDB 索引（`delete_segment_index`），但 `SegmentOffset` 中该 segment 的 start_offset / end_offset / timestamp 字段未随之删除。

**影响**：孤儿 metadata 积累；重启后 `SegmentOffset` 从 RocksDB 恢复时可能读到已删除 segment 的偏移量。

**修复方向**：`delete_local_segment` 末尾追加清理 `SegmentOffset` 相关 key（`offset_segment_start`、`offset_segment_end`、`offset_segment_high_watermark`、`timestamp_segment_start`、`timestamp_segment_end`）。

### P2-4：expire 仅在 leader 上触发，follower 本地文件无独立清理路径 ❌

**现状**：`filesegment/expire.rs` 只有 leader 向 meta 服务发起 `delete_segment` RPC；follower 通过 `BrokerUpdateCacheResourceType::Segment` Delete 通知触发 `delete_local_segment`。

**影响**：若 Delete 通知丢失（网络分区、重启），follower 的 `.msg` 文件将永远保留，磁盘无法回收。

**修复方向**：follower 定期与 meta 服务对比本地 segment 列表，主动清理孤儿 segment。

---

## 六、Offset / 接口统一

### P3-1：`SegmentOffset` 与 `CommitLogOffset` 无统一接口 ❌

**现状**：EngineSegment 用 `SegmentOffset`（segment 粒度），Memory / RocksDB 用 `CommitLogOffset`（shard 粒度），两套接口不兼容，导致 `core/write.rs` 等上层代码无法统一处理。

**修复方向**：抽象一个 `ShardOffsetManager` trait，两种实现各自满足；上层代码依赖 trait，消除 storage_type 分支。

### ~~P3-2：`SegmentOffset::get_high_watermark_offset` 读取了错误的 key~~ ✅ 已修复

`filesegment/offset.rs::get_high_watermark_offset` 现已使用正确的 `offset_segment_high_watermark` key，不再读 `offset_segment_end`。

---

## 七、读路径完整性

### P3-3：`read_by_offset` 不支持跨 segment 连续读取 ❌

**现状**：`core/read_offset.rs::read_by_segment` 只打开单个 segment 文件读取，若请求的 offset 范围跨越 segment 边界，不会自动切换到下一个 segment。

**影响**：消费者在 segment 末尾附近读取时，返回的记录数少于请求的 max_record_num，需要多次 RPC 拼凑。

**修复方向**：读取完当前 segment 后，若未满足 max_record_num / max_size，继续打开下一个 segment 读取。

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
| P2 | P2-1 | ❌ | 活跃 segment end_offset 重启后未上报 meta 服务 |
| P2 | P2-2 | ⚠️ | segment seal 时 end_offset 精度：broker 侧已修（S4），meta seal 侧语义待确认 |
| P2 | P2-3 | ❌ | `delete_local_segment` 未清理 `SegmentOffset` RocksDB 元数据 |
| P2 | P2-4 | ❌ | follower 定期对比 meta，主动清理孤儿 segment |
| P3 | P3-1 | ❌ | `SegmentOffset` / `CommitLogOffset` 统一 trait |
| P3 | P3-2 | ✅ | `get_high_watermark_offset` 读错 key — **已修复** |
| P3 | P3-3 | ❌ | `read_by_offset` 不支持跨 segment 连续读取 |
