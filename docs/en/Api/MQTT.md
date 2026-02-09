# MQTT Broker Management HTTP API

> This document describes all HTTP API interfaces related to MQTT Broker. For general information, please refer to [COMMON.md](COMMON.md).

## API Interface List

### 1. Cluster Overview

#### 1.1 Cluster Overview Information
- **Endpoint**: `POST /api/mqtt/overview`
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
- `connection_num`: Total number of connections (sum of all connection types)
- `session_num`: Total number of sessions
- `topic_num`: Total number of topics
- `placement_status`: Placement Center status (Leader/Follower)
- `tcp_connection_num`: Number of TCP connections
- `tls_connection_num`: Number of TLS connections
- `websocket_connection_num`: Number of WebSocket connections
- `quic_connection_num`: Number of QUIC connections
- `subscribe_num`: Total number of subscriptions (sum of all subscription lists)
- `exclusive_subscribe_num`: Number of exclusive subscriptions
- `exclusive_subscribe_thread_num`: Number of exclusive subscription push threads
- `share_subscribe_group_num`: Number of shared subscription groups
- `share_subscribe_num`: Number of shared subscriptions
- `share_subscribe_thread_num`: Number of shared subscription push threads
- `connector_num`: Total number of connectors
- `connector_thread_num`: Number of active connector threads

#### 1.2 Monitor Data Query
- **Endpoint**: `POST /api/mqtt/monitor/data`
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
    },
    {
      "date": 1640995320,
      "value": 1485
    }
  ]
}
```

**Field Descriptions**:
- `date`: Unix timestamp (seconds)
- `value`: Metric value at that time point

**Request Examples**:

Query connection count:
```json
{
  "data_type": "connection_num"
}
```

Query message count for a specific topic:
```json
{
  "data_type": "topic_in_num",
  "topic_name": "sensor/temperature"
}
```

**Notes**:
- Data retention period: By default, data from the last 1 hour is retained
- Data sampling interval: According to system configuration, typically 60 seconds
- **Parameter Requirements**:
  - Topic-level monitoring (`topic_in_num`, `topic_out_num`): Must provide `topic_name`
  - Subscription-level monitoring (`subscribe_send_success_num`, `subscribe_send_failure_num`): Must provide `client_id` and `path`
  - Subscription-topic-level monitoring (`subscribe_topic_send_success_num`, `subscribe_topic_send_failure_num`): Must provide `client_id`, `path` and `topic_name`
  - Session-level monitoring (`session_in_num`, `session_out_num`): Must provide `client_id`
  - Connector-level monitoring (`connector_send_success`, `connector_send_failure`): Must provide `connector_name`
  - If required parameters are missing, an empty array will be returned
- Returned data is naturally sorted by timestamp

---

### 2. Client Management

#### 2.1 Client List Query
- **Endpoint**: `POST /api/mqtt/client/list`
- **Description**: Query list of clients connected to the cluster
- **Request Parameters**:
```json
{
  "source_ip": "192.168.1.1",      // Optional, filter by source IP
  "connection_id": 12345,           // Optional, filter by connection ID
  "limit": 20,                      // Optional, page size
  "page": 1,                        // Optional, page number
  "sort_field": "connection_id",    // Optional, sort field
  "sort_by": "desc",                // Optional, sort order
  "filter_field": "client_id",      // Optional, filter field (e.g., "connection_id", "client_id")
  "filter_values": ["client001"],   // Optional, filter values
  "exact_match": "true"             // Optional, exact match
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
        "client_id": "client001",
        "connection_id": 12345,
        "mqtt_connection": {
          "connect_id": 12345,
          "client_id": "client001",
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
  - `client_id`: MQTT client ID
  - `session_expiry`: Session expiry interval in seconds
  - `is_contain_last_will`: Whether the session contains a last will message
  - `last_will_delay_interval`: Delay interval for last will message in seconds (optional)
  - `create_time`: Session creation timestamp
  - `connection_id`: Associated connection ID (optional)
  - `broker_id`: Broker node ID hosting the session (optional)
  - `reconnect_time`: Last reconnection timestamp (optional)
  - `distinct_time`: Last disconnection timestamp (optional)

- **heartbeat**: Connection heartbeat information (null if not available)
  - `protocol`: MQTT protocol version (Mqtt3, Mqtt4, Mqtt5)
  - `keep_live`: Keep-alive interval in seconds
  - `heartbeat`: Last heartbeat timestamp

---

### 3. Session Management

#### 3.1 Session List Query
- **Endpoint**: `POST /api/mqtt/session/list`
- **Description**: Query MQTT session list
- **Request Parameters**:
```json
{
  "client_id": "client001",         // Optional, filter by client ID
  "limit": 20,
  "page": 1,
  "sort_field": "create_time",      // Optional, sort field
  "sort_by": "desc",
  "filter_field": "client_id",
  "filter_values": ["client001"],
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

- **last_will**: Last will message information (null if no last will configured)
  - `client_id`: Client ID
  - `last_will`: Last will message content (can be null)
    - `topic`: Last will message topic (Bytes type, displayed as string)
    - `message`: Last will message payload (Bytes type, displayed as string)
    - `qos`: QoS level (`AtMostOnce`/`AtLeastOnce`/`ExactlyOnce`)
    - `retain`: Whether it's a retained message
  - `last_will_properties`: Last will properties (MQTT 5.0, can be null)
    - `delay_interval`: Delay interval in seconds before sending (optional)
    - `payload_format_indicator`: Payload format indicator (0=unspecified, 1=UTF-8, optional)
    - `message_expiry_interval`: Message expiry interval in seconds (optional)
    - `content_type`: Content type (e.g., "text/plain", optional)
    - `response_topic`: Response topic (optional)
    - `correlation_data`: Correlation data (Bytes type, optional)
    - `user_properties`: User properties array (list of key-value pairs)

---

### 4. Topic Management

#### 4.1 Topic List Query
- **Endpoint**: `POST /api/mqtt/topic/list`
- **Description**: Query MQTT topic list
- **Request Parameters**:
```json
{
  "topic_name": "sensor/+",         // Optional, filter by topic name
  "topic_type": "all",              // Optional, topic type: "all"(all topics), "normal"(normal topics), "system"(system topics), defaults to "all"
  "limit": 20,
  "page": 1,
  "sort_field": "topic_name",       // Optional, sort field
  "sort_by": "asc",
  "filter_field": "topic_name",
  "filter_values": ["sensor"],
  "exact_match": "false"
}
```

**Parameter Description**:
- **topic_type**: Topic type filter
  - `"all"` - Return all topics (default)
  - `"normal"` - Return only normal topics (topics not starting with `$`)
  - `"system"` - Return only system topics (topics starting with `$`, such as `$SYS/...`)

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

**Response Field Description**:
- `topic_id`: Topic unique identifier
- `topic_name`: Topic name
- `storage_type`: Storage type (e.g., Memory, RocksDB, etc.)
- `partition`: Number of partitions
- `replication`: Number of replicas
- `storage_name_list`: List of storage names
- `create_time`: Topic creation time (Unix timestamp in seconds)

#### 4.2 Topic Detail Query
- **Endpoint**: `POST /api/mqtt/topic/detail`
- **Description**: Query detailed information for a specific topic, including basic info, retained message, and subscriber list
- **Request Parameters**:
```json
{
  "topic_name": "sensor/temperature"  // Required, topic name
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "topic_info": {
      "cluster_name": "robustmq-cluster",
      "topic_name": "sensor/temperature",
      "create_time": 1640995200
    },
    "retain_message": "eyJ0ZW1wZXJhdHVyZSI6MjUuNX0=",
    "retain_message_at": 1640995300,
    "sub_list": [
      {
        "client_id": "client001",
        "path": "sensor/temperature"
      },
      {
        "client_id": "client002",
        "path": "sensor/+"
      }
    ]
  }
}
```

**Response Field Description**:

- **topic_info**: Topic basic information
  - `cluster_name`: Cluster name
  - `topic_name`: Topic name
  - `create_time`: Topic creation timestamp (seconds)

- **retain_message**: Retained message content
  - Type: `String` or `null`
  - Base64 encoded message content
  - Returns `null` if the topic has no retained message

- **retain_message_at**: Retained message timestamp
  - Type: `u64` or `null`
  - Unix timestamp in milliseconds
  - Indicates when the retained message was created or updated
  - Returns `null` if there is no retained message

- **sub_list**: List of clients subscribed to this topic
  - `client_id`: Subscriber client ID
  - `path`: Subscription path (may include wildcards like `+` or `#`)

**Notes**:
- Returns an error response if the topic does not exist: `{"code": 1, "message": "Topic does not exist."}`
- `sub_list` shows all subscriptions matching this topic, including wildcard subscriptions
- Retained message content is Base64 encoded and needs to be decoded by clients
- `retain_message_at` uses millisecond timestamps while `create_time` uses second timestamps

#### 4.3 Delete Topic
- **Endpoint**: `POST /api/mqtt/topic/delete`
- **Description**: Delete a specified topic
- **Request Parameters**:
```json
{
  "topic_name": "sensor/temperature"  // Required, topic name to delete
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": "success"
}
```

**Field Descriptions**:
- `topic_name`: Name of the topic to delete

**Notes**:
- Deleting a topic will remove all data for that topic, including retained messages
- Returns an error response if the topic does not exist or deletion fails
- This operation is irreversible, use with caution
- Deleting a topic does not automatically unsubscribe from it; subscriptions will remain

#### 4.4 Topic Rewrite Rules List
- **Endpoint**: `POST /api/mqtt/topic-rewrite/list`
- **Description**: Query topic rewrite rules list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
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
- **Endpoint**: `POST /api/mqtt/topic-rewrite/create`
- **Description**: Create new topic rewrite rule
- **Request Parameters**:
```json
{
  "action": "All",                  // Action type: All, Publish, Subscribe
  "source_topic": "old/topic/+",   // Source topic pattern
  "dest_topic": "new/topic/$1",     // Destination topic pattern
  "regex": "^old/topic/(.+)$"       // Regular expression
}
```

- **Parameter Validation Rules**:
  - `action`: Length must be between 1-50 characters, must be `All`, `Publish`, or `Subscribe`
  - `source_topic`: Length must be between 1-256 characters
  - `dest_topic`: Length must be between 1-256 characters
  - `regex`: Length must be between 1-500 characters

- **Response**: Returns "success" on success

#### 4.6 Delete Topic Rewrite Rule
- **Endpoint**: `POST /api/mqtt/topic-rewrite/delete`
- **Description**: Delete topic rewrite rule
- **Request Parameters**:
```json
{
  "action": "All",
  "source_topic": "old/topic/+"
}
```

- **Response**: Returns "success" on success

---

### 5. Subscription Management

#### 5.1 Subscription List Query
- **Endpoint**: `POST /api/mqtt/subscribe/list`
- **Description**: Query subscription list
- **Request Parameters**:
```json
{
  "client_id": "client001",         // Optional, filter by client ID
  "limit": 20,
  "page": 1,
  "sort_field": "create_time",
  "sort_by": "desc",
  "filter_field": "client_id",
  "filter_values": ["client001"],
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

#### 5.2 Subscription Detail Query
- **Endpoint**: `POST /api/mqtt/subscribe/detail`
- **Description**: Query subscription details, supports both exclusive and shared subscription details
- **Request Parameters**:
```json
{
  "client_id": "client001",    // Client ID
  "path": "sensor/temperature" // Subscription path
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

**Field Descriptions**:
- `share_sub`: Whether this is a shared subscription
- `group_leader_info`: Shared subscription group leader information (only for shared subscriptions)
  - `broker_id`: Broker node ID
  - `broker_addr`: Broker address
  - `extend_info`: Extended information
- `sub_data`: Subscription data details
  - `client_id`: Client ID
  - `path`: Subscription path
  - `push_subscribe`: Mapping of topics to subscribers (HashMap<topic_name, Subscriber>)
    - `client_id`: Client ID
    - `sub_path`: Subscription path
    - `rewrite_sub_path`: Rewritten subscription path (optional)
    - `topic_name`: Topic name
    - `group_name`: Group name (for shared subscriptions)
    - `protocol`: MQTT protocol version
    - `qos`: QoS level
    - `no_local`: Whether to exclude local messages
    - `preserve_retain`: Whether to preserve retain flag
    - `retain_forward_rule`: Retain message forwarding rule
    - `subscription_identifier`: Subscription identifier (optional)
    - `create_time`: Creation time (Unix timestamp in seconds)
  - `push_thread`: Mapping of topics to push thread data (HashMap<topic_name, thread_data>)
    - `push_success_record_num`: Successful push count
    - `push_error_record_num`: Failed push count
    - `last_push_time`: Last push time (Unix timestamp in seconds)
    - `last_run_time`: Last run time (Unix timestamp in seconds)
    - `create_time`: Creation time (Unix timestamp in seconds)
    - `bucket_id`: Bucket ID
  - `leader_id`: Leader node ID (for shared subscriptions)
```

**Shared Subscription Response Example**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "share_sub": true,
    "group_leader_info": {        // Leader info for the shared subscription group
      "broker_id": 1,
      "broker_addr": "127.0.0.1:1883",
      "extend_info": "{}"
    },
    "topic_list": [
      {
        "client_id": "client001",
        "path": "$share/group1/sensor/+",
        "topic_name": "sensor/temperature",
        "exclusive_push_data": null,
        "share_push_data": {      // Shared subscription leader push data
          "path": "$share/group1/sensor/+",
          "group_name": "group1",
          "sub_name": "sensor/+",
          "topic_name": "sensor/temperature",
          "sub_list": {           // Subscriber list in the shared group
            "client001": {
              "protocol": "MQTTv5",
              "client_id": "client001",
              "sub_path": "$share/group1/sensor/+",
              "rewrite_sub_path": null,
              "topic_name": "sensor/temperature",
              "group_name": "group1",
              "qos": "AtLeastOnce",
              "nolocal": false,
              "preserve_retain": false,
              "retain_forward_rule": "SendAtSubscribe",
              "subscription_identifier": null,
              "create_time": 1704067200000
            }
          }
        },
        "push_thread": {
          "push_success_record_num": 2540,
          "push_error_record_num": 5,
          "last_push_time": 1704067900000,
          "last_run_time": 1704067910000,
          "create_time": 1704067200000
        }
      }
    ]
  }
}
```

**Field Descriptions**:
- **share_sub**: Boolean value indicating whether this is a shared subscription
- **group_leader_info**: Only returned for shared subscriptions, contains the leader broker information for the shared group
  - `broker_id`: Leader broker's ID
  - `broker_addr`: Leader broker's address
  - `extend_info`: Extended information (JSON string)
- **topic_list**: List of actual topics matching the subscription path
  - `client_id`: Client ID
  - `path`: Subscription path (may include wildcards or shared subscription prefix)
  - `topic_name`: Actual matched topic name
  - `exclusive_push_data`: Push data for exclusive subscriptions (null for shared subscriptions)
  - `share_push_data`: Push data for shared subscriptions (null for exclusive subscriptions)
  - `push_thread`: Statistics for push thread (optional)

**Notes**:
- If the subscription path contains wildcards (like `+` or `#`), `topic_list` may contain multiple actual matched topics
- Exclusive and shared subscriptions have different data structures, distinguished by the `share_sub` field
- Shared subscription path format is `$share/{group_name}/{topic_filter}`
- All timestamps are millisecond Unix timestamps

#### 5.3 Auto Subscribe Rule Management

##### 5.3.1 Auto Subscribe List
- **Endpoint**: `POST /api/mqtt/auto-subscribe/list`
- **Description**: Query auto subscribe rules list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
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
- **Description**: Create new auto subscribe rule
- **Request Parameters**:
```json
{
  "topic": "system/+",              // Topic pattern
  "qos": 1,                         // QoS level: 0, 1, 2
  "no_local": false,                // No local
  "retain_as_published": false,     // Retain as published
  "retained_handling": 0            // Retained message handling: 0, 1, 2
}
```

- **Parameter Validation Rules**:
  - `topic`: Length must be between 1-256 characters
  - `qos`: Must be 0, 1, or 2
  - `no_local`: Boolean value
  - `retain_as_published`: Boolean value
  - `retained_handling`: Must be 0, 1, or 2

- **Response**: Returns "Created successfully!" on success

##### 5.3.3 Delete Auto Subscribe Rule
- **Endpoint**: `POST /api/mqtt/auto-subscribe/delete`
- **Description**: Delete auto subscribe rule
- **Request Parameters**:
```json
{
  "topic_name": "system/+"
}
```

- **Response**: Returns "Deleted successfully!" on success

#### 5.4 Slow Subscribe Monitoring

##### 5.4.1 Slow Subscribe List
- **Endpoint**: `POST /api/mqtt/slow-subscribe/list`
- **Description**: Query slow subscribe list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
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

---

### 6. User Management

#### 6.1 User List Query
- **Endpoint**: `POST /api/mqtt/user/list`
- **Description**: Query MQTT user list
- **Request Parameters**:
```json
{
  "user_name": "admin",             // Optional, filter by username
  "limit": 20,
  "page": 1,
  "sort_field": "username",         // Optional, sort field
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
- `username`: Username
- `is_superuser`: Whether it's a superuser
- `create_time`: User creation timestamp (seconds)

#### 6.2 Create User
- **Endpoint**: `POST /api/mqtt/user/create`
- **Description**: Create new MQTT user
- **Request Parameters**:
```json
{
  "username": "newuser",            // Username
  "password": "password123",        // Password
  "is_superuser": false             // Whether it's a superuser
}
```

- **Parameter Validation Rules**:
  - `username`: Length must be between 1-64 characters
  - `password`: Length must be between 1-128 characters
  - `is_superuser`: Boolean value

- **Response**: Returns "Created successfully!" on success

#### 6.3 Delete User
- **Endpoint**: `POST /api/mqtt/user/delete`
- **Description**: Delete MQTT user
- **Request Parameters**:
```json
{
  "username": "olduser"
}
```

- **Response**: Returns "Deleted successfully!" on success

---

### 7. ACL Management

#### 7.1 ACL List Query
- **Endpoint**: `POST /api/mqtt/acl/list`
- **Description**: Query access control list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
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
- **Endpoint**: `POST /api/mqtt/acl/create`
- **Description**: Create new ACL rule
- **Request Parameters**:
```json
{
  "resource_type": "ClientId",       // Resource type: ClientId, Username, IpAddress
  "resource_name": "client001",      // Resource name
  "topic": "sensor/+",               // Topic pattern
  "ip": "192.168.1.100",             // IP address
  "action": "Publish",               // Action: Publish, Subscribe, All
  "permission": "Allow"              // Permission: Allow, Deny
}
```

- **Parameter Validation Rules**:
  - `resource_type`: Length must be between 1-50 characters, must be `ClientId`, `Username`, or `IpAddress`
  - `resource_name`: Length must be between 1-256 characters
  - `topic`: Length must be between 1-256 characters
  - `ip`: Length cannot exceed 128 characters
  - `action`: Length must be between 1-50 characters, must be `Publish`, `Subscribe`, or `All`
  - `permission`: Length must be between 1-50 characters, must be `Allow` or `Deny`

- **Response**: Returns "Created successfully!" on success

#### 7.3 Delete ACL Rule
- **Endpoint**: `POST /api/mqtt/acl/delete`
- **Description**: Delete ACL rule
- **Request Parameters**:
```json
{
  "resource_type": "ClientId",
  "resource_name": "client001",
  "topic": "sensor/+",
  "ip": "192.168.1.100",
  "action": "Publish",
  "permission": "Allow"
}
```

- **Response**: Returns "Deleted successfully!" on success

---

### 8. Blacklist Management

#### 8.1 Blacklist List Query
- **Endpoint**: `POST /api/mqtt/blacklist/list`
- **Description**: Query blacklist
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
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
- **Endpoint**: `POST /api/mqtt/blacklist/create`
- **Description**: Add new blacklist entry
- **Request Parameters**:
```json
{
  "blacklist_type": "ClientId",        // Blacklist type: ClientId, IpAddress, Username
  "resource_name": "bad_client",       // Resource name
  "end_time": 1735689599,              // End time (Unix timestamp)
  "desc": "Blocked for security"       // Description
}
```

- **Parameter Validation Rules**:
  - `blacklist_type`: Length must be between 1-50 characters, must be `ClientId`, `IpAddress`, or `Username`
  - `resource_name`: Length must be between 1-256 characters
  - `end_time`: Must be greater than 0
  - `desc`: Length cannot exceed 500 characters

- **Response**: Returns "Created successfully!" on success

#### 8.3 Delete Blacklist Entry
- **Endpoint**: `POST /api/mqtt/blacklist/delete`
- **Description**: Delete blacklist entry
- **Request Parameters**:
```json
{
  "blacklist_type": "ClientId",
  "resource_name": "bad_client"
}
```

- **Response**: Returns "Deleted successfully!" on success

---

### 9. Connector Management

#### 9.1 Connector List Query
- **Endpoint**: `POST /api/mqtt/connector/list`
- **Description**: Query connector list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "connector_name": "kafka_connector",
        "connector_type": "kafka",
        "config": "{\"bootstrap_servers\":\"localhost:9092\"}",
        "topic_name": "topic_001",
        "status": "Running",
        "broker_id": "1",
        "create_time": "2024-01-01 10:00:00",
        "update_time": "2024-01-01 11:00:00"
      }
    ],
    "total_count": 8
  }
}
```

#### 9.2 Connector Detail Query
- **Endpoint**: `POST /api/mqtt/connector/detail`
- **Description**: Query detailed runtime status of a specific connector
- **Request Parameters**:
```json
{
  "connector_name": "kafka_connector"  // Connector name
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "last_send_time": 1698765432,        // Last send timestamp (Unix timestamp, seconds)
    "send_success_total": 10245,         // Total successful messages sent
    "send_fail_total": 3,                // Total failed messages
    "last_msg": "Batch sent successfully" // Last message description (may be null)
  }
}
```

- **Error Responses**:
```json
{
  "code": 1,
  "message": "Connector kafka_connector does not exist."
}
```

or

```json
{
  "code": 1,
  "message": "Connector thread kafka_connector does not exist."
}
```

- **Field Descriptions**:
  - `last_send_time`: Timestamp (seconds) of the last message sent by the connector
  - `send_success_total`: Total number of successfully sent messages since connector startup
  - `send_fail_total`: Total number of failed messages since connector startup
  - `last_msg`: Description of the last operation message, may be `null`

- **Usage Notes**:
  - The connector must exist and be currently running (with an active thread) to query details
  - If the connector exists but is not running, an "thread does not exist" error will be returned
  - Statistics data will be reset when the connector restarts

#### 9.3 Create Connector
- **Endpoint**: `POST /api/mqtt/connector/create`
- **Description**: Create new connector
- **Request Parameters**:
```json
{
  "connector_name": "new_connector",   // Connector name
  "connector_type": "kafka",           // Connector type
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",  // Configuration (JSON string)
  "topic_name": "sensor/+",            // Associated topic ID
  "failure_strategy": "{\"Discard\":{}}" // Optional, failure handling strategy (JSON string), defaults to Discard
}
```

- **Parameter Validation Rules**:
  - `connector_name`: Length must be between 1-128 characters
  - `connector_type`: Length must be between 1-50 characters, must be `kafka`, `pulsar`, `rabbitmq`, `greptime`, `postgres`, `mysql`, `mongodb`, `file`, or `elasticsearch`
  - `config`: Length must be between 1-4096 characters
  - `topic_name`: Length must be between 1-256 characters
  - `failure_strategy`: Optional, length must be between 1-1024 characters (JSON string)

**Connector Types and Configuration Examples**：

**Kafka Connector**:
```json
{
  "connector_type": "kafka",
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\",\"key\":\"\"}"
}
```

**Kafka Connector (with advanced configuration)**:
```json
{
  "connector_type": "kafka",
  "config": "{\"bootstrap_servers\":\"127.0.0.1:9092,127.0.0.2:9092\",\"topic\":\"mqtt_messages\",\"key\":\"\",\"compression_type\":\"lz4\",\"batch_size\":32768,\"linger_ms\":10,\"acks\":\"all\",\"retries\":5,\"message_timeout_ms\":60000,\"cleanup_timeout_secs\":15}"
}
```

**Kafka Configuration Parameters**:

**Required Parameters**:
- `bootstrap_servers`: Kafka broker addresses, format: `host1:port1,host2:port2,host3:port3`
  - Supports multiple comma-separated addresses for cluster configuration
  - Each address will be validated for correct format (host:port)
  - At least one broker must be reachable (network connectivity check performed during validation)
  - Example: `"127.0.0.1:9092"` or `"127.0.0.1:9092,127.0.0.2:9092,127.0.0.3:9092"`
- `topic`: Kafka topic name where messages will be published

**Optional Parameters**:
- `key`: Message key for partitioning (default: `""`)
  - Empty string: uses message's inherent key or Kafka round-robin partitioning
  - Non-empty: all messages use this fixed key for partition assignment
  - Max length: 256 characters

**Performance Parameters**:
- `compression_type`: Message compression algorithm (default: `"none"`)
  - Valid values: `"none"`, `"gzip"`, `"snappy"`, `"lz4"`, `"zstd"`
  - Recommended: `"lz4"` for best balance of speed and compression ratio
- `batch_size`: Maximum batch size in bytes (default: `16384`)
  - Range: 1 to 1,048,576 bytes (1MB)
  - Larger values improve throughput but increase latency
- `linger_ms`: Time to wait before sending batch in milliseconds (default: `5`)
  - Range: 0 to 60,000 ms (60 seconds)
  - Higher values batch more messages but increase end-to-end latency

**Reliability Parameters**:
- `acks`: Message acknowledgment level (default: `"1"`)
  - `"0"`: No acknowledgment (fastest, least reliable)
  - `"1"`: Leader acknowledgment only (balanced)
  - `"all"` or `"-1"`: All in-sync replicas acknowledgment (slowest, most reliable)
- `retries`: Maximum number of retry attempts on failure (default: `3`)
  - Range: 0 to 100
- `message_timeout_ms`: Total timeout for message delivery in milliseconds (default: `30000`)
  - Range: 1,000 to 300,000 ms (1 second to 5 minutes)
  - Includes retries and waiting for acknowledgments

**Cleanup Parameters**:
- `cleanup_timeout_secs`: Timeout for flushing messages during connector shutdown (default: `10`)
  - Range: 0 to 300 seconds
  - Ensures buffered messages are sent before connector stops

**Configuration Examples**:

*High throughput configuration*:
```json
{
  "bootstrap_servers": "kafka1:9092,kafka2:9092,kafka3:9092",
  "topic": "mqtt_high_volume",
  "compression_type": "lz4",
  "batch_size": 65536,
  "linger_ms": 50,
  "acks": "1"
}
```

*High reliability configuration*:
```json
{
  "bootstrap_servers": "kafka1:9092,kafka2:9092,kafka3:9092",
  "topic": "mqtt_critical",
  "acks": "all",
  "retries": 10,
  "message_timeout_ms": 60000
}
```

*Low latency configuration*:
```json
{
  "bootstrap_servers": "kafka:9092",
  "topic": "mqtt_realtime",
  "batch_size": 1024,
  "linger_ms": 0,
  "compression_type": "none"
}
```

**Pulsar Connector**:
```json
{
  "connector_type": "pulsar",
  "config": "{\"server\":\"pulsar://localhost:6650\",\"topic\":\"mqtt-messages\",\"token\":\"your-auth-token\"}"
}
```

**Pulsar Connector (with advanced configuration)**:
```json
{
  "connector_type": "pulsar",
  "config": "{\"server\":\"pulsar://pulsar.example.com:6650\",\"topic\":\"mqtt-messages\",\"token\":\"your-auth-token\",\"connection_timeout_secs\":30,\"operation_timeout_secs\":30,\"send_timeout_secs\":30,\"batch_size\":500,\"max_pending_messages\":5000,\"compression\":\"lz4\"}"
}
```

**Pulsar Configuration Parameters**:

**Required Parameters**:
- `server`: Pulsar broker address
  - Format: `pulsar://host:port` or `pulsar+ssl://host:port` for TLS
  - Example: `"pulsar://localhost:6650"` or `"pulsar://broker1.example.com:6650"`
  - Length: 1 to 512 characters
- `topic`: Pulsar topic name where messages will be published
  - Example: `"mqtt-messages"` or `"persistent://tenant/namespace/topic"`
  - Length: 1 to 256 characters
  - Supports full topic format with tenant and namespace

**Authentication Parameters** (choose one method):
- **Token Authentication**:
  - `token`: Authentication token
    - Length: up to 1,024 characters
    - Example: `"eyJhbGciOiJIUzI1NiJ9..."`

- **OAuth2 Authentication**:
  - `oauth`: OAuth2 configuration as JSON string
    - Length: up to 1,024 characters
    - Must be valid JSON containing OAuth2 parameters
    - Example: `"{\"issuer_url\":\"https://auth.example.com\",\"credentials_url\":\"file:///path/to/credentials.json\"}"`

- **Basic Authentication**:
  - `basic_name`: Username for basic authentication
    - Length: up to 256 characters
  - `basic_password`: Password for basic authentication
    - Length: up to 256 characters
    - Both `basic_name` and `basic_password` must be provided together

**Important**: Only one authentication method can be specified. If multiple methods are provided, validation will fail.

**Timeout Parameters**:
- `connection_timeout_secs`: Connection timeout in seconds (default: `30`)
  - Range: 1 to 300 seconds
  - Time to wait when establishing connection to Pulsar broker
- `operation_timeout_secs`: Operation timeout in seconds (default: `30`)
  - Range: 1 to 300 seconds
  - Timeout for Pulsar operations (e.g., creating producer, lookup)
- `send_timeout_secs`: Send timeout in seconds (default: `30`)
  - Range: 1 to 300 seconds
  - Maximum time to wait for message send confirmation

**Performance Parameters**:
- `batch_size`: Number of records to process in a single batch (default: `100`)
  - Range: 1 to 10,000
  - Larger values improve throughput but increase latency and memory usage
  - Used by the connector read loop to determine how many records to fetch
- `max_pending_messages`: Maximum number of pending messages in the queue (default: `1000`)
  - Range: 1 to 100,000
  - Controls memory usage and backpressure
  - Higher values allow more messages to be queued but increase memory usage

**Compression Parameters**:
- `compression`: Compression algorithm for message payload (default: `none`)
  - Valid values: `"none"`, `"lz4"`, `"zlib"`, `"zstd"`, `"snappy"`
  - `none`: No compression (fastest, largest size)
  - `lz4`: Fast compression with decent compression ratio
  - `zlib`: Balanced compression
  - `zstd`: High compression ratio (recommended for bandwidth-limited scenarios)
  - `snappy`: Very fast compression (good for low-latency scenarios)

**Configuration Examples**:

*Basic configuration (development)*:
```json
{
  "server": "pulsar://localhost:6650",
  "topic": "mqtt-messages"
}
```

*Production configuration with token authentication*:
```json
{
  "server": "pulsar://pulsar-broker.example.com:6650",
  "topic": "persistent://public/default/mqtt-messages",
  "token": "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJtcXR0LXVzZXIifQ...",
  "connection_timeout_secs": 30,
  "operation_timeout_secs": 30,
  "send_timeout_secs": 30,
  "batch_size": 200,
  "max_pending_messages": 2000,
  "compression": "lz4"
}
```

*High throughput configuration with compression*:
```json
{
  "server": "pulsar://pulsar-cluster.example.com:6650",
  "topic": "persistent://mqtt/logs/high-volume",
  "token": "production-token",
  "connection_timeout_secs": 15,
  "operation_timeout_secs": 15,
  "send_timeout_secs": 60,
  "batch_size": 1000,
  "max_pending_messages": 10000,
  "compression": "zstd"
}
```

*OAuth2 authentication configuration*:
```json
{
  "server": "pulsar+ssl://secure-pulsar.example.com:6651",
  "topic": "persistent://enterprise/production/mqtt-events",
  "oauth": "{\"issuer_url\":\"https://auth.example.com\",\"credentials_url\":\"file:///etc/pulsar/oauth2.json\",\"audience\":\"urn:pulsar:cluster\"}",
  "connection_timeout_secs": 30,
  "operation_timeout_secs": 30,
  "batch_size": 500,
  "compression": "lz4"
}
```

*Basic authentication configuration*:
```json
{
  "server": "pulsar://internal-broker.example.com:6650",
  "topic": "mqtt-internal",
  "basic_name": "mqtt_user",
  "basic_password": "secure_password",
  "batch_size": 100,
  "max_pending_messages": 1000
}
```

**RabbitMQ Connector**:
```json
{
  "connector_type": "rabbitmq",
  "config": "{\"server\":\"localhost\",\"port\":5672,\"username\":\"guest\",\"password\":\"guest\",\"virtual_host\":\"/\",\"exchange\":\"mqtt_messages\",\"routing_key\":\"sensor.data\",\"delivery_mode\":\"Persistent\",\"enable_tls\":false}"
}
```

**RabbitMQ Connector (with advanced configuration)**:
```json
{
  "connector_type": "rabbitmq",
  "config": "{\"server\":\"rabbitmq.example.com\",\"port\":5672,\"username\":\"mqtt_producer\",\"password\":\"secure_password\",\"virtual_host\":\"/mqtt\",\"exchange\":\"mqtt_messages\",\"routing_key\":\"sensor.#\",\"delivery_mode\":\"Persistent\",\"enable_tls\":false,\"connection_timeout_secs\":30,\"heartbeat_secs\":60,\"batch_size\":100,\"channel_max\":2047,\"frame_max\":131072,\"confirm_timeout_secs\":30,\"publisher_confirms\":true}"
}
```

**RabbitMQ Configuration Parameters**:

**Required Parameters**:
- `server`: RabbitMQ server address (hostname or IP address)
  - Example: `"localhost"`, `"rabbitmq.example.com"`, `"192.168.1.100"`
  - Length: 1 to 512 characters
- `username`: Username for authentication
  - Example: `"guest"`, `"mqtt_producer"`
  - Length: 1 to 256 characters
- `exchange`: Exchange name where messages will be published
  - Example: `"mqtt_messages"`, `"amq.topic"`
  - Length: 1 to 256 characters
  - The exchange should exist before publishing messages

**Optional Parameters**:
- `port`: RabbitMQ server port (default: `5672`)
  - Standard port: `5672` (AMQP), `5671` (AMQPS with TLS)
  - Must be greater than 0
- `password`: Password for authentication (default: `""`)
  - Length: up to 256 characters
  - Can be empty if RabbitMQ allows passwordless authentication
- `virtual_host`: Virtual host name (default: `"/"`)
  - Example: `"/"`, `"/mqtt"`, `"/production"`
  - Length: up to 256 characters
  - Virtual hosts provide logical separation within RabbitMQ
- `routing_key`: Routing key for message routing (default: `""`)
  - Example: `"sensor.temperature"`, `"sensor.#"`, `"*.critical"`
  - Length: up to 256 characters
  - Empty string: messages routed based on exchange type
  - Supports wildcards for topic exchanges: `*` (one word), `#` (zero or more words)
- `delivery_mode`: Message persistence mode (default: `"NonPersistent"`)
  - Valid values: `"NonPersistent"`, `"Persistent"`
  - `NonPersistent`: Faster, messages may be lost on broker restart
  - `Persistent`: Slower, messages survive broker restarts (requires durable exchange and queue)
- `enable_tls`: Enable TLS/SSL connection (default: `false`)
  - `false`: Use AMQP protocol (port 5672)
  - `true`: Use AMQPS protocol (port 5671)

**Connection Parameters**:
- `connection_timeout_secs`: Connection timeout in seconds (default: `30`)
  - Range: 1 to 300 seconds
  - Time to wait when establishing connection to RabbitMQ broker
- `heartbeat_secs`: Heartbeat interval in seconds (default: `60`)
  - Range: 0 to 300 seconds
  - `0`: Disable heartbeat
  - Recommended: 30-120 seconds for production
  - Lower values detect connection failures faster but increase network traffic
- `channel_max`: Maximum number of channels per connection (default: `2047`)
  - Must be greater than 0
  - RabbitMQ protocol maximum is 65535, but most servers limit to 2047
  - This connector uses 1 channel, but the setting is applied to the connection
- `frame_max`: Maximum frame size in bytes (default: `131072`)
  - Range: 4,096 to 1,048,576 bytes (4KB to 1MB)
  - Larger frames reduce protocol overhead but increase memory usage
  - Most RabbitMQ servers default to 128KB (131072 bytes)

**Performance Parameters**:
- `batch_size`: Number of records to process in a single batch (default: `100`)
  - Range: 1 to 10,000
  - Larger values improve throughput when publisher confirms are enabled
  - Used by the connector read loop to determine how many records to fetch
- `publisher_confirms`: Enable publisher confirms for reliability (default: `true`)
  - `true`: Wait for broker acknowledgment (reliable, slower)
  - `false`: Fire-and-forget mode (fast, may lose messages)
  - Recommended: `true` for production to ensure message delivery
- `confirm_timeout_secs`: Timeout for publisher confirms in seconds (default: `30`)
  - Range: 1 to 300 seconds
  - Only applies when `publisher_confirms` is `true`
  - Timeout for waiting for broker acknowledgment per message
  - Higher values accommodate slow brokers but delay error detection

**Configuration Examples**:

*Basic configuration (development)*:
```json
{
  "server": "localhost",
  "port": 5672,
  "username": "guest",
  "password": "guest",
  "virtual_host": "/",
  "exchange": "mqtt_messages",
  "routing_key": "",
  "delivery_mode": "NonPersistent",
  "enable_tls": false
}
```

*Production configuration (high reliability)*:
```json
{
  "server": "rabbitmq-cluster.example.com",
  "port": 5671,
  "username": "mqtt_producer",
  "password": "secure_password",
  "virtual_host": "/production",
  "exchange": "mqtt_messages_persistent",
  "routing_key": "mqtt.messages",
  "delivery_mode": "Persistent",
  "enable_tls": true,
  "connection_timeout_secs": 60,
  "heartbeat_secs": 30,
  "batch_size": 100,
  "confirm_timeout_secs": 30,
  "publisher_confirms": true
}
```

*High throughput configuration*:
```json
{
  "server": "rabbitmq.example.com",
  "port": 5672,
  "username": "mqtt_producer",
  "password": "xxx",
  "virtual_host": "/mqtt",
  "exchange": "mqtt_high_volume",
  "routing_key": "",
  "delivery_mode": "NonPersistent",
  "enable_tls": false,
  "connection_timeout_secs": 30,
  "heartbeat_secs": 60,
  "batch_size": 1000,
  "confirm_timeout_secs": 60,
  "publisher_confirms": true
}
```

*Low latency configuration*:
```json
{
  "server": "localhost",
  "port": 5672,
  "username": "guest",
  "password": "guest",
  "virtual_host": "/",
  "exchange": "mqtt_realtime",
  "routing_key": "realtime",
  "delivery_mode": "NonPersistent",
  "enable_tls": false,
  "batch_size": 10,
  "publisher_confirms": false
}
```

*Topic exchange with routing patterns*:
```json
{
  "server": "rabbitmq.example.com",
  "port": 5672,
  "username": "mqtt_producer",
  "password": "xxx",
  "virtual_host": "/sensors",
  "exchange": "amq.topic",
  "routing_key": "sensor.temperature.room1",
  "delivery_mode": "Persistent",
  "enable_tls": false,
  "batch_size": 50,
  "publisher_confirms": true,
  "confirm_timeout_secs": 30
}
```

**GreptimeDB Connector**:
```json
{
  "connector_type": "greptime",
  "config": "{\"server_addr\":\"localhost:4000\",\"database\":\"public\",\"user\":\"greptime_user\",\"password\":\"greptime_pwd\",\"precision\":\"Second\"}"
}
```

**PostgreSQL Connector**:
```json
{
  "connector_type": "postgres",
  "config": "{\"host\":\"localhost\",\"port\":5432,\"database\":\"mqtt_data\",\"username\":\"postgres\",\"password\":\"password123\",\"table\":\"mqtt_messages\",\"pool_size\":10,\"enable_batch_insert\":true,\"enable_upsert\":false,\"conflict_columns\":\"id\"}"
}
```

**PostgreSQL Connector (with advanced configuration)**:
```json
{
  "connector_type": "postgres",
  "config": "{\"host\":\"postgres.example.com\",\"port\":5432,\"database\":\"mqtt_prod\",\"username\":\"mqtt_user\",\"password\":\"secure_password\",\"table\":\"mqtt_messages\",\"pool_size\":20,\"min_pool_size\":5,\"enable_batch_insert\":true,\"enable_upsert\":true,\"conflict_columns\":\"client_id, topic\",\"connect_timeout_secs\":10,\"acquire_timeout_secs\":30,\"idle_timeout_secs\":600,\"max_lifetime_secs\":1800,\"batch_size\":500}"
}
```

**PostgreSQL Configuration Parameters**:

**Required Parameters**:
- `host`: PostgreSQL server address
  - Example: `"localhost"` or `"postgres.example.com"`
  - Length: 1 to 512 characters
- `port`: PostgreSQL server port (default: `5432`)
- `database`: Database name
  - Length: 1 to 256 characters
- `username`: PostgreSQL username for authentication
  - Length: 1 to 256 characters
- `password`: PostgreSQL password for authentication
  - Length: up to 256 characters
- `table`: Table name where messages will be stored
  - Length: 1 to 256 characters
  - Can only contain letters, numbers, underscores, and dots (for schema.table format)
  - Format validation is performed during connector creation
  - Example: `"mqtt_messages"` or `"public.mqtt_messages"`

**Connection Pool Parameters** (optional):
- `pool_size`: Maximum number of connections in the pool (default: `10`)
  - Range: 1 to 1,000
  - Larger values support higher concurrency
- `min_pool_size`: Minimum number of connections in the pool (default: `2`)
  - Must be less than or equal to `pool_size`
  - Keeps connections warm for faster access

**Timeout Parameters**:
- `connect_timeout_secs`: Connection timeout in seconds (default: `10`)
  - Range: 1 to 300 seconds
  - Time to wait when establishing a new database connection
  - Note: This is controlled by the connection string, not the pool options
- `acquire_timeout_secs`: Connection acquisition timeout in seconds (default: `30`)
  - Range: 1 to 300 seconds
  - Maximum time to wait for getting a connection from the pool
- `idle_timeout_secs`: Idle connection timeout in seconds (default: `600`, 10 minutes)
  - Range: 0 to 3,600 seconds (0 means no timeout)
  - Connections idle longer than this will be closed
- `max_lifetime_secs`: Maximum connection lifetime in seconds (default: `1800`, 30 minutes)
  - Range: 0 to 7,200 seconds (0 means no limit)
  - Connections older than this will be closed and recreated

**Performance Parameters**:
- `batch_size`: Number of records to process in a single batch (default: `100`)
  - Range: 1 to 10,000
  - Larger values improve throughput but increase latency and memory usage
  - Used by the connector read loop to determine how many records to fetch
- `enable_batch_insert`: Whether to use batch insert mode (default: `false`)
  - `true`: Insert multiple records in a single SQL statement (much faster for high throughput)
  - `false`: Insert records one by one (allows custom sql_template)
  - Cannot be used together with `sql_template`
- `enable_upsert`: Whether to enable upsert behavior (default: `false`)
  - `true`: Update existing records on conflict (uses PostgreSQL `ON CONFLICT ... DO UPDATE`)
  - `false`: Insert only (fails on duplicate key)

**Upsert Configuration**:
- `conflict_columns`: Column name(s) to detect conflicts (required when `enable_upsert` is `true`)
  - Example: `"client_id, topic"` or `"id"`
  - Used to identify which records should be updated
  - Must match the unique constraint or primary key in the table

**Custom SQL Configuration**:
- `sql_template`: Custom SQL template for inserts (optional)
  - Must contain exactly 5 placeholders (`$1`-`$5`) in order: `client_id`, `topic`, `timestamp`, `payload`, `data`
  - Example: `"INSERT INTO mqtt_messages (client_id, topic, ts, payload, data) VALUES ($1, $2, $3, $4, $5)"`
  - Cannot be used together with `enable_batch_insert` (will be rejected during validation)
  - Useful for custom table schemas or additional columns with default values
  - Note: PostgreSQL uses `$1`, `$2` syntax for parameters, not `?` like MySQL

**Configuration Examples**:

*Basic configuration (development)*:
```json
{
  "host": "localhost",
  "port": 5432,
  "database": "mqtt_data",
  "username": "postgres",
  "password": "password123",
  "table": "mqtt_messages"
}
```

*Production configuration with connection pooling*:
```json
{
  "host": "postgres-primary.example.com",
  "port": 5432,
  "database": "mqtt_prod",
  "username": "mqtt_user",
  "password": "secure_password",
  "table": "messages",
  "pool_size": 50,
  "min_pool_size": 10,
  "connect_timeout_secs": 10,
  "acquire_timeout_secs": 30,
  "idle_timeout_secs": 600,
  "max_lifetime_secs": 1800,
  "batch_size": 200,
  "enable_batch_insert": true
}
```

*High throughput configuration with upsert*:
```json
{
  "host": "postgres-cluster.example.com",
  "port": 5432,
  "database": "mqtt_logs",
  "username": "mqtt_writer",
  "password": "write_password",
  "table": "high_volume_messages",
  "pool_size": 100,
  "min_pool_size": 20,
  "connect_timeout_secs": 5,
  "acquire_timeout_secs": 15,
  "idle_timeout_secs": 300,
  "max_lifetime_secs": 900,
  "batch_size": 1000,
  "enable_batch_insert": true,
  "enable_upsert": true,
  "conflict_columns": "client_id, topic"
}
```

*High reliability configuration with custom SQL*:
```json
{
  "host": "postgres-replica.example.com",
  "port": 5432,
  "database": "mqtt_critical",
  "username": "mqtt_user",
  "password": "critical_password",
  "table": "critical_messages",
  "pool_size": 20,
  "min_pool_size": 5,
  "connect_timeout_secs": 15,
  "acquire_timeout_secs": 60,
  "idle_timeout_secs": 1200,
  "max_lifetime_secs": 3600,
  "batch_size": 50,
  "enable_batch_insert": false,
  "sql_template": "INSERT INTO critical_messages (client_id, topic, timestamp, payload, data, created_at) VALUES ($1, $2, $3, $4, $5, NOW())"
}
```

**MySQL Connector**:
```json
{
  "connector_type": "mysql",
  "config": "{\"host\":\"localhost\",\"port\":3306,\"database\":\"mqtt_data\",\"username\":\"root\",\"password\":\"password123\",\"table\":\"mqtt_messages\",\"pool_size\":10,\"enable_batch_insert\":true,\"enable_upsert\":false,\"conflict_columns\":\"id\"}"
}
```

**MySQL Connector (with advanced configuration)**:
```json
{
  "connector_type": "mysql",
  "config": "{\"host\":\"mysql.example.com\",\"port\":3306,\"database\":\"mqtt_prod\",\"username\":\"mqtt_user\",\"password\":\"secure_password\",\"table\":\"mqtt_messages\",\"pool_size\":20,\"min_pool_size\":5,\"enable_batch_insert\":true,\"enable_upsert\":true,\"conflict_columns\":\"record_key\",\"connect_timeout_secs\":10,\"acquire_timeout_secs\":30,\"idle_timeout_secs\":600,\"max_lifetime_secs\":1800,\"batch_size\":500}"
}
```

**MySQL Configuration Parameters**:

**Required Parameters**:
- `host`: MySQL server address
  - Example: `"localhost"` or `"mysql.example.com"`
  - Length: 1 to 512 characters
- `port`: MySQL server port (default: `3306`)
- `database`: Database name
  - Length: 1 to 256 characters
- `username`: MySQL username for authentication
  - Length: 1 to 256 characters
- `password`: MySQL password for authentication
  - Length: up to 256 characters
- `table`: Table name where messages will be stored
  - Length: 1 to 256 characters
  - Can only contain letters, numbers, underscores, and dots (for schema.table format)
  - Format validation is performed during connector creation
  - Example: `"mqtt_messages"` or `"mqtt_db.messages"`

**Connection Pool Parameters** (optional):
- `pool_size`: Maximum number of connections in the pool (default: `10`)
  - Range: 1 to 1,000
  - Larger values support higher concurrency
- `min_pool_size`: Minimum number of connections in the pool (default: `2`)
  - Must be less than or equal to `pool_size`
  - Keeps connections warm for faster access

**Timeout Parameters**:
- `connect_timeout_secs`: Connection timeout in seconds (default: `10`)
  - Range: 1 to 300 seconds
  - Time to wait when establishing a new database connection
  - Note: This is controlled by the connection string, not the pool options
- `acquire_timeout_secs`: Connection acquisition timeout in seconds (default: `30`)
  - Range: 1 to 300 seconds
  - Maximum time to wait for getting a connection from the pool
- `idle_timeout_secs`: Idle connection timeout in seconds (default: `600`, 10 minutes)
  - Range: 0 to 3,600 seconds (0 means no timeout)
  - Connections idle longer than this will be closed
- `max_lifetime_secs`: Maximum connection lifetime in seconds (default: `1800`, 30 minutes)
  - Range: 0 to 7,200 seconds (0 means no limit)
  - Connections older than this will be closed and recreated

**Performance Parameters**:
- `batch_size`: Number of records to process in a single batch (default: `100`)
  - Range: 1 to 10,000
  - Larger values improve throughput but increase latency and memory usage
  - Used by the connector read loop to determine how many records to fetch
- `enable_batch_insert`: Whether to use batch insert mode (default: `false`)
  - `true`: Insert multiple records in a single SQL statement (much faster for high throughput)
  - `false`: Insert records one by one (allows custom sql_template)
  - Cannot be used together with `sql_template`
- `enable_upsert`: Whether to enable upsert behavior (default: `false`)
  - `true`: Update existing records on conflict (uses `ON DUPLICATE KEY UPDATE`)
  - `false`: Insert only (fails on duplicate key)
  - Uses MySQL 8.0.19+ syntax: `AS new_vals ON DUPLICATE KEY UPDATE ...`

**Upsert Configuration**:
- `conflict_columns`: Column name(s) to detect conflicts (required when `enable_upsert` is `true`)
  - Example: `"record_key"` or `"id"`
  - Used to identify which records should be updated

**Custom SQL Configuration**:
- `sql_template`: Custom SQL template for inserts (optional)
  - Must contain exactly 3 placeholders (`?`) in order: `record_key`, `payload`, `timestamp`
  - Example: `"INSERT INTO mqtt_messages (key, data, ts) VALUES (?, ?, ?)"`
  - Cannot be used together with `enable_batch_insert` (will be rejected during validation)
  - Useful for custom table schemas or additional columns with default values

**Configuration Examples**:

*Basic configuration (development)*:
```json
{
  "host": "localhost",
  "port": 3306,
  "database": "mqtt_data",
  "username": "root",
  "password": "password123",
  "table": "mqtt_messages"
}
```

*Production configuration with connection pooling*:
```json
{
  "host": "mysql-primary.example.com",
  "port": 3306,
  "database": "mqtt_prod",
  "username": "mqtt_user",
  "password": "secure_password",
  "table": "messages",
  "pool_size": 50,
  "min_pool_size": 10,
  "connect_timeout_secs": 10,
  "acquire_timeout_secs": 30,
  "idle_timeout_secs": 600,
  "max_lifetime_secs": 1800,
  "batch_size": 200,
  "enable_batch_insert": true
}
```

*High throughput configuration with upsert*:
```json
{
  "host": "mysql-cluster.example.com",
  "port": 3306,
  "database": "mqtt_logs",
  "username": "mqtt_writer",
  "password": "write_password",
  "table": "high_volume_messages",
  "pool_size": 100,
  "min_pool_size": 20,
  "connect_timeout_secs": 5,
  "acquire_timeout_secs": 15,
  "idle_timeout_secs": 300,
  "max_lifetime_secs": 900,
  "batch_size": 1000,
  "enable_batch_insert": true,
  "enable_upsert": true,
  "conflict_columns": "record_key"
}
```

*High reliability configuration with custom SQL*:
```json
{
  "host": "mysql-replica.example.com",
  "port": 3306,
  "database": "mqtt_critical",
  "username": "mqtt_user",
  "password": "critical_password",
  "table": "critical_messages",
  "pool_size": 20,
  "min_pool_size": 5,
  "connect_timeout_secs": 15,
  "acquire_timeout_secs": 60,
  "idle_timeout_secs": 1200,
  "max_lifetime_secs": 3600,
  "batch_size": 50,
  "enable_batch_insert": false,
  "sql_template": "INSERT INTO critical_messages (msg_key, msg_payload, msg_timestamp, created_at) VALUES (?, ?, ?, NOW())"
}
```

**MongoDB Connector**:
```json
{
  "connector_type": "mongodb",
  "config": "{\"host\":\"localhost\",\"port\":27017,\"database\":\"mqtt_data\",\"collection\":\"mqtt_messages\",\"username\":\"mqtt_user\",\"password\":\"mqtt_pass\",\"auth_source\":\"admin\",\"deployment_mode\":\"single\",\"enable_tls\":false,\"max_pool_size\":10,\"min_pool_size\":2}"
}
```

**MongoDB Connector (with advanced configuration)**:
```json
{
  "connector_type": "mongodb",
  "config": "{\"host\":\"mongo1.example.com\",\"port\":27017,\"database\":\"mqtt_prod\",\"collection\":\"messages\",\"username\":\"mqtt_user\",\"password\":\"secure_password\",\"deployment_mode\":\"replicaset\",\"replica_set_name\":\"rs0\",\"enable_tls\":true,\"max_pool_size\":50,\"min_pool_size\":5,\"connect_timeout_secs\":10,\"server_selection_timeout_secs\":30,\"socket_timeout_secs\":60,\"batch_size\":500,\"ordered_insert\":false,\"w\":\"majority\"}"
}
```

**MongoDB Configuration Parameters**:

**Required Parameters**:
- `host`: MongoDB server address
  - Example: `"localhost"` or `"mongo.example.com"`
- `port`: MongoDB server port (default: `27017`)
- `database`: Database name
- `collection`: Collection name where messages will be stored

**Authentication Parameters** (optional):
- `username`: MongoDB username for authentication
- `password`: MongoDB password for authentication
- `auth_source`: Authentication database (default: `"admin"`)

**Deployment Parameters**:
- `deployment_mode`: MongoDB deployment mode (default: `"single"`)
  - Valid values: `"single"`, `"replicaset"`, `"sharded"`
- `replica_set_name`: Replica set name (required if `deployment_mode` is `"replicaset"`)
- `enable_tls`: Enable TLS/SSL connection (default: `false`)

**Connection Pool Parameters** (optional):
- `max_pool_size`: Maximum number of connections in the pool (range: 1-1000)
  - Larger values support higher concurrency
- `min_pool_size`: Minimum number of connections in the pool
  - Must be less than or equal to `max_pool_size`

**Timeout Parameters**:
- `connect_timeout_secs`: Connection timeout in seconds (default: `10`)
  - Range: 1 to 300 seconds
  - Prevents hanging during connection establishment
- `server_selection_timeout_secs`: Server selection timeout in seconds (default: `30`)
  - Range: 1 to 300 seconds
  - Time to wait when selecting a server from the cluster
- `socket_timeout_secs`: Socket operation timeout in seconds (default: `60`)
  - Range: 1 to 600 seconds
  - Time to wait for socket operations to complete

**Performance Parameters**:
- `batch_size`: Number of records to insert in a single batch (default: `100`)
  - Range: 1 to 10,000
  - Larger values improve throughput but increase latency and memory usage
- `ordered_insert`: Whether to insert documents in order (default: `false`)
  - `false`: If one document fails, others can still be inserted (recommended for reliability)
  - `true`: Stops insertion on first failure (may cause data loss)
- `w`: Write concern level (default: `"1"`)
  - `"0"`: No acknowledgment (fastest, least reliable)
  - `"1"`: Acknowledgment from primary only (balanced)
  - `"majority"`: Acknowledgment from majority of replica set members (slowest, most reliable)
  - Numbers 2-10: Acknowledgment from specific number of nodes

**Configuration Examples**:

*Basic configuration (development)*:
```json
{
  "host": "localhost",
  "port": 27017,
  "database": "mqtt_data",
  "collection": "messages"
}
```

*Production configuration with replica set*:
```json
{
  "host": "mongo-primary.example.com",
  "port": 27017,
  "database": "mqtt_prod",
  "collection": "messages",
  "username": "mqtt_user",
  "password": "secure_password",
  "auth_source": "admin",
  "deployment_mode": "replicaset",
  "replica_set_name": "rs0",
  "enable_tls": true,
  "max_pool_size": 50,
  "min_pool_size": 10,
  "connect_timeout_secs": 10,
  "server_selection_timeout_secs": 30,
  "batch_size": 500,
  "ordered_insert": false,
  "w": "majority"
}
```

*High throughput configuration*:
```json
{
  "host": "mongodb-cluster.example.com",
  "database": "mqtt_logs",
  "collection": "messages",
  "batch_size": 1000,
  "ordered_insert": false,
  "w": "1",
  "max_pool_size": 100
}
```

*High reliability configuration*:
```json
{
  "host": "mongodb-cluster.example.com",
  "database": "mqtt_critical",
  "collection": "messages",
  "deployment_mode": "replicaset",
  "replica_set_name": "rs0",
  "batch_size": 100,
  "ordered_insert": false,
  "w": "majority",
  "connect_timeout_secs": 15,
  "server_selection_timeout_secs": 60
}
```

**Local File Connector**:
```json
{
  "connector_type": "file",
  "config": "{\"local_file_path\":\"/tmp/mqtt_messages.log\"}"
}
```

**Local File Connector (with Rotation Strategy)**:
```json
{
  "connector_type": "file",
  "config": "{\"local_file_path\":\"/var/log/mqtt/messages.log\",\"rotation_strategy\":\"daily\"}"
}
```

Configuration parameters:
- `local_file_path`: Required, file path
- `rotation_strategy`: Optional, file rotation strategy, values: `none` (default), `size`, `hourly`, `daily`
- `max_size_gb`: Optional, maximum file size in GB, only effective when `rotation_strategy` is `size`, range 1-10, default 1

**Elasticsearch Connector**:
```json
{
  "connector_type": "elasticsearch",
  "config": "{\"url\":\"http://localhost:9200\",\"index\":\"mqtt_messages\",\"auth_type\":\"basic\",\"username\":\"elastic\",\"password\":\"password123\"}"
}
```

Configuration parameters:
- `url`: Required, Elasticsearch server address
- `index`: Required, index name
- `auth_type`: Optional, authentication type, values: `none` (default), `basic`, `apikey`
- `username`: Optional, username (required for Basic auth)
- `password`: Optional, password (required for Basic auth)
- `api_key`: Optional, API key (required for ApiKey auth)
- `enable_tls`: Optional, enable TLS, default false
- `ca_cert_path`: Optional, CA certificate path
- `timeout_secs`: Optional, request timeout in seconds, range 1-300, default 30
- `max_retries`: Optional, maximum retry attempts, max 10, default 3

**Failure Handling Strategy (`failure_strategy`)**:

The `failure_strategy` parameter defines how the connector handles message delivery failures. It's an optional JSON string with the following strategies:

**1. Discard Strategy** (Default):
```json
{
  "failure_strategy": "{\"Discard\":{}}"
}
```
- Immediately discards failed messages
- No retry attempts
- Suitable for scenarios where message loss is acceptable

**2. Discard After Retry Strategy**:
```json
{
  "failure_strategy": "{\"DiscardAfterRetry\":{\"retry_total_times\":3,\"wait_time_ms\":1000}}"
}
```
- Retries delivery for specified number of times before discarding
- `retry_total_times`: Total number of retry attempts (required)
- `wait_time_ms`: Wait time in milliseconds between retries (required)
- Suitable for handling temporary network issues

**3. Dead Message Queue Strategy**:
```json
{
  "failure_strategy": "{\"DeadMessageQueue\":{\"topic_name\":\"dead_letter_queue\"}}"
}
```
- Sends failed messages to a designated dead letter queue topic
- `topic_name`: Name of the dead letter queue topic (required)
- Suitable for scenarios requiring message recovery and analysis
- Note: This feature is currently under development

**Example with failure strategy**:
```json
{
  "connector_name": "kafka_bridge",
  "connector_type": "kafka",
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
  "topic_name": "sensor/+",
  "failure_strategy": "{\"DiscardAfterRetry\":{\"retry_total_times\":5,\"wait_time_ms\":2000}}"
}
```

- **Response**: Returns "Created successfully!" on success

#### 9.4 Delete Connector
- **Endpoint**: `POST /api/mqtt/connector/delete`
- **Description**: Delete connector
- **Request Parameters**:
```json
{
  "connector_name": "old_connector"
}
```

- **Response**: Returns "Deleted successfully!" on success

---

### 10. Schema Management

#### 10.1 Schema List Query
- **Endpoint**: `POST /api/mqtt/schema/list`
- **Description**: Query Schema list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "name": "temperature_schema",
        "schema_type": "json",
        "desc": "Temperature sensor data schema",
        "schema": "{\"type\":\"object\",\"properties\":{\"temp\":{\"type\":\"number\"},\"unit\":{\"type\":\"string\"}}}"
      }
    ],
    "total_count": 12
  }
}
```

#### 10.2 Create Schema
- **Endpoint**: `POST /api/mqtt/schema/create`
- **Description**: Create new Schema
- **Request Parameters**:
```json
{
  "schema_name": "sensor_data_schema",   // Schema name
  "schema_type": "json",                 // Schema type: json, avro, protobuf
  "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\"},\"humidity\":{\"type\":\"number\"}}}",  // Schema definition
  "desc": "Sensor data validation schema"  // Description
}
```

- **Parameter Validation Rules**:
  - `schema_name`: Length must be between 1-128 characters
  - `schema_type`: Length must be between 1-50 characters, must be `json`, `avro`, or `protobuf`
  - `schema`: Length must be between 1-8192 characters
  - `desc`: Length cannot exceed 500 characters

**Schema Type Examples**：

**JSON Schema**:
```json
{
  "schema_type": "json",
  "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\",\"minimum\":-50,\"maximum\":100}}}"
}
```

**AVRO Schema**:
```json
{
  "schema_type": "avro", 
  "schema": "{\"type\":\"record\",\"name\":\"SensorData\",\"fields\":[{\"name\":\"temperature\",\"type\":\"double\"}]}"
}
```

- **Response**: Returns "Created successfully!" on success

#### 10.3 Delete Schema
- **Endpoint**: `POST /api/mqtt/schema/delete`
- **Description**: Delete Schema
- **Request Parameters**:
```json
{
  "schema_name": "old_schema"
}
```

- **Response**: Returns "Deleted successfully!" on success

#### 10.4 Schema Binding Management

##### 10.4.1 Schema Binding List Query
- **Endpoint**: `POST /api/mqtt/schema-bind/list`
- **Description**: Query Schema binding relationship list
- **Request Parameters**:
```json
{
  "resource_name": "sensor/temperature", // Optional, resource name filter
  "schema_name": "temp_schema",          // Optional, Schema name filter
  "limit": 20,
  "page": 1,
  "sort_field": "data_type",
  "sort_by": "asc",
  "filter_field": "data_type",
  "filter_values": ["resource"],
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
        "data_type": "resource",
        "data": ["sensor_data_schema", "device_status_schema"]
      }
    ],
    "total_count": 2
  }
}
```

##### 10.4.2 Create Schema Binding
- **Endpoint**: `POST /api/mqtt/schema-bind/create`
- **Description**: Create Schema binding relationship with resource
- **Request Parameters**:
```json
{
  "schema_name": "sensor_data_schema",  // Schema name
  "resource_name": "sensor/temperature" // Resource name (usually topic name)
}
```

- **Parameter Validation Rules**:
  - `schema_name`: Length must be between 1-128 characters
  - `resource_name`: Length must be between 1-256 characters

- **Response**: Returns "Created successfully!" on success

##### 10.4.3 Delete Schema Binding
- **Endpoint**: `POST /api/mqtt/schema-bind/delete`
- **Description**: Delete Schema binding relationship
- **Request Parameters**:
```json
{
  "schema_name": "sensor_data_schema",
  "resource_name": "sensor/temperature"
}
```

- **Response**: Returns "Deleted successfully!" on success

---

### 11. Message Management

#### 11.1 Send Message
- **Endpoint**: `POST /api/mqtt/message/send`
- **Description**: Send MQTT message to specified topic via HTTP API
- **Request Parameters**:
```json
{
  "topic": "sensor/temperature",  // Required, topic name
  "payload": "25.5",              // Required, message content
  "retain": false                 // Optional, whether to retain message, default false
}
```

- **Parameter Validation Rules**:
  - `topic`: Length must be between 1-256 characters
  - `payload`: Length cannot exceed 1MB (1048576 bytes)
  - `retain`: Boolean value

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "offsets": [12345]  // Offset list of messages in the topic
  }
}
```

**Field Descriptions**:
- `topic`: Target topic for message delivery
- `payload`: Message content (string format)
- `retain`: Whether to retain message
  - `true`: Message will be stored as retained message, new subscribers will receive it
  - `false`: Regular message, will not be retained
- `offsets`: Array of offsets returned after message is successfully written, indicating message position in storage

**Notes**:
- Messages are sent with QoS 1 (at least once) level
- Topic will be automatically created if it doesn't exist
- Default message expiry time is 3600 seconds (1 hour)
- Sender's client_id format: `{cluster_name}_{broker_id}`

#### 11.2 Read Messages
- **Endpoint**: `POST /api/mqtt/message/read`
- **Description**: Read messages from specified topic
- **Request Parameters**:
```json
{
  "topic": "sensor/temperature",  // Required, topic name
  "offset": 0                     // Required, starting offset
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
      },
      {
        "offset": 12346,
        "content": "26.0",
        "timestamp": 1640995260000
      }
    ]
  }
}
```

**Field Descriptions**:
- `topic`: Topic name to read messages from
- `offset`: Starting offset to begin reading messages
- `messages`: Message list (maximum 100 messages returned)
  - `offset`: Message offset
  - `content`: Message content (string format)
  - `timestamp`: Message timestamp (milliseconds)

**Notes**:
- Maximum of 100 messages returned per request
- Offset represents the sequential position of messages in the topic
- Empty message list will be returned if specified offset is out of range
- Timestamp is in millisecond Unix timestamp format

---

### 12. System Monitoring

#### 12.1 System Alarm List
- **Endpoint**: `POST /api/mqtt/system-alarm/list`
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
        "activate_at": "2024-01-01 10:00:00",
        "activated": true
      }
    ],
    "total_count": 3
  }
}
```

#### 12.2 Flapping Detection List
- **Endpoint**: `POST /api/mqtt/flapping_detect/list`
- **Description**: Query flapping detection list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "client_id": "flapping_client",
        "before_last_windows_connections": 15,
        "first_request_time": 1640995200
      }
    ],
    "total_count": 2
  }
}
```

---

## Enumeration Values

### ACL Resource Type (resource_type)
- `ClientId`: Client ID
- `Username`: Username
- `IpAddress`: IP Address

### ACL Action (action)
- `Publish`: Publish message
- `Subscribe`: Subscribe to topic
- `All`: All actions

### ACL Permission (permission)
- `Allow`: Allow
- `Deny`: Deny

### Blacklist Type (blacklist_type)
- `ClientId`: Client ID
- `IpAddress`: IP Address
- `Username`: Username

### Connector Type (connector_type)
- `kafka`: Apache Kafka message queue
- `pulsar`: Apache Pulsar message queue
- `rabbitmq`: RabbitMQ message queue
- `greptime`: GreptimeDB time-series database
- `postgres`: PostgreSQL relational database
- `mysql`: MySQL relational database
- `mongodb`: MongoDB NoSQL database
- `file`: Local file storage
- `elasticsearch`: Elasticsearch search engine

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
curl -X POST http://localhost:8080/api/mqtt/overview \
  -H "Content-Type: application/json" \
  -d '{}'
```

### Query Monitor Data
```bash
# Query connection count monitoring data
curl -X POST http://localhost:8080/api/mqtt/monitor/data \
  -H "Content-Type: application/json" \
  -d '{
    "data_type": "connection_num"
  }'

# Query message received count for a specific topic
curl -X POST http://localhost:8080/api/mqtt/monitor/data \
  -H "Content-Type: application/json" \
  -d '{
    "data_type": "topic_in_num",
    "topic_name": "sensor/temperature"
  }'

# Query subscription send success count
curl -X POST http://localhost:8080/api/mqtt/monitor/data \
  -H "Content-Type: application/json" \
  -d '{
    "data_type": "subscribe_send_success_num",
    "client_id": "client001",
    "path": "sensor/+"
  }'

# Query subscription topic send failure count
curl -X POST http://localhost:8080/api/mqtt/monitor/data \
  -H "Content-Type: application/json" \
  -d '{
    "data_type": "subscribe_topic_send_failure_num",
    "client_id": "client001",
    "path": "sensor/+",
    "topic_name": "sensor/temperature"
  }'

# Query session received message count
curl -X POST http://localhost:8080/api/mqtt/monitor/data \
  -H "Content-Type: application/json" \
  -d '{
    "data_type": "session_in_num",
    "client_id": "client001"
  }'

# Query session sent message count
curl -X POST http://localhost:8080/api/mqtt/monitor/data \
  -H "Content-Type: application/json" \
  -d '{
    "data_type": "session_out_num",
    "client_id": "client001"
  }'

# Query total successful messages sent by all connectors
curl -X POST http://localhost:8080/api/mqtt/monitor/data \
  -H "Content-Type: application/json" \
  -d '{
    "data_type": "connector_send_success_total"
  }'

# Query total failed messages sent by all connectors
curl -X POST http://localhost:8080/api/mqtt/monitor/data \
  -H "Content-Type: application/json" \
  -d '{
    "data_type": "connector_send_failure_total"
  }'

# Query successful messages sent by a specific connector
curl -X POST http://localhost:8080/api/mqtt/monitor/data \
  -H "Content-Type: application/json" \
  -d '{
    "data_type": "connector_send_success",
    "connector_name": "kafka_connector_01"
  }'

# Query failed messages sent by a specific connector
curl -X POST http://localhost:8080/api/mqtt/monitor/data \
  -H "Content-Type: application/json" \
  -d '{
    "data_type": "connector_send_failure",
    "connector_name": "kafka_connector_01"
  }'
```

### Query Client List
```bash
curl -X POST http://localhost:8080/api/mqtt/client/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1,
    "sort_field": "connection_id",
    "sort_by": "desc"
  }'
```

### Delete Topic
```bash
curl -X POST http://localhost:8080/api/mqtt/topic/delete \
  -H "Content-Type: application/json" \
  -d '{
    "topic_name": "sensor/temperature"
  }'
```

### Create User
```bash
curl -X POST http://localhost:8080/api/mqtt/user/create \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "testpass123",
    "is_superuser": false
  }'
```

### Create ACL Rule
```bash
curl -X POST http://localhost:8080/api/mqtt/acl/create \
  -H "Content-Type: application/json" \
  -d '{
    "resource_type": "ClientId",
    "resource_name": "sensor001",
    "topic": "sensor/+",
    "ip": "192.168.1.100",
    "action": "Publish",
    "permission": "Allow"
  }'
```

### Query Connector Detail
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/detail \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "kafka_bridge"
  }'
```

### Create Connector
```bash
# Create basic Kafka connector (uses default settings)
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "kafka_bridge",
    "connector_type": "kafka",
    "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
    "topic_name": "sensor/+"
  }'

# Create Kafka connector with advanced configuration
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "kafka_bridge_advanced",
    "connector_type": "kafka",
    "config": "{\"bootstrap_servers\":\"kafka1:9092,kafka2:9092,kafka3:9092\",\"topic\":\"mqtt_messages\",\"compression_type\":\"lz4\",\"batch_size\":32768,\"linger_ms\":10,\"acks\":\"all\",\"retries\":5}",
    "topic_name": "sensor/+"
  }'

# Create connector with retry failure strategy
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "kafka_bridge_retry",
    "connector_type": "kafka",
    "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
    "topic_name": "sensor/+",
    "failure_strategy": "{\"DiscardAfterRetry\":{\"retry_total_times\":5,\"wait_time_ms\":2000}}"
  }'
```

### Create Schema
```bash
curl -X POST http://localhost:8080/api/mqtt/schema/create \
  -H "Content-Type: application/json" \
  -d '{
    "schema_name": "sensor_schema",
    "schema_type": "json",
    "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\"},\"humidity\":{\"type\":\"number\"}}}",
    "desc": "Sensor data validation schema"
  }'
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

### Read Messages
```bash
curl -X POST http://localhost:8080/api/mqtt/message/read \
  -H "Content-Type: application/json" \
  -d '{
    "topic": "sensor/temperature",
    "offset": 0
  }'
```

---

*Documentation Version: v4.0*  
*Last Updated: 2025-09-20*  
*Based on Code Version: RobustMQ Admin Server v0.1.34*
