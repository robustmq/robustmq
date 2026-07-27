# 存储(Storage)

AMQP 队列的消息存储和 Kafka、MQTT 共用同一套底层引擎——**File Segment**,没有为 AMQP 单独实现一套存储。这也是"一份数据,多种协议视图"架构的具体体现。

## 统一的存储驱动

AMQP、Kafka、MQTT 三个协议 Broker 都通过同一个 `StorageDriverManager` 写入数据,底层落到 `storage-engine` crate 的 File Segment 引擎。一个队列(Queue)在存储层就是一个分片(shard),消息以顺序追加日志的形式写入,通过 offset 定位。

这意味着:

- AMQP 消息的持久化能力、磁盘布局、压缩/清理策略,都与 Kafka/MQTT 完全一致,复用同一套运维和监控经验。
- 不存在"AMQP 专属"的存储配置项——存储相关的调优(如 segment 大小、刷盘策略)在 Broker 层面统一配置,不区分协议。

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
| 惰性队列(Lazy Queue) | ❌ | 没有"内存 vs 磁盘"两级存储的区分,所有消息统一走 File Segment |

## 延伸阅读

- [队列清空与删除](./QueuePurgeAndDelete.md)
- [核心概念](./AMQPCoreConcepts.md)
- [系统架构](./SystemArchitecture.md)
- [兼容性与限制](./Compatibility-and-Limitations.md)
