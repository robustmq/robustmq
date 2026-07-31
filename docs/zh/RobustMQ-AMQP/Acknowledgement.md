# 消息确认(Acknowledgement)

消息被投递(`Basic.Get-Ok` 或 `Basic.Deliver`)之后,如果不是 `no-ack` 模式,就进入**未确认(unacked)**状态,必须被显式确认或拒绝,否则会被视为消费失败并重新投递。

## 未确认索引

RobustMQ 为每条已投递但未确认的消息维护一条索引记录,标记它投给了哪个 connection、哪个 channel、对应的 delivery-tag 是什么。这个索引是 `Basic.Ack`/`Nack`/`Reject`/`Recover` 能定位到具体消息的基础,也是连接/channel 关闭时能批量 requeue 的基础。

## Basic.Ack

确认一条(或多条)消息已处理完成,Broker 据此把消息从存储中删除:

- `delivery-tag` 指定要确认的消息。
- `multiple=true` 表示确认 **小于等于该 delivery-tag 的所有未确认消息**(常用于批量场景,只需在处理完一批后 ack 最后一条)。

## Basic.Nack / Basic.Reject

两者语义相同,`Nack` 是 RabbitMQ 对标准 `Reject` 的扩展,额外支持 `multiple` 批量:

- `requeue=true`:消息被放回队列,等待重新投递(可能投给同一个消费者,也可能投给组内其他成员,取决于 round-robin 顺序)。重新投递时 `redelivered` 标志会被置位。
- `requeue=false`:消息被直接丢弃。RobustMQ 目前**没有死信队列**,丢弃即彻底删除,不会被转发到任何地方。

## Basic.Recover / Basic.Recover-Async

要求 Broker 把这个 channel 上**所有**未确认的消息重新投递(不需要逐条指定 delivery-tag)。两者的区别只是 `Recover` 会回一个 `Recover-Ok` 确认帧,`Recover-Async` 不回。

## 连接/channel 关闭时的自动 requeue

如果一个连接或 channel 在消息还未被 ack 的情况下断开(客户端崩溃、网络中断、正常关闭），该连接/channel 上所有未确认的消息会被自动放回队列,不会丢失。这也是为什么 `no-ack=false` + 显式 ack 是更安全的消费方式:只有真正处理完成的消息才会被删除。

## 示例(Java 客户端)

```java
channel.basicConsume("orders-queue", false, (consumerTag, delivery) -> {
    long tag = delivery.getEnvelope().getDeliveryTag();
    try {
        process(delivery.getBody());
        channel.basicAck(tag, false);
    } catch (RetryableException e) {
        channel.basicNack(tag, false, true);   // 重新入队
    } catch (Exception e) {
        channel.basicNack(tag, false, false);  // 丢弃
    }
}, consumerTag -> {});
```

## 延伸阅读

- [消费](./Consuming.md)
- [核心概念](./AMQPCoreConcepts.md)
- [Publisher Confirm](./PublisherConfirms.md) — 发布侧的可靠性对等物
