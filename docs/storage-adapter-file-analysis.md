# RocksDBStorageAdapter (file) 实现问题分析

基于 MemoryStorageAdapter 的对比分析

## 🔴 严重问题

### 1. **并发写入 Offset 冲突** (Critical)

**位置**: `src/storage-adapter/src/file/mod.rs:100-160`

**问题描述**:
```rust
fn batch_write_internal(&self, shard_name: &str, messages: &[Record]) -> Result<Vec<u64>, CommonError> {
    let offset = self.get_offset(shard_name)?;        // ← 读取 offset
    let mut start_offset = offset;
    // ... 写入数据 ...
    self.save_offset(shard_name, start_offset)?;      // ← 保存 offset
}
```

**问题**:
- `get_offset` 和 `save_offset` 之间没有原子性保证
- 多个并发写入可能读取相同的 offset
- 导致 **offset 重复**，数据相互覆盖

**MemoryStorageAdapter 的正确实现**:
```rust
// Reserve offset range atomically before inserting data
let start_offset = {
    let mut state = self.shard_state.get_mut(shard_name).unwrap();
    let start = state.next_offset;
    state.next_offset = start + messages.len() as u64;  // 原子性预留
    start
};
```

**影响**:
- 数据丢失
- offset 不连续
- 严重的数据一致性问题

**测试验证**: `test_concurrent_write_offset_uniqueness` 已经存在，但可能在实际高并发下失败

---

### 2. **get_offset_by_group() 缺少 shard_name** (High)

**位置**: `src/storage-adapter/src/file/mod.rs:464-482`

**问题代码**:
```rust
async fn get_offset_by_group(&self, group_name: &str) -> Result<Vec<ShardOffset>, CommonError> {
    let raw_offsets = self.db.read_prefix(cf, &group_record_offsets_key_prefix)?;

    for (_, v) in raw_offsets {
        if let Ok(offset) = parse_offset_bytes(&v) {
            offsets.push(ShardOffset {
                offset,
                ..Default::default()  // ❌ shard_name 是空的！
            });
        }
    }
}
```

**问题**: 返回的 `ShardOffset` 没有设置 `shard_name` 字段

**正确实现** (参考 memory 已修复的版本):
```rust
for (key, value) in raw_offsets {
    if let Ok(offset) = parse_offset_bytes(&value) {
        // 从 key 中解析 shard_name
        let shard_name = extract_shard_name_from_key(&key);
        offsets.push(ShardOffset {
            shard_name,
            offset,
            ..Default::default()
        });
    }
}
```

**影响**:
- `MessageStorage.get_group_offset()` 无法找到正确的 shard
- 消费者无法正确恢复消费位置

---

## 🟡 中等问题

### 3. **create_shard 竞态条件** (TOCTOU)

**位置**: `src/storage-adapter/src/file/mod.rs:165-178`

**问题代码**:
```rust
async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError> {
    if self.get_offset(shard_name).is_ok() {  // ← 检查
        return Err(...);
    }
    self.db.write(cf.clone(), &shard_offset_key, &0_u64)?;  // ← 创建
}
```

**问题**: Time-of-check to time-of-use
- 多个并发调用可能同时通过检查
- 导致重复创建或数据不一致

**建议**: 使用 RocksDB 的事务或 merge 操作

---

### 4. **delete_shard 缺少事务保护**

**位置**: `src/storage-adapter/src/file/mod.rs:195-213`

**问题**:
```rust
async fn delete_shard(&self, shard: &str) -> Result<(), CommonError> {
    self.db.delete_prefix(cf.clone(), &record_prefix)?;
    self.db.delete_prefix(cf.clone(), &key_index_prefix)?;
    self.db.delete_prefix(cf.clone(), &tag_index_prefix)?;
    self.db.delete_prefix(cf.clone(), &timestamp_index_prefix)?;
    self.db.delete(cf.clone(), &shard_offset_key)?;
    self.db.delete(cf, &shard_info_key)?;
}
```

**问题**:
- 6 个独立的删除操作，不是原子的
- 如果中间失败，shard 会处于不一致状态
- 可能导致"幽灵 shard" (部分数据残留)

**建议**: 使用 WriteBatch

---

### 5. **read_by_offset 假设连续性**

**位置**: `src/storage-adapter/src/file/mod.rs:256-260`

**问题代码**:
```rust
for record_opt in batch_results {
    let Some(record) = record_opt else {
        break;  // ❌ 遇到第一个空就停止
    };
}
```

**问题**:
- 假设 offset 是严格连续的
- 如果有消息过期或删除，会提前终止读取
- 可能导致读不到后面的有效消息

**MemoryStorageAdapter** 的处理更健壮

---

## 🟢 次要问题

### 6. **timestamp 索引稀疏**

**位置**: `src/storage-adapter/src/file/mod.rs:143`

```rust
if msg.timestamp > 0 && start_offset % 5000 == 0 {
    // 只索引每 5000 条记录
}
```

**问题**:
- `get_offset_by_timestamp()` 精度最多差 5000 条消息
- 对于时间敏感的场景可能不够精确

---

### 7. **错误处理不一致**

**问题**:
- `get_offset()` 在 shard 不存在时返回错误
- 但某些场景下 (如首次写入) 应该自动创建

**建议**: 添加 `get_or_create_shard()` 方法

---

### 8. **性能优化机会**

#### a) commit_offset 已优化 ✅
```rust
// 使用 WriteBatch，性能很好
let mut batch = WriteBatch::default();
for (shard_name, offset) in offsets.iter() {
    batch.put_cf(...);
}
self.db.write_batch(batch)?;
```

#### b) read_by_tag 使用迭代器 ✅
```rust
// 避免加载所有 tag 到内存，很好
let mut iter = self.db.db.raw_iterator_cf(&cf);
```

#### c) read_by_offset 使用 multi_get ✅
```rust
// 批量读取，性能优秀
let batch_results = self.db.multi_get::<Record>(cf, &keys)?;
```

---

## 📊 对比总结

| 功能 | MemoryStorageAdapter | RocksDBStorageAdapter | 状态 |
|------|---------------------|----------------------|------|
| 并发 offset 分配 | ✅ 原子操作 | ❌ 无保护 | 🔴 严重 |
| get_offset_by_group | ✅ 返回 shard_name | ❌ 缺失 shard_name | 🔴 严重 |
| create_shard 并发 | ✅ DashMap 保护 | ❌ TOCTOU | 🟡 中等 |
| delete_shard 原子性 | ✅ 内存操作原子 | ❌ 多步操作 | 🟡 中等 |
| 读取连续性假设 | ✅ 健壮 | ❌ 脆弱 | 🟡 中等 |
| 性能优化 | ✅ 内存快 | ✅ 批量操作优化好 | ✅ 良好 |

---

## 🔧 修复优先级

### P0 (必须修复):
1. **并发写入 offset 冲突** - 使用分布式锁或 RocksDB 事务
2. **get_offset_by_group() 缺少 shard_name** - 从 key 解析 shard_name

### P1 (强烈建议):
3. **create_shard 竞态条件** - 使用 CAS 或事务
4. **delete_shard 事务保护** - 使用 WriteBatch

### P2 (改进):
5. **read_by_offset 连续性假设** - 支持稀疏读取
6. **timestamp 索引精度** - 可配置化

---

## 🧪 测试建议

### 缺失的测试用例:

1. **并发 offset 冲突测试**
   ```rust
   #[tokio::test]
   async fn test_concurrent_offset_conflict() {
       // 100 个并发写入同一个 shard
       // 验证所有 offset 唯一且连续
   }
   ```
   **注**: `test_concurrent_write_offset_uniqueness` 已存在但需加强

2. **get_offset_by_group 功能测试**
   ```rust
   #[tokio::test]
   async fn test_get_offset_with_shard_name() {
       // 验证返回的 ShardOffset 包含正确的 shard_name
   }
   ```

3. **delete_shard 部分失败测试**
   ```rust
   #[tokio::test]
   async fn test_delete_shard_partial_failure() {
       // 模拟删除中间失败的情况
   }
   ```

4. **稀疏 offset 读取测试**
   ```rust
   #[tokio::test]
   async fn test_read_with_gaps() {
       // 写入 [0,1,2,5,6,7]，删除 [3,4]
       // 验证能读取全部 6 条记录
   }
   ```

---

## 💡 架构建议

考虑引入:
1. **分布式锁服务** (如 etcd) 用于 offset 分配
2. **RocksDB 事务** 用于原子操作
3. **Raft 协议** 用于多副本一致性

---

生成时间: 2025-12-05
分析基准: MemoryStorageAdapter (已修复版本)
