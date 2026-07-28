# Kafka Configuration

> This page describes every Kafka protocol configuration item. For global/base configuration see [Broker Configuration](BROKER.md); for other protocols see [MQTT Configuration](MQTTConfig.md), [AMQP Configuration](AMQPConfig.md), [NATS Configuration](NATSConfig.md).

## [kafka_runtime]

Kafka protocol service configuration.

```toml
[kafka_runtime]
tcp_port = 9092
max_fetch_bytes = 4194304
max_message_bytes = 1048588
max_describe_topic_partitions = 2000
auto_create_topics_enable = true

[kafka_runtime.sasl]
enabled = false
mechanisms = ["SCRAM-SHA-256", "SCRAM-SHA-512"]
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `tcp_port` | `u32` | `9092` | Kafka protocol TCP listener port |
| `max_fetch_bytes` | `u32` | `4194304` (4 MB) | Upper bound, per partition, on how many bytes a Fetch response may return, regardless of the client's `max_bytes`/`partition_max_bytes` |
| `max_message_bytes` | `u32` | `1048588` | Upper bound on the size of a single produced record batch (matches Kafka's `message.max.bytes`/`max.message.bytes`); a larger batch is rejected with `MESSAGE_TOO_LARGE` |
| `max_describe_topic_partitions` | `u32` | `2000` | Upper bound on how many partitions a single `DescribeTopicPartitions` response may return, regardless of the client's `response_partition_limit` |
| `auto_create_topics_enable` | `bool` | `true` | Whether to auto-create a Topic on first produce/fetch to an unknown name (overridable via cluster dynamic config; the `config_type` is still `KafkaDynamic`) |

**[kafka_runtime.sasl] SASL authentication configuration:**

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enabled` | `bool` | `false` | Whether SASL authentication is enabled; when `false`, connections are accepted without authentication and the handshake/authenticate handlers stay inert |
| `mechanisms` | `array` | `["SCRAM-SHA-256", "SCRAM-SHA-512"]` | SASL mechanisms the broker offers |

## Further Reading

- [Broker Configuration](BROKER.md) — global/base configuration
- [RobustMQ Kafka Core Concepts](../RobustMQ-Kafka/KafkaCoreConcepts.md)
- [RobustMQ Kafka SASL/SCRAM Authentication](../RobustMQ-Kafka/Security/Authentication-SASL-SCRAM.md)
