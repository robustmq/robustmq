# MQTT Configuration

> This page describes every MQTT protocol configuration item. For global/base configuration see [Broker Configuration](BROKER.md); for other protocols see [Kafka Configuration](KafkaConfig.md), [AMQP Configuration](AMQPConfig.md), [NATS Configuration](NATSConfig.md).

## Overview

All MQTT-related configuration is grouped under the TOML `[mqtt_runtime]` table (corresponding to the `MqttRuntime` struct on the Rust side, organized the same way as `[kafka_runtime]`). This page documents it sub-table by sub-table. The environment-variable override rules are the same as in [Broker Configuration](BROKER.md#environment-variable-overrides), for example:

```bash
export ROBUST_MQ_SERVER_MQTT_RUNTIME_SERVER_TCP_PORT=1883
```

---

## 1. MQTT Server Configuration

### [mqtt_runtime.server]

MQTT protocol listener port configuration.

```toml
[mqtt_runtime.server]
tcp_port = 1883
tls_port = 1885
websocket_port = 8083
websockets_port = 8085
quic_port = 9083
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `tcp_port` | `u32` | `1883` | MQTT over TCP port |
| `tls_port` | `u32` | `1885` | MQTT over TLS port |
| `websocket_port` | `u32` | `8083` | MQTT over WebSocket port |
| `websockets_port` | `u32` | `8085` | MQTT over WebSocket Secure port |
| `quic_port` | `u32` | `9083` | MQTT over QUIC port |

---

## 2. MQTT Auth & Runtime Configuration

### [mqtt_runtime.auth]

```toml
[mqtt_runtime.auth]
default_user = "admin"
default_password = "robustmq"
durable_sessions_enable = false
secret_free_login = false
is_self_protection_status = false
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `default_user` | `string` | `"admin"` | Default system username |
| `default_password` | `string` | `"robustmq"` | Default system password |
| `durable_sessions_enable` | `bool` | `false` | Enable durable sessions (`false` = transient, memory-only, better performance) |
| `secret_free_login` | `bool` | `false` | Whether to allow passwordless login |
| `is_self_protection_status` | `bool` | `false` | Whether the node is in self-protection mode (rejects new connections under overload) |

> MQTT's network threads reuse the shared `[broker_network]` configuration (see [Broker Configuration](BROKER.md)) — there is no separate `[mqtt_runtime.auth.network]`.

---

## 3. MQTT Keep Alive Configuration

### [mqtt_runtime.keep_alive]

```toml
[mqtt_runtime.keep_alive]
enable = true
default_time = 180
max_time = 3600
default_timeout = 2
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `true` | Enable Keep Alive heartbeat detection |
| `default_time` | `u16` | `180` | Default heartbeat interval (seconds) |
| `max_time` | `u16` | `3600` | Maximum heartbeat interval (seconds) |
| `default_timeout` | `u16` | `2` | Number of consecutive timeouts before disconnecting |

---

## 4. MQTT Protocol Configuration

### [mqtt_runtime.protocol]

```toml
[mqtt_runtime.protocol]
max_session_expiry_interval = 1800
default_session_expiry_interval = 30
topic_alias_max = 65535
max_packet_size = 10485760
receive_max = 65535
max_message_expiry_interval = 3600
client_pkid_persistent = false
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `max_session_expiry_interval` | `u32` | `1800` | Maximum session expiry interval (seconds) |
| `default_session_expiry_interval` | `u32` | `30` | Default session expiry interval (seconds) |
| `topic_alias_max` | `u16` | `65535` | Maximum number of topic aliases |
| `max_packet_size` | `u32` | `10485760` (10 MB) | Maximum size of a single MQTT packet (bytes) |
| `receive_max` | `u16` | `65535` | Maximum number of unacknowledged PUBLISH packets |
| `max_message_expiry_interval` | `u64` | `3600` | Maximum message expiry interval (seconds) |
| `client_pkid_persistent` | `bool` | `false` | Whether to persist client Packet IDs |

---

## 5. MQTT Rate Limiting Configuration

### [mqtt_runtime.limit]

Cluster- and tenant-level connection/traffic rate limiting configuration.

```toml
[mqtt_runtime.limit.cluster]
max_connections_per_node = 10000000
max_connection_rate = 100000
max_topics = 5000000
max_sessions = 50000000
max_publish_rate = 10000

[mqtt_runtime.limit.tenant]
max_connections_per_node = 1000000
max_connection_rate = 10000
max_topics = 500000
max_sessions = 5000000
max_publish_rate = 10000
```

| Configuration | Type | Description |
|---------------|------|-------------|
| `max_connections_per_node` | `u64` | Maximum connections per node |
| `max_connection_rate` | `u32` | Maximum new connection rate per second |
| `max_topics` | `u64` | Maximum number of Topics |
| `max_sessions` | `u64` | Maximum number of Sessions |
| `max_publish_rate` | `u32` | Maximum Publish message rate per second |

---

## 6. MQTT Offline Message Configuration

### [mqtt_runtime.offline_message]

Configuration for storing messages while a client is offline.

```toml
[mqtt_runtime.offline_message]
enable = true
expire_ms = 0
max_messages_num = 0
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `true` | Enable offline messages |
| `expire_ms` | `u32` | `0` | Offline message expiry time (ms); `0` means never expires |
| `max_messages_num` | `u32` | `0` | Maximum offline messages per client; `0` means unlimited |

---

## 7. MQTT Connection Flapping Detection

### [mqtt_runtime.flapping_detect]

Detects clients that connect/disconnect too frequently ("flapping") and bans them automatically.

```toml
[mqtt_runtime.flapping_detect]
enable = false
window_time = 1
max_client_connections = 15
ban_time = 5
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `false` | Enable connection flapping detection |
| `window_time` | `u32` | `1` | Detection time window (seconds) |
| `max_client_connections` | `u64` | `15` | Maximum connection attempts within the window |
| `ban_time` | `u32` | `5` | Ban duration after flapping is triggered (seconds) |

---

## 8. MQTT Slow Subscribe Detection

### [mqtt_runtime.slow_subscribe]

Slow-subscription monitoring for detecting message delivery latency.

```toml
[mqtt_runtime.slow_subscribe]
enable = false
record_time = 1000
delay_type = "Whole"
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `false` | Enable slow-subscription detection |
| `record_time` | `u64` | `1000` | Slow-subscription record threshold (ms) |
| `delay_type` | `string` | `"Whole"` | Latency calculation type: `Whole` (end-to-end) or `Partial` |

---

## 9. MQTT Schema Validation Configuration

### [mqtt_runtime.schema]

Message schema validation configuration.

```toml
[mqtt_runtime.schema]
enable = true
strategy = "ALL"
failed_operation = "Discard"
echo_log = true
log_level = "info"
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `true` | Enable schema validation |
| `strategy` | `string` | `"ALL"` | Validation strategy |
| `failed_operation` | `string` | `"Discard"` | What to do on validation failure |
| `echo_log` | `bool` | `true` | Whether to log schema validation |
| `log_level` | `string` | `"info"` | Schema validation log level |

**Validation strategy:**
- `ALL`: the message must pass every bound schema
- `Any`: the message only needs to pass any one bound schema

**Failed operation:**
- `Discard`: discard the message that failed validation
- `DisconnectAndDiscard`: disconnect and discard the message
- `Ignore`: ignore the failure and keep processing

---

## 10. MQTT System Monitor Configuration

### [mqtt_runtime.system_monitor]

System resource monitoring configuration.

```toml
[mqtt_runtime.system_monitor]
enable = false
os_cpu_high_watermark = 70.0
os_memory_high_watermark = 80.0
system_topic_interval_ms = 60000
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `false` | Enable system resource monitoring |
| `os_cpu_high_watermark` | `f32` | `70.0` | CPU usage high watermark (%) |
| `os_memory_high_watermark` | `f32` | `80.0` | Memory usage high watermark (%) |
| `system_topic_interval_ms` | `u64` | `60000` | System topic metrics publish interval (ms) |

---

## Full Example

```toml
[mqtt_runtime.server]
tcp_port = 1883
tls_port = 1885
websocket_port = 8083
websockets_port = 8085
quic_port = 9083

[mqtt_runtime.auth]
default_user = "admin"
default_password = "your_secure_password"
durable_sessions_enable = false
secret_free_login = false
is_self_protection_status = false

[mqtt_runtime.keep_alive]
enable = true
default_time = 180
max_time = 3600
default_timeout = 2

[mqtt_runtime.protocol]
max_session_expiry_interval = 1800
default_session_expiry_interval = 30
topic_alias_max = 65535
max_packet_size = 10485760
receive_max = 65535
max_message_expiry_interval = 3600
client_pkid_persistent = false

[mqtt_runtime.offline_message]
enable = true
expire_ms = 0
max_messages_num = 0

[mqtt_runtime.flapping_detect]
enable = false
window_time = 1
max_client_connections = 15
ban_time = 5

[mqtt_runtime.slow_subscribe]
enable = false
record_time = 1000
delay_type = "Whole"

[mqtt_runtime.schema]
enable = true
strategy = "ALL"
failed_operation = "Discard"
echo_log = true
log_level = "info"

[mqtt_runtime.system_monitor]
enable = false
os_cpu_high_watermark = 70.0
os_memory_high_watermark = 80.0
system_topic_interval_ms = 60000
```

## Further Reading

- [Broker Configuration](BROKER.md) — global/base configuration
- [RobustMQ MQTT Core Concepts](../RobustMQ-MQTT/MQTTCoreConcepts.md)
