# Broker Configuration Reference

> This document describes all configuration items for the RobustMQ Broker service. For logging configuration, see [Logging.md](Logging.md).

## Overview

RobustMQ uses TOML format configuration files for system configuration. The main configuration file is `config/server.toml`.

### Configuration Loading Priority

1. Environment variables (highest)
2. Configuration file
3. Default values (lowest)

### Environment Variable Override

Configuration file settings can be overridden using environment variables. Naming convention:

```
ROBUST_MQ_SERVER_{SECTION}_{KEY}
```

- Top-level items: `ROBUST_MQ_SERVER_{KEY}`
- Section items: `ROBUST_MQ_SERVER_{SECTION}_{KEY}`
- All letters uppercase, `.` replaced with `_`

Examples:

```bash
export ROBUST_MQ_SERVER_CLUSTER_NAME="my-cluster"
export ROBUST_MQ_SERVER_MQTT_SERVER_TCP_PORT=1883
export ROBUST_MQ_SERVER_PROMETHEUS_PORT=9091
```

---

## 1. Basic Configuration

Top-level configuration items defining cluster and node information.

```toml
cluster_name = "robust_mq_cluster_default"
broker_id = 1
broker_ip = "127.0.0.1"
roles = ["broker", "meta"]
grpc_port = 1228
http_port = 8080

[meta_addrs]
1 = "127.0.0.1:1228"
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `cluster_name` | `string` | `"robust_mq_cluster_default"` | Cluster name, must be consistent across all nodes |
| `broker_id` | `u64` | `1` | Unique node identifier |
| `broker_ip` | `string` | Auto-detect local IP | Node IP address |
| `roles` | `array` | `["broker", "meta"]` | Node role list, options: `meta`, `broker`, `engine` |
| `grpc_port` | `u32` | `1228` | gRPC service port |
| `http_port` | `u32` | `8080` | HTTP API service port |
| `meta_addrs` | `table` | `{1 = "127.0.0.1:1228"}` | Meta node address mapping, key is node ID, value is `IP:port` |

### Deployment Modes

- **Integrated deployment**: `roles = ["meta", "broker", "engine"]`
- **Separated deployment**:
  - Meta nodes: `roles = ["meta"]`
  - Broker nodes: `roles = ["broker"]`
  - Engine nodes: `roles = ["engine"]`

---

## 2. Runtime Configuration

### [runtime]

Tokio runtime and TLS configuration.

```toml
[runtime]
runtime_worker_threads = 8
tls_cert = "./config/certs/cert.pem"
tls_key = "./config/certs/key.pem"
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `runtime_worker_threads` | `usize` | Auto-detect CPU cores | Tokio runtime worker thread count |
| `tls_cert` | `string` | `"./config/certs/cert.pem"` | TLS certificate file path |
| `tls_key` | `string` | `"./config/certs/key.pem"` | TLS private key file path |

---

## 3. Network Configuration

### [network]

Network layer thread and queue configuration.

```toml
[network]
accept_thread_num = 8
handler_thread_num = 32
response_thread_num = 8
queue_size = 1000
lock_max_try_mut_times = 30
lock_try_mut_sleep_time_ms = 50
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `accept_thread_num` | `usize` | `8` | Thread count for accepting new connections |
| `handler_thread_num` | `usize` | `32` | Request handler thread count |
| `response_thread_num` | `usize` | `8` | Response sender thread count |
| `queue_size` | `usize` | `1000` | Internal message queue size |
| `lock_max_try_mut_times` | `u64` | `30` | Maximum lock acquisition retry count |
| `lock_try_mut_sleep_time_ms` | `u64` | `50` | Lock retry interval (milliseconds) |

---

## 4. Meta Runtime Configuration

### [meta_runtime]

Metadata service heartbeat and Raft configuration.

```toml
[meta_runtime]
heartbeat_timeout_ms = 30000
heartbeat_check_time_ms = 1000
raft_write_timeout_sec = 30
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `heartbeat_timeout_ms` | `u64` | `30000` | Node heartbeat timeout (ms), node marked unavailable after timeout |
| `heartbeat_check_time_ms` | `u64` | `1000` | Heartbeat check interval (ms) |
| `raft_write_timeout_sec` | `u64` | `30` | Raft write operation timeout (seconds) |

---

## 5. RocksDB Configuration

### [rocksdb]

Local RocksDB storage configuration.

```toml
[rocksdb]
data_path = "./data"
max_open_files = 10000
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `data_path` | `string` | `"./data"` | RocksDB data storage directory |
| `max_open_files` | `i32` | `10000` | Maximum simultaneously open files |

---

## 6. Storage Engine Runtime Configuration

### [storage_runtime]

Journal storage engine runtime configuration.

```toml
[storage_runtime]
tcp_port = 1778
max_segment_size = 1073741824
io_thread_num = 8
data_path = []
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `tcp_port` | `u32` | `1778` | Storage engine TCP port |
| `max_segment_size` | `u32` | `1073741824` (1 GB) | Maximum segment file size (bytes) |
| `io_thread_num` | `u32` | `8` | IO processing thread count |
| `data_path` | `array` | `[]` | Data storage path list |

---

## 7. Message Storage Configuration

### [message_storage]

Message persistence storage backend configuration.

```toml
[message_storage]
storage_type = "EngineMemory"
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `storage_type` | `string` | `"EngineMemory"` | Storage type |

**Available storage types:**

| Value | Description |
|-------|-------------|
| `EngineMemory` | In-memory storage (data lost on restart, suitable for testing) |
| `EngineSegment` | Segment-based storage engine |
| `EngineRocksDB` | RocksDB-based local storage |
| `Mysql` | MySQL database storage |
| `MinIO` | MinIO object storage |
| `S3` | AWS S3 object storage |

Depending on the selected `storage_type`, configure the corresponding sub-items:

**memory_config (optional for EngineMemory):**

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `max_records_per_shard` | `usize` | `1000` | Maximum records per shard |
| `max_shard_size_limit` | `usize` | `10000000` | Maximum total size per shard |

**mysql_config (for Mysql):**

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `mysql_addr` | `string` | `""` | MySQL database address |

**minio_config (for MinIO):**

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `data_dir` | `string` | `""` | MinIO data directory |
| `bucket` | `string` | `""` | MinIO bucket name |

**s3_config (for S3):**

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `endpoint` | `string` | `""` | S3 endpoint address |
| `bucket` | `string` | `""` | S3 bucket name |
| `region` | `string` | `""` | S3 region |
| `access_key` | `string` | `""` | Access key |
| `secret_key` | `string` | `""` | Secret key |
| `enable_virtual_host_style` | `bool` | `false` | Whether to use virtual host style access |

---

## 8. Offset Storage Configuration

### [storage_offset]

Message consumption offset cache configuration.

```toml
[storage_offset]
enable_cache = true
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable_cache` | `bool` | `true` | Whether to enable offset caching |

---

## 9. MQTT Server Configuration

### [mqtt_server]

MQTT protocol listener port configuration.

```toml
[mqtt_server]
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

## 10. MQTT Runtime Configuration

### [mqtt_runtime]

MQTT runtime basic parameters.

```toml
[mqtt_runtime]
default_user = "admin"
default_password = "robustmq"
max_connection_num = 1000000
durable_sessions_enable = false
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `default_user` | `string` | `"admin"` | System default username |
| `default_password` | `string` | `"robustmq"` | System default password |
| `max_connection_num` | `usize` | `1000000` | Maximum connections per node |
| `durable_sessions_enable` | `bool` | `false` | Whether to enable durable sessions (`false` for transient sessions, better performance) |

---

## 11. MQTT Keep Alive Configuration

### [mqtt_keep_alive]

MQTT heartbeat keep-alive configuration.

```toml
[mqtt_keep_alive]
enable = true
default_time = 180
max_time = 3600
default_timeout = 2
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `true` | Whether to enable Keep Alive heartbeat detection |
| `default_time` | `u16` | `180` | Default heartbeat interval (seconds) |
| `max_time` | `u16` | `3600` | Maximum heartbeat interval (seconds) |
| `default_timeout` | `u16` | `2` | Disconnect after consecutive timeout count |

---

## 12. MQTT Protocol Configuration

### [mqtt_protocol_config]

MQTT protocol parameter configuration.

```toml
[mqtt_protocol_config]
max_session_expiry_interval = 1800
default_session_expiry_interval = 30
topic_alias_max = 65535
max_qos_flight_message = 2
max_packet_size = 10485760
receive_max = 65535
max_message_expiry_interval = 3600
client_pkid_persistent = false
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `max_session_expiry_interval` | `u32` | `1800` | Maximum session expiry time (seconds) |
| `default_session_expiry_interval` | `u32` | `30` | Default session expiry time (seconds) |
| `topic_alias_max` | `u16` | `65535` | Maximum number of topic aliases |
| `max_qos_flight_message` | `u8` | `2` | Maximum QoS in-flight messages |
| `max_packet_size` | `u32` | `10485760` (10 MB) | Maximum MQTT packet size (bytes) |
| `receive_max` | `u16` | `65535` | Maximum unacknowledged PUBLISH packets |
| `max_message_expiry_interval` | `u64` | `3600` | Maximum message expiry time (seconds) |
| `client_pkid_persistent` | `bool` | `false` | Whether to persist client Packet IDs |

---

## 13. MQTT Security Configuration

### [mqtt_security]

MQTT cluster security dynamic configuration.

```toml
[mqtt_security]
is_self_protection_status = false
secret_free_login = false
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `is_self_protection_status` | `bool` | `false` | Whether to enable self-protection mode |
| `secret_free_login` | `bool` | `false` | Whether to allow password-free login |

---

## 14. MQTT Authentication & Authorization Configuration

### [mqtt_auth_config]

MQTT authentication (AuthN) and authorization (AuthZ) configuration.

```toml
[mqtt_auth_config.authn_config]
authn_type = "password_based"

[mqtt_auth_config.authn_config.password_based_config]
# Password-based authentication configuration

[mqtt_auth_config.authn_config.password_based_config.storage_config]
storage_type = "placement"

[mqtt_auth_config.authn_config.password_based_config.password_config]
credential_type = "username"
algorithm = "plain"
salt_position = "disable"

[mqtt_auth_config.authz_config.storage_config]
storage_type = "placement"
```

**Authentication types (authn_type):** `password_based`, `JWT`

**Password algorithms (algorithm):** `plain`, `md5`, `sha`, `sha256`, `sha512`, `bcrypt`, `pbkdf2`

**Auth data storage types (storage_type):** `placement`, `mysql`, `postgresql`, `redis`, `http`, `file` (AuthZ only)

---

## 15. MQTT Offline Message Configuration

### [mqtt_offline_message]

Message storage configuration for offline clients.

```toml
[mqtt_offline_message]
enable = true
expire_ms = 0
max_messages_num = 0
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `true` | Whether to enable offline messages |
| `expire_ms` | `u32` | `0` | Offline message expiry time (ms), `0` means no expiry |
| `max_messages_num` | `u32` | `0` | Maximum offline messages per client, `0` means unlimited |

---

## 16. MQTT Flapping Detection Configuration

### [mqtt_flapping_detect]

Detect clients that connect/disconnect frequently (flapping) and auto-ban them.

```toml
[mqtt_flapping_detect]
enable = false
window_time = 1
max_client_connections = 15
ban_time = 5
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `false` | Whether to enable flapping detection |
| `window_time` | `u32` | `1` | Detection time window (seconds) |
| `max_client_connections` | `u64` | `15` | Maximum connection attempts within time window |
| `ban_time` | `u32` | `5` | Ban duration after triggering flapping (seconds) |

---

## 17. MQTT Slow Subscribe Detection Configuration

### [mqtt_slow_subscribe_config]

Slow subscription monitoring for detecting message delivery delays.

```toml
[mqtt_slow_subscribe_config]
enable = false
record_time = 1000
delay_type = "Whole"
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `false` | Whether to enable slow subscribe detection |
| `record_time` | `u64` | `1000` | Slow subscribe threshold (milliseconds) |
| `delay_type` | `string` | `"Whole"` | Delay calculation type: `Whole` (end-to-end), `Partial` (partial) |

---

## 18. MQTT Schema Validation Configuration

### [mqtt_schema]

Message Schema validation configuration.

```toml
[mqtt_schema]
enable = true
strategy = "ALL"
failed_operation = "Discard"
echo_log = true
log_level = "info"
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `true` | Whether to enable Schema validation |
| `strategy` | `string` | `"ALL"` | Validation strategy |
| `failed_operation` | `string` | `"Discard"` | Operation when validation fails |
| `echo_log` | `bool` | `true` | Whether to output Schema validation logs |
| `log_level` | `string` | `"info"` | Schema validation log level |

**Validation strategies (strategy):**
- `ALL`: Message must pass all bound Schema validations
- `Any`: Message only needs to pass any Schema validation

**Failed operations (failed_operation):**
- `Discard`: Discard messages that fail validation
- `DisconnectAndDiscard`: Disconnect and discard messages
- `Ignore`: Ignore validation failures and continue processing

---

## 19. MQTT System Monitor Configuration

### [mqtt_system_monitor]

System resource monitoring configuration.

```toml
[mqtt_system_monitor]
enable = false
os_cpu_high_watermark = 70.0
os_memory_high_watermark = 80.0
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `false` | Whether to enable system resource monitoring |
| `os_cpu_high_watermark` | `f32` | `70.0` | CPU usage high watermark (%) |
| `os_memory_high_watermark` | `f32` | `80.0` | Memory usage high watermark (%) |

---

## 20. gRPC Client Configuration

### [grpc_client]

Controls the connection pool behavior of the Broker's internal gRPC client. RobustMQ uses HTTP/2 for inter-node communication. Each Channel is an independent TCP connection that supports multiplexing.

```toml
[grpc_client]
channels_per_address = 4
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `channels_per_address` | `usize` | `4` | Number of HTTP/2 Channels maintained per gRPC server address |

**Tuning Guide:**

Each HTTP/2 Channel supports approximately 200 concurrent Streams (concurrent RPC requests). The default value of `4` supports approximately 800 concurrent gRPC requests, which covers the vast majority of production scenarios.

| Scenario | Recommended Value |
|----------|-------------------|
| Default / general production | `4` |
| High concurrency (tens of thousands of MQTT connections) | `8` ~ `16` |
| Extreme concurrency / stress testing | `32` |

> **Note:** Setting this value too high will cause a surge in open TCP file descriptors (each Channel occupies one fd). In environments with a low `ulimit -n`, this may trigger a `Too many open files` error.

**Environment variable:**
```bash
export ROBUST_MQ_SERVER_GRPC_CLIENT_CHANNELS_PER_ADDRESS=8
```

---

## 21. Monitoring Configuration

### [prometheus]

Prometheus metrics exposure configuration.

```toml
[prometheus]
enable = true
port = 9090
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `true` | Whether to enable Prometheus metrics collection |
| `port` | `u32` | `9090` | Prometheus metrics exposure port |

### [p_prof]

PProf performance profiling configuration.

```toml
[p_prof]
enable = false
port = 6060
frequency = 100
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `false` | Whether to enable PProf profiling |
| `port` | `u16` | `6060` | PProf service port |
| `frequency` | `i32` | `100` | Sampling frequency |

---

## Complete Configuration Example

```toml
# ========== Basic Configuration ==========
cluster_name = "production-cluster"
broker_id = 1
roles = ["meta", "broker", "engine"]
grpc_port = 1228
http_port = 8080

[meta_addrs]
1 = "192.168.1.10:1228"
2 = "192.168.1.11:1228"
3 = "192.168.1.12:1228"

# ========== Runtime ==========
[runtime]
runtime_worker_threads = 8
tls_cert = "./config/certs/cert.pem"
tls_key = "./config/certs/key.pem"

# ========== Network ==========
[network]
accept_thread_num = 8
handler_thread_num = 32
response_thread_num = 8
queue_size = 2000

# ========== Meta ==========
[meta_runtime]
heartbeat_timeout_ms = 30000
heartbeat_check_time_ms = 1000
raft_write_timeout_sec = 30

# ========== RocksDB ==========
[rocksdb]
data_path = "/data/robustmq"
max_open_files = 20000

# ========== Storage Engine ==========
[storage_runtime]
tcp_port = 1778
max_segment_size = 1073741824
io_thread_num = 8

# ========== Message Storage ==========
[message_storage]
storage_type = "EngineMemory"

# ========== Offset Cache ==========
[storage_offset]
enable_cache = true

# ========== MQTT Server ==========
[mqtt_server]
tcp_port = 1883
tls_port = 1885
websocket_port = 8083
websockets_port = 8085
quic_port = 9083

# ========== MQTT Runtime ==========
[mqtt_runtime]
default_user = "admin"
default_password = "your_secure_password"
max_connection_num = 1000000
durable_sessions_enable = false

# ========== MQTT Keep Alive ==========
[mqtt_keep_alive]
enable = true
default_time = 180
max_time = 3600
default_timeout = 2

# ========== MQTT Protocol ==========
[mqtt_protocol_config]
max_session_expiry_interval = 1800
default_session_expiry_interval = 30
topic_alias_max = 65535
max_qos_flight_message = 2
max_packet_size = 10485760
receive_max = 65535
max_message_expiry_interval = 3600
client_pkid_persistent = false

# ========== MQTT Security ==========
[mqtt_security]
is_self_protection_status = false
secret_free_login = false

# ========== MQTT Authentication ==========
[mqtt_auth_config.authn_config]
authn_type = "password_based"

[mqtt_auth_config.authz_config.storage_config]
storage_type = "placement"

# ========== MQTT Offline Messages ==========
[mqtt_offline_message]
enable = true
expire_ms = 0
max_messages_num = 0

# ========== MQTT Flapping Detection ==========
[mqtt_flapping_detect]
enable = false
window_time = 1
max_client_connections = 15
ban_time = 5

# ========== MQTT Slow Subscribe ==========
[mqtt_slow_subscribe_config]
enable = false
record_time = 1000
delay_type = "Whole"

# ========== MQTT Schema ==========
[mqtt_schema]
enable = true
strategy = "ALL"
failed_operation = "Discard"
echo_log = true
log_level = "info"

# ========== MQTT System Monitor ==========
[mqtt_system_monitor]
enable = false
os_cpu_high_watermark = 70.0
os_memory_high_watermark = 80.0

# ========== Monitoring ==========
[prometheus]
enable = true
port = 9090

[p_prof]
enable = false
port = 6060
frequency = 100

# ========== gRPC Client ==========
[grpc_client]
channels_per_address = 4

# ========== Logging ==========
[log]
log_config = "./config/broker-tracing.toml"
log_path = "./logs"
```
