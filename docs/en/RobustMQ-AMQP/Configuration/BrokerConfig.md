# Broker Configuration

AMQP-related broker configuration is minimal — currently a single port setting. Every other behavior (timeouts, heartbeats, authentication, etc.) is shared with RobustMQ's unified configuration.

## Configuration Fields

In the config file (e.g. `config/server.toml`), AMQP has its own `[amqp_runtime]` section:

```toml
[amqp_runtime]
tcp_port = 5672
```

| Field | Type | Default | Description |
|---|---|---|---|
| `tcp_port` | u32 | `5672` | The TCP port the AMQP protocol listens on, matching RabbitMQ's default port |

There is currently **no** `tls_port` field — AMQP does not support TLS; see [Security Overview](../Security/Overview.md).

## Configuration Shared with Other Protocols

The AMQP broker reuses the following RobustMQ-wide settings — there's nothing AMQP-specific to configure for these:

- **User authentication data**: shared with MQTT and Kafka's user table — see [Authentication (SASL)](../Security/Authentication-SASL.md).
- **Storage engine parameters** (File Segment's segment size, cleanup policy, etc.): configured once at the broker level, independent of protocol — see [Storage](../Storage.md).
- **meta-service/Raft cluster configuration**: cluster topology, election parameters, etc. are protocol-agnostic.

## Example: A Full Multi-Protocol Snippet

```toml
[network]
# general network configuration

[mqtt_runtime]
tcp_port = 1883

[kafka_runtime]
tcp_port = 9092

[amqp_runtime]
tcp_port = 5672
```

Changing `tcp_port` requires a broker restart to take effect — dynamic hot-reload of the AMQP port isn't supported yet.

## Further Reading

- [Security Overview](../Security/Overview.md)
- [Exchange & Queue Configuration](./ExchangeAndQueueConfig.md)
- [Quick Start](../QuickStart.md)
