# Java Client

RobustMQ presents a standard Kafka Broker, so the official `kafka-clients` connects directly with no proprietary dependency. The examples below use `kafka-clients` 4.0. Other-language clients (the librdkafka family: C/C++, Python `confluent-kafka`, Go, etc.) work the same way — they all speak the standard Kafka wire protocol.

## Dependency

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

// acks: 0 / 1 / all (all recommended — waits for ISR acknowledgement)
props.put(ProducerConfig.ACKS_CONFIG, "all");
// Optional: enable idempotence to avoid duplicates from retries (supported)
props.put(ProducerConfig.ENABLE_IDEMPOTENCE_CONFIG, true);

try (Producer<String, String> producer = new KafkaProducer<>(props)) {
    producer.send(new ProducerRecord<>("my-topic", "key", "value"));
    producer.flush();
}
```

::: tip About acks and batch size
`acks=all` waits for ISR replica acknowledgement, giving stronger durability together with ISR multi-replica. A single record batch cannot exceed the broker's `max_message_bytes` (default ~1 MiB), otherwise it is rejected with `MESSAGE_TOO_LARGE`, see [Broker Configuration](../Configuration/BrokerConfig.md).
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
    // subscribe: partition assignment is done by the group coordinator (classic or KIP-848)
    consumer.subscribe(List.of("my-topic"));
    while (true) {
        ConsumerRecords<String, String> records = consumer.poll(Duration.ofMillis(500));
        for (ConsumerRecord<String, String> r : records) {
            System.out.printf("offset=%d key=%s value=%s%n", r.offset(), r.key(), r.value());
        }
    }
}
```

- **subscribe(topic)**: join a consumer group; the coordinator assigns partitions. Both the classic and the KIP-848 consumer-group protocols are supported.
- **assign(partitions)**: manually pin partitions without participating in group rebalance.

## AdminClient

```java
Properties props = new Properties();
props.put(AdminClientConfig.BOOTSTRAP_SERVERS_CONFIG, "localhost:9092");

try (Admin admin = Admin.create(props)) {
    // Create topic: 3 partitions, 1 replica, with optional topic config
    NewTopic topic = new NewTopic("my-topic", 3, (short) 1)
        .configs(Map.of("retention.ms", "604800000"));
    admin.createTopics(List.of(topic)).all().get();

    // List consumer groups
    admin.listConsumerGroups().all().get()
        .forEach(g -> System.out.println(g.groupId()));
}
```

Config passed via `configs` at create time is persisted and echoed by `DescribeConfigs`, but some keys are currently stored only and not yet fully wired into engine behavior, see [Topic Configuration](../Configuration/TopicConfig.md).

## SASL_PLAINTEXT + SCRAM

With SASL enabled on the broker (`kafka_runtime.sasl.enabled = true`, mechanisms default to `SCRAM-SHA-256` / `SCRAM-SHA-512`, see [Broker Configuration](../Configuration/BrokerConfig.md)), configure the client as:

```java
props.put("security.protocol", "SASL_PLAINTEXT");
props.put("sasl.mechanism", "SCRAM-SHA-256");
props.put("sasl.jaas.config",
    "org.apache.kafka.common.security.scram.ScramLoginModule required "
  + "username=\"<user>\" password=\"<password>\";");
```

> Inject the username/password via environment variables / a secrets manager — do not hardcode them in code or config.

## Other languages

Under the standard protocol, the following clients also work; just point `bootstrap.servers` at RobustMQ:

- **C / C++**: librdkafka
- **Python**: `confluent-kafka` (built on librdkafka)
- **Go**: `confluent-kafka-go` / `franz-go`

Related: [Deployment](../Operations/Deployment.md) · [Advertised Address](../Operations/AdvertisedListeners.md) (for remote connections, always verify the advertised address is reachable).
