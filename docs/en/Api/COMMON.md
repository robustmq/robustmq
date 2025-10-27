# RobustMQ Admin Server HTTP API Common Guide

## Overview

RobustMQ Admin Server is an HTTP management interface service, providing comprehensive management capabilities for RobustMQ clusters.

- **Base URL**: `http://localhost:8080`
- **API Prefix**: `/api` (all management interfaces use this prefix)
- **Request Method**: Mainly uses `POST` method
- **Data Format**: JSON
- **Response Format**: JSON

## API Documentation Navigation

- 📋 **[Cluster Management API](CLUSTER.md)** - Cluster configuration and status management
- 🔧 **[MQTT Broker API](MQTT.md)** - All MQTT broker related management interfaces

---

## Common Response Format

### Success Response
```json
{
  "code": 0,
  "message": "success",
  "data": {...}
}
```

### Error Response
```json
{
  "code": 500,
  "message": "error message",
  "data": null
}
```

### Paginated Response Format
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [...],
    "total_count": 100
  }
}
```

---

## Common Request Parameters

Most list query interfaces support the following common parameters:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | `u32` | No | Page size, default 10000 |
| `page` | `u32` | No | Page number, starts from 1, default 1 |
| `sort_field` | `string` | No | Sort field |
| `sort_by` | `string` | No | Sort order: asc/desc |
| `filter_field` | `string` | No | Filter field |
| `filter_values` | `array` | No | Filter value list |
| `exact_match` | `string` | No | Exact match: true/false |

### Pagination Parameters Example
```json
{
  "limit": 20,
  "page": 1,
  "sort_field": "create_time",
  "sort_by": "desc",
  "filter_field": "status",
  "filter_values": ["active"],
  "exact_match": "false"
}
```

---

## Basic Interface

### Service Version Query
- **Endpoint**: `GET /`
- **Description**: Get service version information
- **Request Parameters**: None
- **Response Example**:
```json
"RobustMQ API v0.1.34"
```

### Cluster Status Query
- **Endpoint**: `POST /api/status`
- **Description**: Get cluster status, version, and node information
- **Request Parameters**: `{}`（empty object）
- **Response Example**:
```json
{
  "code": 0,
  "data": {
    "version": "0.2.1",
    "cluster_name": "broker-server",
    "start_time": 1760828141,
    "broker_node_list": [
      {
        "cluster_name": "broker-server",
        "roles": ["meta", "broker"],
        "extend": "{\"mqtt\":{\"grpc_addr\":\"192.168.100.100:1228\",\"mqtt_addr\":\"192.168.100.100:1883\",\"mqtts_addr\":\"192.168.100.100:1885\",\"websocket_addr\":\"192.168.100.100:8083\",\"websockets_addr\":\"192.168.100.100:8085\",\"quic_addr\":\"192.168.100.100:9083\"}}",
        "node_id": 1,
        "node_ip": "192.168.100.100",
        "node_inner_addr": "192.168.100.100:1228",
        "start_time": 1760828141,
        "register_time": 1760828142
      }
    ],
    "meta": {
      "running_state": {
        "Ok": null
      },
      "id": 1,
      "current_term": 1,
      "vote": {
        "leader_id": {
          "term": 1,
          "node_id": 1
        },
        "committed": true
      },
      "last_log_index": 422,
      "last_applied": {
        "leader_id": {
          "term": 1,
          "node_id": 1
        },
        "index": 422
      },
      "snapshot": null,
      "purged": null,
      "state": "Leader",
      "current_leader": 1,
      "millis_since_quorum_ack": 0,
      "last_quorum_acked": 1760828146763525625,
      "membership_config": {
        "log_id": {
          "leader_id": {
            "term": 0,
            "node_id": 0
          },
          "index": 0
        },
        "membership": {
          "configs": [[1]],
          "nodes": {
            "1": {
              "node_id": 1,
              "rpc_addr": "127.0.0.1:1228"
            }
          }
        }
      },
      "heartbeat": {
        "1": 1760828146387602084
      },
      "replication": {
        "1": {
          "leader_id": {
            "term": 1,
            "node_id": 1
          },
          "index": 422
        }
      }
    }
  }
}
```

**Response Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `version` | `string` | RobustMQ version number |
| `cluster_name` | `string` | Cluster name |
| `start_time` | `u64` | Service start time (Unix timestamp in seconds) |
| `broker_node_list` | `array` | List of broker nodes |
| `meta` | `object` | Meta cluster Raft state information (structured object) |

**Broker Node Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `node_id` | `u64` | Node ID |
| `node_ip` | `string` | Node IP address |
| `node_inner_addr` | `string` | Node internal communication address (gRPC address) |
| `cluster_name` | `string` | Cluster name |
| `roles` | `array` | Node role list (e.g., `["meta", "broker"]`) |
| `extend` | `string` | Extended information (JSON string), containing protocol listening addresses |
| `start_time` | `u64` | Node start time (Unix timestamp in seconds) |
| `register_time` | `u64` | Node registration time (Unix timestamp in seconds) |

**Extended Information (extend) Fields**:

The `extend` field is a JSON string containing MQTT protocol-related address information:

```json
{
  "mqtt": {
    "grpc_addr": "192.168.100.100:1228",
    "mqtt_addr": "192.168.100.100:1883",
    "mqtts_addr": "192.168.100.100:1885",
    "websocket_addr": "192.168.100.100:8083",
    "websockets_addr": "192.168.100.100:8085",
    "quic_addr": "192.168.100.100:9083"
  }
}
```

| Field | Description |
|-------|-------------|
| `grpc_addr` | gRPC service address |
| `mqtt_addr` | MQTT protocol listening address |
| `mqtts_addr` | MQTT over TLS listening address |
| `websocket_addr` | WebSocket protocol listening address |
| `websockets_addr` | WebSocket over TLS listening address |
| `quic_addr` | QUIC protocol listening address |

**Meta Cluster State (meta) Fields**:

The `meta` field is a structured object containing Meta cluster Raft state information:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `u64` | Node ID |
| `state` | `string` | Current node state (Leader/Follower/Candidate) |
| `current_leader` | `u64` | Current leader node ID |
| `current_term` | `u64` | Current term number |
| `last_log_index` | `u64` | Last log index |
| `running_state` | `object` | Running state (typically `{"Ok": null}` for healthy) |
| `vote` | `object` | Vote information, including `leader_id` and `committed` |
| `last_applied` | `object` | Last applied log information |
| `snapshot` | `object/null` | Snapshot information |
| `purged` | `object/null` | Purge information |
| `millis_since_quorum_ack` | `u64` | Milliseconds since quorum acknowledgment |
| `last_quorum_acked` | `u128` | Last quorum acknowledged timestamp (nanoseconds) |
| `membership_config` | `object` | Cluster membership configuration |
| `heartbeat` | `object` | Heartbeat information (node ID to timestamp mapping) |
| `replication` | `object` | Replication state information |

---

## Error Code Description

| Error Code | Description |
|------------|-------------|
| 0 | Request successful |
| 400 | Request parameter error |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Resource not found |
| 500 | Internal server error |

---

## Usage Examples

### Basic Request Example
```bash
# Get service version
curl -X GET http://localhost:8080/

# Get cluster status
curl -X POST http://localhost:8080/api/status \
  -H "Content-Type: application/json" \
  -d '{}'

# List query with pagination
curl -X POST http://localhost:8080/api/mqtt/user/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1,
    "sort_field": "username",
    "sort_by": "asc"
  }'
```

### Error Handling Example
```bash
# When request fails, detailed error information is returned
{
  "code": 400,
  "message": "Invalid parameter: username is required",
  "data": null
}
```

---

## Notes

1. **Request Method**: 
   - Root path `/` uses GET method
   - All other interfaces (including `/api/status`) use POST method
2. **Request Body**: Even for query operations, a JSON format request body needs to be sent (can be an empty object `{}`)
3. **Time Format**: 
   - Input time uses Unix timestamp (seconds)
   - Output time uses local time format string "YYYY-MM-DD HH:MM:SS"
4. **Pagination**: Page number `page` starts counting from 1
5. **Configuration Validation**: Resource creation validates configuration format correctness
6. **Access Control**: It's recommended to add appropriate authentication and authorization mechanisms in production environments
7. **Error Handling**: All errors return detailed error information for easy debugging
8. **Content Type**: Requests must set the `Content-Type: application/json` header

---

## Development and Debugging

### Start Service
```bash
# Start admin-server
cargo run --bin admin-server

# Or use compiled binary
./target/release/admin-server
```

### Test Connection
```bash
# Test if service is running normally
curl -X GET http://localhost:8080/
```

### Log Viewing
The service outputs detailed log information during runtime, including:
- Request paths and parameters
- Response status and data
- Error messages and stack traces

---

*Documentation Version: v4.0*  
*Last Updated: 2025-09-20*  
*Based on Code Version: RobustMQ Admin Server v0.1.34*
