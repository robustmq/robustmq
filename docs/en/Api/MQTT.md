# MQTT Broker Management HTTP API

> This document describes all HTTP API interfaces related to MQTT Broker. For general information, please refer to [COMMON.md](COMMON.md).

## API Interface List

### 1. Cluster Overview

#### 1.1 Cluster Overview Information
- **Endpoint**: `POST /mqtt/overview`
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

#### 1.2 Cluster Monitoring Metrics
- **Endpoint**: `POST /mqtt/overview/metrics`
- **Description**: Get cluster monitoring metrics for specified time range
- **Request Parameters**:
```json
{
  "start_time": 1640995200,  // Unix timestamp, start time
  "end_time": 1640998800     // Unix timestamp, end time
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "connection_num": "[{\"timestamp\":1640995200,\"value\":1500}]",
    "topic_num": "[{\"timestamp\":1640995200,\"value\":50}]", 
    "subscribe_num": "[{\"timestamp\":1640995200,\"value\":2000}]",
    "message_in_num": "[{\"timestamp\":1640995200,\"value\":10000}]",
    "message_out_num": "[{\"timestamp\":1640995200,\"value\":8500}]",
    "message_drop_num": "[{\"timestamp\":1640995200,\"value\":15}]"
  }
}
```

---

### 2. Client Management

#### 2.1 Client List Query
- **Endpoint**: `POST /mqtt/client/list`
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
  "filter_field": "protocol",       // Optional, filter field
  "filter_values": ["MQTT"],        // Optional, filter values
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
        "connection_id": 12345,
        "connection_type": "TCP",
        "protocol": "MQTT",
        "source_addr": "192.168.1.100:52341",
        "create_time": "2024-01-01 10:00:00"
      }
    ],
    "total_count": 100
  }
}
```

---

### 3. Session Management

#### 3.1 Session List Query
- **Endpoint**: `POST /mqtt/session/list`
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
        "distinct_time": 1640995400
      }
    ],
    "total_count": 50
  }
}
```

---

### 4. Topic Management

#### 4.1 Topic List Query
- **Endpoint**: `POST /mqtt/topic/list`
- **Description**: Query MQTT topic list
- **Request Parameters**:
```json
{
  "topic_name": "sensor/+",         // Optional, filter by topic name
  "limit": 20,
  "page": 1,
  "sort_field": "topic_name",       // Optional, sort field
  "sort_by": "asc",
  "filter_field": "topic_name",
  "filter_values": ["sensor"],
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
        "topic_id": "topic_001",
        "topic_name": "sensor/temperature",
        "is_contain_retain_message": true
      }
    ],
    "total_count": 25
  }
}
```

#### 4.2 Topic Rewrite Rules List
- **Endpoint**: `POST /mqtt/topic-rewrite/list`
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
- **Endpoint**: `POST /mqtt/topic-rewrite/create`
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
- **Endpoint**: `POST /mqtt/topic-rewrite/delete`
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
- **Endpoint**: `POST /mqtt/subscribe/list`
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
- **Endpoint**: `POST /mqtt/subscribe/detail`
- **Description**: Query subscription details
- **Request Parameters**: Empty JSON object
- **Response**: Currently returns empty string (feature to be implemented)

#### 5.3 Auto Subscribe Rule Management

##### 5.3.1 Auto Subscribe List
- **Endpoint**: `POST /mqtt/auto-subscribe/list`
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
- **Endpoint**: `POST /mqtt/auto-subscribe/create`
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
- **Endpoint**: `POST /mqtt/auto-subscribe/delete`
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
- **Endpoint**: `POST /mqtt/slow-subscribe/list`
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
- **Endpoint**: `POST /mqtt/user/list`
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
- **Endpoint**: `POST /mqtt/user/create`
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
- **Endpoint**: `POST /mqtt/user/delete`
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
- **Endpoint**: `POST /mqtt/acl/list`
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
- **Endpoint**: `POST /mqtt/acl/create`
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
- **Endpoint**: `POST /mqtt/acl/delete`
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
- **Endpoint**: `POST /mqtt/blacklist/list`
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
- **Endpoint**: `POST /mqtt/blacklist/create`
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
- **Endpoint**: `POST /mqtt/blacklist/delete`
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
- **Endpoint**: `POST /mqtt/connector/list`
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
        "topic_id": "topic_001",
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
- **Endpoint**: `POST /mqtt/connector/create`
- **Description**: Create new connector
- **Request Parameters**:
```json
{
  "connector_name": "new_connector",   // Connector name
  "connector_type": "Kafka",           // Connector type
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",  // Configuration (JSON string)
  "topic_id": "sensor/+"               // Associated topic ID
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
- **Endpoint**: `POST /mqtt/connector/delete`
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
- **Endpoint**: `POST /mqtt/schema/list`
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
- **Endpoint**: `POST /mqtt/schema/create`
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
- **Endpoint**: `POST /mqtt/schema/delete`
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
- **Endpoint**: `POST /mqtt/schema-bind/list`
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
- **Endpoint**: `POST /mqtt/schema-bind/create`
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
- **Endpoint**: `POST /mqtt/schema-bind/delete`
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
- **Endpoint**: `POST /mqtt/system-alarm/list`
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
- **Endpoint**: `POST /mqtt/flapping_detect/list`
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
curl -X POST http://localhost:8080/mqtt/overview \
  -H "Content-Type: application/json" \
  -d '{}'
```

### Query Client List
```bash
curl -X POST http://localhost:8080/mqtt/client/list \
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
curl -X POST http://localhost:8080/mqtt/user/create \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "testpass123",
    "is_superuser": false
  }'
```

### Create ACL Rule
```bash
curl -X POST http://localhost:8080/mqtt/acl/create \
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
curl -X POST http://localhost:8080/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "kafka_bridge",
    "connector_type": "Kafka",
    "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
    "topic_id": "sensor/+"
  }'
```

### Create Schema
```bash
curl -X POST http://localhost:8080/mqtt/schema/create \
  -H "Content-Type: application/json" \
  -d '{
    "schema_name": "sensor_schema",
    "schema_type": "json",
    "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\"},\"humidity\":{\"type\":\"number\"}}}",
    "desc": "Sensor data validation schema"
  }'
```

---

*Documentation Version: v3.0*  
*Last Updated: 2024-01-01*  
*Based on Code Version: RobustMQ Admin Server v0.1.31*
