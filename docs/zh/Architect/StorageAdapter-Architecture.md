# Storage Adapter 架构

Storage Adapter 是 RobustMQ 的存储抽象层，位于 Broker（MQTT、Kafka 等协议处理层）和 Storage Engine（底层存储引擎）之间。它的核心职责是：**将不同协议的存储概念统一抽象为 Shard，并将读写操作透明路由到对应的存储后端，使上层协议与底层存储完全解耦。**

---

## 整体结构

![img](../../images/storage-adapter.png)

## 核心概念：Shard

Shard 是 Storage Adapter 的核心抽象单元，将不同协议的存储概念统一映射：

| 协议 | 原始概念 | 映射为 |
|------|---------|--------|
| MQTT | Topic | Shard |
| Kafka | Partition | Shard |
| AMQP（规划中）| Queue | Shard |

一个 Topic 对应一组 Shard（多分区），Shard 命名规则为：

```
{topic_id}_{partition_seq}
```

每个 Shard 独立配置存储类型（`EngineMemory` / `EngineRocksDB` / `EngineSegment`），由 Meta Service 统一管理 Shard 和 Segment 的元数据。

---

## 关键组件

### StorageAdapter trait

所有存储后端需要实现的统一接口，定义了 Broker 可以调用的全部存储操作：

| 方法 | 说明 |
|------|------|
| `create_shard` | 创建 Shard（对应创建 Topic / Partition） |
| `delete_shard` | 删除 Shard |
| `list_shard` | 查询 Shard 列表 |
| `write` | 写入单条消息，返回 Offset |
| `batch_write` | 批量写入消息 |
| `read_by_offset` | 按 Offset 读取消息 |
| `read_by_tag` | 按 Tag 读取消息 |
| `read_by_key` | 按 Key 读取消息 |
| `delete_by_key` | 按 Key 删除消息 |
| `delete_by_offset` | 按 Offset 删除消息 |
| `get_offset_by_timestamp` | 按时间戳查找对应 Offset |
| `get_offset_by_group` | 查询消费组的 Offset |
| `commit_offset` | 提交消费组 Offset |

### StorageDriverManager

Storage Adapter 对外的入口，Broker 直接调用此组件。核心字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| `driver_list` | `DashMap<String, ArcStorageAdapter>` | 按存储类型缓存已初始化的 Driver，避免重复创建 |
| `engine_storage_handler` | `Arc<StorageEngineHandler>` | 底层引擎处理器 |
| `broker_cache` | `Arc<BrokerCacheManager>` | Topic 元数据缓存，路由时查询 |
| `offset_manager` | `Arc<OffsetManager>` | 消费组 Offset 管理 |
| `message_seq` | `AtomicU64` | 全局写入序号，用于轮询分区选择 |

**Topic 路由逻辑（`build_driver`）：**

1. 从 `BrokerCacheManager` 查询 Topic 配置（含 `storage_type`）
2. 按 `storage_type` 查 `driver_list` 缓存，命中则直接返回
3. 未命中则初始化对应 Driver，写入缓存

### EngineStorageAdapter

实现了 `StorageAdapter` trait，将接口调用委托给 `StorageEngineHandler`，是 Storage Adapter 与 Storage Engine 之间的桥接层。

---

## 写入流程

```
Broker::publish(topic, message)
        │
        ▼
StorageDriverManager::write(topic_name, records)
        │
        ├── 1. 查询 Topic 配置（BrokerCacheManager）
        ├── 2. 选择目标分区：message_seq % topic.partition（轮询）
        ├── 3. 构造 shard_name = build_storage_name(topic_id, partition)
        │
        ▼
EngineStorageAdapter::batch_write(shard_name, records)
        │
        ▼
StorageEngineHandler::batch_write(shard_name, records)
        │
        ├── 查询 StorageCacheManager 获取 Shard 配置和 active_segment
        ├── 判断当前节点是否为 active_segment 的 Leader
        │
        ├── 是 Leader：本地写入
        │       ├── EngineMemory → MemoryStorageEngine（DashMap 内存存储）
        │       ├── EngineRocksDB → RocksDBStorageEngine（RocksDB Column Family）
        │       └── EngineSegment → WriteManager（分段文件日志）
        │
        └── 非 Leader：转发到 Leader 节点
                └── ClientConnectionManager::write_send(leader_broker_id, ...)
```

### 分区选择策略

写入时通过全局原子计数器实现简单轮询：

```
partition = message_seq.fetch_add(1) % topic.partition
```

---

## Topic 创建流程

Topic 创建是一个跨 Meta Service 和 Storage Engine 的协调过程：

```
1. Broker 调用 create_topic_full(topic, shard_config)
        │
        ▼
2. 向 Meta Service 注册 Topic（gRPC: placement_create_topic）
        │
        ▼
3. 等待 Topic 出现在 BrokerCacheManager（超时 30s）
   Meta Service → 推送配置 → Broker 本地缓存更新
        │
        ▼
4. StorageDriverManager::create_storage_resource(topic_name)
        │
        ▼
5. 向 Meta Service 注册 Shard（gRPC: create_shard）
        │
        ▼
6. 等待 Shard + Segment 出现在 StorageCacheManager（超时 3s）
```

---

## 三种存储后端

### EngineMemory（内存存储）

基于 `DashMap` 实现的纯内存存储，包含多维度索引：

| 索引 | 存储结构 | 用途 |
|------|---------|------|
| 主数据 | `DashMap<shard, DashMap<offset, Record>>` | 按 Offset 读取 |
| Tag 索引 | `DashMap<shard, DashMap<tag, Vec<offset>>>` | 按 Tag 查询 |
| Key 索引 | `DashMap<shard, DashMap<key, offset>>` | 按 Key 查询（Key 唯一） |
| 时间戳索引 | `DashMap<shard, DashMap<timestamp, offset>>` | 按时间查 Offset |

**特点**：微秒级延迟，进程重启后数据丢失。适合实时指标、临时通知等场景。

### EngineRocksDB（RocksDB 持久化存储）

基于 RocksDB 的持久化存储，使用专用 Column Family（`DB_COLUMN_FAMILY_BROKER`）存储消息数据，同时在内存中维护写锁避免并发冲突。

**特点**：毫秒级延迟，持久化存储，支持按 Offset / Tag / Key 查询。适合 IoT 设备消息、离线消息等场景。

### EngineSegment（分段文件存储）

基于分段日志文件的存储引擎，由 `WriteManager` 管理写入通道：

- 每个 Shard 由多个 Segment 文件组成（滚动日志）
- 每个 Segment 有一个 Leader Broker 节点负责写入
- Segment 元数据（起止 Offset、状态等）由 Meta Service 统一管理
- 写入通过异步通道（`WriteManager`）批量处理，提高吞吐
- 支持 Tag 和 Key 的索引构建（`save_index`）

**特点**：高吞吐持久化存储，支持 Segment 滚动。适合大数据流处理、Kafka 场景。

---

## Offset 管理

`OffsetManager` 负责消费组 Offset 的管理，支持两种存储策略：

| 策略 | 实现 | 说明 |
|------|------|------|
| 缓存存储（`enable_cache=true`） | `OffsetCacheManager` | RocksDB 本地缓存，低延迟 |
| 持久化存储 | `OffsetStorageManager` | 写入 Meta Service，强一致 |

Offset 的查询支持按时间戳定位策略（`AdapterOffsetStrategy`），可以根据业务需求查找对应时间点的 Offset。

---

## 设计特点

| 特点 | 说明 |
|------|------|
| **协议无感** | Broker 只需传入 Topic 名称，存储类型由 Topic 配置决定，对 Broker 透明 |
| **Topic 级存储配置** | 不同 Topic 可以使用不同存储后端，同一集群支持混合存储 |
| **Leader 转发** | 写入时自动识别 Segment Leader，非 Leader 节点透明转发，Broker 无需感知拓扑 |
| **懒初始化 Driver** | Driver 按需初始化并缓存，避免启动时的资源竞争 |
| **统一 Offset 语义** | 三种存储后端都遵循统一的 Offset 模型，上层消费逻辑一致 |
