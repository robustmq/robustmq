# 快速开始

本文帮助你在几分钟内启动一个本地 RobustMQ 实例,并用 AMQP 客户端完成一次发布-消费。

## 1. 启动 Broker

按照仓库根目录的构建/运行说明启动单机版 RobustMQ(具体编译与启动方式请参考项目 README)。确认配置文件中 `[amqp_runtime]` 段的端口(默认 `5672`):

```toml
[amqp_runtime]
tcp_port = 5672
```

## 2. 添加依赖(Java)

```xml
<dependency>
    <groupId>com.rabbitmq</groupId>
    <artifactId>amqp-client</artifactId>
    <version>5.21.0</version>
</dependency>
```

## 3. 连接、声明、发布、消费

```java
ConnectionFactory factory = new ConnectionFactory();
factory.setHost("127.0.0.1");
factory.setPort(5672);
factory.setUsername("app_user");     // 需要提前在 RobustMQ 中创建该用户
factory.setPassword("app_password");

try (Connection connection = factory.newConnection();
     Channel channel = connection.createChannel()) {

    // 声明交换机与队列,并绑定
    channel.exchangeDeclare("demo-exchange", BuiltinExchangeType.DIRECT, true);
    channel.queueDeclare("demo-queue", true, false, false, null);
    channel.queueBind("demo-queue", "demo-exchange", "demo-key");

    // 发布一条消息(开启 confirm 保证可靠投递)
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

预期输出:

```text
received: hello robustmq
```

## 下一步

- 想了解 Exchange 类型、Queue 与共享消费组的关系,阅读 [核心概念](./AMQPCoreConcepts.md)。
- 想用 `Basic.Consume` 做持续消费而不是轮询 `Basic.Get`,阅读 [消费](./Consuming.md)。
- 想了解目前哪些 AMQP 特性还未实现,阅读 [兼容性与限制](./Compatibility-and-Limitations.md)。
- 想配置认证用户,阅读 [认证(SASL)](./Security/Authentication-SASL.md)。
