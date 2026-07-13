# 快速开始

本文带你在几分钟内启动 RobustMQ,用官方 Kafka CLI 完成建 topic / 生产 / 消费,并用 Java `kafka-clients` 跑通一个最小示例。

## 前置条件

- 一套可用的 Kafka CLI(官方发行版里的 `kafka-*.sh`,任意 3.x / 4.x 版本即可)。
- 如需 Java 示例:JDK 8+ 与 Maven / Gradle。
- Kafka 协议默认端口 `9092`。

## 启动 RobustMQ(单节点)

```bash
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
robust-server start
```

Kafka 监听端口由配置文件的 `[kafka_runtime]` 段决定,默认 `9092`:

```toml
[kafka_runtime]
tcp_port = 9092
```

> 单节点默认**不开启** SASL,连接无需认证,便于快速上手。开启认证见文末"SASL 连接"。

## 用官方 CLI 验证

### 建 topic

```bash
kafka-topics.sh --bootstrap-server localhost:9092 \
  --create --topic quickstart --partitions 3

# 查看
kafka-topics.sh --bootstrap-server localhost:9092 --describe --topic quickstart
```

> RobustMQ 默认开启自动创建 topic,直接生产也会隐式建 topic;显式创建更可控。

### 生产

```bash
kafka-console-producer.sh --bootstrap-server localhost:9092 --topic quickstart
>hello
>robustmq kafka
# Ctrl-C 结束
```

### 消费

```bash
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic quickstart --from-beginning
```

能看到刚才生产的两行,即表示数据面工作正常。

### 用消费组消费并查看进度

```bash
# 以消费组 g1 消费
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic quickstart --group g1 --from-beginning

# 查看该组的位点与 lag
kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group g1
```

更多命令见 [CLI 操作指南](./CLI-Guide.md)。

## Java kafka-clients 最小示例

依赖(Maven):

```xml
<dependency>
    <groupId>org.apache.kafka</groupId>
    <artifactId>kafka-clients</artifactId>
    <version>3.7.0</version>
</dependency>
```

生产者:

```java
Properties props = new Properties();
props.put("bootstrap.servers", "localhost:9092");
props.put("key.serializer", "org.apache.kafka.common.serialization.StringSerializer");
props.put("value.serializer", "org.apache.kafka.common.serialization.StringSerializer");
// 可选:开启幂等生产(RobustMQ 支持)
props.put("enable.idempotence", "true");

try (KafkaProducer<String, String> producer = new KafkaProducer<>(props)) {
    producer.send(new ProducerRecord<>("quickstart", "k1", "hello from java"));
    producer.flush();
}
```

消费者:

```java
Properties props = new Properties();
props.put("bootstrap.servers", "localhost:9092");
props.put("group.id", "g-java");
props.put("auto.offset.reset", "earliest");
props.put("key.deserializer", "org.apache.kafka.common.serialization.StringDeserializer");
props.put("value.deserializer", "org.apache.kafka.common.serialization.StringDeserializer");

try (KafkaConsumer<String, String> consumer = new KafkaConsumer<>(props)) {
    consumer.subscribe(List.of("quickstart"));
    while (true) {
        ConsumerRecords<String, String> records = consumer.poll(Duration.ofMillis(500));
        records.forEach(r -> System.out.printf("%s => %s%n", r.key(), r.value()));
    }
}
```

> **提示**:`enable.idempotence=true` 可用(RobustMQ 支持幂等 Producer);但**不要**开启事务(`transactional.id`),RobustMQ 暂不支持事务,详见 [兼容性与限制](./Compatibility-and-Limitations.md)。

## SASL 连接(可选)

RobustMQ 支持 **SASL/SCRAM**(SCRAM-SHA-256 / SCRAM-SHA-512)。开启后,客户端需提供 SASL 配置:

```properties
# client.properties
security.protocol=SASL_PLAINTEXT
sasl.mechanism=SCRAM-SHA-256
sasl.jaas.config=org.apache.kafka.common.security.scram.ScramLoginModule required \
  username="alice" password="alice-secret";
```

```bash
kafka-console-producer.sh --bootstrap-server localhost:9092 \
  --topic quickstart --producer.config client.properties
```

SCRAM 用户的创建与完整的安全配置说明,见 [CLI 操作指南](./CLI-Guide.md#scram-用户管理) 与 [兼容性与限制](./Compatibility-and-Limitations.md)。

## 下一步

- [核心概念](./KafkaCoreConcepts.md) — 理解 Topic / Offset / 消费组 / Coordinator
- [协议兼容矩阵](./Protocol.md) — 你的客户端能用哪些 API
- [CLI 操作指南](./CLI-Guide.md) — 官方命令行工具操作大全
