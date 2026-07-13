# 新一代消费组协议(KIP-848)

KIP-848 是 Kafka 新一代消费组协议,把分区分配从客户端搬到**服务端**,并用增量心跳取代经典协议的"停顿式"再均衡。RobustMQ 支持该协议:客户端设置 `group.protocol=consumer` 即启用。经典协议见[消费组](./ConsumerGroup.md)。

## 启用

| 配置 | 值 | 效果 |
|---|---|---|
| `group.protocol` | `consumer` | 使用 KIP-848 协议 |
| `group.protocol` | `classic`(默认) | 使用经典协议 |

## 工作方式

![RobustMQ Kafka KIP-848](../../images/kafka-kip848.svg)

核心是单一的 `ConsumerGroupHeartbeat`,取代了经典协议的 `JoinGroup` / `SyncGroup` / `Heartbeat` 三件套:

1. **上报订阅**:消费者心跳携带自己订阅的 topic 列表。
2. **服务端分配**:**Broker(协调器)计算**目标分配方案并在心跳响应中下发——这是与经典协议最大的不同。
3. **增量 reconcile**:消费者按目标分配增量地"先释放、再获取"分区,不需要全组停顿。
4. **稳态心跳**:达到目标分配后,心跳携带 epoch 做确认,进入稳态。

## 与经典协议对比

| 维度 | 经典协议 | KIP-848 |
|---|---|---|
| 分配计算方 | 客户端 leader | **服务端(Broker)** |
| 协议消息 | JoinGroup + SyncGroup + Heartbeat | 单一 ConsumerGroupHeartbeat |
| 再均衡方式 | eager(停顿式,全组先释放再重分配) | 增量 reconcile(逐步迁移) |
| assignor 位置 | 客户端 | 服务端 |
| 启用方式 | 默认 | `group.protocol=consumer` |

两种协议**共享同一份已提交位点和同一个协调器**(Raft Leader),因此位点管理方式一致(见[位点管理](./OffsetManagement.md))。

## 限制

| 项 | 状态 |
|---|---|
| `group.protocol=consumer` | 支持 |
| 服务端分配 + 增量心跳 | 支持 |
| `subscribed_topic_regex`(正则订阅) | **不支持** |

## 相关文档

- [消费组(经典协议)](./ConsumerGroup.md)
- [位点管理](./OffsetManagement.md)
- [消费者](./Consumer.md)
- [系统架构](./SystemArchitecture.md)
