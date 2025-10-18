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
        "node_id": 1,
        "node_ip": "192.168.1.100",
        "node_inner_addr": "192.168.1.100:9981",
        "extend_info": "{}",
        "create_time": 1640995200
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
    "share_subscribe_leader_num": 300,
    "share_subscribe_resub_num": 200,
    "exclusive_subscribe_thread_num": 8,
    "share_subscribe_leader_thread_num": 4,
    "share_subscribe_follower_thread_num": 4
  }
}
```

#### 1.2 Monitor Data Query
- **Endpoint**: `POST /api/mqtt/monitor/data`
- **Description**: Get time series monitoring data for a specified metric type
- **Request Parameters**:
```json
{
  "data_type": "connection_num",      // Required, monitoring data type
  "topic_name": "sensor/temperature", // Optional, required for certain types
  "client_id": "client001",           // Optional, required for certain types
  "path": "sensor/+"                  // Optional, required for certain types
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
        "topic_name": "topic_001",
        "topic_name": "sensor/temperature",
        "is_contain_retain_message": true,
        "create_time": 1640995200
      }
    ],
    "total_count": 25
  }
}
```

**Response Field Description**:
- `topic_name`: Topic ID
- `topic_name`: Topic name
- `is_contain_retain_message`: Whether contains retained message
- `create_time`: Topic creation timestamp

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

#### 4.3 Topic Rewrite Rules List
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

#### 4.3 Create Topic Rewrite Rule
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

- **Response**: Returns "success" on success

#### 4.4 Delete Topic Rewrite Rule
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
    "share_sub": false,        // Whether this is a shared subscription
    "group_leader_info": null, // Shared subscription group leader info (only for shared subscriptions)
    "topic_list": [            // List of matched topics
      {
        "client_id": "client001",
        "path": "sensor/temperature",
        "topic_name": "sensor/temperature",
        "exclusive_push_data": {  // Exclusive subscription push data (null for shared subscriptions)
          "protocol": "MQTTv5",
          "client_id": "client001",
          "sub_path": "sensor/temperature",
          "rewrite_sub_path": null,
          "topic_name": "sensor/temperature",
          "group_name": null,
          "qos": "AtLeastOnce",
          "nolocal": false,
          "preserve_retain": true,
          "retain_forward_rule": "SendAtSubscribe",
          "subscription_identifier": null,
          "create_time": 1704067200000
        },
        "share_push_data": null,  // Shared subscription push data (null for exclusive subscriptions)
        "push_thread": {          // Push thread statistics (optional)
          "push_success_record_num": 1520,  // Successful push count
          "push_error_record_num": 3,       // Failed push count
          "last_push_time": 1704067800000,  // Last push time (millisecond timestamp)
          "last_run_time": 1704067810000,   // Last run time (millisecond timestamp)
          "create_time": 1704067200000      // Create time (millisecond timestamp)
        }
      }
    ]
  }
}
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
        "is_superuser": true
      }
    ],
    "total_count": 10
  }
}
```

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
        "connector_type": "Kafka",
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

#### 9.2 Create Connector
- **Endpoint**: `POST /api/mqtt/connector/create`
- **Description**: Create new connector
- **Request Parameters**:
```json
{
  "connector_name": "new_connector",   // Connector name
  "connector_type": "Kafka",           // Connector type
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",  // Configuration (JSON string)
  "topic_name": "sensor/+"               // Associated topic ID
}
```

**Connector Types and Configuration Examples**：

**Kafka Connector**:
```json
{
  "connector_type": "Kafka",
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\",\"acks\":\"all\"}"
}
```

**File Connector**:
```json
{
  "connector_type": "File",
  "config": "{\"path\":\"/tmp/mqtt_messages.log\",\"max_file_size\":\"100MB\"}"
}
```

- **Response**: Returns "Created successfully!" on success

#### 9.3 Delete Connector
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

### 11. System Monitoring

#### 11.1 System Alarm List
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

#### 11.2 Flapping Detection List
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
- `Kafka`: Kafka message queue
- `File`: Local file

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

### Create Connector
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "kafka_bridge",
    "connector_type": "Kafka",
    "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
    "topic_name": "sensor/+"
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

---

*Documentation Version: v4.0*  
*Last Updated: 2025-09-20*  
*Based on Code Version: RobustMQ Admin Server v0.1.34*
