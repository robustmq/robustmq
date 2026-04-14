# MQTT Broker Management HTTP API

> This document describes all HTTP API interfaces related to MQTT Broker. For general information, please refer to [COMMON.md](COMMON.md).

## API Interface List

### 1. Cluster Overview

#### 1.1 Cluster Overview Information
- **Endpoint**: `GET /api/mqtt/overview`
- **Description**: Get MQTT cluster overview information
- **Request Parameters**: Empty JSON object
```json
{}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "node_list": [
      {
        "roles": ["mqtt-broker"],
        "extend": [],
        "node_id": 1,
        "node_ip": "192.168.1.100",
        "grpc_addr": "192.168.1.100:9981",
        "engine_addr": "192.168.1.100:9982",
        "start_time": 1640995200,
        "register_time": 1640995200,
        "storage_fold": []
      }
    ],
    "cluster_name": "robustmq-cluster",
    "message_in_rate": 100,
    "message_out_rate": 85,
    "connection_num": 1500,
    "session_num": 1200,
    "topic_num": 50,
    "placement_status": "Leader",
    "tcp_connection_num": 800,
    "tls_connection_num": 400,
    "websocket_connection_num": 200,
    "quic_connection_num": 100,
    "subscribe_num": 2000,
    "exclusive_subscribe_num": 1500,
    "exclusive_subscribe_thread_num": 8,
    "share_subscribe_group_num": 50,
    "share_subscribe_num": 500,
    "share_subscribe_thread_num": 50,
    "connector_num": 5,
    "connector_thread_num": 3
  }
}
```

**Field Descriptions**:
- `node_list`: Cluster node list
  - `roles`: Node role list
  - `extend`: Extended information (byte array)
  - `node_id`: Node ID
  - `node_ip`: Node IP address
  - `grpc_addr`: gRPC communication address
  - `engine_addr`: Storage engine address
  - `start_time`: Node start timestamp
  - `register_time`: Node registration timestamp
  - `storage_fold`: Storage directory list
- `cluster_name`: Cluster name
- `message_in_rate`: Message receive rate (messages/second)
- `message_out_rate`: Message send rate (messages/second)
- `connection_num`: Total number of connections (sum of all tenants, all connection types)
- `session_num`: Total number of sessions (sum across all tenants)
- `topic_num`: Total number of topics (sum across all tenants)
- `placement_status`: Placement Center status (Leader/Follower)
- `tcp_connection_num`: Number of TCP connections
- `tls_connection_num`: Number of TLS connections
- `websocket_connection_num`: Number of WebSocket connections
- `quic_connection_num`: Number of QUIC connections
- `subscribe_num`: Total number of subscriptions (sum of all tenants, all subscriptions)
- `exclusive_subscribe_num`: Total number of exclusive subscriptions
- `exclusive_subscribe_thread_num`: Number of exclusive subscription push threads
- `share_subscribe_group_num`: Number of shared subscription groups
- `share_subscribe_num`: Total number of shared subscriptions
- `share_subscribe_thread_num`: Number of shared subscription push threads
- `connector_num`: Total number of connectors (sum across all tenants)
- `connector_thread_num`: Number of active connector threads

#### 1.2 Monitor Data Query
- **Endpoint**: `GET /api/mqtt/monitor/data`
- **Description**: Get time series monitoring data for a specified metric type
- **Request Parameters**:
```json
{
  "data_type": "connection_num",      // Required, monitoring data type
  "topic_name": "sensor/temperature", // Optional, required for certain types
  "client_id": "client001",           // Optional, required for certain types
  "path": "sensor/+",                 // Optional, required for certain types
  "connector_name": "kafka_conn_01"   // Optional, required for connector monitoring types
}
```

**Supported Monitoring Data Types (data_type)**:

**Basic Monitoring Types** (no additional parameters required):
- `connection_num` - Number of connections
- `topic_num` - Number of topics
- `subscribe_num` - Number of subscriptions
- `message_in_num` - Number of messages received
- `message_out_num` - Number of messages sent
- `message_drop_num` - Number of messages dropped

**Topic-Level Monitoring Types** (requires `topic_name`):
- `topic_in_num` - Number of messages received for a specific topic
- `topic_out_num` - Number of messages sent for a specific topic

**Subscription-Level Monitoring Types** (requires `client_id` and `path`):
- `subscribe_send_success_num` - Number of successfully sent messages to subscription
- `subscribe_send_failure_num` - Number of failed sent messages to subscription

**Subscription-Topic-Level Monitoring Types** (requires `client_id`, `path` and `topic_name`):
- `subscribe_topic_send_success_num` - Number of successfully sent messages to subscription for specific topic
- `subscribe_topic_send_failure_num` - Number of failed sent messages to subscription for specific topic

**Session-Level Monitoring Types** (requires `client_id`):
- `session_in_num` - Number of messages received by session
- `session_out_num` - Number of messages sent by session

**Connector Monitoring Types**:
- `connector_send_success_total` - Total number of successfully sent messages across all connectors (no additional parameters required)
- `connector_send_failure_total` - Total number of failed sent messages across all connectors (no additional parameters required)
- `connector_send_success` - Number of successfully sent messages for a specific connector (requires `connector_name`)
- `connector_send_failure` - Number of failed sent messages for a specific connector (requires `connector_name`)

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": [
    {
      "date": 1640995200,
      "value": 1500
    },
    {
      "date": 1640995260,
      "value": 1520
    }
  ]
}
```

**Field Descriptions**:
- `date`: Unix timestamp (seconds)
- `value`: Metric value at that time point

**Notes**:
- Data retention period: By default, data from the last 1 hour is retained
- Data sampling interval: According to system configuration, typically 60 seconds
- If required parameters are missing, an empty array will be returned
- Returned data is naturally sorted by timestamp

---

### 2. Client Management

#### 2.1 Client List Query
- **Endpoint**: `GET /api/mqtt/client/list`
- **Description**: Query the list of clients connected to the cluster, supports filtering by tenant and fuzzy search by client_id
- **Request Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tenant` | string | No | Filter exactly by tenant; when specified, queries that tenant's cache directly (better performance) |
| `client_id` | string | No | Fuzzy search by client ID (contains match) |
| `limit` | u32 | No | Page size, maximum 100 records sampled |
| `page` | u32 | No | Page number, starting from 1 |
| `sort_field` | string | No | Sort field, supports `client_id`, `connection_id` |
| `sort_by` | string | No | Sort direction: `asc` / `desc` |

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "client_id": "client001",
        "connection_id": 12345,
        "mqtt_connection": {
          "connect_id": 12345,
          "client_id": "client001",
          "tenant": "default",
          "is_login": true,
          "source_ip_addr": "192.168.1.100",
          "login_user": "user001",
          "keep_alive": 60,
          "topic_alias": {},
          "client_max_receive_maximum": 65535,
          "max_packet_size": 268435455,
          "topic_alias_max": 65535,
          "request_problem_info": 1,
          "receive_qos_message": 0,
          "sender_qos_message": 0,
          "create_time": 1640995200
        },
        "network_connection": {
          "connection_type": "Tcp",
          "connection_id": 12345,
          "protocol": "MQTT5",
          "addr": "192.168.1.100:52341",
          "last_heartbeat_time": 1640995200,
          "create_time": 1640995200
        },
        "session": {
          "client_id": "client001",
          "session_expiry": 3600,
          "is_contain_last_will": true,
          "last_will_delay_interval": 30,
          "create_time": 1640995200,
          "connection_id": 12345,
          "broker_id": 1,
          "reconnect_time": 1640995300,
          "distinct_time": 1640995400
        },
        "heartbeat": {
          "protocol": "Mqtt5",
          "keep_live": 60,
          "heartbeat": 1640995500
        }
      }
    ],
    "total_count": 100
  }
}
```

**Field Descriptions**:

- **mqtt_connection**: MQTT protocol layer connection information
  - `connect_id`: Connection ID
  - `client_id`: MQTT client ID
  - `tenant`: Tenant name
  - `is_login`: Whether the client is logged in
  - `source_ip_addr`: Source IP address of the client
  - `login_user`: Authenticated username
  - `keep_alive`: Keep-alive interval in seconds
  - `topic_alias`: Topic alias mappings for this connection
  - `client_max_receive_maximum`: Maximum number of QoS 1 and QoS 2 messages that can be received simultaneously
  - `max_packet_size`: Maximum packet size in bytes
  - `topic_alias_max`: Maximum number of topic aliases
  - `request_problem_info`: Whether to return detailed error information (0 or 1)
  - `receive_qos_message`: Number of QoS 1/2 messages pending receive
  - `sender_qos_message`: Number of QoS 1/2 messages pending send
  - `create_time`: Connection creation timestamp

- **network_connection**: Network layer connection information (null if disconnected)
  - `connection_type`: Connection type (Tcp, Tls, Websocket, Websockets, Quic)
  - `connection_id`: Network connection ID
  - `protocol`: Protocol version (MQTT3, MQTT4, MQTT5)
  - `addr`: Client socket address
  - `last_heartbeat_time`: Last heartbeat timestamp
  - `create_time`: Network connection creation timestamp

- **session**: MQTT session information (null if no session exists)
  - Same fields as the session management interface response

- **heartbeat**: Connection heartbeat information (null if not available)
  - `protocol`: MQTT protocol version (Mqtt3, Mqtt4, Mqtt5)
  - `keep_live`: Keep-alive interval in seconds
  - `heartbeat`: Last heartbeat timestamp

- **total_count**: Actual total number of connections for that tenant (or the entire cluster)

---

### 3. Session Management

#### 3.1 Session List Query
- **Endpoint**: `GET /api/mqtt/session/list`
- **Description**: Query MQTT session list, supports filtering by tenant and fuzzy search by client_id
- **Request Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tenant` | string | No | Filter exactly by tenant; when specified, queries that tenant's cache directly (better performance) |
| `client_id` | string | No | Fuzzy search by client ID (contains match) |
| `limit` | u32 | No | Page size, maximum 100 records sampled |
| `page` | u32 | No | Page number, starting from 1 |
| `sort_field` | string | No | Sort field, supports `client_id`, `create_time` |
| `sort_by` | string | No | Sort direction: `asc` / `desc` |

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant": "default",
        "client_id": "client001",
        "session_expiry": 3600,
        "is_contain_last_will": true,
        "last_will_delay_interval": 30,
        "create_time": 1640995200,
        "connection_id": 12345,
        "broker_id": 1,
        "reconnect_time": 1640995300,
        "distinct_time": 1640995400,
        "last_will": {
          "client_id": "client001",
          "last_will": {
            "topic": "device/client001/status",
            "message": "offline",
            "qos": "AtLeastOnce",
            "retain": true
          },
          "last_will_properties": {
            "delay_interval": 30,
            "payload_format_indicator": 0,
            "message_expiry_interval": 3600,
            "content_type": "text/plain",
            "response_topic": null,
            "correlation_data": null,
            "user_properties": []
          }
        }
      }
    ],
    "total_count": 50
  }
}
```

**Field Descriptions**:

- `client_id`: MQTT client ID
- `session_expiry`: Session expiry interval in seconds
- `is_contain_last_will`: Whether the session contains a last will message
- `last_will_delay_interval`: Delay interval for last will message in seconds (optional)
- `create_time`: Session creation timestamp
- `connection_id`: Associated connection ID (optional)
- `broker_id`: Broker node ID hosting the session (optional)
- `reconnect_time`: Last reconnection timestamp (optional)
- `distinct_time`: Last disconnection timestamp (optional)
- `last_will`: Last will message information (null if no last will configured)
  - `client_id`: Client ID
  - `last_will`: Last will message content
    - `topic`: Last will message topic
    - `message`: Last will message payload
    - `qos`: QoS level (`AtMostOnce`/`AtLeastOnce`/`ExactlyOnce`)
    - `retain`: Whether it's a retained message
  - `last_will_properties`: Last will properties (MQTT 5.0, can be null)
- **total_count**: Actual total number of sessions for that tenant (or the entire cluster)

---

### 4. Topic Management

#### 4.1 Topic List Query
- **Endpoint**: `GET /api/cluster/topic/list`
- **Description**: Query MQTT topic list, supports filtering by tenant, fuzzy search by topic name, and topic type filtering
- **Request Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tenant` | string | No | Filter exactly by tenant; when specified, queries that tenant's cache directly (better performance) |
| `topic_name` | string | No | Fuzzy search by topic name (contains match) |
| `topic_type` | string | No | Topic type: `all` (default), `normal` (normal topics), `system` (system topics containing `$`) |
| `limit` | u32 | No | Page size |
| `page` | u32 | No | Page number, starting from 1 |
| `sort_field` | string | No | Sort field, supports `topic_name`, `tenant` |
| `sort_by` | string | No | Sort direction: `asc` / `desc` |

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "topic_id": "01J9K5FHQP8NWXYZ1234567890",
        "topic_name": "sensor/temperature",
        "tenant": "default",
        "storage_type": "Memory",
        "partition": 1,
        "replication": 1,
        "storage_name_list": [],
        "create_time": 1640995200
      }
    ],
    "total_count": 25
  }
}
```

#### 4.2 Topic Detail Query
- **Endpoint**: `GET /api/cluster/topic/detail`
- **Description**: Query detailed information for a specific topic, including basic topic information, retained message, and subscriber list
- **Request Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tenant` | string | Yes | Tenant name |
| `topic_name` | string | Yes | Topic name (exact match) |


- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "topic_info": {
      "topic_id": "01J9K5FHQP8NWXYZ1234567890",
      "topic_name": "sensor/temperature",
      "tenant": "default",
      "storage_type": "Memory",
      "partition": 1,
      "replication": 1,
      "storage_name_list": [],
      "create_time": 1640995200
    },
    "retain_message": { ... },
    "retain_message_at": 1640995300,
    "sub_list": [
      { "client_id": "client001", "path": "sensor/temperature" }
    ],
    "storage_list": { ... }
  }
}
```

#### 4.3 Delete Topic
- **Endpoint**: `POST /api/cluster/topic/delete`
- **Description**: Delete a specified topic
- **Request Parameters**:

| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| `tenant` | string | Yes | - | Tenant name |
| `topic_name` | string | Yes | Length 1-256 | Name of the topic to delete |


- **Response**: Returns "success" on success

**Notes**:
- Deleting a topic will remove all data for that topic, including retained messages
- This operation is irreversible, use with caution

#### 4.4 Topic Rewrite Rules List
- **Endpoint**: `GET /api/cluster/topic-rewrite/list`
- **Description**: Query topic rewrite rules list, supports fuzzy search by tenant and rule name
- **Request Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tenant` | string | No | Fuzzy filter by tenant |
| `name` | string | No | Fuzzy filter by rule name |
| `limit` | u32 | No | Page size |
| `page` | u32 | No | Page number, starting from 1 |
| `sort_field` | string | No | Sort field, supports `name`, `source_topic`, `dest_topic`, `action`, `tenant` |
| `sort_by` | string | No | Sort direction: `asc` / `desc` |


- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "name": "my-rule",
        "desc": "rewrite sensor topics",
        "tenant": "default",
        "source_topic": "old/topic/+",
        "dest_topic": "new/topic/$1",
        "regex": "^old/topic/(.+)$",
        "action": "All"
      }
    ],
    "total_count": 10
  }
}
```

#### 4.5 Create Topic Rewrite Rule
- **Endpoint**: `POST /api/cluster/topic-rewrite/create`
- **Description**: Create a new topic rewrite rule, `name` is used as the unique identifier
- **Request Parameters**:
```json
{
  "name": "my-rule",                // Required, rule name, globally unique identifier
  "desc": "rewrite sensor topics",  // Optional, rule description
  "tenant": "default",              // Required, tenant name, length 1-128
  "action": "All",                  // Required, action type: All, Publish, Subscribe
  "source_topic": "old/topic/+",   // Required, source topic pattern, length 1-256
  "dest_topic": "new/topic/$1",    // Required, destination topic pattern, length 1-256
  "regex": "^old/topic/(.+)$"      // Required, regular expression, length 1-500
}
```

- **Response**: Returns "success" on success

#### 4.6 Delete Topic Rewrite Rule
- **Endpoint**: `POST /api/cluster/topic-rewrite/delete`
- **Description**: Delete topic rewrite rule by rule name
- **Request Parameters**:
```json
{
  "tenant": "default",    // Required, tenant name
  "name": "my-rule"       // Required, rule name
}
```

- **Response**: Returns "success" on success

---

### 5. Subscription Management

#### 5.1 Subscription List Query
- **Endpoint**: `GET /api/mqtt/subscribe/list`
- **Description**: Query subscription list, supports filtering by tenant and fuzzy search by client_id
- **Request Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tenant` | string | No | Filter exactly by tenant; when specified, queries that tenant's cache directly (better performance) |
| `client_id` | string | No | Fuzzy search by client ID (contains match) |
| `limit` | u32 | No | Page size |
| `page` | u32 | No | Page number, starting from 1 |
| `sort_field` | string | No | Sort field, supports `client_id`, `tenant` |
| `sort_by` | string | No | Sort direction: `asc` / `desc` |


- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant": "default",
        "client_id": "client001",
        "path": "sensor/+",
        "broker_id": 1,
        "protocol": "MQTTv5",
        "qos": "QoS1",
        "no_local": 0,
        "preserve_retain": 0,
        "retain_handling": "SendAtSubscribe",
        "create_time": "2024-01-01 10:00:00",
        "pk_id": 1,
        "properties": "{}",
        "is_share_sub": false
      }
    ],
    "total_count": 30
  }
}
```

**New Field**:
- `tenant`: Tenant name the subscription belongs to

#### 5.2 Subscription Detail Query
- **Endpoint**: `GET /api/mqtt/subscribe/detail`
- **Description**: Query subscription details, supports both exclusive and shared subscription details
- **Request Parameters**:
```json
{
  "tenant": "default",          // Required, tenant name
  "client_id": "client001",     // Required, client ID
  "path": "sensor/temperature"  // Required, subscription path
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "share_sub": false,
    "group_leader_info": null,
    "sub_data": {
      "client_id": "client001",
      "path": "sensor/temperature",
      "push_subscribe": {
        "sensor/temperature": {
          "client_id": "client001",
          "sub_path": "sensor/temperature",
          "rewrite_sub_path": null,
          "tenant": "default",
          "topic_name": "sensor/temperature",
          "group_name": "",
          "protocol": "MQTTv5",
          "qos": "AtLeastOnce",
          "no_local": false,
          "preserve_retain": true,
          "retain_forward_rule": "SendAtSubscribe",
          "subscription_identifier": null,
          "create_time": 1704067200
        }
      },
      "push_thread": {
        "sensor/temperature": {
          "push_success_record_num": 1520,
          "push_error_record_num": 3,
          "last_push_time": 1704067800,
          "last_run_time": 1704067810,
          "create_time": 1704067200,
          "bucket_id": "bucket_0"
        }
      },
      "leader_id": null
    }
  }
}
```

#### 5.3 Auto Subscribe Rule Management

##### 5.3.1 Auto Subscribe List
- **Endpoint**: `GET /api/mqtt/auto-subscribe/list`
- **Description**: Query auto subscribe rules list, supports fuzzy search by tenant and name
- **Request Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| tenant | string | No | Fuzzy search by tenant |
| name | string | No | Fuzzy search by rule name |
| limit | number | No | Number of records per page |
| page | number | No | Page number |
| sort_field | string | No | Sort field |
| sort_by | string | No | Sort direction asc/desc |

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "name": "rule-1",
        "desc": "auto subscribe for system topics",
        "tenant": "default",
        "topic": "system/+",
        "qos": "QoS1",
        "no_local": false,
        "retain_as_published": false,
        "retained_handling": "SendAtSubscribe"
      }
    ],
    "total_count": 5
  }
}
```

##### 5.3.2 Create Auto Subscribe Rule
- **Endpoint**: `POST /api/mqtt/auto-subscribe/create`
- **Description**: Create a new auto subscribe rule, name is the unique identifier
- **Request Parameters**:
```json
{
  "name": "rule-1",                 // Required, unique rule name, length 1-256
  "desc": "optional description",  // Optional, rule description
  "tenant": "default",              // Required, tenant name, length 1-256
  "topic": "system/+",              // Required, topic pattern, length 1-256
  "qos": 1,                         // Required, QoS level: 0, 1, 2
  "no_local": false,                // Required, no local
  "retain_as_published": false,     // Required, retain as published
  "retained_handling": 0            // Required, retained message handling: 0, 1, 2
}
```

- **Response**: Returns "success" on success

##### 5.3.3 Delete Auto Subscribe Rule
- **Endpoint**: `POST /api/mqtt/auto-subscribe/delete`
- **Description**: Delete auto subscribe rule by name
- **Request Parameters**:
```json
{
  "tenant": "default",              // Required, tenant name
  "name": "rule-1"                  // Required, rule name
}
```

- **Response**: Returns "success" on success

#### 5.4 Slow Subscribe Monitoring

##### 5.4.1 Slow Subscribe List
- **Endpoint**: `GET /api/mqtt/slow-subscribe/list`
- **Description**: Query slow subscribe list, supports filtering by tenant and fuzzy search by client_id
- **Request Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tenant` | string | No | Filter by tenant (when specified, only that tenant's records are scanned) |
| `client_id` | string | No | Fuzzy search by client ID (contains match) |
| `limit` | u32 | No | Page size |
| `page` | u32 | No | Page number, starting from 1 |
| `sort_field` | string | No | Sort field, supports `tenant`, `client_id`, `topic_name` |
| `sort_by` | string | No | Sort direction: `asc` / `desc` |

- **Response Data Structure**:

```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant": "default",
        "client_id": "slow_client",
        "topic_name": "heavy/topic",
        "time_span": 5000,
        "node_info": "node1",
        "create_time": "2024-01-01 10:00:00",
        "subscribe_name": "sub001"
      }
    ],
    "total_count": 3
  }
}
```

**Field Descriptions**:

- `tenant`: Tenant name
- `client_id`: Client ID
- `topic_name`: Subscription topic
- `time_span`: Slow subscription latency (milliseconds)
- `node_info`: Node information
- `create_time`: Record creation time (local time format)
- `subscribe_name`: Subscription name

---

### 6. User Management

#### 6.1 User List Query
- **Endpoint**: `GET /api/cluster/user/list`
- **Description**: Query MQTT user list, supports filtering by tenant and fuzzy search by username
- **Request Parameters**:
```json
{
  "tenant": "default",              // Optional, filter exactly by tenant
  "user_name": "admin",             // Optional, fuzzy search by username (contains match)
  "limit": 20,
  "page": 1,
  "sort_field": "username",
  "sort_by": "asc",
  "filter_field": "username",
  "filter_values": ["admin"],
  "exact_match": "false"
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant": "default",
        "username": "admin",
        "is_superuser": true,
        "create_time": 1640995200
      }
    ],
    "total_count": 10
  }
}
```

**Field Descriptions**:
- `tenant`: Tenant the user belongs to
- `username`: Username
- `is_superuser`: Whether it's a superuser
- `create_time`: User creation timestamp (seconds)

#### 6.2 Create User
- **Endpoint**: `POST /api/cluster/user/create`
- **Description**: Create new MQTT user
- **Request Parameters**:
```json
{
  "tenant": "default",              // Required, tenant name, length 1-64
  "username": "newuser",            // Required, username, length 1-64
  "password": "password123",        // Required, password, length 1-128
  "is_superuser": false             // Required, whether it's a superuser
}
```

- **Response**: Returns "success" on success

#### 6.3 Delete User
- **Endpoint**: `POST /api/cluster/user/delete`
- **Description**: Delete MQTT user
- **Request Parameters**:
```json
{
  "tenant": "default",              // Required, tenant name, length 1-64
  "username": "olduser"             // Required, username, length 1-64
}
```

- **Response**: Returns "success" on success

---

### 7. ACL Management

#### 7.1 ACL List Query
- **Endpoint**: `GET /api/cluster/acl/list`
- **Description**: Query access control list, supports fuzzy search by tenant, name, and resource name (contains match)
- **Request Parameters**:
```json
{
  "tenant": "default",              // Optional, fuzzy search by tenant (contains match)
  "name": "sensor",                 // Optional, fuzzy search by ACL name (contains match)
  "resource_name": "client001",     // Optional, fuzzy search by resource name (contains match, applies to client_id or user)
  "limit": 20,
  "page": 1,
  "sort_field": "name",
  "sort_by": "asc"
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant": "default",
        "name": "sensor-publish-rule",
        "desc": "Allow sensor devices to publish data",
        "resource_type": "ClientId",
        "resource_name": "client001",
        "topic": "sensor/+",
        "ip": "192.168.1.0/24",
        "action": "Publish",
        "permission": "Allow"
      }
    ],
    "total_count": 15
  }
}
```

#### 7.2 Create ACL Rule
- **Endpoint**: `POST /api/cluster/acl/create`
- **Description**: Create new ACL rule, `name` is the unique identifier (unique within a tenant)
- **Request Parameters**:
```json
{
  "tenant": "default",               // Required, tenant name, length 1-64
  "name": "sensor-publish-rule",     // Required, ACL rule name, unique within tenant, length 1-128
  "desc": "Allow sensor devices to publish data", // Optional, description, length no more than 256
  "resource_type": "ClientId",       // Required, resource type: ClientId, User, Ip
  "resource_name": "client001",      // Required, resource name, length 1-256
  "topic": "sensor/+",               // Optional, topic pattern, length no more than 256
  "ip": "192.168.1.100",             // Optional, IP address, length no more than 128
  "action": "Publish",               // Required, action: Publish, Subscribe, All
  "permission": "Allow"              // Required, permission: Allow, Deny
}
```

- **Parameter Validation Rules**:
  - `resource_type`: Must be `ClientId`, `User`, or `Ip`
  - `action`: Must be `Publish`, `Subscribe`, or `All`
  - `permission`: Must be `Allow` or `Deny`

- **Response**: Returns "success" on success

#### 7.3 Delete ACL Rule
- **Endpoint**: `POST /api/cluster/acl/delete`
- **Description**: Delete ACL rule by `name`
- **Request Parameters**:
```json
{
  "tenant": "default",
  "name": "sensor-publish-rule"
}
```

- **Response**: Returns "success" on success

---

### 8. Blacklist Management

#### 8.1 Blacklist List Query
- **Endpoint**: `GET /api/cluster/blacklist/list`
- **Description**: Query blacklist, supports fuzzy search by tenant, name, and resource name
- **Request Parameters**:
```json
{
  "tenant": "default",              // Optional, fuzzy search by tenant
  "name": "bl-bad",                 // Optional, fuzzy search by name
  "resource_name": "bad_client",    // Optional, fuzzy search by resource name
  "limit": 20,
  "page": 1,
  "sort_field": "resource_name",
  "sort_by": "asc"
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "name": "bl-malicious-client",
        "tenant": "default",
        "blacklist_type": "ClientId",
        "resource_name": "malicious_client",
        "end_time": "2024-12-31 23:59:59",
        "desc": "Blocked due to suspicious activity"
      }
    ],
    "total_count": 5
  }
}
```

#### 8.2 Create Blacklist Entry
- **Endpoint**: `POST /api/cluster/blacklist/create`
- **Description**: Add new blacklist entry, `name` is the unique identifier
- **Request Parameters**:
```json
{
  "name": "bl-bad-client",            // Required, unique name, length 1-128
  "tenant": "default",                 // Required, tenant name, length 1-64
  "blacklist_type": "ClientId",        // Required, blacklist type (see enum description)
  "resource_name": "bad_client",       // Required, resource name, length 1-256
  "end_time": 1735689599,              // Required, end time (Unix timestamp), must be greater than 0
  "desc": "Blocked for security"       // Optional, description, length no more than 500
}
```

- **Parameter Validation Rules**:
  - `blacklist_type`: Must be `ClientId`, `User`, `Ip`, `ClientIdMatch`, `UserMatch`, or `IPCIDR`
  - `end_time`: Must be greater than 0

- **Response**: Returns "success" on success

#### 8.3 Delete Blacklist Entry
- **Endpoint**: `POST /api/cluster/blacklist/delete`
- **Description**: Delete blacklist entry by `name`
- **Request Parameters**:
```json
{
  "tenant": "default",
  "name": "bl-bad-client"
}
```

- **Response**: Returns "success" on success

---

### 9. Connector Management

> The Connector API has extensive content and has been extracted into a separate document. Please refer to [Connector Management HTTP API](Connector.md).

**Key Changes**: The connector `list`, `detail`, and `delete` interfaces all have a new `tenant` parameter added. See Connector.md for details.

---

### 10. Schema Management

#### 10.1 Schema List Query
- **Endpoint**: `GET /api/cluster/schema/list`
- **Description**: Query Schema list, supports fuzzy search by tenant and name
- **Request Parameters**:
```json
{
  "tenant": "default",              // Optional, fuzzy search by tenant
  "name": "sensor",                 // Optional, fuzzy search by Schema name
  "limit": 20,
  "page": 1,
  "sort_field": "name",
  "sort_by": "asc"
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant": "default",
        "name": "temperature_schema",
        "schema_type": "json",
        "desc": "Temperature sensor data schema",
        "schema": "{\"type\":\"object\",\"properties\":{\"temp\":{\"type\":\"number\"}}}"
      }
    ],
    "total_count": 12
  }
}
```

#### 10.2 Create Schema
- **Endpoint**: `POST /api/cluster/schema/create`
- **Description**: Create new Schema
- **Request Parameters**:
```json
{
  "tenant": "default",                   // Required, tenant name, length 1-128
  "schema_name": "sensor_data_schema",   // Required, Schema name, length 1-128
  "schema_type": "json",                 // Required, Schema type: json, avro, protobuf
  "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\"}}}",
  "desc": "Sensor data validation schema"  // Optional, description, length no more than 500
}
```

- **Response**: Returns "success" on success

#### 10.3 Delete Schema
- **Endpoint**: `POST /api/cluster/schema/delete`
- **Description**: Delete Schema
- **Request Parameters**:
```json
{
  "tenant": "default",              // Required, tenant name
  "schema_name": "old_schema"       // Required, Schema name
}
```

- **Response**: Returns "success" on success

#### 10.4 Schema Binding Management

##### 10.4.1 Schema Binding List Query
- **Endpoint**: `GET /api/cluster/schema-bind/list`
- **Description**: Query Schema binding relationship list
- **Request Parameters**:
```json
{
  "tenant": "default",                    // Optional, tenant name
  "resource_name": "sensor/temperature",  // Optional, resource name filter
  "schema_name": "temp_schema",           // Optional, Schema name filter
  "limit": 20,
  "page": 1,
  "sort_field": "data_type",
  "sort_by": "asc"
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "data_type": "resource",
        "data": ["sensor_data_schema", "device_status_schema"]
      }
    ],
    "total_count": 2
  }
}
```

##### 10.4.2 Create Schema Binding
- **Endpoint**: `POST /api/cluster/schema-bind/create`
- **Description**: Create Schema binding relationship with resource
- **Request Parameters**:
```json
{
  "tenant": "default",                   // Required, tenant name, length 1-128
  "schema_name": "sensor_data_schema",   // Required, Schema name, length 1-128
  "resource_name": "sensor/temperature"  // Required, resource name (usually topic name), length 1-256
}
```

- **Response**: Returns "success" on success

##### 10.4.3 Delete Schema Binding
- **Endpoint**: `POST /api/cluster/schema-bind/delete`
- **Description**: Delete Schema binding relationship
- **Request Parameters**:
```json
{
  "tenant": "default",
  "schema_name": "sensor_data_schema",
  "resource_name": "sensor/temperature"
}
```

- **Response**: Returns "success" on success

---

### 11. Message Management

#### 11.1 Send Message
- **Endpoint**: `POST /api/mqtt/message/send`
- **Description**: Send MQTT message to specified topic via HTTP API
- **Request Parameters**:
```json
{
  "tenant": "default",            // Required, tenant name, length 1-256
  "topic": "sensor/temperature",  // Required, topic name, length 1-256
  "payload": "25.5",              // Required, message content, no more than 1MB
  "retain": false                 // Optional, whether to retain message, default false
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "offsets": [12345]
  }
}
```

**Notes**:
- Messages are sent with QoS 1 (at least once) level
- Topic will be automatically created if it doesn't exist
- Default message expiry time is 3600 seconds (1 hour)

#### 11.2 Read Messages
- **Endpoint**: `POST /api/mqtt/message/read`
- **Description**: Read messages from specified topic
- **Request Parameters**:
```json
{
  "tenant": "default",            // Required, tenant name
  "topic": "sensor/temperature",  // Required, topic name
  "offset": 0                     // Required, starting offset, reads from this position
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "messages": [
      {
        "offset": 12345,
        "content": "25.5",
        "timestamp": 1640995200000
      }
    ]
  }
}
```

**Notes**:
- Maximum of 100 messages returned per request
- Timestamp is in millisecond Unix timestamp format

---

### 12. System Monitoring

#### 12.1 System Alarm List
- **Endpoint**: `GET /api/mqtt/system-alarm/list`
- **Description**: Query system alarm list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "name": "High Memory Usage",
        "message": "Memory usage exceeded 80% threshold",
        "create_time": 1640995200
      }
    ],
    "total_count": 3
  }
}
```

#### 12.2 Flapping Detection List

- **Endpoint**: `GET /api/mqtt/flapping_detect/list`
- **Description**: Query flapping detection list, supports filtering by tenant and fuzzy search by client_id
- **Request Parameters**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tenant` | string | No | Filter exactly by tenant (when specified, returns only that tenant's data, better performance) |
| `client_id` | string | No | Fuzzy search by client ID (contains match) |
| `limit` | u32 | No | Page size |
| `page` | u32 | No | Page number, starting from 1 |
| `sort_field` | string | No | Sort field, supports `tenant`, `client_id` |
| `sort_by` | string | No | Sort direction: `asc` / `desc` |

- **Response Data Structure**:

```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant": "default",
        "client_id": "flapping_client",
        "before_last_window_connections": 15,
        "first_request_time": 1640995200
      }
    ],
    "total_count": 2
  }
}
```

**Field Descriptions**:

- `tenant`: Tenant name
- `client_id`: Client ID
- `before_last_window_connections`: Number of connections in the previous detection window
- `first_request_time`: Timestamp of first trigger (seconds)

#### 12.3 Ban Log List

- **Endpoint**: `GET /api/mqtt/ban-log/list`
- **Description**: Query client ban log, supports filtering by tenant
- **Request Parameters**:

```json
{
  "tenant": "default",              // Optional, filter by tenant (when specified, scans only that tenant's records)
  "limit": 20,
  "page": 1,
  "sort_field": "create_time",
  "sort_by": "desc",
  "filter_field": "resource_name",
  "filter_values": ["bad_client"],
  "exact_match": "false"
}
```

- **Response Data Structure**:

```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant": "default",
        "ban_type": "ClientId",
        "resource_name": "bad_client_001",
        "ban_source": "manual",
        "end_time": "2024-12-31 23:59:59",
        "create_time": "2024-01-01 10:00:00"
      }
    ],
    "total_count": 5
  }
}
```

**Field Descriptions**:

- `tenant`: Tenant name
- `ban_type`: Ban type (`ClientId` / `User` / `Ip` / `ClientIdMatch` / `UserMatch` / `IPCIDR`)
- `resource_name`: Banned resource name (client ID, username, or IP address)
- `ban_source`: Ban source (e.g. `manual` or `auto`)
- `end_time`: Ban expiry time (local time format)
- `create_time`: Ban creation time (local time format)

---

### 13. Tenant Management

> MQTT Broker supports multi-tenant isolation. A tenant is the top-level namespace for resources (users, ACLs, topics, subscriptions, etc.).

#### 13.1 Tenant List Query
- **Endpoint**: `GET /api/cluster/tenant/list`
- **Description**: Query MQTT tenant list, supports direct query by tenant name
- **Request Parameters**:
```json
{
  "tenant_name": "default",         // Optional, exact query by tenant name
  "limit": 20,
  "page": 1,
  "sort_field": "tenant_name",      // Optional, sort field
  "sort_by": "asc",
  "filter_field": "tenant_name",
  "filter_values": ["default"],
  "exact_match": "false"
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant_name": "default",
        "desc": "Default tenant",
        "create_time": 1640995200,
        "max_connections_per_node": 1000000,
        "max_create_connection_rate_per_second": 10000,
        "max_topics": 500000,
        "max_sessions": 5000000,
        "max_publish_rate": 10000
      }
    ],
    "total_count": 3
  }
}
```

**Field Descriptions**:
- `tenant_name`: Tenant name
- `desc`: Tenant description
- `create_time`: Tenant creation timestamp (seconds)
- `max_connections_per_node`: Maximum connections per node
- `max_create_connection_rate_per_second`: Maximum new connection rate per second
- `max_topics`: Maximum number of topics
- `max_sessions`: Maximum number of sessions
- `max_publish_rate`: Maximum publish message rate per second

#### 13.2 Create Tenant
- **Endpoint**: `POST /api/cluster/tenant/create`
- **Description**: Create new MQTT tenant
- **Request Parameters**:
```json
{
  "tenant_name": "new_tenant",                        // Required, tenant name, length 1-128
  "desc": "Production environment",                   // Optional, tenant description, length no more than 500
  "max_connections_per_node": 1000000,                // Optional, maximum connections per node
  "max_create_connection_rate_per_second": 10000,     // Optional, maximum new connection rate per second
  "max_topics": 500000,                               // Optional, maximum number of topics
  "max_sessions": 5000000,                            // Optional, maximum number of sessions
  "max_publish_rate": 10000                           // Optional, maximum publish message rate per second
}
```

- **Response**: Returns "success" on success

#### 13.3 Delete Tenant
- **Endpoint**: `POST /api/cluster/tenant/delete`
- **Description**: Delete MQTT tenant
- **Request Parameters**:
```json
{
  "tenant_name": "old_tenant"        // Required, tenant name, length 1-128
}
```

- **Response**: Returns "success" on success

**Notes**:
- Before deleting a tenant, ensure users, ACLs, subscriptions, and other resources under that tenant have been cleaned up
- It is not recommended to delete the system default tenant `default`

---

## Enumeration Values

### ACL Resource Type (resource_type)
- `ClientId`: Client ID
- `User`: Username
- `Ip`: IP Address

### ACL Action (action)
- `Publish`: Publish message
- `Subscribe`: Subscribe to topic
- `All`: All actions

### ACL Permission (permission)
- `Allow`: Allow
- `Deny`: Deny

### Blacklist Type (blacklist_type)
- `ClientId`: Exact match by client ID
- `User`: Exact match by username
- `Ip`: Exact match by IP address
- `ClientIdMatch`: Wildcard match by client ID (supports `*` wildcard)
- `UserMatch`: Wildcard match by username (supports `*` wildcard)
- `IPCIDR`: CIDR range match (e.g. `192.168.1.0/24`)

### Connector Type (connector_type)

> For the complete list of connector types and configuration parameters, please refer to [Connector Management HTTP API](Connector.md#enum-reference).

### Schema Type (schema_type)
- `json`: JSON Schema
- `avro`: Apache Avro
- `protobuf`: Protocol Buffers

### QoS Level
- `0`: At most once delivery
- `1`: At least once delivery
- `2`: Exactly once delivery

### Retained Message Handling (retained_handling)
- `0`: Send retained messages
- `1`: Send retained messages only on new subscription
- `2`: Don't send retained messages

---

## Usage Examples

### Query Cluster Overview
```bash
curl -X GET http://localhost:8080/api/mqtt/overview
```

### Query Clients for a Specific Tenant
```bash
curl "http://localhost:8080/api/mqtt/client/list?tenant=default&limit=10&page=1"
```

### Query Sessions for a Specific Tenant
```bash
curl "http://localhost:8080/api/mqtt/session/list?tenant=default&limit=20&page=1"
```

### Query Subscription List (Specific Tenant)
```bash
curl "http://localhost:8080/api/mqtt/subscribe/list?tenant=default&limit=20&page=1"
```

### Create Tenant
```bash
curl -X POST http://localhost:8080/api/cluster/tenant/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_name": "production",
    "desc": "Production environment tenant"
  }'
```

### Delete Topic
```bash
curl -X POST http://localhost:8080/api/cluster/topic/delete \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "topic_name": "sensor/temperature"
  }'
```

### Create User
```bash
curl -X POST http://localhost:8080/api/cluster/user/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "username": "testuser",
    "password": "testpass123",
    "is_superuser": false
  }'
```

### Create ACL Rule
```bash
curl -X POST http://localhost:8080/api/cluster/acl/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "name": "sensor-publish-rule",
    "desc": "Allow sensor devices to publish data",
    "resource_type": "ClientId",
    "resource_name": "sensor001",
    "topic": "sensor/+",
    "action": "Publish",
    "permission": "Allow"
  }'
```

### Create Blacklist Entry (Wildcard Match)
```bash
curl -X POST http://localhost:8080/api/cluster/blacklist/create \
  -H "Content-Type: application/json" \
  -d '{
    "name": "bl-bad-client-match",
    "tenant": "default",
    "blacklist_type": "ClientIdMatch",
    "resource_name": "bad_client_*",
    "end_time": 1735689599,
    "desc": "Block all clients matching bad_client_*"
  }'
```

### Create Schema
```bash
curl -X POST http://localhost:8080/api/cluster/schema/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "schema_name": "sensor_schema",
    "schema_type": "json",
    "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\"}}}",
    "desc": "Sensor data validation schema"
  }'
```

### Query Monitor Data
```bash
# Query connection count monitoring data
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=connection_num"

# Query message received count for a specific topic
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=topic_in_num&topic_name=sensor/temperature"
```

### Send Message
```bash
curl -X POST http://localhost:8080/api/mqtt/message/send \
  -H "Content-Type: application/json" \
  -d '{
    "topic": "sensor/temperature",
    "payload": "25.5",
    "retain": false
  }'
```

---

*Documentation Version: v5.0*
*Last Updated: 2026-03-15*
*Based on Code Version: RobustMQ Admin Server v0.1.35*
