# Java 客户端接入

RobustMQ 对外是标准 Kafka Broker,直接用官方 `kafka-clients` 即可接入,无需任何私有依赖。下面以 `kafka-clients` 4.0 为例。其它语言客户端(librdkafka 系列:C/C++、Python `confluent-kafka`、Go 等)同样适用——因为走的是标准 Kafka 线上协议。

## 依赖

```xml
<dependency>
  <groupId>org.apache.kafka</groupId>
  <artifactId>kafka-clients</artifactId>
  <version>4.0.0</version>
</dependency>
```

## Producer

```java
Properties props = new Properties();
props.put(ProducerConfig.BOOTSTRAP_SERVERS_CONFIG, "localhost:9092");
props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, StringSerializer.class.getName());
props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, StringSerializer.class.getName());

// acks: 0 / 1 / all(推荐 all,等待 ISR 确认)
props.put(ProducerConfig.ACKS_CONFIG, "all");
// 可选:开启幂等,避免重试导致的重复写入(已支持)
props.put(ProducerConfig.ENABLE_IDEMPOTENCE_CONFIG, true);

try (Producer<String, String> producer = new KafkaProducer<>(props)) {
    producer.send(new ProducerRecord<>("my-topic", "key", "value"));
    producer.flush();
}
```

::: tip 关于 acks 与批大小
`acks=all` 会等待 ISR 副本确认,配合 ISR 多副本提供更强的持久性。单个 record batch 不能超过 broker 的 `max_message_bytes`(默认约 1 MiB),否则返回 `MESSAGE_TOO_LARGE`,见 [Broker 配置](../Configuration/BrokerConfig.md)。
:::

## Consumer

```java
Properties props = new Properties();
props.put(ConsumerConfig.BOOTSTRAP_SERVERS_CONFIG, "localhost:9092");
props.put(ConsumerConfig.GROUP_ID_CONFIG, "my-group");
props.put(ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG, StringDeserializer.class.getName());
props.put(ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG, StringDeserializer.class.getName());
props.put(ConsumerConfig.AUTO_OFFSET_RESET_CONFIG, "earliest");

try (Consumer<String, String> consumer = new KafkaConsumer<>(props)) {
    // subscribe:交给消费组协调器做分区分配(经典或 KIP-848 协议)
    consumer.subscribe(List.of("my-topic"));
    while (true) {
        ConsumerRecords<String, String> records = consumer.poll(Duration.ofMillis(500));
        for (ConsumerRecord<String, String> r : records) {
            System.out.printf("offset=%d key=%s value=%s%n", r.offset(), r.key(), r.value());
        }
    }
}
```

- **subscribe(topic)**:加入消费组,由协调器分配分区,支持经典与 KIP-848 两种消费组协议。
- **assign(partitions)**:手动指定分区,不参与消费组 rebalance。

## AdminClient

```java
Properties props = new Properties();
props.put(AdminClientConfig.BOOTSTRAP_SERVERS_CONFIG, "localhost:9092");

try (Admin admin = Admin.create(props)) {
    // 建 topic:3 分区 1 副本,可带 topic 配置
    NewTopic topic = new NewTopic("my-topic", 3, (short) 1)
        .configs(Map.of("retention.ms", "604800000"));
    admin.createTopics(List.of(topic)).all().get();

    // 查消费组
    admin.listConsumerGroups().all().get()
        .forEach(g -> System.out.println(g.groupId()));
}
```

建 topic 时传入的 `configs` 会被持久化并可通过 `DescribeConfigs` 回显,但部分配置目前仅存储、尚未全部接入引擎行为,见 [Topic 配置](../Configuration/TopicConfig.md)。

## SASL_PLAINTEXT + SCRAM

Broker 开启 SASL 后(`kafka_runtime.sasl.enabled = true`,机制默认 `SCRAM-SHA-256` / `SCRAM-SHA-512`,见 [Broker 配置](../Configuration/BrokerConfig.md)),客户端配置如下:

```java
props.put("security.protocol", "SASL_PLAINTEXT");
props.put("sasl.mechanism", "SCRAM-SHA-256");
props.put("sasl.jaas.config",
    "org.apache.kafka.common.security.scram.ScramLoginModule required "
  + "username=\"<user>\" password=\"<password>\";");
```

> 请通过环境变量 / 密钥管理注入用户名密码,不要硬编码在代码或配置里。

## 其它语言

标准协议下,以下客户端同样可用,只需把 `bootstrap.servers` 指向 RobustMQ:

- **C / C++**:librdkafka
- **Python**:`confluent-kafka`(基于 librdkafka)
- **Go**:`confluent-kafka-go` / `franz-go`

相关文档:[部署](../Operations/Deployment.md) · [广告地址机制](../Operations/AdvertisedListeners.md)(远程连接务必确认广告地址可达)。
