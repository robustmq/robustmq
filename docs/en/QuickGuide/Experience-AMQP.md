# Experience RobustMQ AMQP

## Prerequisite: Start the Broker

Follow the [Quick Install](Quick-Install.md) guide to install RobustMQ, then start the service:

```bash
robust-server start
```

The AMQP protocol starts together with RobustMQ, listening on port `5672` by default — no extra configuration needed. **It's the same broker process as MQTT, Kafka, NATS, and mq9** — one command starts every protocol at once.

---

## Choose an AMQP Client

RobustMQ follows the standard AMQP 0-9-1 wire protocol, so any standard client library (e.g. the [RabbitMQ Java Client](https://github.com/rabbitmq/rabbitmq-java-client)) connects directly — no RobustMQ-specific SDK needed. The example below uses Java.

```xml
<dependency>
    <groupId>com.rabbitmq</groupId>
    <artifactId>amqp-client</artifactId>
    <version>5.21.0</version>
</dependency>
```

---

## Connect, Declare, Publish, Consume

```java
ConnectionFactory factory = new ConnectionFactory();
factory.setHost("127.0.0.1");
factory.setPort(5672);
factory.setUsername("admin");
factory.setPassword("robustmq");

try (Connection connection = factory.newConnection();
     Channel channel = connection.createChannel()) {

    // Declare an exchange and queue, and bind them
    channel.exchangeDeclare("demo-exchange", BuiltinExchangeType.DIRECT, true);
    channel.queueDeclare("demo-queue", true, false, false, null);
    channel.queueBind("demo-queue", "demo-exchange", "demo-key");

    // Publish a message (with confirms enabled for reliable delivery)
    channel.confirmSelect();
    channel.basicPublish("demo-exchange", "demo-key", null, "hello robustmq".getBytes());
    channel.waitForConfirms(3000);

    // Consume it
    GetResponse resp = channel.basicGet("demo-queue", true);
    if (resp != null) {
        System.out.println("received: " + new String(resp.getBody()));
    }
}
```

Expected output:

```text
received: hello robustmq
```

---

## Next Steps

- [Core Concepts](../RobustMQ-AMQP/AMQPCoreConcepts.md) — exchanges, queues, shared consume groups
- [Quick Start (full version)](../RobustMQ-AMQP/QuickStart.md)
- [Protocol Support](../RobustMQ-AMQP/Protocol.md) — real support status per method
- [Compatibility & Limitations](../RobustMQ-AMQP/Compatibility-and-Limitations.md)
