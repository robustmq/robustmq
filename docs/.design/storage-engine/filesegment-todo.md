# FileSegment 待完善任务列表

基于对 `filesegment`、`commitlog`、`handler`、ISR 四个模块的对比分析，整理出以下待完善任务。

---

## 零、Segment Scroll 重新设计 + Write 优化（当前最高优先级）

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

- [ ] **W2** mmap 用 `written_watermark` 替代全量 `clear_cache`
  - 文件：`src/storage-engine/src/filesegment/segment_file.rs`
  - 在 `SegmentFile` 增加 `written_watermark: AtomicU64`；`write()` 后更新 watermark 而非清缓存；读时超出 watermark 部分走 write_buffer 或 fallback 到文件读
  - 测试：写入后立即通过 mmap 路径能读到新记录；大文件不触发全量重新 mmap

---

## 一、ISR / 复制（最高优先级）

### P0-1：为 EngineSegment 实现 `ReplicaLog` trait

**现状**：`ReplicaLog` trait 只有 `RocksDBStorageEngine` 和 `MemoryStorageEngine` 两个实现；
`isr/handle_fetch.rs` 的 `FetchEngines` 结构体只含 `memory` / `rocksdb`，无 `segment` 字段。

**影响**：EngineSegment 的 follower 永远无法从 leader fetch 数据，ISR 复制对该存储类型完全不工作。

**需要实现**：
- `append_at(base_offset, records)` — 写入时校验 base_offset == LEO，批量追加到 segment 文件
- `read_from(start_offset, max_bytes)` — 从 segment 文件读出记录，供 leader 向 follower 发送
- `truncate_to(offset)` — leader 切换后截断尾部不一致日志
- `clear()` — 清空 shard 所有 segment 数据
- `update_high_watermark(hw)` — follower 收到 leader HW 后更新本地 HW
- 将 EngineSegment 接入 `FetchEngines` 和 `handle_offsets_for_leader_epoch`

### P0-2：EngineSegment 的 HW（高水位）追踪缺失

**现状**：`core/write.rs::batch_write` 中 `advance_hw` 始终操作 `memory_storage_engine.commit_log_offset`；
`core/shard_offset.rs::get_high_water_offset` 对所有存储类型直接 `return Ok(0)`。

**影响**：EngineSegment 写入后 `acks=all` 语义无法保证；consumer 通过 HW 判断可消费偏移量时始终得到 0。

**需要实现**：
- `FileSegmentOffset` 暴露 shard 级别的 `save_high_watermark` / `get_high_watermark`
- `get_high_water_offset` 对 EngineSegment 走 `FileSegmentOffset`，不再返回 0
- `batch_write` 写完后对 EngineSegment 调用对应的 HW advance 逻辑

### P0-3：EngineSegment 缺少 epoch / truncation 支持

**现状**：`OffsetsForLeaderEpochReq` 在 `command.rs` 中构造 `FetchEngines` 后调用 `handle_offsets_for_leader_epoch`，该函数内部没有 EngineSegment 的处理路径。

**影响**：follower 重连时无法通过 `OffsetsForLeaderEpoch` 协商截断点，导致日志分叉无法修复。

---

## 二、Handler 入口（中优先级）

### P1-1：`shard_offset_req` 对 EngineSegment 返回 `(0, 0)`

**现状**：`handler/data.rs::shard_offset_req` 的 storage_type 匹配走到 `_ => (0, 0)`。

**影响**：客户端查询 shard 的 earliest/latest offset 时始终得到全零，`seek_to_beginning` / `seek_to_end` 语义错误。

**修复方向**：使用 `FileSegmentOffset::get_earliest_offset` 和 `get_latest_offset` 填充返回值。

### P1-2：`get_offset_by_timestamp` 对 EngineSegment 未接入 handler

**现状**：`shard_offset_req` 的 `by_timestamp` 分支对 EngineSegment 走到 `_ => 0`；
但 `filesegment/offset.rs::FileSegmentOffset::get_offset_by_timestamp` 已经实现了按 timestamp 查找 segment。

**修复方向**：将 `FileSegmentOffset::get_offset_by_timestamp` 接入 handler 的 by_timestamp 分支。

### P1-3：tag / key 读取向全集群广播，而非只查副本节点

**现状**：`core/read_key.rs` 对 EngineSegment 调用 `call_read_data_by_all_node`，向集群内所有 broker 广播读请求。

**影响**：集群规模大时流量放大；不持有该 shard 任何副本的节点也会被无谓查询。

**修复方向**：改为只向持有该 shard 相关 segment 副本的节点发送请求（从 segment meta 中读取 replicas 列表）。

---

## 三、mmap 缓存正确性（高优先级，正确性 bug）

### P0-4：写入后 mmap 缓存不失效，新记录不可见

**现状**：`SegmentFile::mmap_cache` 在 `ensure_mmap()` 初始化后不会随 `write()` 自动失效；
`clear_cache()` 方法存在但只在 benchmark 测试中调用，写路径从未调用它。

**影响**：通过 mmap 路径（`read_by_offset` 的主路径）读取时，`write()` 新追加的记录不可见，
直到进程重启或手动触发 `clear_cache()`。

**修复方向**：每次 `write()` 后调用 `clear_cache()`；或改为写时 remap（重新 mmap 最新 file_size）。

---

## 四、Segment Meta 同步

### P2-1：活跃 segment 的 end_offset 不实时同步到 meta 服务

**现状**：`FileSegmentOffset::save_latest_offset` 只写本地 RocksDB，不通知 meta 服务。

**影响**：broker 崩溃重启后，meta 服务侧的 `end_offset` 是上次 seal（滚动）时的值，可能远落后于实际写入进度；
重启后消费者通过 meta 服务查到的 latest_offset 偏小。

**修复方向**：重启时在本地恢复完成后，将实际 LEO 上报给 meta 服务；或 seal 时写入准确的 end_offset。

### P2-2：segment 切换时 end_offset 使用硬编码估算值

**现状**：`scroll.rs` 中 `end_offset = last_offset + SEGMENT_SCROLL_OFFSET_BUFFER (10000)`，属于估算。

**影响**：真正封存时 end_offset 可能与下一个 segment 的 start_offset 不连续，导致 offset 路由出现空洞或重叠。

**修复方向**：seal 时将 end_offset 精确设置为下一个 segment 的 `start_offset - 1`，而非估算值。

---

## 五、过期清理

### P2-3：删除 segment 后本地 `SegmentOffset` 元数据未清理

**现状**：`core/segment.rs::delete_local_segment` 删除了 `.msg` 文件和 RocksDB 索引（`delete_segment_index`），
但 `SegmentOffset` 中该 segment 的 start_offset / end_offset / timestamp 字段未随之删除。

**影响**：孤儿 metadata 积累；重启后 `FileSegmentOffset` 从 RocksDB 恢复时可能读到已删除 segment 的偏移量。

### P2-4：expire 仅在 leader 上触发，follower 本地文件无独立清理路径

**现状**：`filesegment/expire.rs` 只有 leader 向 meta 服务发起 `delete_segment` RPC；
follower 通过 `BrokerUpdateCacheResourceType::Segment` Delete 通知触发 `delete_local_segment`。

**影响**：若 Delete 通知丢失（网络分区、重启），follower 的 `.msg` 文件将永远保留，磁盘无法回收。

**修复方向**：follower 定期与 meta 服务对比本地 segment 列表，主动清理孤儿 segment。

---

## 六、Offset / 接口统一

### P3-1：`FileSegmentOffset` 与 `CommitLogOffset` 无统一接口

**现状**：EngineSegment 用 `FileSegmentOffset`（segment 粒度），Memory / RocksDB 用 `CommitLogOffset`（shard 粒度），
两套接口不兼容，导致 `core/write.rs` 等上层代码无法统一处理。

**修复方向**：抽象一个 `ShardOffsetManager` trait，两种实现各自满足；上层代码依赖 trait，消除 storage_type 分支。

### P3-2：`SegmentOffset::get_high_watermark_offset` 读取了错误的 key

**现状**：`filesegment/segment_offset.rs` 第 90 行附近，`get_high_watermark_offset` 使用了 `offset_segment_end` key 而非 `offset_segment_high_watermark` key。

**影响**：读到的是 end_offset，而非 high_watermark，使 HW 语义混乱。

---

## 七、读路径完整性

### P3-3：`read_by_offset` 不支持跨 segment 连续读取

**现状**：`core/read_offset.rs::read_by_segment` 只打开单个 segment 文件读取，
若请求的 offset 范围跨越 segment 边界，不会自动切换到下一个 segment。

**影响**：消费者在 segment 末尾附近读取时，返回的记录数少于请求的 max_record_num，需要多次 RPC 拼凑。

**修复方向**：读取完当前 segment 后，若未满足 max_record_num / max_size，继续打开下一个 segment 读取。

---

## 优先级汇总

| 优先级 | 编号 | 任务 |
|--------|------|------|
| P0 | P0-4 | mmap 缓存写后不失效（正确性 bug） |
| P0 | P0-1 | EngineSegment 实现 ReplicaLog trait，接入 ISR |
| P0 | P0-2 | EngineSegment HW 追踪（write.rs + shard_offset） |
| P0 | P0-3 | EngineSegment epoch / truncation 支持 |
| P1 | P1-1 | shard_offset_req 支持 EngineSegment（非零返回） |
| P1 | P1-2 | get_offset_by_timestamp 接入 EngineSegment handler |
| P1 | P1-3 | tag/key 读取改为只查副本节点，不广播全集群 |
| P2 | P2-1 | 活跃 segment end_offset 重启后上报 meta 服务 |
| P2 | P2-2 | segment 切换时 end_offset 使用精确值而非估算 |
| P2 | P2-3 | delete_local_segment 同时清理 SegmentOffset 元数据 |
| P2 | P2-4 | follower 定期对比 meta，主动清理孤儿 segment |
| P3 | P3-1 | FileSegmentOffset / CommitLogOffset 统一 trait |
| P3 | P3-2 | SegmentOffset::get_high_watermark_offset 读错 key 修复 |
| P3 | P3-3 | read_by_offset 支持跨 segment 连续读取 |
