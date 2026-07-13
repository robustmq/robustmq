# Quick Start

This guide gets you from zero to producing and consuming in a few minutes: start RobustMQ, use the official Kafka CLI to create a topic / produce / consume, and run a minimal Java `kafka-clients` example.

## Prerequisites

- A working Kafka CLI (the `kafka-*.sh` scripts from any official 3.x / 4.x release).
- For the Java example: JDK 8+ and Maven / Gradle.
- The Kafka protocol listens on port `9092` by default.

## Start RobustMQ (single node)

```bash
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
robust-server start
```

The Kafka listener port is set by the `[kafka_runtime]` section of the config file, default `9092`:

```toml
[kafka_runtime]
tcp_port = 9092
```

> A single node does **not** enable SASL by default, so connections need no authentication — ideal for getting started. To enable authentication, see "SASL connection" below.

## Verify with the official CLI

### Create a topic

```bash
kafka-topics.sh --bootstrap-server localhost:9092 \
  --create --topic quickstart --partitions 3

# Describe
kafka-topics.sh --bootstrap-server localhost:9092 --describe --topic quickstart
```

> RobustMQ auto-creates topics by default, so producing directly also creates the topic implicitly; explicit creation gives you more control.

### Produce

```bash
kafka-console-producer.sh --bootstrap-server localhost:9092 --topic quickstart
>hello
>robustmq kafka
# Ctrl-C to finish
```

### Consume

```bash
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic quickstart --from-beginning
```

Seeing the two lines you just produced confirms the data plane works.

### Consume with a group and inspect progress

```bash
# Consume with consumer group g1
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic quickstart --group g1 --from-beginning

# Inspect the group's offsets and lag
kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group g1
```

For more commands, see the [CLI Guide](./CLI-Guide.md).

## Minimal Java kafka-clients example

Dependency (Maven):

```xml
<dependency>
    <groupId>org.apache.kafka</groupId>
    <artifactId>kafka-clients</artifactId>
    <version>3.7.0</version>
</dependency>
```

Producer:

```java
Properties props = new Properties();
props.put("bootstrap.servers", "localhost:9092");
props.put("key.serializer", "org.apache.kafka.common.serialization.StringSerializer");
props.put("value.serializer", "org.apache.kafka.common.serialization.StringSerializer");
// Optional: enable idempotent production (supported by RobustMQ)
props.put("enable.idempotence", "true");

try (KafkaProducer<String, String> producer = new KafkaProducer<>(props)) {
    producer.send(new ProducerRecord<>("quickstart", "k1", "hello from java"));
    producer.flush();
}
```

Consumer:

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

> **Tip**: `enable.idempotence=true` works (RobustMQ supports idempotent producers); but do **not** enable transactions (`transactional.id`) — RobustMQ does not support transactions yet, see [Compatibility & Limitations](./Compatibility-and-Limitations.md).

## SASL connection (optional)

RobustMQ supports **SASL/SCRAM** (SCRAM-SHA-256 / SCRAM-SHA-512). Once enabled, clients must provide SASL settings:

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

For creating SCRAM users and the full security configuration, see the [CLI Guide](./CLI-Guide.md#scram-user-management) and [Compatibility & Limitations](./Compatibility-and-Limitations.md).

## Next steps

- [Core Concepts](./KafkaCoreConcepts.md) — understand Topic / Offset / consumer groups / Coordinator
- [Protocol Compatibility Matrix](./Protocol.md) — which APIs your client can use
- [CLI Guide](./CLI-Guide.md) — complete guide to the official command-line tools
