# RobustMQ Admin Server HTTP API Common Guide

## Overview

RobustMQ Admin Server is an HTTP management interface service, providing comprehensive management capabilities for RobustMQ clusters.

- **Base URL**: `http://localhost:58080`
- **API Prefix**: `/api` (all management interfaces use this prefix)
- **Request Method**: Use `GET` for list/detail queries, `POST` for create/delete operations
- **Data Format**: JSON
- **Response Format**: JSON

## API Documentation Navigation

- 🔐 **[Authentication](AUTH.md)** - Login endpoint, JWT token usage and configuration
- 📋 **[Cluster Management API](CLUSTER.md)** - Cluster configuration and status management
- 🔧 **[MQTT Broker API](MQTT.md)** - All MQTT broker related management interfaces
- 🔌 **[Connector API](Connector.md)** - Connector management interfaces

---

## Full API URI List

### /cluster — Cluster Resource APIs

| Category | Method | URI | Description |
|----------|--------|-----|-------------|
| Config | `GET` | `/api/cluster/config/get` | Get cluster configuration |
| Config | `POST` | `/api/cluster/config/set` | Update cluster configuration |
| Health | `GET` | `/api/cluster/healthy` | Cluster health check |
| Tenant | `GET` | `/api/cluster/tenant/list` | List tenants |
| Tenant | `POST` | `/api/cluster/tenant/create` | Create tenant |
| Tenant | `POST` | `/api/cluster/tenant/update` | Update tenant |
| Tenant | `POST` | `/api/cluster/tenant/delete` | Delete tenant |
| Topic | `GET` | `/api/cluster/topic/list` | List topics |
| Topic | `GET` | `/api/cluster/topic/detail` | Get topic detail |
| Topic | `POST` | `/api/cluster/topic/create` | Create topic |
| Topic | `POST` | `/api/cluster/topic/delete` | Delete topic |
| Topic | `GET` | `/api/cluster/topic-rewrite/list` | List topic rewrite rules |
| Topic | `POST` | `/api/cluster/topic-rewrite/create` | Create topic rewrite rule |
| Topic | `POST` | `/api/cluster/topic-rewrite/delete` | Delete topic rewrite rule |
| User | `GET` | `/api/cluster/user/list` | List users |
| User | `POST` | `/api/cluster/user/create` | Create user |
| User | `POST` | `/api/cluster/user/delete` | Delete user |
| ACL | `GET` | `/api/cluster/acl/list` | List ACL rules |
| ACL | `POST` | `/api/cluster/acl/create` | Create ACL rule |
| ACL | `POST` | `/api/cluster/acl/delete` | Delete ACL rule |
| Blacklist | `GET` | `/api/cluster/blacklist/list` | List blacklist entries |
| Blacklist | `POST` | `/api/cluster/blacklist/create` | Create blacklist entry |
| Blacklist | `POST` | `/api/cluster/blacklist/delete` | Delete blacklist entry |
| Connector | `GET` | `/api/cluster/connector/list` | List connectors |
| Connector | `GET` | `/api/cluster/connector/detail` | Get connector detail |
| Connector | `POST` | `/api/cluster/connector/create` | Create connector |
| Connector | `POST` | `/api/cluster/connector/delete` | Delete connector |
| Schema | `GET` | `/api/cluster/schema/list` | List schemas |
| Schema | `POST` | `/api/cluster/schema/create` | Create schema |
| Schema | `POST` | `/api/cluster/schema/delete` | Delete schema |
| Schema | `GET` | `/api/cluster/schema-bind/list` | List schema bindings |
| Schema | `POST` | `/api/cluster/schema-bind/create` | Create schema binding |
| Schema | `POST` | `/api/cluster/schema-bind/delete` | Delete schema binding |
| Offset | `POST` | `/api/cluster/offset/timestamp` | Query offset by timestamp |
| Offset | `POST` | `/api/cluster/offset/group` | Query offset by consumer group |
| Offset | `POST` | `/api/cluster/offset/commit` | Commit offset |

### /mqtt — MQTT Broker APIs

| Category | Method | URI | Description |
|----------|--------|-----|-------------|
| Overview | `GET` | `/api/mqtt/overview` | Cluster overview information |
| Monitor | `GET` | `/api/mqtt/monitor/data` | Monitor data query |
| Client | `GET` | `/api/mqtt/client/list` | List clients |
| Session | `GET` | `/api/mqtt/session/list` | List sessions |
| Subscribe | `GET` | `/api/mqtt/subscribe/list` | List subscriptions |
| Subscribe | `GET` | `/api/mqtt/subscribe/detail` | Get subscription detail |
| Subscribe | `GET` | `/api/mqtt/auto-subscribe/list` | List auto-subscribe rules |
| Subscribe | `POST` | `/api/mqtt/auto-subscribe/create` | Create auto-subscribe rule |
| Subscribe | `POST` | `/api/mqtt/auto-subscribe/delete` | Delete auto-subscribe rule |
| Subscribe | `GET` | `/api/mqtt/slow-subscribe/list` | List slow subscriptions |
| Flapping | `GET` | `/api/mqtt/flapping_detect/list` | List flapping detection records |
| Alarm | `GET` | `/api/mqtt/system-alarm/list` | List system alarms |
| Alarm | `GET` | `/api/mqtt/ban-log/list` | List ban log entries |
| Message | `POST` | `/api/mqtt/message/send` | Send message |
| Message | `POST` | `/api/mqtt/message/read` | Read message |

### /storage-engine — Storage Engine APIs

| Category | Method | URI | Description |
|----------|--------|-----|-------------|
| Shard | `POST` | `/api/storage-engine/shard/list` | List shards |
| Shard | `POST` | `/api/storage-engine/shard/create` | Create shard |
| Shard | `POST` | `/api/storage-engine/shard/delete` | Delete shard |
| Segment | `POST` | `/api/storage-engine/segment/list` | List segments |
| Segment | `POST` | `/api/storage-engine/segment/detail` | Get segment detail (with per-replica state) |
| Segment | `POST` | `/api/storage-engine/segment/replica-state` | Get local node replica state (internal) |

### Common APIs

| Category | Method | URI | Description |
|----------|--------|-----|-------------|
| Info | `GET` | `/` | Get service version information |
| Status | `GET` | `/api/status` | Cluster status query |
| Health | `GET` | `/health/ready` | Readiness health check |
| Health | `GET` | `/health/node` | Node health check |
| Health | `GET` | `/health/cluster` | Cluster health check |

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
- **Endpoint**: `GET /api/status`
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

## Storage Engine APIs

### Get Segment Detail
- **Endpoint**: `POST /api/storage-engine/segment/detail`
- **Description**: Returns detailed information for a specific segment, including basic metadata and real-time state from all replica nodes (HW, LEO, role, etc.). The local node's data is read directly from cache; remote replicas are fetched via internal HTTP forwarding.
- **Request Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `shard_name` | `string` | Yes | Shard name |
| `segment_seq` | `u32` | Yes | Segment sequence number |

- **Request Example**:
```json
{
  "shard_name": "my-shard",
  "segment_seq": 0
}
```

- **Response Example**:
```json
{
  "code": 0,
  "data": {
    "segment": {
      "shard_name": "my-shard",
      "segment_seq": 0,
      "leader": 1,
      "replicas": [{"node_id": 1}, {"node_id": 2}],
      "isr": [1, 2],
      "config": {}
    },
    "segment_meta": null,
    "replicas": [
      {
        "node_id": 1,
        "is_leader": true,
        "in_isr": true,
        "role": "LeaderActive",
        "leader_epoch": 1,
        "segment_epoch": 0,
        "leo": 100,
        "high_watermark": 98,
        "log_start_offset": 0,
        "follower_progress": [
          {
            "node_id": 2,
            "leo": 98,
            "last_known_leader_epoch": 1,
            "last_fetch_ts": 1749100000,
            "last_caught_up_ts": 1749099990
          }
        ],
        "available": true,
        "error": null
      },
      {
        "node_id": 2,
        "is_leader": false,
        "in_isr": true,
        "role": "Follower",
        "leader_epoch": 1,
        "segment_epoch": 0,
        "leo": 98,
        "high_watermark": 98,
        "log_start_offset": 0,
        "follower_progress": [],
        "available": true,
        "error": null
      }
    ]
  }
}
```

**Response Fields**:

`segment` contains the segment's basic information; `segment_meta` is optional metadata; `replicas` is the list of per-node replica states.

| Field | Type | Description |
|-------|------|-------------|
| `node_id` | `u64` | Node ID hosting this replica |
| `is_leader` | `bool` | Whether this replica is the current Leader |
| `in_isr` | `bool` | Whether this replica is in the ISR list |
| `role` | `string` | Replica role: `LeaderActive`, `Follower`, `Unknown`, etc. |
| `leader_epoch` | `u32` | Leader epoch, incremented on each leader change |
| `segment_epoch` | `u32` | Segment epoch |
| `leo` | `u64` | Log End Offset — the latest offset written on this replica |
| `high_watermark` | `u64` | High watermark — the highest offset replicated to all ISR members |
| `log_start_offset` | `u64` | Earliest available offset in the log |
| `follower_progress` | `array` | Follower sync progress list (only populated on Leader nodes) |
| `available` | `bool` | Whether the node was reachable |
| `error` | `string/null` | Error message if the node was unreachable |

`follower_progress` fields:

| Field | Type | Description |
|-------|------|-------------|
| `node_id` | `u64` | Follower node ID |
| `leo` | `u64` | LEO the follower has synced up to |
| `last_known_leader_epoch` | `u32` | Last leader epoch known to this follower |
| `last_fetch_ts` | `u64` | Timestamp of the last fetch request (milliseconds) |
| `last_caught_up_ts` | `u64` | Timestamp when the follower last caught up to the leader (milliseconds) |

---

### Get Local Node Replica State (Internal)
- **Endpoint**: `POST /api/storage-engine/segment/replica-state`
- **Description**: Internal endpoint called by `segment/detail` via HTTP forwarding. Returns the local node's replica state for a given segment. Not intended for direct external use.
- **Request Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `shard_name` | `string` | Yes | Shard name |
| `segment_seq` | `u32` | Yes | Segment sequence number |

- **Response**: Single `SegmentReplicaStateResp` object — same structure as one element in the `replicas` array of `segment/detail`.

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
curl -X GET http://localhost:58080/

# Get cluster status
curl -X GET http://localhost:58080/api/status

# List query with pagination
curl "http://localhost:58080/api/cluster/user/list?limit=10&page=1&sort_field=username&sort_by=asc"
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
   - List/detail queries (`/list`, `/detail`, `/api/status`, etc.) use `GET`, with parameters passed via query string
   - Create/delete operations (`/create`, `/delete`) use `POST`, with parameters passed as JSON body
2. **Request Body**: POST interfaces require `Content-Type: application/json` and a JSON body; GET interfaces pass parameters via URL query string
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
curl -X GET http://localhost:58080/
```

### Log Viewing
The service outputs detailed log information during runtime, including:
- Request paths and parameters
- Response status and data
- Error messages and stack traces

---

Last Updated: 2026-03-21
