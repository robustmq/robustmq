# NATS Configuration

> This page describes every NATS (including mq9) protocol configuration item. For global/base configuration see [Broker Configuration](BROKER.md); for other protocols see [MQTT Configuration](MQTTConfig.md), [Kafka Configuration](KafkaConfig.md), [AMQP Configuration](AMQPConfig.md).

## [nats_runtime]

NATS Core / mq9 protocol service configuration.

```toml
[nats_runtime]
tcp_port = 4222
tls_port = 4223
ws_port = 4080
wss_port = 4443
max_payload = 1048576
auth_required = false
ping_interval = 60
ping_max = 3
ping_send_chunk = 10000
core_shard_num = 10
push_thread_num = 1
push_queue_thread_num = 10
mq9_mailbox_default_ttl = 86400
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `tcp_port` | `u32` | `4222` | NATS TCP listener port |
| `tls_port` | `u32` | `4223` | NATS TLS listener port |
| `ws_port` | `u32` | `4080` | NATS WebSocket listener port |
| `wss_port` | `u32` | `4443` | NATS WebSocket Secure listener port |
| `max_payload` | `u64` | `1048576` (1 MB) | Maximum payload size per message (bytes) |
| `auth_required` | `bool` | `false` | Whether client authentication is required |
| `ping_interval` | `u64` | `60` | Interval between server-initiated PINGs (seconds) |
| `ping_max` | `u64` | `3` | Maximum unanswered PINGs before the connection is closed |
| `ping_send_chunk` | `usize` | `10000` | Number of connections processed per PING send batch |
| `core_shard_num` | `usize` | `10` | Number of internal core shards |
| `push_thread_num` | `usize` | `1` | Number of direct-push threads (one per bucket) |
| `push_queue_thread_num` | `usize` | `10` | Number of queue-push threads (one per queue-group bucket) |
| `mq9_mailbox_default_ttl` | `u64` | `86400` | Default TTL (seconds) for mq9 mailboxes when the client doesn't specify one |

## Further Reading

- [Broker Configuration](BROKER.md) — global/base configuration
- [RobustMQ NATS Overview](../nats/Overview.md)
- [JetStream](../nats/JetStream.md)
