# AMQP Configuration

> This page describes every AMQP protocol configuration item. For global/base configuration see [Broker Configuration](BROKER.md); for other protocols see [MQTT Configuration](MQTTConfig.md), [Kafka Configuration](KafkaConfig.md), [NATS Configuration](NATSConfig.md).

## [amqp_runtime]

AMQP protocol service configuration. Currently only a port setting exists — there is no TLS port or dynamic configuration yet; see [AMQP Compatibility & Limitations](../RobustMQ-AMQP/Compatibility-and-Limitations.md).

```toml
[amqp_runtime]
tcp_port = 5672
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `tcp_port` | `u32` | `5672` | AMQP protocol TCP listener port, matching RabbitMQ's default port |

## Further Reading

- [Broker Configuration](BROKER.md) — global/base configuration
- [RobustMQ AMQP Core Concepts](../RobustMQ-AMQP/AMQPCoreConcepts.md)
- [AMQP Compatibility & Limitations](../RobustMQ-AMQP/Compatibility-and-Limitations.md)
