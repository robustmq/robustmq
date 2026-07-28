# Broker Configuration Reference

> This document describes RobustMQ's global/base configuration items. For logging configuration, see [Logging.md](Logging.md). Each protocol's own configuration now lives in its own page: [MQTT Configuration](MQTTConfig.md), [Kafka Configuration](KafkaConfig.md), [AMQP Configuration](AMQPConfig.md), [NATS Configuration](NATSConfig.md).

## Overview

RobustMQ uses TOML format configuration files for system configuration. The main configuration file is `config/server.toml`.

### Configuration Loading Priority

1. Environment variables (highest)
2. Configuration file
3. Default values (lowest)

### Environment Variable Override

Configuration file settings can be overridden using environment variables. Naming convention:

```text
ROBUST_MQ_SERVER_{SECTION}_{KEY}
```

- Top-level items: `ROBUST_MQ_SERVER_{KEY}`
- Section items: `ROBUST_MQ_SERVER_{SECTION}_{KEY}`
- All letters uppercase, `.` replaced with `_`

Examples:

```bash
export ROBUST_MQ_SERVER_CLUSTER_NAME="my-cluster"
export ROBUST_MQ_SERVER_MQTT_RUNTIME_SERVER_TCP_PORT=1883
export ROBUST_MQ_SERVER_RUNTIME_CHANNELS_PER_ADDRESS=8
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
http_port = 58080

[meta_addrs]
1 = "127.0.0.1:1228"
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `cluster_name` | `string` | `"robust_mq_cluster_default"` | Cluster name, must be identical across all nodes |
| `broker_id` | `u64` | `1` | Unique node identifier |
| `broker_ip` | `string` | Auto-detect local IP | Node IP address |
| `roles` | `array` | `["broker", "meta"]` | Node role list, options: `meta`, `broker`, `engine` |
| `grpc_port` | `u32` | `1228` | gRPC service port |
| `http_port` | `u32` | `58080` | HTTP API service port |
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

Tokio runtime, gRPC client connection pool, TLS, and pprof collection configuration. RobustMQ uses three independent Tokio runtimes internally, each serving a distinct role that can be tuned separately.

```toml
[runtime]
tls_cert = "./config/certs/cert.pem"
tls_key = "./config/certs/key.pem"
channels_per_address = 4
# Worker threads per runtime, 0 = auto (recommended)
# server_worker_threads = 0
# meta_worker_threads = 0
# broker_worker_threads = 0
# runtime_worker_threads = 1  # Legacy compat field, prefer per-runtime fields
# pprof_enable = false
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `tls_cert` | `string` | `"./config/certs/cert.pem"` | TLS certificate file path |
| `tls_key` | `string` | `"./config/certs/key.pem"` | TLS private key file path |
| `channels_per_address` | `usize` | `4` | Number of HTTP/2 Channels (TCP connections) maintained per gRPC server address |
| `server_worker_threads` | `usize` | `0` (auto) | server-runtime worker threads, auto = `max(4, CPU / 2)` |
| `meta_worker_threads` | `usize` | `0` (auto) | meta-runtime worker threads, auto = `max(4, CPU / 2)` |
| `broker_worker_threads` | `usize` | `0` (auto) | broker-runtime worker threads, auto = `CPU cores` |
| `runtime_worker_threads` | `usize` | `1` | Legacy global thread multiplier, used as fallback when per-runtime fields are 0 |
| `pprof_enable` | `bool` | `false` | Enable built-in pprof profiling collection; the resulting flamegraph is exposed via the Admin HTTP API (shares `http_port`), there is no separate port |

**Runtime Roles:**

| Runtime | Responsibilities | Default Threads |
|---------|-----------------|-----------------|
| `server-runtime` | gRPC service, HTTP Admin API, Prometheus metrics | `max(4, CPU/2)` |
| `meta-runtime` | Raft state machines, RocksDB writes | `max(4, CPU/2)` |
| `broker-runtime` | MQTT connection handling, message delivery hot path | `CPU cores` |

> **Tuning tip:** Keep the default `0`. Use the `tokio_runtime_busy_ratio` metric in Grafana to guide adjustments: if a runtime's busy ratio consistently exceeds 80%, consider increasing its thread count.

**gRPC client connection pool tuning:** Each HTTP/2 Channel supports approximately 200 concurrent Streams (concurrent RPC requests). The default value of `4` supports approximately 800 concurrent gRPC requests, covering the vast majority of production scenarios.

| Scenario | Recommended Value |
|----------|-------------------|
| Default / general production | `4` |
| High concurrency (tens of thousands of MQTT connections) | `8` ~ `16` |
| Extreme concurrency / stress testing | `32` |

> **Note:** Setting this value too high causes a surge in open TCP file descriptors (each Channel occupies one fd). In environments with a low `ulimit -n`, this may trigger `Too many open files`.

---

## 3. Meta Runtime Configuration

### [meta_runtime]

Metadata service heartbeat and Raft configuration.

```toml
[meta_runtime]
heartbeat_timeout_ms = 30000
heartbeat_check_time_ms = 1000
raft_write_timeout_sec = 30
offset_raft_group_num = 1
data_raft_group_num = 1
group_offset_expire_sec = 604800
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `heartbeat_timeout_ms` | `u64` | `30000` | Node heartbeat timeout (ms); node marked unavailable after timeout |
| `heartbeat_check_time_ms` | `u64` | `1000` | Heartbeat check interval (ms) |
| `raft_write_timeout_sec` | `u64` | `30` | Raft write operation timeout (seconds) |
| `offset_raft_group_num` | `u32` | `1` | Number of Offset Raft groups |
| `data_raft_group_num` | `u32` | `1` | Number of Data Raft groups |
| `group_offset_expire_sec` | `u64` | `604800` | Consumer group offset expiry time (seconds), default 7 days |

---

## 4. RocksDB Configuration

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

## 5. Storage Engine Runtime Configuration

### [storage_runtime]

Journal storage engine runtime configuration.

```toml
[storage_runtime]
tcp_port = 1778
max_segment_size = 1073741824
io_thread_num = 8
data_path = []
expire_scan_task_num = 10
offset_enable_cache = true
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `tcp_port` | `u32` | `1778` | Storage engine TCP port |
| `max_segment_size` | `u32` | `1073741824` (1 GB) | Maximum segment file size (bytes) |
| `io_thread_num` | `u32` | `8` | IO processing thread count |
| `data_path` | `array` | `[]` | Data storage path list |
| `expire_scan_task_num` | `usize` | `10` | Concurrent expired data scan tasks |
| `offset_enable_cache` | `bool` | `true` | Whether to enable consumer offset caching |

> The storage engine's network threads reuse the shared [`[broker_network]`](#7-broker-network-configuration) configuration — there is no separate `[storage_runtime.network]`.

---

## 6. Delay Task Configuration

### [delay_task]

Delayed message processing task queue configuration.

```toml
[delay_task]
delay_task_queue_num = 100
delay_task_handler_concurrency = 100
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `delay_task_queue_num` | `usize` | `100` | Number of delay task queues |
| `delay_task_handler_concurrency` | `usize` | `100` | Delay task handler concurrency |

---

## 7. Broker Network Configuration

### [broker_network]

Broker internal general network thread configuration.

```toml
[broker_network]
accept_thread_num = 2
handler_thread_num = 16
queue_size = 1000
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `accept_thread_num` | `usize` | `2` | Threads for accepting new connections |
| `handler_thread_num` | `usize` | `16` | Request handler thread count |
| `queue_size` | `usize` | `1000` | Internal processing queue size |

---

## 8. LLM Client Configuration

### [llm_client]

Configures the Broker's unified LLM client (`LLMClient`). This section is optional. If omitted, the LLM client is not enabled.

```toml
[llm_client]
platform = "open_ai"
model = "gpt-4o-mini"
token = "your_api_token"
# Optional: useful for OpenAI-compatible gateways or private deployments
# base_url = "https://api.openai.com/v1/"
# embedding = "text-embedding-3-small"
# embedding_model_path = "./models/embedding"
```

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `platform` | `string` | none | LLM provider identifier |
| `model` | `string` | none | Model name, e.g. `gpt-4o-mini`, `claude-3-5-sonnet`, `gemini-2.0-flash` |
| `token` | `string` | none | Access token. Required for all providers except `ollama` |
| `base_url` | `string` | none | Custom API base URL (optional) |
| `embedding` | `string` | none | Embedding model name (optional) |
| `embedding_model_path` | `string` | none | Local embedding model file path (optional) |

**`base_url` behavior (important):**

- If `base_url` is omitted, `genai` uses the provider's default official endpoint.
- Set `base_url` when using a proxy gateway, an OpenAI-compatible service, private deployment, or internal routing.
- For `ollama`, if omitted, the default is `http://localhost:11434/v1/`.

**Default endpoint behavior when `base_url` is omitted:**

| `platform` | Can omit `base_url` | Default endpoint |
|------------|----------------------|------------------|
| `open_ai` / `open_ai_resp` | yes | OpenAI official |
| `gemini` | yes | Google Gemini official |
| `anthropic` | yes | Anthropic official |
| `cohere` | yes | Cohere official |
| `xai` | yes | xAI official |
| `deep_seek` | yes | DeepSeek official |
| `groq` / `together` / `fireworks` / `nebius` / `mimo` / `zai` / `big_model` | yes | Each provider official |
| `ollama` | yes | `http://localhost:11434/v1/` |

**Allowed `platform` values:**

- `open_ai`, `open_ai_resp`, `gemini`, `anthropic`, `fireworks`, `together`, `groq`
- `mimo`, `nebius`, `xai`, `deep_seek`, `zai`, `big_model`, `cohere`, `ollama`

**Environment variable example:**

```bash
export ROBUST_MQ_SERVER_LLM_CLIENT_PLATFORM=open_ai
export ROBUST_MQ_SERVER_LLM_CLIENT_MODEL=gpt-4o-mini
export ROBUST_MQ_SERVER_LLM_CLIENT_TOKEN=your_api_token
```

---

## 9. Admin HTTP API Authentication

### [admin]

Authentication configuration for the Admin HTTP API. See [API Authentication](../Api/AUTH.md) for details.

```toml
[admin]
username = "admin"
password = "admin"
jwt_secret = "robustmq-change-me-in-production"
token_ttl_hours = 8
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `username` | `string` | `"admin"` | Admin username |
| `password` | `string` | `"admin"` | Admin password — **change this in production** |
| `jwt_secret` | `string` | `"robustmq-change-me-in-production"` | HMAC-SHA256 secret used to sign JWT tokens — use a random string of 32+ chars in production |
| `token_ttl_hours` | `u64` | `8` | Token validity period in hours |

> ⚠️ **Security notice**: The default `password` and `jwt_secret` values are insecure. Always change them before deploying to production.

**Auth rules:**
- Requests from `127.0.0.1` / `::1` (loopback): **no token required**, allowed through directly
- Requests from any other IP: must include `Authorization: Bearer <token>`
- `/api/v1/login`, `/health/*`, `/metrics`: always public, no auth required

---

## 10. Monitoring & Profiling

RobustMQ has no separate `[prometheus]` or `[pprof]` configuration section — both reuse the Admin HTTP API's `http_port` and have no dedicated configurable port:

- **Prometheus metrics**: always exposed via `GET /metrics` on the Admin HTTP API (see `http_port` in [Basic Configuration](#1-basic-configuration)); there is no enable/disable switch or separate port.
- **pprof profiling**: controlled by `runtime.pprof_enable` (see [2. Runtime Configuration](#2-runtime-configuration)); the resulting flamegraph is likewise exposed via the Admin HTTP API — there is no separate `port`/`frequency` configuration.

---

## Complete Configuration Example

This example covers every base configuration item. See each protocol's own page for its full example: [MQTT](MQTTConfig.md#full-example), [Kafka](KafkaConfig.md), [AMQP](AMQPConfig.md), [NATS](NATSConfig.md).

```toml
# ========== Basic Configuration ==========
cluster_name = "production-cluster"
broker_id = 1
roles = ["meta", "broker", "engine"]
grpc_port = 1228
http_port = 58080

[meta_addrs]
1 = "192.168.1.10:1228"
2 = "192.168.1.11:1228"
3 = "192.168.1.12:1228"

# ========== Runtime ==========
[runtime]
tls_cert = "./config/certs/cert.pem"
tls_key = "./config/certs/key.pem"
channels_per_address = 4
# server_worker_threads = 0
# meta_worker_threads = 0
# broker_worker_threads = 0
# pprof_enable = false

# ========== Meta ==========
[meta_runtime]
heartbeat_timeout_ms = 30000
heartbeat_check_time_ms = 1000
raft_write_timeout_sec = 30
offset_raft_group_num = 1
data_raft_group_num = 1
group_offset_expire_sec = 604800

# ========== RocksDB ==========
[rocksdb]
data_path = "/data/robustmq"
max_open_files = 20000

# ========== Storage Engine ==========
[storage_runtime]
tcp_port = 1778
max_segment_size = 1073741824
io_thread_num = 8
expire_scan_task_num = 10
offset_enable_cache = true

# ========== Delay Task ==========
[delay_task]
delay_task_queue_num = 100
delay_task_handler_concurrency = 100

# ========== Broker Network ==========
[broker_network]
accept_thread_num = 2
handler_thread_num = 16
queue_size = 1000

# ========== LLM Client (optional) ==========
[llm_client]
platform = "open_ai"
model = "gpt-4o-mini"
token = "your_api_token"
# base_url = "https://api.openai.com/v1/"
# embedding = "text-embedding-3-small"
# embedding_model_path = "./models/embedding"

# ========== Admin Authentication ==========
[admin]
username = "admin"
password = "your_secure_password"
jwt_secret = "your-random-jwt-secret-32-chars-min"
token_ttl_hours = 8

# ========== Logging ==========
[log]
log_config = "./config/broker-tracing.toml"
log_path = "./logs"
```
