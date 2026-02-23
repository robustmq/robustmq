# Cluster Management HTTP API

> This document describes HTTP API interfaces related to cluster configuration and management. For general information, please refer to [COMMON.md](COMMON.md).

## Cluster Configuration Management

### 1. Get Cluster Configuration

- **Endpoint**: `GET /api/cluster/config/get`
- **Description**: Get current cluster configuration information
- **Request Parameters**:
```json
{}
```

- **Response Example**:
```json
{
  "code": 0,
  "message": "success",
  "data": "Configuration information as JSON string"
}
```

- **Configuration Information Structure** (BrokerConfig):
```json
{
  "cluster_name": "robustmq-cluster",
  "broker_id": 1,
  "meta_service": {
    "addr": ["127.0.0.1:1228"],
    "timeout_ms": 30000
  },
  "network": {
    "tcp_port": 1883,
    "tls_port": 1885,
    "ws_port": 8083,
    "wss_port": 8085,
    "quic_port": 9083
  },
  "mqtt_offline_message": {
    "enable": true,
    "storage_type": "memory",
    "max_num": 1000
  },
  "mqtt_slow_subscribe": {
    "enable": true,
    "threshold_ms": 1000
  },
  "mqtt_system_monitor": {
    "enable": true,
    "os_cpu_high_watermark": 80.0,
    "os_cpu_low_watermark": 40.0,
    "os_memory_high_watermark": 80.0,
    "os_cpu_check_interval_ms": 5000
  },
  "mqtt_flapping_detect": {
    "enable": true,
    "window_time": 60,
    "max_client_connections": 10,
    "ban_time": 300
  }
}
```

### 2. Set Cluster Configuration

- **Endpoint**: `POST /api/cluster/config/set`
- **Description**: Set cluster configuration
- **Request Parameters**:
```json
{
  "config_type": "broker",           // Configuration type, currently supports "broker"
  "config": "{\"cluster_name\":\"new-cluster\",\"broker_id\":2,...}"  // Complete configuration JSON string
}
```

- **Response Example**:
```json
{
  "code": 0,
  "message": "success",
  "data": "Configuration updated successfully"
}
```

---

## Usage Examples

### Get Cluster Configuration
```bash
curl -X GET http://localhost:8080/api/cluster/config/get
```

### Set Cluster Configuration
```bash
# Set offline message feature
curl -X POST http://localhost:8080/api/cluster/config/set \
  -H "Content-Type: application/json" \
  -d '{
    "config_type": "broker",
    "config": "{\"mqtt_offline_message\":{\"enable\":true,\"storage_type\":\"rocksdb\",\"max_num\":10000}}"
  }'
```

### Enable System Monitoring
```bash
curl -X POST http://localhost:8080/api/cluster/config/set \
  -H "Content-Type: application/json" \
  -d '{
    "config_type": "broker",
    "config": "{\"mqtt_system_monitor\":{\"enable\":true,\"os_cpu_high_watermark\":90.0,\"os_cpu_low_watermark\":30.0}}"
  }'
```

### Configure Flapping Detection
```bash
curl -X POST http://localhost:8080/api/cluster/config/set \
  -H "Content-Type: application/json" \
  -d '{
    "config_type": "broker",
    "config": "{\"mqtt_flapping_detect\":{\"enable\":true,\"window_time\":120,\"max_client_connections\":20,\"ban_time\":600}}"
  }'
```

---

## Configuration Item Description

### Offline Message Configuration (mqtt_offline_message)
| Field | Type | Description |
|-------|------|-------------|
| `enable` | `bool` | Whether to enable offline messages |
| `storage_type` | `string` | Storage type: memory, rocksdb, mysql |
| `max_num` | `u32` | Maximum number of offline messages |

### Slow Subscribe Configuration (mqtt_slow_subscribe)
| Field | Type | Description |
|-------|------|-------------|
| `enable` | `bool` | Whether to enable slow subscribe detection |
| `threshold_ms` | `u64` | Slow subscribe threshold (milliseconds) |

### System Monitor Configuration (mqtt_system_monitor)
| Field | Type | Description |
|-------|------|-------------|
| `enable` | `bool` | Whether to enable system monitoring |
| `os_cpu_high_watermark` | `f32` | CPU high watermark (percentage) |
| `os_cpu_low_watermark` | `f32` | CPU low watermark (percentage) |
| `os_memory_high_watermark` | `f32` | Memory high watermark (percentage) |
| `os_cpu_check_interval_ms` | `u64` | CPU check interval (milliseconds) |

### Flapping Detection Configuration (mqtt_flapping_detect)
| Field | Type | Description |
|-------|------|-------------|
| `enable` | `bool` | Whether to enable flapping detection |
| `window_time` | `u64` | Time window (seconds) |
| `max_client_connections` | `u32` | Maximum connection count |
| `ban_time` | `u64` | Ban duration (seconds) |

---

## Cluster Status

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
      "offset": { "state": "Leader", "last_log_index": 1, "...": "..." },
      "meta":   { "state": "Leader", "last_log_index": 42853, "...": "..." }
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

## Notes

1. **Configuration Format**: Configuration must be a valid JSON string
2. **Configuration Validation**: Service validates configuration format and validity
3. **Hot Update**: Most configurations support hot updates without service restart
4. **Backup Recommendation**: It's recommended to get current configuration for backup before making changes
5. **Permission Requirements**: Configuration modifications usually require administrator privileges

---

*Documentation Version: v4.0*
*Last Updated: 2025-09-20*
*Based on Code Version: RobustMQ Admin Server v0.1.34*
