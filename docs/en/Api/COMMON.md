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
- **Description**: Get complete cluster status information, including RobustMQ version, cluster name, start time, broker node list, and Meta cluster Raft state
- **Request Parameters**: 
```json
{}
```
(empty object)

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
        "roles": ["mqtt-broker"],
        "extend": [],
        "node_id": 1,
        "node_ip": "192.168.100.100",
        "grpc_addr": "192.168.100.100:1228",
        "engine_addr": "192.168.100.100:1229",
        "start_time": 1760828141,
        "register_time": 1760828142,
        "storage_fold": []
      }
    ],
    "nodes": ["192.168.100.100", "127.0.0.1"],
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
| `nodes` | `array` | List of unique node IP addresses in the cluster (deduplicated) |
| `meta` | `object` | Meta cluster Raft state information (structured object) |

**Broker Node Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `roles` | `array` | Node role list (e.g., `["mqtt-broker"]`) |
| `extend` | `array` | Extended information (byte array) |
| `node_id` | `u64` | Node ID |
| `node_ip` | `string` | Node IP address |
| `grpc_addr` | `string` | gRPC communication address |
| `engine_addr` | `string` | Storage engine address |
| `start_time` | `u64` | Node start time (Unix timestamp in seconds) |
| `register_time` | `u64` | Node registration time (Unix timestamp in seconds) |
| `storage_fold` | `array` | Storage directory list |

---

**Meta Cluster State (meta) Fields**:

The `meta` field contains the Raft consensus state information of the Meta cluster, used to monitor the distributed consistency state:

| Field | Type | Description |
|-------|------|-------------|
| `running_state` | `object` | Running state, `{"Ok": null}` indicates healthy |
| `id` | `u64` | Current node ID |
| `current_term` | `u64` | Raft current term number |
| `vote` | `object` | Vote information |
| `vote.leader_id` | `object` | Leader identifier, containing `term` and `node_id` |
| `vote.committed` | `boolean` | Whether the vote has been committed |
| `last_log_index` | `u64` | Index of the last log entry |
| `last_applied` | `object` | Last applied log information |
| `last_applied.leader_id` | `object` | Leader identifier |
| `last_applied.index` | `u64` | Index of applied log |
| `snapshot` | `object/null` | Snapshot information (if exists) |
| `purged` | `object/null` | Purged log information (if exists) |
| `state` | `string` | Current node Raft state: `Leader`, `Follower`, or `Candidate` |
| `current_leader` | `u64` | Current leader node ID |
| `millis_since_quorum_ack` | `u64` | Milliseconds since last quorum acknowledgment |
| `last_quorum_acked` | `u128` | Last quorum acknowledged timestamp (nanosecond precision) |
| `membership_config` | `object` | Cluster membership configuration |
| `membership_config.log_id` | `object` | Log ID corresponding to the configuration |
| `membership_config.membership` | `object` | Membership information |
| `membership_config.membership.configs` | `array` | Configuration array, e.g., `[[1]]` represents node 1 |
| `membership_config.membership.nodes` | `object` | Node mapping, key is node ID string, value is node info |
| `heartbeat` | `object` | Heartbeat mapping, key is node ID, value is heartbeat timestamp (nanoseconds) |
| `replication` | `object` | Replication state mapping, key is node ID, value contains `leader_id` and `index` |

**Use Cases**:
- Check if node is Leader using the `state` field
- Find the current cluster Leader through `current_leader` field
- Monitor log synchronization status via `last_log_index` and `last_applied.index`
- Monitor cluster node activity through `heartbeat`
- Understand cluster membership configuration via `membership_config`

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
