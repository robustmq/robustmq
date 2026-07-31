# 消费(Consuming)

AMQP 0-9-1 提供两种互补的消费方式:`Basic.Get`(同步拉取一条)和 `Basic.Consume`(注册消费者,由 Broker 主动推送)。两者共享同一套底层读游标——对一个队列先 `Get` 再 `Consume`(或反过来),不会重复消费或漏消费。

## 拉取消费:`Basic.Get`

`Basic.Get` 是一次性、同步的操作:发一个请求,拿到一条消息(或 `Basic.Get-Empty` 表示队列为空)。

- 如果这个队列当前的[共享消费组 leader](./SharedQueueGroup.md) 不是处理请求的这个节点,RobustMQ 会通过一次内部 gRPC 调用把 `Get` 转发给 leader 节点完成,对客户端完全透明。
- `no-ack=true` 时消息读出即视为已消费,不进入未确认状态;`no-ack=false`(默认)时消息进入未确认状态,需要显式 `Basic.Ack`/`Nack`/`Reject`。

`Basic.Get` 适合低频轮询或一次性任务,不适合高吞吐场景——高吞吐请使用 `Basic.Consume`。

## 推送消费:`Basic.Consume`

`Basic.Consume` 注册一个消费者后,Broker 会持续把消息通过 `Basic.Deliver` 主动推给它,直到被 `Basic.Cancel` 取消或连接/channel 关闭。

- 多个消费者(同一个连接的多个 channel,或来自不同节点的不同连接)对同一个队列 `Basic.Consume`,即成为该队列[共享消费组](./SharedQueueGroup.md)的多个成员,彼此**竞争消费**——每条消息只会投给其中一个。
- 队列 leader 节点上的推送任务负责实际投递;消费者所在节点如果不是 leader,leader 会通过内部 gRPC(`SendShareGroupMessage`)把消息投到消费者所在的连接。
- `consumer-tag` 留空时由 Broker 生成唯一值,避免同一连接上多个消费者因为都留空而互相覆盖。
- `no-local`:声明后不影响行为——不会过滤掉本连接自己发布的消息(与 RabbitMQ 经典队列行为一致,该标志本身在经典队列上就没有强制实现)。
- `exclusive=true`:要求独占消费这个队列。如果队列已经有其他消费者(或已有另一个独占消费者),注册会被拒绝(`403 ACCESS_REFUSED`)。

## 预取(Prefetch)与 `Basic.Qos`

`Basic.Qos` 的 `prefetch-count` 限制一个消费者在还没 ack 的情况下最多能同时拿到多少条未确认消息,用于防止消费者被冲垮。

- 当消费者所在节点**就是**队列的共享消费组 leader 节点时,prefetch 会被严格执行:达到上限后,推送任务会跳过这个消费者,直到它 ack 掉一部分消息腾出配额。
- 当消费者所在节点**不是** leader 节点时,prefetch 目前是尽力而为——跨节点的实时预取配额同步尚未实现,详见 [兼容性与限制](./Compatibility-and-Limitations.md)。
- `prefetch-size`(按字节限流)不生效;`global` 标志目前统一按 channel 粒度处理。

## delivery-tag

每个 channel 维护自己独立的 delivery-tag 计数器,从 1 开始单调递增,`Basic.Get-Ok` 和 `Basic.Deliver` 共用同一个计数器序列,channel 生命周期内不会重复,用于后续 `Basic.Ack`/`Nack`/`Reject` 定位具体消息。

## 示例(Java 客户端)

```java
// 拉取
GetResponse resp = channel.basicGet("orders-queue", false);
if (resp != null) {
    System.out.println(new String(resp.getBody()));
    channel.basicAck(resp.getEnvelope().getDeliveryTag(), false);
}

// 推送:设置 prefetch=1,手动 ack
channel.basicQos(1);
channel.basicConsume("orders-queue", false, (consumerTag, delivery) -> {
    System.out.println(new String(delivery.getBody()));
    channel.basicAck(delivery.getEnvelope().getDeliveryTag(), false);
}, consumerTag -> {});
```

## 延伸阅读

- [共享队列组](./SharedQueueGroup.md) — 竞争消费者与 leader 选举
- [消息确认](./Acknowledgement.md) — ack / nack / reject / recover
- [核心概念](./AMQPCoreConcepts.md)
