# Experience RobustMQ Kafka

## Prerequisite: Start the Broker

Follow the [Quick Install](Quick-Install.md) guide to install RobustMQ, then start the service:

```bash
robust-server start
```

The Kafka protocol starts together with RobustMQ, listening on port `9092` by default — no extra configuration needed. **It's the same broker process as MQTT, NATS, mq9, and AMQP** — one command starts every protocol at once.

---

## Choose a Kafka Client

Use either of the following to test Kafka produce/consume.

### Option 1: Official Kafka CLI

Use the `kafka-*.sh` command-line tools bundled with any official Kafka distribution (3.x / 4.x) — no RobustMQ-specific tooling needed.

### Option 2: robust-bench (built-in)

`robust-bench` is RobustMQ's built-in benchmarking tool — no extra installation needed. See the full usage guide: [Bench CLI Documentation](../Bench/Bench-CLI.md)

---

## Create a Topic, Produce, and Consume

```bash
# create a topic
kafka-topics.sh --bootstrap-server localhost:9092 \
  --create --topic quickstart --partitions 3

# produce
kafka-console-producer.sh --bootstrap-server localhost:9092 --topic quickstart
>hello
>robustmq kafka
# Ctrl-C to end

# consume
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic quickstart --from-beginning
```

Seeing the two lines you just produced confirms the data path is working.

> RobustMQ auto-creates topics by default, so producing directly will implicitly create the topic too; creating it explicitly gives you more control.

### Consume with a Consumer Group

```bash
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic quickstart --group g1 --from-beginning

kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group g1
```

More commands in the [CLI Guide](../RobustMQ-Kafka/CLI-Guide.md).

### Benchmark (robust-bench)

```bash
robust-bench kafka pub --count 100 --duration-secs 30
robust-bench kafka sub --count 100 --duration-secs 30
```

More options in the [Bench CLI Documentation](../Bench/Bench-CLI.md).

---

## SDK Integration

RobustMQ is compatible with the standard Kafka protocol — any standard community Kafka client library works out of the box.

```java
Properties props = new Properties();
props.put("bootstrap.servers", "localhost:9092");
props.put("key.serializer", "org.apache.kafka.common.serialization.StringSerializer");
props.put("value.serializer", "org.apache.kafka.common.serialization.StringSerializer");

try (KafkaProducer<String, String> producer = new KafkaProducer<>(props)) {
    producer.send(new ProducerRecord<>("quickstart", "k1", "hello from java"));
    producer.flush();
}
```

## Next Steps

- [Core Concepts](../RobustMQ-Kafka/KafkaCoreConcepts.md) — Topics, offsets, consumer groups, coordinators
- [Quick Start (full version)](../RobustMQ-Kafka/QuickStart.md) — SASL connections, Java consumer example
- [Protocol Support](../RobustMQ-Kafka/Protocol.md) — which APIs your client can use
- [Compatibility & Limitations](../RobustMQ-Kafka/Compatibility-and-Limitations.md)
