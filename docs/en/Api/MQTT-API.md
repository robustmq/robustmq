# RobustMQ Admin Server HTTP API Documentation

## Overview

RobustMQ Admin Server is an HTTP management interface service built on the Axum framework, providing comprehensive management capabilities for MQTT clusters.

- **Base URL**: `http://localhost:{port}`
- **Request Method**: Primarily uses `POST` method
- **Data Format**: JSON
- **Response Format**: JSON

## Common Response Format

### Success Response
```json
{
  "code": 200,
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
  "code": 200,
  "message": "success",
  "data": {
    "data": [...],
    "total_count": 100
  }
}
```

## Common Request Parameters

Most list query interfaces support the following common parameters:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page_num` | `u32` | No | Page number, starting from 1 |
| `page` | `u32` | No | Page size |
| `sort_field` | `string` | No | Sort field |
| `sort_by` | `string` | No | Sort order: asc/desc |
| `filter_field` | `string` | No | Filter field |
| `filter_values` | `array` | No | Filter value list |
| `exact_match` | `string` | No | Exact match |

---

## API Interface List

### 1. Basic Interfaces

#### 1.1 Service Status Query
- **Interface**: `GET /`
- **Description**: Get service version information
- **Request Parameters**: None
- **Response Example**:
```json
"RobustMQ v0.1.0"
```

---

### 2. MQTT Cluster Overview

#### 2.1 Cluster Overview Information
- **Interface**: `POST /mqtt/overview`
- **Description**: Get MQTT cluster overview information
- **Request Parameters**: Empty request body (empty JSON object `{}`)
- **Response Data Structure**:
```json
{
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
```

#### 2.2 Cluster Monitoring Metrics
- **Interface**: `POST /mqtt/overview/metrics`
- **Description**: Get cluster monitoring metrics for a specified time range
- **Request Parameters**:
```json
{
  "start_time": 1640995200,  // Unix timestamp, defaults to 1 hour ago if 0
  "end_time": 1640998800     // Unix timestamp, defaults to current time if 0
}
```
- **Response Data Structure**:
```json
{
  "connection_num": "[{\"timestamp\":1640995200,\"value\":1500}]",
  "topic_num": "[{\"timestamp\":1640995200,\"value\":50}]",
  "subscribe_num": "[{\"timestamp\":1640995200,\"value\":2000}]",
  "message_in_num": "[{\"timestamp\":1640995200,\"value\":10000}]",
  "message_out_num": "[{\"timestamp\":1640995200,\"value\":8500}]",
  "message_drop_num": "[{\"timestamp\":1640995200,\"value\":15}]"
}
```

---

### 3. Client Management

#### 3.1 Client List Query
- **Interface**: `POST /mqtt/client/list`
- **Description**: Query the list of clients connected to the cluster
- **Request Parameters**:
```json
{
  "source_ip": "192.168.1.1",      // Optional, filter by source IP
  "connection_id": 12345,           // Optional, filter by connection ID
  "page_num": 1,                    // Optional, page number
  "page": 20,                       // Optional, page size
  "sort_field": "connection_id",    // Optional, sort field: connection_id, connection_type, protocol, source_addr
  "sort_by": "desc",                // Optional, sort order
  "filter_field": "protocol",       // Optional, filter field
  "filter_values": ["MQTT"],        // Optional, filter values
  "exact_match": "true"             // Optional, exact match
}
```
- **Response Data Structure**:
```json
{
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
```

---

### 4. Session Management

#### 4.1 Session List Query
- **Interface**: `POST /mqtt/session/list`
- **Description**: Query MQTT session list
- **Request Parameters**:
```json
{
  "client_id": "client001",         // Optional, filter by client ID
  "page_num": 1,
  "page": 20,
  "sort_field": "create_time",      // Optional, sort field: client_id
  "sort_by": "desc",
  "filter_field": "client_id",
  "filter_values": ["client001"],
  "exact_match": "false"
}
```
- **Response Data Structure**:
```json
{
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
```

---

### 5. Topic Management

#### 5.1 Topic List Query
- **Interface**: `POST /mqtt/topic/list`
- **Description**: Query MQTT topic list
- **Request Parameters**:
```json
{
  "topic_name": "sensor/+",         // Optional, filter by topic name
  "page_num": 1,
  "page": 20,
  "sort_field": "topic_name",       // Optional, sort field: topic_name
  "sort_by": "asc",
  "filter_field": "topic_name",
  "filter_values": ["sensor"],
  "exact_match": "false"
}
```
- **Response Data Structure**:
```json
{
  "data": [
    {
      "topic_id": "topic_001",
      "topic_name": "sensor/temperature",
      "is_contain_retain_message": true
    }
  ],
  "total_count": 25
}
```

#### 5.2 Topic Rewrite Rules List
- **Interface**: `POST /mqtt/topic-rewrite/list`
- **Description**: Query topic rewrite rules list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
  "data": [
    {
      "source_topic": "old/topic",
      "dest_topic": "new/topic", 
      "regex": "^old/(.*)$",
      "action": "publish"
    }
  ],
  "total_count": 10
}
```

#### 5.3 Create Topic Rewrite Rule
- **Interface**: `POST /mqtt/topic-rewrite/create`
- **Description**: Create a new topic rewrite rule
- **Request Parameters**:
```json
{
  "action": "publish",              // Action type
  "source_topic": "old/topic",      // Source topic
  "dest_topic": "new/topic",        // Destination topic
  "regex": "^old/(.*)$"             // Regular expression
}
```
- **Response**: Returns "success" on success

#### 5.4 Delete Topic Rewrite Rule
- **Interface**: `POST /mqtt/topic-rewrite/delete`
- **Description**: Delete topic rewrite rule
- **Request Parameters**:
```json
{
  "action": "publish",
  "source_topic": "old/topic"
}
```
- **Response**: Returns "success" on success

---

### 6. Subscription Management

#### 6.1 Subscription List Query
- **Interface**: `POST /mqtt/subscribe/list`
- **Description**: Query subscription list
- **Request Parameters**:
```json
{
  "client_id": "client001",         // Optional, filter by client ID
  "page_num": 1,
  "page": 20,
  "sort_field": "create_time",
  "sort_by": "desc",
  "filter_field": "client_id",      // Optional, filter field: client_id
  "filter_values": ["client001"],
  "exact_match": "false"
}
```
- **Response Data Structure**:
```json
{
  "data": [
    {
      "client_id": "client001",
      "path": "sensor/+",
      "broker_id": 1,
      "protocol": "MQTTv4",
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
```

#### 6.2 Subscription Detail Query
- **Interface**: `POST /mqtt/subscribe/detail`
- **Description**: Query subscription details
- **Request Parameters**:
```json
{}
```
- **Response**: Currently returns empty string (feature pending implementation)

#### 6.3 Auto Subscribe List
- **Interface**: `POST /mqtt/auto-subscribe/list`
- **Description**: Query auto subscription rules list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
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
```

#### 6.4 Create Auto Subscribe Rule
- **Interface**: `POST /mqtt/auto-subscribe/create`
- **Description**: Create a new auto subscription rule
- **Request Parameters**:
```json
{
  "topic": "system/+",              // Topic pattern
  "qos": 1,                         // QoS level: 0, 1, 2
  "no_local": false,                // No local flag
  "retain_as_published": false,     // Retain as published
  "retained_handling": 0            // Retained message handling: 0, 1, 2
}
```
- **Response**: Returns "success" on success

#### 6.5 Delete Auto Subscribe Rule
- **Interface**: `POST /mqtt/auto-subscribe/delete`
- **Description**: Delete auto subscription rule
- **Request Parameters**:
```json
{
  "topic_name": "system/+"
}
```
- **Response**: Returns "success" on success

#### 6.6 Slow Subscribe List
- **Interface**: `POST /mqtt/slow-subscribe/list`
- **Description**: Query slow subscription list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
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
```

---

### 7. User Management

#### 7.1 User List Query
- **Interface**: `POST /mqtt/user/list`
- **Description**: Query MQTT user list
- **Request Parameters**:
```json
{
  "user_name": "admin",             // Optional, filter by username
  "page_num": 1,
  "page": 20,
  "sort_field": "username",         // Optional, sort field: username
  "sort_by": "asc",
  "filter_field": "username",
  "filter_values": ["admin"],
  "exact_match": "false"
}
```
- **Response Data Structure**:
```json
{
  "data": [
    {
      "username": "admin",
      "is_superuser": true
    }
  ],
  "total_count": 10
}
```

#### 7.2 Create User
- **Interface**: `POST /mqtt/user/create`
- **Description**: Create a new MQTT user
- **Request Parameters**:
```json
{
  "username": "newuser",            // Username
  "password": "password123",        // Password
  "is_superuser": false             // Is superuser
}
```
- **Response**: Returns "success" on success

#### 7.3 Delete User
- **Interface**: `POST /mqtt/user/delete`
- **Description**: Delete MQTT user
- **Request Parameters**:
```json
{
  "username": "olduser"
}
```
- **Response**: Returns "success" on success

---

### 8. ACL Management

#### 8.1 ACL List Query
- **Interface**: `POST /mqtt/acl/list`
- **Description**: Query access control list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
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
```

#### 8.2 Create ACL Rule
- **Interface**: `POST /mqtt/acl/create`
- **Description**: Create a new ACL rule
- **Request Parameters**:
```json
{
  "resource_type": "ClientId",       // Resource type: ClientId, Username, IpAddress, All
  "resource_name": "client001",      // Resource name
  "topic": "sensor/+",               // Topic pattern
  "ip": "192.168.1.100",             // IP address
  "action": "Publish",               // Action: Publish, Subscribe, All
  "permission": "Allow"              // Permission: Allow, Deny
}
```
- **Response**: Returns "success" on success

#### 8.3 Delete ACL Rule
- **Interface**: `POST /mqtt/acl/delete`
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
- **Response**: Returns "success" on success

---

### 9. Blacklist Management

#### 9.1 Blacklist List Query
- **Interface**: `POST /mqtt/blacklist/list`
- **Description**: Query blacklist
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
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
```

#### 9.2 Create Blacklist
- **Interface**: `POST /mqtt/blacklist/create`
- **Description**: Add new blacklist item
- **Request Parameters**:
```json
{
  "blacklist_type": "ClientId",        // Blacklist type: ClientId, IpAddress, Username
  "resource_name": "bad_client",       // Resource name
  "end_time": 1735689599,              // End time (Unix timestamp)
  "desc": "Blocked for security"       // Description
}
```
- **Response**: Returns "success" on success

#### 9.3 Delete Blacklist
- **Interface**: `POST /mqtt/blacklist/delete`
- **Description**: Delete blacklist item
- **Request Parameters**:
```json
{
  "blacklist_type": "ClientId",
  "resource_name": "bad_client"
}
```
- **Response**: Returns "success" on success

---

### 10. Connector Management

#### 10.1 Connector List Query
- **Interface**: `POST /mqtt/connector/list`
- **Description**: Query connector list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
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
```

#### 10.2 Create Connector
- **Interface**: `POST /mqtt/connector/create`
- **Description**: Create a new connector
- **Request Parameters**:
```json
{
  "connector_name": "new_connector",   // Connector name
  "connector_type": "Kafka",           // Connector type: LocalFile, Kafka, GreptimeDB
  "config": "{\"path\":\"/tmp/mqtt.log\"}",  // Configuration (JSON string)
  "topic_id": "topic_001"              // Associated topic ID
}
```

**Connector Configuration Examples**:

**LocalFile Connector**:
```json
{
  "connector_type": "LocalFile",
  "config": "{\"path\":\"/tmp/mqtt_messages.log\",\"max_file_size\":\"100MB\"}"
}
```

**Kafka Connector**:
```json
{
  "connector_type": "Kafka",
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\",\"acks\":\"all\"}"
}
```

**GreptimeDB Connector**:
```json
{
  "connector_type": "GreptimeDB",
  "config": "{\"host\":\"localhost\",\"port\":4001,\"database\":\"mqtt_data\",\"table\":\"messages\"}"
}
```

- **Response**: Returns "success" on success

#### 10.3 Delete Connector
- **Interface**: `POST /mqtt/connector/delete`
- **Description**: Delete connector
- **Request Parameters**:
```json
{
  "connector_name": "old_connector"
}
```
- **Response**: Returns "success" on success

---

### 11. Schema Management

#### 11.1 Schema List Query
- **Interface**: `POST /mqtt/schema/list`
- **Description**: Query schema list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
  "data": [
    {
      "name": "temperature_schema",
      "schema_type": "JSON",
      "desc": "Temperature sensor data schema",
      "schema": "{\"type\":\"object\",\"properties\":{\"temp\":{\"type\":\"number\"},\"unit\":{\"type\":\"string\"}}}"
    }
  ],
  "total_count": 12
}
```

#### 11.2 Create Schema
- **Interface**: `POST /mqtt/schema/create`
- **Description**: Create a new schema
- **Request Parameters**:
```json
{
  "schema_name": "sensor_data_schema",   // Schema name
  "schema_type": "json",                 // Schema type: json, avro, protobuf
  "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\"},\"humidity\":{\"type\":\"number\"}}}",  // Schema definition
  "desc": "Sensor data validation schema"  // Description
}
```

**Schema Type Examples**:

**JSON Schema**:
```json
{
  "schema_type": "json",
  "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\",\"minimum\":-50,\"maximum\":100},\"humidity\":{\"type\":\"number\",\"minimum\":0,\"maximum\":100}}}"
}
```

**AVRO Schema**:
```json
{
  "schema_type": "avro",
  "schema": "{\"type\":\"record\",\"name\":\"SensorData\",\"fields\":[{\"name\":\"temperature\",\"type\":\"double\"},{\"name\":\"humidity\",\"type\":\"double\"}]}"
}
```

**Protobuf Schema**:
```json
{
  "schema_type": "protobuf",
  "schema": "syntax = \"proto3\"; message SensorData { double temperature = 1; double humidity = 2; }"
}
```

- **Response**: Returns "success" on success

#### 11.3 Delete Schema
- **Interface**: `POST /mqtt/schema/delete`
- **Description**: Delete schema
- **Request Parameters**:
```json
{
  "schema_name": "old_schema"
}
```
- **Response**: Returns "success" on success

#### 11.4 Schema Binding List Query
- **Interface**: `POST /mqtt/schema-bind/list`
- **Description**: Query schema binding relationship list
- **Request Parameters**:
```json
{
  "resource_name": "topic001",         // Optional, filter by resource name
  "schema_name": "temp_schema",         // Optional, filter by schema name
  "page_num": 1,
  "page": 20,
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
  "data": [
    {
      "data_type": "resource",
      "data": ["sensor_data_schema", "device_status_schema"]
    },
    {
      "data_type": "schema",
      "data": ["sensor/temperature", "sensor/humidity", "device/status"]
    }
  ],
  "total_count": 2
}
```

#### 11.5 Create Schema Binding
- **Interface**: `POST /mqtt/schema-bind/create`
- **Description**: Create schema-resource binding relationship
- **Request Parameters**:
```json
{
  "schema_name": "sensor_data_schema",  // Schema name
  "resource_name": "sensor/temperature" // Resource name (usually topic name)
}
```
- **Response**: Returns "success" on success

#### 11.6 Delete Schema Binding
- **Interface**: `POST /mqtt/schema-bind/delete`
- **Description**: Delete schema binding relationship
- **Request Parameters**:
```json
{
  "schema_name": "sensor_data_schema",
  "resource_name": "sensor/temperature"
}
```
- **Response**: Returns "success" on success

---

### 12. System Management

#### 12.1 System Alarm List
- **Interface**: `POST /mqtt/system-alarm/list`
- **Description**: Query system alarm list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
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
```

#### 12.2 Cluster Configuration Set
- **Interface**: `POST /mqtt/cluster-config/set`
- **Description**: Set cluster configuration
- **Request Parameters**:
```json
{
  "config_type": "SlowSubscribe",      // Configuration type: SlowSubscribe, OfflineMessage
  "config": "{\"enable\":true,\"threshold\":1000}" // Configuration content (JSON string)
}
```
- **Response**: Returns "success" on success
- **Note**: This interface functionality is pending completion in the current implementation

#### 12.3 Flapping Detection List
- **Interface**: `POST /mqtt/flapping_detect/list`
- **Description**: Query connection flapping detection list
- **Request Parameters**: Supports common pagination and filtering parameters
- **Response Data Structure**:
```json
{
  "data": [
    {
      "client_id": "flapping_client",
      "before_last_windows_connections": 15,
      "first_request_time": 1640995200
    }
  ],
  "total_count": 2
}
```

---

## Error Codes

| Error Code | Description |
|------------|-------------|
| 200 | Request successful |
| 400 | Bad request parameters |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Resource not found |
| 500 | Internal server error |

---

## Enumeration Values

### ACL Resource Type (resource_type)
- `ClientId`: Client ID
- `Username`: Username
- `IpAddress`: IP Address
- `All`: All resources

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
- `LocalFile`: Local file
- `Kafka`: Kafka message queue
- `GreptimeDB`: GreptimeDB time-series database

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
- `2`: Do not send retained messages

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
    "page_num": 1,
    "page": 10,
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

## Important Notes

1. **Request Method**: Except for the root path `/` which uses GET method, all other interfaces use POST method
2. **Request Body**: Even for query operations, a JSON format request body is required
3. **Time Format**: 
   - Input time uses Unix timestamp (seconds)
   - Output time uses local time format string "YYYY-MM-DD HH:MM:SS"
4. **Pagination**: Page number `page_num` starts from 1
5. **Configuration Validation**: Configuration format is validated when creating connectors
6. **Schema Validation**: Schema syntax is validated when creating schemas
7. **Access Control**: It is recommended to add appropriate authentication and authorization mechanisms in production environments
8. **Error Handling**: All errors return detailed error information for debugging

---

*Document Version: v2.0*  
*Last Updated: 2024-01-01*  
*Based on Code Version: RobustMQ Admin Server*
