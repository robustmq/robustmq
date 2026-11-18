# 存储(Storage)

AMQP 队列消息使用统一的 `StorageDriverManager` 和 `EngineStorageAdapter`，
但当前 AMQP 队列创建流程选择的是 **`EngineRocksDB`**。Kafka Topic 当前选择
`EngineSegment`，MQTT Topic 选择 `EngineRocksDB`。因此各协议共享适配器和存储
引擎接口，但不会自动共享同一份物理 File Segment 数据。

## 统一的存储驱动

AMQP、Kafka、MQTT 都通过同一个 `StorageDriverManager` 写入数据，由 Topic 配置
选择具体存储引擎。AMQP 队列在存储层表示为内部 Topic 和一个或多个 Shard；当前
队列路径创建 RocksDB Shard，消息通过 Storage Adapter 使用 offset 定位。

这意味着:

- AMQP 的持久化和清理行为遵循 RocksDB Shard 的配置，并不使用 Kafka 的 File Segment 磁盘布局。
- 队列存储仍复用统一的 Topic/Shard 配置流程；保留时间、记录数限制等存储参数来自 Topic 配置。

## Publisher Confirm 语义

在 confirm 模式下,路由成功的 `Basic.Publish` 会先在内存中暂存(`PendingPublish`);Broker 等待每个目标队列的 `StorageDriverManager::write` 结果后,再构造并发送 `Basic.Ack`/`Basic.Nack`。对于已路由的消息,`Basic.Ack` 表示处理节点已成功写入配置的存储,不表示跨节点副本确认。当前实现也会确认未路由的 publish,即使没有写入队列;若设置 `mandatory=true`,还会收到 `Basic.Return`。

## 没有实现的能力

以下是常见消息队列存储特性中,RobustMQ AMQP **目前未实现**的部分,使用时需要注意:

| 特性 | 状态 | 说明 |
|---|---|---|
| 消息 TTL / 过期 | ❌ | 没有为队列或消息设置过期时间的机制 |
| 队列最大长度(max-length) | ❌ | 没有队列容量上限,也没有超限丢弃/拒绝策略 |
| 死信队列(DLX) | ❌ | 拒绝或丢弃的消息直接删除,不会转发到死信交换机 |
| 消息优先级队列 | ❌ | 未实现优先级调度 |
| 惰性队列(Lazy Queue) | ❌ | 没有"内存 vs 磁盘"两级存储的区分,所有消息统一走当前配置的 RocksDB 队列存储路径 |

## 延伸阅读

- [队列清空与删除](./QueuePurgeAndDelete.md)
- [核心概念](./AMQPCoreConcepts.md)
- [系统架构](./SystemArchitecture.md)
- [兼容性与限制](./Compatibility-and-Limitations.md)
