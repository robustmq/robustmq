# 体验 RobustMQ AMQP

## 前提：启动 Broker

参考 [快速安装](Quick-Install.md) 完成安装，然后启动服务：

```bash
robust-server start
```

AMQP 协议随 RobustMQ 一起启动，默认监听端口 `5672`，无需额外配置——**与 MQTT、Kafka、NATS、mq9 是同一个 Broker 进程**，一条命令启动后所有协议同时就绪。

---

## 准备 AMQP 客户端

RobustMQ 兼容标准 AMQP 0-9-1 协议帧格式，任意标准客户端库（如 [RabbitMQ Java Client](https://github.com/rabbitmq/rabbitmq-java-client)）都可以直接连接，无需 RobustMQ 专用 SDK。以下以 Java 为例。

```xml
<dependency>
    <groupId>com.rabbitmq</groupId>
    <artifactId>amqp-client</artifactId>
    <version>5.21.0</version>
</dependency>
```

---

## 连接、声明、发布、消费

```java
ConnectionFactory factory = new ConnectionFactory();
factory.setHost("127.0.0.1");
factory.setPort(5672);
factory.setUsername("admin");
factory.setPassword("robustmq");

try (Connection connection = factory.newConnection();
     Channel channel = connection.createChannel()) {

    // 声明交换机与队列，并绑定
    channel.exchangeDeclare("demo-exchange", BuiltinExchangeType.DIRECT, true);
    channel.queueDeclare("demo-queue", true, false, false, null);
    channel.queueBind("demo-queue", "demo-exchange", "demo-key");

    // 发布一条消息（开启 confirm 保证可靠投递）
    channel.confirmSelect();
    channel.basicPublish("demo-exchange", "demo-key", null, "hello robustmq".getBytes());
    channel.waitForConfirms(3000);

    // 消费这条消息
    GetResponse resp = channel.basicGet("demo-queue", true);
    if (resp != null) {
        System.out.println("received: " + new String(resp.getBody()));
    }
}
```

预期输出：

```text
received: hello robustmq
```

---

## 下一步

- [核心概念](../RobustMQ-AMQP/AMQPCoreConcepts.md) — 理解 Exchange / Queue / 共享消费组
- [快速开始（完整版）](../RobustMQ-AMQP/QuickStart.md)
- [协议支持](../RobustMQ-AMQP/Protocol.md) — 各方法的真实支持状态
- [兼容性与限制](../RobustMQ-AMQP/Compatibility-and-Limitations.md)
