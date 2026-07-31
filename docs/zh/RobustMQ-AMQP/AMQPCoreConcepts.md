# 核心概念

本文解释 RobustMQ AMQP 中的核心概念,以及它们在 RobustMQ 统一内核上的具体落地方式。如果你熟悉原生 RabbitMQ / AMQP 0-9-1,这些概念完全一致;差异之处会明确标注。

## Connection 与 Channel

**Connection** 是一条 TCP 长连接,承载 SASL 认证与 vhost(租户)选择;**Channel** 是复用在一条连接上的多路逻辑通道,几乎所有业务方法(Exchange/Queue/Basic)都在某个 channel 上执行。

- 建连握手:`Connection.Start` → `Start-Ok`(SASL PLAIN 凭据)→ `Tune`/`Tune-Ok`(协商 channel-max、frame-max、heartbeat)→ `Open`/`Open-Ok`(选择 vhost)。
- AMQP 的 `virtual-host` 对应 RobustMQ 的**租户(tenant)**:`Connection.Open` 传入的 vhost 名会被解析为租户,空字符串映射到默认租户。
- `Channel.Open` 之后,该 channel 的 `Basic.Deliver`/`Basic.Get-Ok` 的 delivery-tag 从 1 开始单调递增,channel 生命周期内不重复。

## Exchange(交换机)

Exchange 是消息路由的入口,`Basic.Publish` 总是先投给一个 exchange,再由 exchange 按规则转发到匹配的队列。RobustMQ 支持全部四种标准类型:

| 类型 | 路由规则 |
|---|---|
| `direct` | routing-key 与 binding-key **精确相等**才路由 |
| `fanout` | 忽略 routing-key,广播给所有绑定队列 |
| `topic` | routing-key 与 binding pattern 按 `.` 分段通配(`*` 匹配一段,`#` 匹配零到多段) |
| `headers` | 忽略 routing-key,按绑定时声明的 header 键值匹配(`x-match: all` / `any`) |

- **默认 exchange**(名字为空字符串 `""`)是隐式的:每个 queue 都天然按队列名绑定到默认 exchange,`Basic.Publish("", queue_name, ...)` 直接按名字投递到同名队列,无需显式声明或绑定。
- **Exchange-to-Exchange 绑定**(`Exchange.Bind`)支持链式路由:一个 exchange 可以绑定到另一个 exchange 作为其"目的地",消息会沿绑定链继续路由,并对循环绑定做了保护。

## Queue(队列)与共享消费组

Queue 是消息的最终落地点,也是 RobustMQ AMQP 里**最关键的概念映射**:每个 queue 在内部对应一个 [共享消费组](./SharedQueueGroup.md)。

- `Queue.Declare` 会创建队列的元数据(持久化在 Raft)以及承载消息的底层存储分片(shard)。
- 队列不需要预先注册消费组——第一次有消费者调用 `Basic.Consume`(或 `Basic.Get`)时,RobustMQ 会按需创建共享消费组并在集群内选举出负责这个队列的 leader 节点。
- 多个消费者(可能连在不同节点上)`Basic.Consume` 同一个队列,就是这个共享消费组的多个成员——这正是 AMQP 语义里的**竞争消费者(competing consumers)**:每条消息只会投给其中一个消费者。

## Binding(绑定)

Binding 把 exchange 和 queue(或 exchange 和 exchange)关联起来,并携带一个 routing-key(以及 headers 类型下的匹配参数)。同一个 queue 可以被多个 binding 绑定到多个 exchange 甚至同一个 exchange 的多个 routing-key。

## 消息发布与投递

- **发布**:`Basic.Publish` 携带 exchange + routing-key,紧跟 Content Header(属性)和 Content Body(payload)两个帧。
- **拉取消费**:`Basic.Get` 是一次性同步拉取,如果队列在其他节点当 leader,会对该请求做一次内部 gRPC 转发,对客户端透明。
- **推送消费**:`Basic.Consume` 注册后,由队列 leader 节点的推送任务主动把消息 `Basic.Deliver` 给消费者,遵守 `Basic.Qos` 设置的 prefetch 窗口(见 [消费](./Consuming.md))。

## 消息确认与未确认索引

未使用 `no-ack` 的消息在投递后进入"未确认(unacked)"状态,由 RobustMQ 维护一个未确认索引(记录哪条消息投给了哪个 connection/channel)。

- `Basic.Ack` 确认后从存储中删除对应记录。
- `Basic.Nack` / `Basic.Reject`(`requeue=true`)或 `Basic.Recover` 会把消息重新放回队列等待重投;`requeue=false` 则直接丢弃。
- Connection 或 Channel 关闭时,该连接/通道上所有未确认的消息会被自动 requeue。

详见 [消息确认](./Acknowledgement.md)。

## Publisher Confirm

`Confirm.Select` 开启后,该 channel 上每一次 `Basic.Publish` 都会在消息真正写入存储后收到对应的 `Basic.Ack`(成功)或 `Basic.Nack`(失败),用 publish 时分配的递增序号(delivery-tag)对应。详见 [Publisher Confirm](./PublisherConfirms.md)。

## 概念映射一览

| AMQP 概念 | 在 RobustMQ 的落地 |
|---|---|
| Virtual Host | 租户(tenant) |
| Queue | 共享消费组 + 底层存储 shard |
| Basic.Consume 的消费者 | 共享消费组成员 |
| 队列消息拉取/推送的协调者 | meta-service 选举出的组 leader 节点 |
| Exchange / Queue / Binding 元数据 | Raft 复制的集群元数据 |
| 消息存储 | File Segment 引擎(与 Kafka / MQTT 共享) |

## 延伸阅读

- [系统架构](./SystemArchitecture.md)
- [协议兼容矩阵](./Protocol.md)
- [兼容性与限制](./Compatibility-and-Limitations.md)
