# 概览

RobustMQ AMQP 是构建在 RobustMQ 统一内核之上的 **AMQP 0-9-1 协议兼容层**——它不是一个独立的 RabbitMQ 发行版,而是让标准 AMQP 0-9-1 客户端生态直接连接 RobustMQ 的一层协议实现。官方 RabbitMQ 客户端(Java `amqp-client`、`pika`、`amqplib` 等)都可直连,默认端口 `5672`。

## 设计取向:复用共享消费组模型,而非另起炉灶

RobustMQ AMQP 最核心的设计取向是**复用 RobustMQ 已有的共享消费组基础设施**,而不是为 AMQP 单独实现一套队列引擎。AMQP 的"一个队列、多个竞争消费者"模型,和 MQTT/NATS 的共享订阅在本质上是同一件事:都是"一组消费者竞争消费同一份数据,由某个节点做协调"。因此:

- **队列 = 共享消费组**:每个 AMQP 队列在内部对应一个共享消费组,`Basic.Consume` 注册的消费者就是这个组的成员。
- **组 Leader 由 meta-service 选举**:每个队列的消息拉取与投递,由集群按负载选出的 leader 节点驱动;`Basic.Get` 在非 leader 节点上会通过一次 gRPC 转发到 leader 完成。
- **Exchange / Queue / Binding 元数据持久化在 Raft**:声明的 exchange、queue、binding 是集群级元数据,通过 meta-service 的 Raft 层复制,重启不丢失,任意节点可见。
- **消息存储 = File Segment 引擎**:与 Kafka、MQTT 共享同一套底层存储(段追加写、offset 索引),AMQP 的每个 queue 对应一个内部 topic/shard。

详见 [系统架构](./SystemArchitecture.md)。

## 能力总览

| 能力 | 状态 | 说明 |
|---|---|---|
| Connection / Channel 握手 | ✅ | SASL PLAIN 认证,`Tune`/`TuneOk` 协商 |
| Exchange 管理 | ✅ | `direct` / `fanout` / `topic` / `headers` 四种类型;declare / delete / bind / unbind |
| Queue 管理 | ✅ | declare(含 passive)/ delete(if-unused / if-empty)/ bind / unbind / purge |
| 发布与路由 | ✅ | `Basic.Publish`,默认 exchange 直连路由 + 四种 exchange 类型匹配,`mandatory` 退回 |
| 拉取消费(`Basic.Get`) | ✅ | 单条拉取,跨节点自动转发到队列 leader |
| 推送消费(`Basic.Consume`) | ✅ | 竞争消费者模型,基于共享消费组 leader 推送 |
| 消息确认 | ✅ | `Basic.Ack` / `Nack` / `Reject` / `Recover`(requeue / 丢弃) |
| Publisher Confirm | ✅ | `Confirm.Select` 后,每条 publish 收到落盘结果对应的 ack/nack |
| QoS 预取(prefetch) | 🟡 | 消费者与队列 leader 同节点时强制生效,跨节点为尽力而为 |
| Channel.Flow | 🟡 | 仅对本节点推送生效 |
| 事务(Tx 类) | ❌ | 握手可用,`Tx.Commit`/`Rollback` 不做真实事务语义 |
| 死信队列 / 消息 TTL | ❌ | 不支持 |

> 逐 method 的支持版本与差异见 [协议兼容矩阵](./Protocol.md);完整的"支持 / 部分 / 不支持"清单与根因见 [兼容性与限制](./Compatibility-and-Limitations.md)。

## 快速上手

启动单节点后,用官方 RabbitMQ Java 客户端建队列、发布、消费:

```java
ConnectionFactory factory = new ConnectionFactory();
factory.setHost("localhost");
factory.setPort(5672);
try (Connection connection = factory.newConnection();
     Channel channel = connection.createChannel()) {
    channel.queueDeclare("quickstart", true, false, false, null);
    channel.basicPublish("", "quickstart", null, "hello".getBytes());
    GetResponse resp = channel.basicGet("quickstart", true);
    System.out.println(new String(resp.getBody()));
}
```

完整步骤(含多语言客户端示例、共享队列组验证)见 [快速开始](./QuickStart.md)。

## 文档导航

| 文档 | 内容 |
|---|---|
| [核心概念](./AMQPCoreConcepts.md) | Connection / Channel / Exchange / Queue / Binding / 共享消费组 |
| [系统架构](./SystemArchitecture.md) | 分层架构、请求走向、与原生 RabbitMQ 的关键差异 |
| [协议兼容矩阵](./Protocol.md) | 逐 method 的支持状态与说明 |
| [快速开始](./QuickStart.md) | 单节点启动、Java 客户端最小示例 |
| [兼容性与限制](./Compatibility-and-Limitations.md) | 支持 / 部分 / 不支持清单及根因 |
