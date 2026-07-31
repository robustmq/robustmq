# Java Client

RobustMQ's AMQP implementation follows the standard AMQP 0-9-1 wire protocol, so **no RobustMQ-specific SDK is needed** — any standard AMQP 0-9-1 client library connects and works directly. This page uses the most common one, the [RabbitMQ Java Client](https://github.com/rabbitmq/rabbitmq-java-client), as the example.

## Dependency

```xml
<dependency>
    <groupId>com.rabbitmq</groupId>
    <artifactId>amqp-client</artifactId>
    <version>5.21.0</version>
</dependency>
```

## Establishing a Connection

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

## Declaring an Exchange and Queue

```java
channel.exchangeDeclare("orders-exchange", BuiltinExchangeType.TOPIC, true);
channel.queueDeclare("orders-queue", true, false, false, null);
channel.queueBind("orders-queue", "orders-exchange", "order.*");
```

## Publishing (with Publisher Confirms)

```java
channel.confirmSelect();
channel.basicPublish("orders-exchange", "order.created",
    MessageProperties.PERSISTENT_TEXT_PLAIN,
    payload.getBytes(StandardCharsets.UTF_8));
channel.waitForConfirms(5000);
```

## Consuming

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

## Usage Notes

- Read this alongside [Protocol Support](../Protocol.md) and [Compatibility & Limitations](../Compatibility-and-Limitations.md) — don't assume every RabbitMQ feature (dead-lettering, TTL, priority queues, etc.) is available.
- In multi-node deployments, `Basic.Qos` prefetch enforcement is currently best-effort across nodes — see [Shared Queue Group](../SharedQueueGroup.md).
- In production, control access at the network layer (security groups/firewalls) — there's no ACL authorization or TLS yet; see [Security Overview](../Security/Overview.md).

## Other Languages

Since the protocol is standard AMQP 0-9-1, client libraries in Python (`pika`), Go (`amqp091-go`), .NET (`RabbitMQ.Client`), and others should connect and work the same way they would against RabbitMQ — just keep the functional boundaries above in mind.

## Further Reading

- [Quick Start](../QuickStart.md)
- [Publishing](../Publishing.md)
- [Consuming](../Consuming.md)
