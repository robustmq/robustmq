# 发布(Publishing)

`Basic.Publish` 是 AMQP 里唯一的消息写入路径。一次发布由三个帧组成:`Basic.Publish` method 帧(携带 exchange、routing-key、mandatory 标志)、Content Header 帧(消息属性)、一个或多个 Content Body 帧(消息体,按协商的 `frame-max` 分片)。

## 路由规则

发布消息不是直接写队列,而是先投给一个 exchange,由 exchange 按类型把消息路由到匹配的队列(可能是 0 个、1 个或多个):

- **默认 exchange**(`exchange=""`):隐式路由,routing-key 就是目标队列名,直接按名字写入该队列。如果目标队列还没有被显式 `Queue.Declare` 过,RobustMQ 会在发布时自动创建它——这是为了兼容"先发消息、后声明队列"的客户端用法。
- **`direct`**:routing-key 与某个 binding 的 binding-key **完全相等**才路由。
- **`fanout`**:忽略 routing-key,广播给该 exchange 绑定的所有队列。
- **`topic`**:routing-key 按 `.` 分段,与 binding pattern 做通配匹配(`*` 匹配一段,`#` 匹配零到多段)。
- **`headers`**:忽略 routing-key,按绑定时声明的 header 键值和 `x-match`(`all`/`any`)匹配。
- **Exchange-to-Exchange 绑定**:如果目标是另一个 exchange(而非队列),消息会沿绑定链继续路由,直到落到具体队列;循环绑定会被检测并跳过,不会无限转发。

## `mandatory` 与不可路由消息

`Basic.Publish` 的 `mandatory` 标志决定"路由不到任何队列"时的行为:

- `mandatory=true`:Broker 会把消息通过 `Basic.Return` 退回给发布者(`reply-code=312 NO_ROUTE`),紧跟原始的 Content Header + Body,方便客户端拿到完整的被退回消息。
- `mandatory=false`(默认):静默丢弃,不通知发布者。

`immediate` 标志(要求消息必须能立即投给一个在线消费者,否则退回)在 AMQP 0-9-1 中已被广泛认为是反模式,RobustMQ 与现代 RabbitMQ 一致,不做特殊处理。

## 消息属性

Content Header 携带的标准属性(`content-type`、`content-encoding`、`delivery-mode`、`priority`、`correlation-id`、`reply-to`、`expiration`、`message-id`、`timestamp`、`type`、`user-id`、`app-id`、`cluster-id`、自定义 `headers` 键值表)会被完整保存,并在消费(`Basic.Get-Ok` / `Basic.Deliver`)以及消息重投时原样带回。

> `delivery-mode=2`(持久化)与 `delivery-mode=1`(非持久化)目前不影响存储行为——RobustMQ 的存储引擎按队列统一持久化,不区分单条消息的持久化标志。

## 可靠发布

`Basic.Publish` 本身是异步的,不等待任何回执。如果需要确认消息是否真正写入成功,使用 [Publisher Confirm](./PublisherConfirms.md)(`Confirm.Select`);如果需要在路由不到队列时得到通知,使用上面的 `mandatory` 标志。

## 示例(Java 客户端)

```java
Channel channel = connection.createChannel();

// 通过默认 exchange 直接发布到队列(会按需自动创建队列)
channel.basicPublish("", "orders", null, "order-1".getBytes());

// 通过具名 direct exchange 路由
channel.exchangeDeclare("orders-exchange", "direct", true);
channel.queueDeclare("orders-queue", true, false, false, null);
channel.queueBind("orders-queue", "orders-exchange", "orders.created");
channel.basicPublish("orders-exchange", "orders.created", null, "order-2".getBytes());

// mandatory:路由不到时要求退回
AMQP.BasicProperties props = new AMQP.BasicProperties.Builder()
        .contentType("application/json")
        .build();
channel.basicPublish("orders-exchange", "orders.cancelled", true, props, "order-3".getBytes());
```

## 延伸阅读

- [核心概念](./AMQPCoreConcepts.md) — Exchange 类型与路由规则
- [Publisher Confirm](./PublisherConfirms.md) — 可靠发布
- [消费](./Consuming.md)
