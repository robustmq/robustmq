# 存储(Storage)

AMQP 队列消息使用统一的 `StorageDriverManager`,但物理后端由队列 Topic
配置决定。当前 AMQP 实现创建的队列 Topic 使用 **`EngineRocksDB`**；Kafka
Topic 使用 `EngineSegment`，MQTT 则使用自身的 Topic 配置。各协议共享存储
基础设施，但不是强制使用同一个物理引擎，也不是自动共享协议命名空间。

## 统一的存储驱动

AMQP、Kafka、MQTT 三个协议 Broker 都通过同一个 `StorageDriverManager`
写入数据，由每个 Topic 的存储类型选择具体 Driver。AMQP 队列在存储层表现
为一个 `TopicSource::AMQP` Topic 及其底层 Shard，读写都通过统一 Adapter，
而不是 AMQP 专用存储 API。

这意味着:

- AMQP 的持久化和清理行为遵循所选后端及 Topic 配置，并不等同于 Kafka 的
  File Segment 布局。
- 不存在独立的 AMQP 存储 API；后端相关设置通过统一 Topic 和存储配置生效。

## 发布即持久化

在 confirm 模式下,消息的落盘和确认之间没有"抢跑":`Basic.Publish` 收到消息后先在内存中暂存(`PendingPublish`),真正执行写入时会完整等待 `StorageDriverManager::write` 返回成功,然后才构造并发送 `Basic.Ack`/`Basic.Nack`。也就是说,只要客户端收到了 `Basic.Ack`,消息就已经真实落盘,不存在"先 ack 后台再异步写盘"的情况。

## 没有实现的能力

以下是常见消息队列存储特性中,RobustMQ AMQP **目前未实现**的部分,使用时需要注意:

| 特性 | 状态 | 说明 |
|---|---|---|
| 消息 TTL / 过期 | ❌ | 没有为队列或消息设置过期时间的机制 |
| 队列最大长度(max-length) | ❌ | 没有队列容量上限,也没有超限丢弃/拒绝策略 |
| 死信队列(DLX) | ❌ | 拒绝或丢弃的消息直接删除,不会转发到死信交换机 |
| 消息优先级队列 | ❌ | 未实现优先级调度 |
| 惰性队列(Lazy Queue) | ❌ | 没有 RabbitMQ 风格的内存/磁盘两级队列模式,队列统一使用其配置的后端 |

## 延伸阅读

- [队列清空与删除](./QueuePurgeAndDelete.md)
- [核心概念](./AMQPCoreConcepts.md)
- [系统架构](./SystemArchitecture.md)
- [兼容性与限制](./Compatibility-and-Limitations.md)
