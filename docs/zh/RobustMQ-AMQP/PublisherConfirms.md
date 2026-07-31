# Publisher Confirm

Publisher Confirm(发布确认)是 AMQP 0-9-1 的扩展协议(`Confirm` 类),让生产者能够可靠地知道一条消息是否已经被 Broker 接收并持久化,而不是发出去就不管。RobustMQ 已实现真实的 Publisher Confirm 语义。

## 为什么需要它

普通的 `Basic.Publish` 是"发后不理"(fire-and-forget):没有 Publisher Confirm 时,生产者无法区分"消息已成功写入"和"消息在网络中丢失"或"Broker 处理失败"。一旦 channel 进入 confirm 模式,Broker 会在消息真正落盘之后,主动回一个 `Basic.Ack`(或失败时 `Basic.Nack`)给生产者。

## Confirm.Select

生产者通过 `Confirm.Select` 把 channel 切换到 confirm 模式,Broker 回复 `Confirm.Select-Ok`。切换之后,这个 channel 上所有 `Basic.Publish` 都会被追踪。

## 发布序号与确认

- 进入 confirm 模式后,该 channel 上每一条 `Basic.Publish` 都会被分配一个从 1 开始单调递增的 **publish sequence number**(与 `Basic.Ack`/`Nack` 的 `delivery-tag` 是不同的计数器,分别对应发布方向和消费方向)。
- 消息被路由、写入存储(File Segment)成功后,Broker 回一个 `Basic.Ack`,携带对应的序号。
- `multiple=true` 表示确认小于等于该序号的所有消息(批量确认,减少往返)。
- 如果消息因为路由失败(没有匹配的队列)或写入失败,Broker 回 `Basic.Nack`。

## 与 mandatory 的配合

`mandatory=true` 用于检测"消息没有匹配到任何队列"的情况(见 [发布](./Publishing.md))。在 confirm 模式下,一条 `mandatory` 消息如果没有路由到任何队列,会先收到 `Basic.Return`,随后仍然会收到对应的 `Basic.Ack`/`Nack`(RobustMQ 目前将"未路由到队列"视为发布成功,只要交换机匹配逻辑本身没有出错就 ack——这与部分 Broker 实现的处理方式一致)。

## 示例(Java 客户端)

```java
channel.confirmSelect();
channel.basicPublish("orders-exchange", "order.created", null, payload);

// 同步等待这一条确认(不推荐在高吞吐场景使用,仅作示例)
if (!channel.waitForConfirms(5000)) {
    // 处理失败,考虑重试或告警
}
```

更高吞吐的做法是异步监听:

```java
channel.confirmSelect();
channel.addConfirmListener(
    (deliveryTag, multiple) -> { /* 确认成功 */ },
    (deliveryTag, multiple) -> { /* 确认失败,重发或记录 */ }
);
```

## 局限性

- Publisher Confirm 只保证消息被本节点成功写入存储,不涉及跨节点副本确认(RobustMQ 的高可用由 File Segment 存储引擎和 Raft 元数据层负责,不是每条消息级别的多副本同步写)。
- 事务(`Tx.Select`/`Tx.Commit`)目前**不提供**真实的原子提交语义,只回复确认帧,不建议依赖它做可靠性保证,请使用 Publisher Confirm。

## 延伸阅读

- [发布](./Publishing.md)
- [消息确认](./Acknowledgement.md) — 消费侧的可靠性对等物
- [协议支持](./Protocol.md)
