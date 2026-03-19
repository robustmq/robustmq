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
- **Endpoint**: `GET /api/mqtt/client/list`
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
- **Endpoint**: `GET /api/mqtt/session/list`
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
- **Endpoint**: `GET /api/mqtt/topic/list`
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
- **Endpoint**: `GET /api/mqtt/topic/detail`
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
      "topic_id": "01J9K5FHQP8NWXYZ1234567890",
      "topic_name": "sensor/temperature",
      "storage_type": "Memory",
      "partition": 1,
      "replication": 1,
      "storage_name_list": [],
      "create_time": 1640995200
    },
    "retain_message": {
      "client_id": "client001",
      "dup": false,
      "qos": "AtLeastOnce",
      "pkid": 1,
      "retain": true,
      "topic": "sensor/temperature",
      "payload": "eyJ0ZW1wZXJhdHVyZSI6MjUuNX0=",
      "format_indicator": null,
      "expiry_interval": 0,
      "response_topic": null,
      "correlation_data": null,
      "user_properties": null,
      "subscription_identifiers": null,
      "content_type": null,
      "create_time": 1640995300
    },
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
    ],
    "storage_list": {
      "0": {
        "shard_uid": "shard_01J9K5FHQP8NWXYZ",
        "shard_name": "sensor_temperature_p0",
        "start_segment_seq": 0,
        "active_segment_seq": 1,
        "last_segment_seq": 1,
        "status": "Run",
        "config": {
          "replica_num": 1,
          "storage_type": "Memory",
          "max_segment_size": 104857600,
          "retention_sec": 86400
        },
        "create_time": 1640995200
      }
    }
  }
}
```

**Response Field Description**:

- **topic_info**: Complete topic information (Topic object)
  - `topic_id`: Topic unique identifier
  - `topic_name`: Topic name
  - `storage_type`: Storage type (e.g., Memory, RocksDB, etc.)
  - `partition`: Number of partitions
  - `replication`: Number of replicas
  - `storage_name_list`: List of storage names
  - `create_time`: Topic creation time (Unix timestamp in seconds)

- **retain_message**: Complete retained message object (MqttMessage or null)
  - `client_id`: Client ID that sent the message
  - `dup`: Whether this is a duplicate message
  - `qos`: QoS level
  - `pkid`: Packet identifier
  - `retain`: Whether this is a retained message
  - `topic`: Message topic
  - `payload`: Message content (Base64 encoded)
  - `format_indicator`: Format indicator (optional)
  - `expiry_interval`: Expiry interval in seconds
  - `response_topic`: Response topic (optional)
  - `correlation_data`: Correlation data (optional)
  - `user_properties`: User properties (optional)
  - `subscription_identifiers`: List of subscription identifiers (optional)
  - `content_type`: Content type (optional)
  - `create_time`: Message creation time (Unix timestamp in seconds)

- **retain_message_at**: Retained message timestamp
  - Type: `u64` or `null`
  - Unix timestamp in seconds
  - Returns `null` if there is no retained message

- **sub_list**: List of clients subscribed to this topic (HashSet)
  - `client_id`: Subscriber client ID
  - `path`: Subscription path (may include wildcards like `+` or `#`)

- **storage_list**: Storage shard mapping (HashMap<partition_number, EngineShard>)
  - Key: Partition number (u32)
  - Value: EngineShard object
    - `shard_uid`: Shard unique identifier
    - `shard_name`: Shard name
    - `start_segment_seq`: Start segment sequence number
    - `active_segment_seq`: Active segment sequence number
    - `last_segment_seq`: Last segment sequence number
    - `status`: Shard status (Run, PrepareDelete, Deleting)
    - `config`: Shard configuration
      - `replica_num`: Number of replicas
      - `storage_type`: Storage type
      - `max_segment_size`: Maximum segment size in bytes
      - `retention_sec`: Retention time in seconds
    - `create_time`: Shard creation time (Unix timestamp in seconds)

**Notes**:
- Returns an error response if the topic does not exist
- `sub_list` shows all subscriptions matching this topic, including wildcard subscriptions
- `storage_list` provides detailed storage engine shard information for each partition
- The `payload` field in retained message is Base64 encoded

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
- **Endpoint**: `GET /api/mqtt/topic-rewrite/list`
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
- **Endpoint**: `GET /api/mqtt/subscribe/list`
- **Description**: Query subscription list
- **Request Parameters**:
```json
{
  "tenant": "default",              // Optional, filter by tenant name; if omitted, returns subscriptions across all tenants
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

**Field Descriptions**:
- `tenant`: Tenant name the subscription belongs to
- `total_count`: Total number of subscriptions across all tenants (when `tenant` is not specified)

#### 5.2 Subscription Detail Query
- **Endpoint**: `GET /api/mqtt/subscribe/detail`
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
- **Endpoint**: `GET /api/mqtt/auto-subscribe/list`
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

- **Endpoint**: `GET /api/mqtt/slow-subscribe/list`
- **Description**: Query slow subscribe list, supports filtering by tenant
- **Request Parameters**:

```json
{
  "tenant": "default",              // Optional, filter by tenant (scans only that tenant's records)
  "limit": 20,
  "page": 1,
  "sort_field": "time_span",
  "sort_by": "desc",
  "filter_field": "client_id",
  "filter_values": ["slow_client"],
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
- **Endpoint**: `GET /api/mqtt/user/list`
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
- **Endpoint**: `GET /api/mqtt/acl/list`
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
- **Endpoint**: `GET /api/mqtt/blacklist/list`
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
  "blacklist_type": "ClientId",        // Blacklist type: ClientId, ClientIdMatch, Username, UserMatch, IpAddress, IPCIDR
  "resource_name": "bad_client",       // Resource name
  "end_time": 1735689599,              // End time (Unix timestamp)
  "desc": "Blocked for security"       // Description
}
```

- **Parameter Validation Rules**:
  - `blacklist_type`: Length must be between 1-50 characters, must be one of `ClientId`, `ClientIdMatch`, `Username`, `UserMatch`, `IpAddress`, `IPCIDR`
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

> Connector API documentation has been moved to a dedicated document. Please refer to [Connector Management HTTP API](Connector.md).

---

### 10. Schema Management

#### 10.1 Schema List Query
- **Endpoint**: `GET /api/mqtt/schema/list`
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
- **Endpoint**: `GET /api/mqtt/schema-bind/list`
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
        "activate_at": "2024-01-01 10:00:00",
        "activated": true
      }
    ],
    "total_count": 3
  }
}
```

#### 12.2 Flapping Detection List

- **Endpoint**: `GET /api/mqtt/flapping_detect/list`
- **Description**: Query flapping detection list, supports filtering by tenant
- **Request Parameters**:

```json
{
  "tenant": "default",              // Optional, filter by tenant (returns only that tenant's data)
  "limit": 20,
  "page": 1,
  "sort_field": "first_request_time",
  "sort_by": "desc",
  "filter_field": "client_id",
  "filter_values": ["flapping_client"],
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
  "tenant": "default",              // Optional, filter by tenant (scans only that tenant's records)
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

#### 13.1 Tenant List Query
- **Endpoint**: `GET /api/mqtt/tenant/list`
- **Description**: Query MQTT tenant list
- **Request Parameters**:
```json
{
  "tenant_name": "default",         // Optional, filter by exact tenant name
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
        "max_mqtt_qos2_num": 1000,
        "max_publish_rate": 10000
      }
    ],
    "total_count": 1
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
- `max_mqtt_qos2_num`: Maximum concurrent QoS 2 messages
- `max_publish_rate`: Maximum publish message rate per second

#### 13.2 Create Tenant
- **Endpoint**: `POST /api/mqtt/tenant/create`
- **Description**: Create new MQTT tenant
- **Request Parameters**:
```json
{
  "tenant_name": "new_tenant",                        // Required, tenant name, length 1-128 characters
  "desc": "My new tenant",                            // Optional, description, length cannot exceed 500 characters
  "max_connections_per_node": 1000000,                // Optional, maximum connections per node
  "max_create_connection_rate_per_second": 10000,     // Optional, maximum new connection rate per second
  "max_topics": 500000,                               // Optional, maximum number of topics
  "max_sessions": 5000000,                            // Optional, maximum number of sessions
  "max_mqtt_qos2_num": 1000,                          // Optional, maximum concurrent QoS 2 messages
  "max_publish_rate": 10000                           // Optional, maximum publish message rate per second
}
```

- **Parameter Validation Rules**:
  - `tenant_name`: Length must be between 1-128 characters
  - `desc`: Length cannot exceed 500 characters

- **Response**: Returns `"success"` on success

#### 13.3 Delete Tenant
- **Endpoint**: `POST /api/mqtt/tenant/delete`
- **Description**: Delete MQTT tenant
- **Request Parameters**:
```json
{
  "tenant_name": "old_tenant"       // Tenant name, length 1-128 characters
}
```

- **Response**: Returns `"success"` on success

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

| Value | Description |
|-------|-------------|
| `ClientId` | Exact match by client ID |
| `ClientIdMatch` | Wildcard/regex match by client ID |
| `Username` | Exact match by username |
| `UserMatch` | Wildcard/regex match by username |
| `IpAddress` | Exact match by IP address |
| `IPCIDR` | CIDR range match by IP address |

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

### Query Monitor Data
```bash
# Query connection count monitoring data
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=connection_num"

# Query message received count for a specific topic
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=topic_in_num&topic_name=sensor/temperature"

# Query subscription send success count
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=subscribe_send_success_num&client_id=client001&path=sensor/%2B"

# Query subscription topic send failure count
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=subscribe_topic_send_failure_num&client_id=client001&path=sensor/%2B&topic_name=sensor/temperature"

# Query session received message count
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=session_in_num&client_id=client001"

# Query session sent message count
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=session_out_num&client_id=client001"

# Query total successful messages sent by all connectors
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=connector_send_success_total"

# Query total failed messages sent by all connectors
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=connector_send_failure_total"

# Query successful messages sent by a specific connector
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=connector_send_success&connector_name=kafka_connector_01"

# Query failed messages sent by a specific connector
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=connector_send_failure&connector_name=kafka_connector_01"
```

### Query Client List
```bash
curl "http://localhost:8080/api/mqtt/client/list?limit=10&page=1&sort_field=connection_id&sort_by=desc"
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
curl "http://localhost:8080/api/mqtt/connector/detail?connector_name=kafka_bridge"
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

*Documentation Version: v5.0*
*Last Updated: 2026-03-15*
*Based on Code Version: RobustMQ Admin Server v0.3.2*
