# Storage Adapter 架构

Storage Adapter 位于 Broker 和 Storage Engine 之间，将不同协议的存储概念统一抽象为 Shard，并将读写操作路由到对应的存储后端。

---

## Shard 抽象

各协议的存储概念统一映射为 Shard：

| 协议 | 原始概念 | 映射 |
|------|---------|------|
| MQTT | Topic | Shard |
| Kafka | Partition | Shard |
| AMQP（规划中）| Queue | Shard |

一个 Topic 对应一组 Shard（多分区），命名规则：

```
{topic_id}_{partition_seq}
```

每个 Shard 独立配置存储类型（`EngineMemory` / `EngineRocksDB` / `EngineSegment`），Shard 和 Segment 元数据由 Meta Service 统一管理。

---

## 核心组件

### StorageAdapter trait

所有存储后端实现的统一接口：

| 方法 | 说明 |
|------|------|
| `create_shard` | 创建 Shard |
| `delete_shard` | 删除 Shard |
| `write` | 写入单条消息，返回 Offset |
| `batch_write` | 批量写入消息 |
| `read_by_offset` | 按 Offset 读取 |
| `read_by_tag` | 按 Tag 读取 |
| `read_by_key` | 按 Key 读取 |
| `get_offset_by_timestamp` | 按时间戳查 Offset |
| `get_offset_by_group` | 查询消费组 Offset |
| `commit_offset` | 提交消费组 Offset |

### StorageDriverManager

Broker 直接调用的入口组件：

| 字段 | 类型 | 说明 |
|------|------|------|
| `driver_list` | `DashMap<String, ArcStorageAdapter>` | 按存储类型缓存已初始化的 Driver |
| `engine_storage_handler` | `Arc<StorageEngineHandler>` | 底层引擎处理器 |
| `broker_cache` | `Arc<BrokerCacheManager>` | Topic 元数据缓存，路由时查询 |
| `offset_manager` | `Arc<OffsetManager>` | 消费组 Offset 管理 |
| `message_seq` | `AtomicU64` | 全局写入序号，用于轮询分区选择 |

Driver 按需初始化并缓存，避免重复创建。

### EngineStorageAdapter

实现 `StorageAdapter` trait，将接口调用委托给 `StorageEngineHandler`，是 Storage Adapter 与 Storage Engine 之间的桥接层。

---

## 写入流程

Broker 收到消息后，调用 `StorageDriverManager::write` 进入写入流程：

1. 从 `BrokerCacheManager` 查找 Topic 的分区数和存储类型配置
2. 用 `message_seq.fetch_add(1)` 取自增序号，对分区数取模，选出目标 Shard（`topic_id_{seq % partition_count}`）
3. 从 `driver_list` 查找对应存储类型的 Driver，不存在则按需初始化并缓存
4. 调用 Driver 的 `write` 方法写入，返回该消息的 Offset

批量写入（`batch_write`）逻辑相同，对每条消息独立选 Shard 后批量提交。

![Storage Adapter 写入流程](../../images/arch_adapter_write.png)

---

## Topic 创建流程

创建 Topic 时，Broker 向 Meta Service 申请 Shard 配置，Meta Service 负责分配分区序号和 Segment 资源，再将 Shard 元数据下发给 Storage Engine 初始化存储结构：

1. 客户端请求创建 Topic，Broker 校验参数后调用 Meta Service gRPC 接口
2. Meta Service 在 metadata Raft Group 中写入 Topic 和 Shard 配置
3. Meta Service 通过 `UpdateCache` 通知 Broker 刷新本地 `BrokerCacheManager`
4. Storage Engine 收到 Shard 创建指令，初始化对应的 Active Segment

![Topic 创建流程](../../images/arch_topic_create.png)

---

## Offset 管理

`OffsetManager` 支持两种存储策略：

| 策略 | 实现 | 说明 |
|------|------|------|
| 缓存存储（`enable_cache=true`） | `OffsetCacheManager` | RocksDB 本地缓存，低延迟 |
| 持久化存储 | `OffsetStorageManager` | 写入 Meta Service，强一致 |

---

## 分层关系

![Storage Adapter 分层架构](../../images/arch_adapter_layers.png)
