# Quick Start

This page gets you from zero to a working publish-and-consume round trip against a local RobustMQ instance in a few minutes.

## 1. Start the Broker

Follow the build/run instructions in the repository root to start a single-node RobustMQ instance (see the project README for exact build and startup steps). Confirm the port under the `[amqp_runtime]` section in your config file (default `5672`):

```toml
[amqp_runtime]
tcp_port = 5672
```

## 2. Add the Dependency (Java)

```xml
<dependency>
    <groupId>com.rabbitmq</groupId>
    <artifactId>amqp-client</artifactId>
    <version>5.21.0</version>
</dependency>
```

## 3. Connect, Declare, Publish, Consume

```java
ConnectionFactory factory = new ConnectionFactory();
factory.setHost("127.0.0.1");
factory.setPort(5672);
factory.setUsername("app_user");     // this user must already exist in RobustMQ
factory.setPassword("app_password");

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

## Next Steps

- To understand exchange types and how queues map to shared consume groups, read [Core Concepts](./AMQPCoreConcepts.md).
- To do continuous consumption with `Basic.Consume` instead of polling `Basic.Get`, read [Consuming](./Consuming.md).
- To see which AMQP features aren't implemented yet, read [Compatibility & Limitations](./Compatibility-and-Limitations.md).
- To set up authenticated users, read [Authentication (SASL)](./Security/Authentication-SASL.md).
