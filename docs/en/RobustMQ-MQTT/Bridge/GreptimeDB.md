# GreptimeDB Connector

## Overview

The GreptimeDB connector is a data integration component provided by RobustMQ for bridging MQTT messages to GreptimeDB time-series database systems. This connector supports high-concurrency time-series data writing, suitable for scenarios such as IoT data storage, real-time monitoring, time-series analysis, and data visualization.

## Configuration

### Connector Configuration

The GreptimeDB connector uses the `GreptimeDBConnectorConfig` structure for configuration:

```rust
pub struct GreptimeDBConnectorConfig {
    pub server_addr: String,        // GreptimeDB server address
    pub database: String,           // Database name
    pub user: String,               // Username
    pub password: String,           // Password
    pub precision: TimePrecision,   // Time precision
}
```

### Configuration Parameters

| Parameter | Type | Required | Description | Example |
|-----------|------|----------|-------------|---------|
| `server_addr` | String | Yes | GreptimeDB server address | `localhost:4000` |
| `database` | String | Yes | Database name | `public` |
| `user` | String | Yes | Username | `greptime_user` |
| `password` | String | Yes | Password | `greptime_pwd` |
| `precision` | String | No | Time precision, default is second | `s`, `ms`, `us`, `ns` |

### Time Precision Description

| Precision Value | Description | Example |
|-----------------|-------------|---------|
| `s` | Second precision | `1640995200` |
| `ms` | Millisecond precision | `1640995200000` |
| `us` | Microsecond precision | `1640995200000000` |
| `ns` | Nanosecond precision | `1640995200000000000` |

### Configuration Examples

#### JSON Configuration Format
```json
{
  "server_addr": "localhost:4000",
  "database": "public",
  "user": "greptime_user",
  "password": "greptime_pwd",
  "precision": "s"
}
```

#### Complete Connector Configuration
```json
{
  "cluster_name": "default",
  "connector_name": "greptimedb_connector_01",
  "connector_type": "GreptimeDB",
  "config": "{\"server_addr\": \"localhost:4000\", \"database\": \"public\", \"user\": \"greptime_user\", \"password\": \"greptime_pwd\", \"precision\": \"s\"}",
  "topic_id": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## Message Format

### Transmission Format
The GreptimeDB connector converts MQTT messages to InfluxDB Line Protocol format and sends them to GreptimeDB, supporting structured storage of time-series data.

### Message Structure

```json
{
  "topic": "sensor/temperature",
  "qos": 1,
  "retain": false,
  "payload": "eyJ0ZW1wZXJhdHVyZSI6IDI1LjUsICJodW1pZGl0eSI6IDYwfQ==",
  "client_id": "sensor_001",
  "username": "sensor_user",
  "timestamp": 1640995200,
  "message_id": 12345,
  "header": [
    {
      "key": "content-type",
      "value": "application/json"
    }
  ],
  "key": "sensor_001",
  "data": "eyJ0ZW1wZXJhdHVyZSI6IDI1LjUsICJodW1pZGl0eSI6IDYwfQ==",
  "tags": ["sensor", "temperature"],
  "timestamp": 1640995200,
  "crc_num": 1234567890
}
```

### InfluxDB Line Protocol Format

The GreptimeDB connector converts messages to the following format:

```
measurement,tag1=value1,tag2=value2 field1=value1,field2=value2 timestamp
```

#### Conversion Example

**Input Message**:
```json
{
  "key": "sensor_001",
  "header": [{"name": "device_type", "value": "temperature"}],
  "data": {"temperature": 25.5, "humidity": 60},
  "tags": ["sensor", "iot"],
  "timestamp": 1640995200,
  "crc_num": 1234567890,
  "offset": 100
}
```

**Converted Line Protocol**:
```
sensor_001,device_type=temperature data="{\"temperature\":25.5,\"humidity\":60}",tags="[\"sensor\",\"iot\"]",crc_num=1234567890i,offset=100i 1640995200
```

### Field Description

| Field | Type | Description |
|-------|------|-------------|
| `measurement` | String | Measurement name (from `key` field) |
| `tags` | String | Tags (from `header` and `tags` fields) |
| `fields` | String | Fields (including `data`, `tags`, `crc_num`, `offset`) |
| `timestamp` | Number | Timestamp |

## Creating GreptimeDB Connectors with robust-ctl

### Basic Syntax

Use the `robust-ctl` command-line tool to easily create and manage GreptimeDB connectors:

```bash
robust-ctl mqtt connector create \
  --connector-name <connector_name> \
  --connector-type <connector_type> \
  --config <config> \
  --topic-id <topic_id>
```

### Creating GreptimeDB Connectors

#### 1. Basic Create Command

```bash
# Create GreptimeDB connector
robust-ctl mqtt connector create \
  --connector-name "greptimedb_connector_01" \
  --connector-type "GreptimeDB" \
  --config '{"server_addr": "localhost:4000", "database": "public", "user": "greptime_user", "password": "greptime_pwd", "precision": "s"}' \
  --topic-id "sensor/data"
```

#### 2. Parameter Description

| Parameter | Description | Example Value |
|-----------|-------------|---------------|
| `--connector-name` | Connector name, must be unique | `greptimedb_connector_01` |
| `--connector-type` | Connector type, fixed as `GreptimeDB` | `GreptimeDB` |
| `--config` | Configuration information in JSON format | `{"server_addr": "localhost:4000", "database": "public", "user": "greptime_user", "password": "greptime_pwd", "precision": "s"}` |
| `--topic-id` | MQTT topic to monitor | `sensor/data` |

#### 3. Configuration Example

```bash
# Create sensor data GreptimeDB connector
robust-ctl mqtt connector create \
  --connector-name "sensor_greptimedb" \
  --connector-type "GreptimeDB" \
  --config '{"server_addr": "greptimedb:4000", "database": "iot_data", "user": "iot_user", "password": "iot_pass", "precision": "ms"}' \
  --topic-id "sensors/+/data"
```

### Managing Connectors

#### 1. List All Connectors
```bash
# List all connectors
robust-ctl mqtt connector list

# List connector with specific name
robust-ctl mqtt connector list --connector-name "greptimedb_connector_01"
```

#### 2. Delete Connector
```bash
# Delete specific connector
robust-ctl mqtt connector delete --connector-name "greptimedb_connector_01"
```

### Complete Operation Example

#### Scenario: Creating IoT Time-Series Data Storage System

```bash
# 1. Create sensor data GreptimeDB connector
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_greptimedb" \
  --connector-type "GreptimeDB" \
  --config '{"server_addr": "greptimedb:4000", "database": "iot_sensors", "user": "sensor_user", "password": "sensor_pass", "precision": "ms"}' \
  --topic-id "iot/sensors/+/data"

# 2. Create device status GreptimeDB connector
robust-ctl mqtt connector create \
  --connector-name "device_status_greptimedb" \
  --connector-type "GreptimeDB" \
  --config '{"server_addr": "greptimedb:4000", "database": "device_status", "user": "device_user", "password": "device_pass", "precision": "s"}' \
  --topic-id "iot/devices/+/status"

# 3. Create alarm message GreptimeDB connector
robust-ctl mqtt connector create \
  --connector-name "alarm_greptimedb" \
  --connector-type "GreptimeDB" \
  --config '{"server_addr": "greptimedb:4000", "database": "alarms", "user": "alarm_user", "password": "alarm_pass", "precision": "s"}' \
  --topic-id "iot/alarms/#"

# 4. View created connectors
robust-ctl mqtt connector list

# 5. Test connector (publish test message)
robust-ctl mqtt publish \
  --username "test_user" \
  --password "test_pass" \
  --topic "iot/sensors/temp_001/data" \
  --qos 1 \
  --message '{"temperature": 25.5, "humidity": 60, "timestamp": 1640995200}'
```

### GreptimeDB Deployment Example

#### Docker Deployment
```bash
# Start GreptimeDB service
docker run -p 4000-4004:4000-4004 \
  -p 4242:4242 \
  -v "$(pwd)/greptimedb:/tmp/greptimedb" \
  --name greptime --rm \
  greptime/greptimedb standalone start \
  --http-addr 0.0.0.0:4000 \
  --rpc-addr 0.0.0.0:4001 \
  --mysql-addr 0.0.0.0:4002 \
  --user-provider=static_user_provider:cmd:greptime_user=greptime_pwd
```

#### Connection Configuration
- **HTTP Address**: `http://localhost:4000`
- **Username**: `greptime_user`
- **Password**: `greptime_pwd`
- **Default Database**: `public`


## Summary

The GreptimeDB connector is an important component of RobustMQ's data integration system, providing efficient time-series data storage capabilities. Through reasonable configuration and usage, it can meet various business requirements such as IoT data storage, real-time monitoring, time-series analysis, and data visualization.

This connector fully leverages GreptimeDB's time-series database characteristics, combined with Rust's memory safety and zero-cost abstraction advantages, achieving high-performance and high-reliability time-series data storage, making it an important tool for building modern IoT data platforms and real-time analysis systems.
