# Cluster Management HTTP API

> This document describes HTTP API interfaces related to cluster configuration and management. For general information, please refer to [COMMON.md](COMMON.md).

## Common Response Format

All interfaces return a unified JSON response structure:

**Success Response**:
```json
{
  "code": 0,
  "data": "...",
  "error": null
}
```

**Error Response**:
```json
{
  "code": 100,
  "data": "",
  "error": "error message"
}
```

---

## Cluster Configuration Management

### 1. Get Cluster Configuration

- **Endpoint**: `GET /api/cluster/config/get`
- **Description**: Get the complete `BrokerConfig` configuration of the current cluster
- **Request Parameters**: None

- **Response Example**:
```json
{
  "code": 0,
  "data": {
    "cluster_name": "robust_mq_cluster_default",
    "broker_id": 1,
    "broker_ip": "192.168.1.100",
    "roles": ["broker", "meta"],
    "grpc_port": 1228,
    "http_port": 8080,
    "meta_addrs": {
      "1": "127.0.0.1:1228"
    },
    "prometheus": {
      "enable": true,
      "port": 9090
    },
    "log": {
      "log_path": "./logs",
      "log_config": "./config/broker-tracing.toml"
    },
    "runtime": {
      "runtime_worker_threads": 1,
      "server_worker_threads": 0,
      "meta_worker_threads": 0,
      "broker_worker_threads": 0,
      "channels_per_address": 10,
      "tls_cert": "./config/certs/cert.pem",
      "tls_key": "./config/certs/key.pem"
    },
    "network": {
      "accept_thread_num": 8,
      "handler_thread_num": 32,
      "queue_size": 1000
    },
    "pprof": {
      "enable": false,
      "port": 6060,
      "frequency": 100
    },
    "rocksdb": {
      "data_path": "./data",
      "max_open_files": 10000
    },
    "llm_client": null,
    "cluster_limit": {
      "max_network_connection": 100000000,
      "max_network_connection_rate": 10000,
      "max_admin_http_uri_rate": 50
    },
    "meta_runtime": {
      "heartbeat_timeout_ms": 30000,
      "heartbeat_check_time_ms": 1000,
      "raft_write_timeout_sec": 30,
      "offset_raft_group_num": 1,
      "data_raft_group_num": 1
    },
    "storage_runtime": {
      "tcp_port": 1778,
      "max_segment_size": 1073741824,
      "io_thread_num": 8,
      "data_path": [],
      "offset_enable_cache": true
    },
    "mqtt_server": {
      "tcp_port": 1883,
      "tls_port": 1885,
      "websocket_port": 8083,
      "websockets_port": 8085,
      "quic_port": 9083
    },
    "mqtt_keep_alive": {
      "enable": true,
      "default_time": 180,
      "max_time": 3600,
      "default_timeout": 2
    },
    "mqtt_runtime": {
      "default_user": "admin",
      "default_password": "robustmq",
      "durable_sessions_enable": false,
      "secret_free_login": false,
      "is_self_protection_status": false
    },
    "mqtt_offline_message": {
      "enable": true,
      "expire_ms": 0,
      "max_messages_num": 0
    },
    "mqtt_slow_subscribe": {
      "enable": false,
      "record_time": 1000,
      "delay_type": "Whole"
    },
    "mqtt_flapping_detect": {
      "enable": false,
      "window_time": 1,
      "max_client_connections": 15,
      "ban_time": 5
    },
    "mqtt_protocol": {
      "max_session_expiry_interval": 1800,
      "default_session_expiry_interval": 30,
      "topic_alias_max": 65535,
      "max_packet_size": 10485760,
      "receive_max": 65535,
      "max_message_expiry_interval": 3600,
      "client_pkid_persistent": false
    },
    "mqtt_schema": {
      "enable": true,
      "strategy": "ALL",
      "failed_operation": "Discard",
      "echo_log": true,
      "log_level": "info"
    },
    "mqtt_system_monitor": {
      "enable": false,
      "os_cpu_high_watermark": 70.0,
      "os_memory_high_watermark": 80.0,
      "system_topic_interval_ms": 60000
    },
    "mqtt_limit": {
      "cluster": {
        "max_connections_per_node": 10000000,
        "max_connection_rate": 100000,
        "max_topics": 5000000,
        "max_sessions": 50000000,
        "max_publish_rate": 10000
      },
      "tenant": {
        "max_connections_per_node": 1000000,
        "max_connection_rate": 10000,
        "max_topics": 500000,
        "max_sessions": 5000000,
        "max_publish_rate": 10000
      }
    }
  },
  "error": null
}
```

---

### 2. Set Cluster Configuration

- **Endpoint**: `POST /api/cluster/config/set`
- **Description**: Dynamically update cluster configuration. Changes take effect immediately and are persisted to Meta storage.
- **Request Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `config_type` | string | Yes | Configuration type, see options below |
| `config` | string | Yes | JSON string of the corresponding configuration type |

`config_type` options and their `config` field structure:

---

#### `MqttSlowSubscribeConfig` — Slow Subscribe Detection

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enable` | bool | `false` | Whether to enable |
| `record_time` | u64 | `1000` | Slow subscribe threshold (ms); subscriptions exceeding this time are recorded |
| `delay_type` | string | `"CreateTime"` | Delay measurement method: `"CreateTime"` or `"PublishTime"` |

```json
{
  "config_type": "MqttSlowSubscribeConfig",
  "config": "{\"enable\":true,\"record_time\":500,\"delay_type\":\"CreateTime\"}"
}
```

---

#### `MqttFlappingDetect` — Connection Flapping Detection

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enable` | bool | `false` | Whether to enable |
| `window_time` | u32 | `60` | Detection window duration (seconds) |
| `max_client_connections` | u64 | `5` | Maximum allowed connections in the window; exceeding this triggers a ban |
| `ban_time` | u32 | `300` | Ban duration (seconds) |

```json
{
  "config_type": "MqttFlappingDetect",
  "config": "{\"enable\":true,\"window_time\":60,\"max_client_connections\":5,\"ban_time\":300}"
}
```

---

#### `MqttProtocol` — MQTT Protocol Parameters

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_session_expiry_interval` | u32 | `2592000` | Maximum session expiry interval (seconds) |
| `default_session_expiry_interval` | u32 | `3600` | Default session expiry interval (seconds) |
| `topic_alias_max` | u16 | `65535` | Maximum number of topic aliases |
| `max_packet_size` | u32 | `10485760` | Maximum packet size (bytes, default 10 MB) |
| `receive_max` | u16 | `65535` | Receive window size |
| `max_message_expiry_interval` | u64 | `86400` | Maximum message expiry interval (seconds) |
| `client_pkid_persistent` | bool | `false` | Whether to persist client Packet IDs |

```json
{
  "config_type": "MqttProtocol",
  "config": "{\"max_session_expiry_interval\":2592000,\"default_session_expiry_interval\":3600,\"topic_alias_max\":65535,\"max_packet_size\":10485760,\"receive_max\":65535,\"max_message_expiry_interval\":86400,\"client_pkid_persistent\":false}"
}
```

---

#### `MqttOfflineMessage` — Offline Messages

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enable` | bool | `false` | Whether to enable |
| `expire_ms` | u32 | `86400000` | Offline message expiry time (ms) |
| `max_messages_num` | u32 | `1000` | Maximum offline messages stored per client |

```json
{
  "config_type": "MqttOfflineMessage",
  "config": "{\"enable\":true,\"expire_ms\":86400000,\"max_messages_num\":1000}"
}
```

---

#### `MqttSystemMonitor` — System Monitor

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enable` | bool | `false` | Whether to enable system monitoring |
| `os_cpu_high_watermark` | f32 | `0.7` | CPU usage high watermark (0~1) |
| `os_memory_high_watermark` | f32 | `0.8` | Memory usage high watermark (0~1) |
| `system_topic_interval_ms` | u64 | `60000` | System topic publish interval (ms) |

```json
{
  "config_type": "MqttSystemMonitor",
  "config": "{\"enable\":true,\"os_cpu_high_watermark\":0.7,\"os_memory_high_watermark\":0.8,\"system_topic_interval_ms\":60000}"
}
```

---

#### `MqttSchema` — Schema Validation

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enable` | bool | `false` | Whether to enable schema validation |
| `strategy` | string | `"Forward"` | Validation strategy: `"Forward"` (pass-through) or `"Strict"` |
| `failed_operation` | string | `"Disconnect"` | Action on validation failure: `"Disconnect"` or `"Ignore"` |
| `echo_log` | bool | `false` | Whether to log schema validation results |
| `log_level` | string | `"info"` | Log level |

```json
{
  "config_type": "MqttSchema",
  "config": "{\"enable\":true,\"strategy\":\"Strict\",\"failed_operation\":\"Disconnect\",\"echo_log\":false,\"log_level\":\"info\"}"
}
```

---

#### `MqttLimit` — MQTT Rate Limits

| Field | Type | Description |
|-------|------|-------------|
| `cluster` | object | Cluster-level limits, see `LimitQuota` |
| `tenant` | object | Tenant default limits, see `LimitQuota` |

**`LimitQuota` fields**:

| Field | Type | Description |
|-------|------|-------------|
| `max_connections_per_node` | u64 | Maximum connections per node |
| `max_connection_rate` | u32 | Maximum connection rate (connections/second) |
| `max_topics` | u64 | Maximum number of topics |
| `max_sessions` | u64 | Maximum number of sessions |
| `max_publish_rate` | u32 | Maximum publish rate (messages/second) |

```json
{
  "config_type": "MqttLimit",
  "config": "{\"cluster\":{\"max_connections_per_node\":10000000,\"max_connection_rate\":100000,\"max_topics\":5000000,\"max_sessions\":50000000,\"max_publish_rate\":10000},\"tenant\":{\"max_connections_per_node\":1000000,\"max_connection_rate\":10000,\"max_topics\":500000,\"max_sessions\":5000000,\"max_publish_rate\":10000}}"
}
```

---

#### `ClusterLimit` — Cluster Access Limits

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_network_connection` | u64 | `100000000` | Maximum total network connections |
| `max_network_connection_rate` | u32 | `10000` | Maximum network connection rate (connections/second) |
| `max_admin_http_uri_rate` | u32 | `50` | Maximum Admin HTTP request rate (requests/second) |

```json
{
  "config_type": "ClusterLimit",
  "config": "{\"max_network_connection\":100000000,\"max_network_connection_rate\":10000,\"max_admin_http_uri_rate\":50}"
}
```

---

- **Response Example**:
```json
{
  "code": 0,
  "data": "success",
  "error": null
}
```

---

## Cluster Information

### 3. Get Cluster Status

- **Endpoint**: `GET /api/cluster/status`
- **Description**: Returns cluster runtime status, including version, node list, and Raft group status for each internal group (`mqtt`, `offset`, `meta`).

- **Response Example**:
```json
{
  "code": 0,
  "data": {
    "version": "0.3.0",
    "cluster_name": "robust_mq_cluster_default",
    "start_time": 1738800000,
    "broker_node_list": [],
    "nodes": ["127.0.0.1"],
    "meta": {
      "mqtt": {
        "running_state": { "Ok": null },
        "id": 1,
        "current_term": 1,
        "vote": { "leader_id": { "term": 1, "node_id": 1 }, "committed": true },
        "last_log_index": 30001,
        "last_applied": { "leader_id": { "term": 1, "node_id": 1 }, "index": 30001 },
        "snapshot": { "leader_id": { "term": 1, "node_id": 1 }, "index": 30001 },
        "purged": { "leader_id": { "term": 1, "node_id": 1 }, "index": 30001 },
        "state": "Leader",
        "current_leader": 1,
        "millis_since_quorum_ack": 0,
        "membership_config": {
          "log_id": { "leader_id": { "term": 0, "node_id": 0 }, "index": 0 },
          "membership": {
            "configs": [[1]],
            "nodes": { "1": { "node_id": 1, "rpc_addr": "127.0.0.1:1228" } }
          }
        },
        "replication": { "1": { "leader_id": { "term": 1, "node_id": 1 }, "index": 30001 } }
      },
      "offset": {
        "state": "Leader",
        "current_leader": 1,
        "last_log_index": 1,
        "...": "..."
      },
      "meta": {
        "state": "Leader",
        "current_leader": 1,
        "last_log_index": 42853,
        "...": "..."
      }
    }
  },
  "error": null
}
```

**`data` fields**:

| Field | Type | Description |
|-------|------|-------------|
| `version` | string | Current Broker version |
| `cluster_name` | string | Cluster name |
| `start_time` | u64 | Process start time (Unix seconds) |
| `broker_node_list` | array | List of all Broker nodes in the cluster |
| `nodes` | string[] | Deduplicated list of cluster node IPs |
| `meta` | object | Raft group status map, keyed by group name |

**`meta` keys**:

| Key | Description |
|-----|-------------|
| `mqtt` | MQTT data Raft group status |
| `offset` | Offset data Raft group status |
| `meta` | Metadata Raft group status |

**Per-group status (`MetaStatus`) fields**:

| Field | Type | Description |
|-------|------|-------------|
| `running_state` | object | Runtime health; `{"Ok": null}` means healthy |
| `id` | u64 | Current node ID |
| `current_term` | u64 | Current Raft term |
| `vote` | object | Current vote information |
| `last_log_index` | u64 | Latest log index |
| `last_applied` | object | Log position last applied to the state machine |
| `snapshot` | object/null | Latest snapshot position |
| `purged` | object/null | Oldest log position that has been purged |
| `state` | string | Node role: `Leader`, `Follower`, or `Candidate` |
| `current_leader` | u64 | Current Leader node ID |
| `millis_since_quorum_ack` | u64 | Milliseconds since last quorum acknowledgement |
| `membership_config` | object | Cluster membership configuration |
| `replication` | object | Per-node replication progress (key = node ID) |

---

## BrokerConfig Field Reference

### Base Configuration

| Field | Type | Description |
|-------|------|-------------|
| `cluster_name` | string | Cluster name |
| `broker_id` | u64 | Broker node ID |
| `broker_ip` | string/null | Broker IP address |
| `roles` | string[] | Node roles: `meta`, `broker`, `engine` |
| `grpc_port` | u32 | gRPC service port |
| `http_port` | u32 | HTTP API service port |
| `meta_addrs` | object | Meta node address map (node ID → address) |

### prometheus

| Field | Type | Description |
|-------|------|-------------|
| `enable` | bool | Whether to enable Prometheus metrics |
| `port` | u16 | Prometheus metrics port |

### log

| Field | Type | Description |
|-------|------|-------------|
| `log_path` | string | Log output directory |
| `log_config` | string | Log tracing config file path |

### runtime

| Field | Type | Description |
|-------|------|-------------|
| `runtime_worker_threads` | usize | Legacy global thread multiplier; used as fallback when per-runtime fields are 0 |
| `server_worker_threads` | usize | Server runtime threads (0 = auto, equals CPU core count) |
| `meta_worker_threads` | usize | Meta runtime threads (0 = auto) |
| `broker_worker_threads` | usize | Broker runtime threads (0 = auto, hot-path runtime) |
| `channels_per_address` | usize | Number of gRPC connection channels per address |
| `tls_cert` | string | TLS certificate file path |
| `tls_key` | string | TLS private key file path |

### network

| Field | Type | Description |
|-------|------|-------------|
| `accept_thread_num` | usize | Number of connection accept threads |
| `handler_thread_num` | usize | Number of message handler threads |
| `queue_size` | usize | Internal queue size |

### pprof

| Field | Type | Description |
|-------|------|-------------|
| `enable` | bool | Whether to enable pprof profiling |
| `port` | u16 | pprof HTTP port |
| `frequency` | i32 | Sampling frequency |

### meta_runtime

| Field | Type | Description |
|-------|------|-------------|
| `heartbeat_timeout_ms` | u64 | Heartbeat timeout (ms) |
| `heartbeat_check_time_ms` | u64 | Heartbeat check interval (ms) |
| `raft_write_timeout_sec` | u64 | Raft write timeout (seconds) |
| `offset_raft_group_num` | u32 | Number of Offset Raft shard groups (default 1) |
| `data_raft_group_num` | u32 | Number of data Raft shard groups (default 1) |

### rocksdb

| Field | Type | Description |
|-------|------|-------------|
| `data_path` | string | RocksDB data directory |
| `max_open_files` | i32 | Maximum open files |

### storage_runtime

| Field | Type | Description |
|-------|------|-------------|
| `tcp_port` | u32 | Storage engine TCP port |
| `max_segment_size` | u32 | Maximum segment file size (bytes) |
| `io_thread_num` | u32 | Number of IO threads |
| `data_path` | string[] | List of data storage paths |
| `offset_enable_cache` | bool | Whether to enable offset cache |

### mqtt_server

| Field | Type | Description |
|-------|------|-------------|
| `tcp_port` | u32 | MQTT TCP port |
| `tls_port` | u32 | MQTT TLS port |
| `websocket_port` | u32 | MQTT WebSocket port |
| `websockets_port` | u32 | MQTT WebSocket Secure port |
| `quic_port` | u32 | MQTT QUIC port |

### mqtt_keep_alive

| Field | Type | Description |
|-------|------|-------------|
| `enable` | bool | Whether to enable keep-alive detection |
| `default_time` | u16 | Default keep-alive time (seconds) |
| `max_time` | u16 | Maximum keep-alive time (seconds) |
| `default_timeout` | u16 | Timeout multiplier |

### mqtt_runtime

| Field | Type | Description |
|-------|------|-------------|
| `default_user` | string | Default username |
| `default_password` | string | Default password |
| `durable_sessions_enable` | bool | Whether to enable durable sessions |
| `secret_free_login` | bool | Whether to allow password-free login |
| `is_self_protection_status` | bool | Whether the node is in self-protection mode |

### mqtt_offline_message

| Field | Type | Description |
|-------|------|-------------|
| `enable` | bool | Whether to enable offline messages |
| `expire_ms` | u32 | Expiry time (ms), 0 means no expiry |
| `max_messages_num` | u32 | Maximum offline messages per client, 0 means unlimited |

### mqtt_slow_subscribe

| Field | Type | Description |
|-------|------|-------------|
| `enable` | bool | Whether to enable slow subscribe detection |
| `record_time` | u64 | Recording threshold (ms) |
| `delay_type` | string | Delay type: `Whole`, `Internal`, `Response` |

### mqtt_flapping_detect

| Field | Type | Description |
|-------|------|-------------|
| `enable` | bool | Whether to enable connection flapping detection |
| `window_time` | u32 | Detection window (minutes) |
| `max_client_connections` | u64 | Maximum connections in the window |
| `ban_time` | u32 | Ban duration (minutes) |

### mqtt_protocol

| Field | Type | Description |
|-------|------|-------------|
| `max_session_expiry_interval` | u32 | Maximum session expiry interval (seconds) |
| `default_session_expiry_interval` | u32 | Default session expiry interval (seconds) |
| `topic_alias_max` | u16 | Maximum Topic Alias value |
| `max_packet_size` | u32 | Maximum packet size (bytes) |
| `receive_max` | u16 | Receive maximum |
| `max_message_expiry_interval` | u64 | Maximum message expiry interval (seconds) |
| `client_pkid_persistent` | bool | Whether to persist client Packet IDs |

### mqtt_schema

| Field | Type | Description |
|-------|------|-------------|
| `enable` | bool | Whether to enable schema validation |
| `strategy` | string | Validation strategy: `ALL`, `Any` |
| `failed_operation` | string | Action on failure: `Discard`, `DisconnectAndDiscard`, `Ignore` |
| `echo_log` | bool | Whether to output validation logs |
| `log_level` | string | Log level |

### mqtt_system_monitor

| Field | Type | Description |
|-------|------|-------------|
| `enable` | bool | Whether to enable system monitoring |
| `os_cpu_high_watermark` | f32 | CPU high watermark (percentage) |
| `os_memory_high_watermark` | f32 | Memory high watermark (percentage) |
| `system_topic_interval_ms` | u64 | System topic metrics publish interval (ms) |

### cluster_limit

| Field | Type | Description |
|-------|------|-------------|
| `max_network_connection` | u64 | Maximum total network connections in the cluster |
| `max_network_connection_rate` | u32 | Maximum new connection rate per second in the cluster |
| `max_admin_http_uri_rate` | u32 | Maximum Admin HTTP request rate per second |

### mqtt_limit

Both `cluster` and `tenant` use the `LimitQuota` structure:

| Field | Type | Description |
|-------|------|-------------|
| `max_connections_per_node` | u64 | Maximum connections per node |
| `max_connection_rate` | u32 | Maximum new connection rate per second |
| `max_topics` | u64 | Maximum number of topics |
| `max_sessions` | u64 | Maximum number of sessions |
| `max_publish_rate` | u32 | Maximum publish rate per second |

### llm_client

Optional configuration; `null` when not set.

| Field | Type | Description |
|-------|------|-------------|
| `platform` | string | LLM platform, e.g. `open_ai`, `anthropic`, `ollama` |
| `model` | string | Model name |
| `token` | string/null | API token (not required for Ollama) |
| `base_url` | string/null | Custom API base URL (must start with `http://` or `https://`) |

---

## Usage Examples

### Get Cluster Configuration
```bash
curl -X GET http://localhost:8080/api/cluster/config/get
```

### Get Cluster Status
```bash
curl -X GET http://localhost:8080/api/cluster/status
```

### Set Flapping Detection Configuration
```bash
curl -X POST http://localhost:8080/api/cluster/config/set \
  -H "Content-Type: application/json" \
  -d '{
    "config_type": "MqttFlappingDetect",
    "config": "{\"enable\":true,\"window_time\":2,\"max_client_connections\":20,\"ban_time\":10}"
  }'
```

---

## Tenant Management

A **Tenant** is the core multi-tenancy concept in RobustMQ, providing logical isolation of cluster resources. Use tenants when a single cluster serves multiple business units, multiple environments (dev / staging / prod), or multiple teams.

### 4. List Tenants

- **Endpoint**: `GET /api/tenant/list`
- **Description**: List all tenants. Supports pagination, sorting, and filtering.
- **Query Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tenant_name` | string | No | Fuzzy search by tenant name (contains match) |
| `page` | u32 | No | Page number, starting from 1 (default: 1) |
| `limit` | u32 | No | Items per page (default: 100) |
| `sort_field` | string | No | Sort field; supports `tenant_name` |
| `sort_by` | string | No | Sort direction: `asc` or `desc` |

- **Response Example**:
```json
{
  "code": 0,
  "data": {
    "data": [
      {
        "tenant_name": "business-a",
        "desc": "Business A tenant",
        "config": {
          "max_connections_per_node": 10000000,
          "max_create_connection_rate_per_second": 10000,
          "max_topics": 5000000,
          "max_sessions": 50000000,
          "max_publish_rate": 10000
        },
        "create_time": 1738800000
      }
    ],
    "total_count": 1
  },
  "error": null
}
```

- **curl Example**:
```bash
# List all tenants
curl -X GET "http://localhost:8080/api/tenant/list"

# Fuzzy search tenants with "business" in the name
curl -X GET "http://localhost:8080/api/tenant/list?tenant_name=business"
```

---

### 5. Create Tenant

- **Endpoint**: `POST /api/tenant/create`
- **Description**: Create a new tenant.
- **Request Body**:

| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| `tenant_name` | string | Yes | Length 1–128 | Tenant name, must be globally unique |
| `desc` | string | No | Length ≤ 500 | Human-readable description |
| `config` | object | No | - | Tenant resource quota; defaults are used if not set |
| `config.max_connections_per_node` | u64 | No | - | Max connections per node (default: 10000000) |
| `config.max_create_connection_rate_per_second` | u32 | No | - | Max new connection rate per second (default: 10000) |
| `config.max_topics` | u64 | No | - | Max topics (default: 5000000) |
| `config.max_sessions` | u64 | No | - | Max sessions (default: 50000000) |
| `config.max_publish_rate` | u32 | No | - | Max publish rate per second (default: 10000) |

- **Request Example**:
```json
{
  "tenant_name": "business-a",
  "desc": "Business A tenant",
  "config": {
    "max_connections_per_node": 50000,
    "max_topics": 100000,
    "max_sessions": 200000,
    "max_publish_rate": 5000
  }
}
```

- **Response Example**:
```json
{
  "code": 0,
  "data": "success",
  "error": null
}
```

- **curl Example**:
```bash
curl -X POST http://localhost:8080/api/tenant/create \
  -H "Content-Type: application/json" \
  -d '{"tenant_name": "business-a", "desc": "Business A tenant"}'
```

---

### 6. Delete Tenant

- **Endpoint**: `POST /api/tenant/delete`
- **Description**: Delete a tenant by name. After deletion, metadata belonging to the tenant is no longer managed by it.
- **Request Body**:

| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| `tenant_name` | string | Yes | Length 1–128 | Name of the tenant to delete |

- **Request Example**:
```json
{
  "tenant_name": "business-a"
}
```

- **Response Example**:
```json
{
  "code": 0,
  "data": "success",
  "error": null
}
```

- **curl Example**:
```bash
curl -X POST http://localhost:8080/api/tenant/delete \
  -H "Content-Type: application/json" \
  -d '{"tenant_name": "business-a"}'
```

---

### 7. Update Tenant

- **Endpoint**: `POST /api/tenant/update`
- **Description**: Update a tenant's description and resource quota. The tenant must already exist. If `config` is omitted, the existing configuration is preserved.
- **Request Body**:

| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| `tenant_name` | string | Yes | Length 1–128 | Name of the tenant to update |
| `desc` | string | No | Length ≤ 500 | New description |
| `config` | object | No | - | Resource quota; existing config is kept if omitted |
| `config.max_connections_per_node` | u64 | No | - | Max connections per node |
| `config.max_create_connection_rate_per_second` | u32 | No | - | Max new connection rate per second |
| `config.max_topics` | u64 | No | - | Max topics |
| `config.max_sessions` | u64 | No | - | Max sessions |
| `config.max_publish_rate` | u32 | No | - | Max publish rate per second |

- **Request Example**:
```json
{
  "tenant_name": "business-a",
  "desc": "Business A tenant (updated)",
  "config": {
    "max_connections_per_node": 100000,
    "max_publish_rate": 20000
  }
}
```

- **Response Example**:
```json
{
  "code": 0,
  "data": "success",
  "error": null
}
```

- **Error Response** (when tenant does not exist):
```json
{
  "code": 1,
  "message": "Tenant business-a not found",
  "data": null
}
```

- **curl Example**:
```bash
curl -X POST http://localhost:8080/api/tenant/update \
  -H "Content-Type: application/json" \
  -d '{"tenant_name": "business-a", "desc": "Business A tenant (updated)", "config": {"max_connections_per_node": 100000}}'
```

---

## Health Check

### 8. Liveness Check

- **Endpoint**: `GET /cluster/healthy`
- **Description**: Check if the service is alive. Returns `true` when healthy.
- **Request Parameters**: None
- **Response Example**:
```json
{
  "code": 0,
  "message": "success",
  "data": true
}
```

---

### 9. Readiness Check

- **Endpoint**: `GET /health/ready`
- **Description**: Check whether all configured ports are ready. Intended for use as a Kubernetes readiness probe.
- **Request Parameters**: None
- **Response**:
  - **200 OK** — All ports ready:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "status": "ok",
    "check_type": "ready",
    "message": "all configured ports are ready"
  }
}
```
  - **503 Service Unavailable** — Ports not ready:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "status": "not_ready",
    "check_type": "ready",
    "message": "one or more configured ports are not ready"
  }
}
```

---

### 10. Node Health Check

- **Endpoint**: `GET /health/node`
- **Description**: Node-level health check (placeholder implementation, always returns ok).
- **Request Parameters**: None
- **Response Example**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "status": "ok",
    "check_type": "node",
    "message": "health check placeholder"
  }
}
```

---

### 11. Cluster Health Check

- **Endpoint**: `GET /health/cluster`
- **Description**: Cluster-level health check (placeholder implementation, always returns ok).
- **Request Parameters**: None
- **Response Example**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "status": "ok",
    "check_type": "cluster",
    "message": "health check placeholder"
  }
}
```

---

## Notes

1. **Response Format**: On success, `code` is `0` and `error` is `null`; on failure, `code` is `100` and `error` contains the error message.
2. **Configuration Format**: The `config` field in `config/set` must be a valid JSON string.
3. **Hot Update**: Most configurations support hot updates without restarting the service.
4. **Backup Recommendation**: It is recommended to fetch the current configuration via `config/get` before making changes.
5. **Tenant Isolation**: Tenants provide logical isolation, suitable for private deployments. For SaaS multi-tenancy requiring stronger isolation boundaries, consider physical separation.
