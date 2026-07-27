# Java 客户端

RobustMQ 的 AMQP 实现遵循标准 AMQP 0-9-1 协议帧格式,因此**不需要专门的 RobustMQ SDK**——任何标准 AMQP 0-9-1 客户端库都可以直接连接使用。本文以最主流的 [RabbitMQ Java Client](https://github.com/rabbitmq/rabbitmq-java-client) 为例。

## 依赖

```xml
<dependency>
    <groupId>com.rabbitmq</groupId>
    <artifactId>amqp-client</artifactId>
    <version>5.21.0</version>
</dependency>
```

## 建立连接

```java
ConnectionFactory factory = new ConnectionFactory();
factory.setHost("127.0.0.1");
factory.setPort(5672);
factory.setUsername("app_user");
factory.setPassword("app_password");
factory.setVirtualHost("/");

Connection connection = factory.newConnection();
Channel channel = connection.createChannel();
```

## 声明 Exchange 与 Queue

```java
channel.exchangeDeclare("orders-exchange", BuiltinExchangeType.TOPIC, true);
channel.queueDeclare("orders-queue", true, false, false, null);
channel.queueBind("orders-queue", "orders-exchange", "order.*");
```

## 发布(带 Publisher Confirm)

```java
channel.confirmSelect();
channel.basicPublish("orders-exchange", "order.created",
    MessageProperties.PERSISTENT_TEXT_PLAIN,
    payload.getBytes(StandardCharsets.UTF_8));
channel.waitForConfirms(5000);
```

## 消费

```java
channel.basicQos(10); // prefetch
channel.basicConsume("orders-queue", false, (consumerTag, delivery) -> {
    try {
        handle(delivery.getBody());
        channel.basicAck(delivery.getEnvelope().getDeliveryTag(), false);
    } catch (Exception e) {
        channel.basicNack(delivery.getEnvelope().getDeliveryTag(), false, true);
    }
}, consumerTag -> {});
```

## 使用建议

- 结合 RobustMQ 当前的实现状态阅读 [协议支持](../Protocol.md) 和 [兼容性与限制](../Compatibility-and-Limitations.md),不要假设所有 RabbitMQ 特性(死信、TTL、优先级队列等)都可用。
- 在跨节点部署下,`Basic.Qos` prefetch 目前是尽力而为,详见 [共享队列组](../SharedQueueGroup.md)。
- 生产环境请通过网络层(安全组/防火墙)控制访问,当前没有 ACL 授权和 TLS,详见 [安全概览](../Security/Overview.md)。

## 其他语言

由于协议是标准 AMQP 0-9-1,理论上 Python(`pika`)、Go(`amqp091-go`)、.NET(`RabbitMQ.Client`)等客户端库同样可以连接使用,使用方式与连接 RabbitMQ 基本一致,只需注意上述功能边界。

## 延伸阅读

- [快速开始](../QuickStart.md)
- [发布](../Publishing.md)
- [消费](../Consuming.md)
