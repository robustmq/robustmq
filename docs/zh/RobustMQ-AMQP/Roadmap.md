# 路线图

RobustMQ AMQP 已经能让标准 AMQP 0-9-1 客户端(如 RabbitMQ Java Client)端到端跑通核心的发布/消费/确认流程,同时还有一批 RabbitMQ 生态常见的能力尚未实现。本页给出诚实的现状,详细的逐方法状态见 [协议支持](./Protocol.md),整体边界见 [兼容性与限制](./Compatibility-and-Limitations.md)。

## 已完成

| 能力 | 说明 |
|---|---|
| 连接与 Channel | 生命周期管理、心跳、`Channel.Flow` |
| 交换机 | direct/fanout/topic/headers 四种类型,交换机到交换机绑定 |
| 队列 | 声明(含 passive 真实计数)、绑定、清空、删除(含 if-unused/if-empty) |
| 发布与消费 | `Basic.Publish`/`Get`/`Consume`/`Cancel` |
| 确认 | `Basic.Ack`/`Nack`/`Reject`/`Recover`,断线自动 requeue |
| 可靠发布 | 真实 Publisher Confirm(落盘后 ack) |
| 独占消费 | `exclusive` 强制生效 |
| 认证 | SASL PLAIN,复用统一用户体系 |
| 共享消费组 | 队列复用 MQTT/NATS 共享消费组基础设施,支持竞争消费与跨节点转发 |

## 进行中 / 规划中

| 能力 | 说明 |
|---|---|
| ACL / 操作授权 | 目前认证通过后无权限校验,规划引入 exchange/queue 级授权 |
| TLS/SSL | AMQP 端口目前只支持明文 TCP |
| 声明参数生效 | `x-message-ttl`/`x-expires`/`x-max-length`/`x-max-priority` 等策略参数目前只存储不生效 |
| 死信队列(DLX) | 被拒绝/丢弃的消息目前直接删除,规划支持转发到死信交换机 |
| `durable`/`auto-delete` 真实语义 | 目前持久化行为与 `durable` 取值无关,规划区分 |
| 跨节点 prefetch | `Basic.Qos` 跨节点场景目前尽力而为,规划强一致化 |
| 事务(Tx) | 目前只回确认帧,规划真实原子提交语义或明确废弃 |
| AMQPLAIN | 目前只支持 SASL PLAIN |

## 如何理解当前状态

RobustMQ AMQP 的实现遵循"先把核心消息路径打通,再补齐管理面能力"的节奏:发布、路由、消费、确认这条主链路已经是真实语义,而声明参数(TTL/DLX/优先级等)、精细化权限、传输加密等管理面/边缘能力还在规划中。如果你的场景强依赖这些能力,请先阅读 [兼容性与限制](./Compatibility-and-Limitations.md) 评估影响。

## 延伸阅读

- [协议支持](./Protocol.md)
- [兼容性与限制](./Compatibility-and-Limitations.md)
- [系统架构](./SystemArchitecture.md)
