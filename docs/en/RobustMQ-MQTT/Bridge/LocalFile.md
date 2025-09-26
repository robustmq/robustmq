# Local File Connector

## Overview

The Local File connector is a data integration component provided by RobustMQ for writing MQTT messages to the local file system. This connector is simple and reliable, suitable for scenarios such as data backup, log recording, and offline analysis.

## Configuration

### Connector Configuration

The Local File connector uses the `LocalFileConnectorConfig` structure for configuration:

```rust
pub struct LocalFileConnectorConfig {
    pub local_file_path: String,  // Local file path
}
```

### Configuration Parameters

| Parameter | Type | Required | Description | Example |
|-----------|------|----------|-------------|---------|
| `local_file_path` | String | Yes | Complete path to the local file | `/var/log/mqtt_messages.log` |

### Configuration Examples

#### JSON Configuration Format
```json
{
  "local_file_path": "/var/log/mqtt_messages.log"
}
```

#### Complete Connector Configuration
```json
{
  "cluster_name": "default",
  "connector_name": "file_connector_01",
  "connector_type": "LocalFile",
  "config": "{\"local_file_path\": \"/var/log/mqtt_messages.log\"}",
  "topic_id": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## Message Format

### Storage Format
The Local File connector converts MQTT messages to JSON format for storage, with each message on a separate line.

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

### Field Description

| Field | Type | Description |
|-------|------|-------------|
| `topic` | String | MQTT topic name |
| `qos` | Number | QoS level (0, 1, 2) |
| `retain` | Boolean | Retain flag |
| `payload` | String | Message payload (Base64 encoded) |
| `client_id` | String | Client ID |
| `username` | String | Username |
| `timestamp` | Number | Message timestamp (seconds) |
| `message_id` | Number | Message ID |
| `header` | Array | Message header information array |
| `key` | String | Message key value |
| `data` | String | Message content (Base64 encoded) |
| `tags` | Array | Message tags array |
| `timestamp` | Number | Message timestamp (seconds) |
| `crc_num` | Number | Message CRC checksum value |

## Creating File Connectors with robust-ctl

### Basic Syntax

Use the `robust-ctl` command-line tool to easily create and manage local file connectors:

```bash
robust-ctl mqtt connector create \
  --connector-name <connector_name> \
  --connector-type <connector_type> \
  --config <config> \
  --topic-id <topic_id>
```

### Creating Local File Connectors

#### 1. Basic Create Command

```bash
# Create local file connector
robust-ctl mqtt connector create \
  --connector-name "file_connector_01" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/mqtt_messages.log"}' \
  --topic-id "sensor/data"
```

#### 2. Parameter Description

| Parameter | Description | Example Value |
|-----------|-------------|---------------|
| `--connector-name` | Connector name, must be unique | `file_connector_01` |
| `--connector-type` | Connector type, fixed as `LocalFile` | `LocalFile` |
| `--config` | Configuration information in JSON format | `{"local_file_path": "/path/to/file.log"}` |
| `--topic-id` | MQTT topic to monitor | `sensor/data` |

#### 3. Configuration Example

```bash
# Create sensor data logging connector
robust-ctl mqtt connector create \
  --connector-name "sensor_logger" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/sensor_data.log"}' \
  --topic-id "sensors/+/data"
```

### Managing Connectors

#### 1. List All Connectors
```bash
# List all connectors
robust-ctl mqtt connector list

# List connector with specific name
robust-ctl mqtt connector list --connector-name "file_connector_01"
```

#### 2. Delete Connector
```bash
# Delete specific connector
robust-ctl mqtt connector delete --connector-name "file_connector_01"
```

### Complete Operation Example

#### Scenario: Creating IoT Sensor Data Logging System

```bash
# 1. Create sensor data connector
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_logger" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/iot_sensors.log"}' \
  --topic-id "iot/sensors/+/data"

# 2. Create device status connector
robust-ctl mqtt connector create \
  --connector-name "device_status_logger" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/device_status.log"}' \
  --topic-id "iot/devices/+/status"

# 3. Create alarm message connector
robust-ctl mqtt connector create \
  --connector-name "alarm_logger" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/alarms.log"}' \
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


## Summary

The Local File connector is an important component of RobustMQ's data integration system, providing simple and reliable message persistence capabilities. Through reasonable configuration and usage, it can meet various business requirements such as data backup, log recording, and offline analysis.

This connector fully leverages the advantages of the Rust language, providing memory safety and concurrency safety features while ensuring high performance, making it an important tool for building reliable IoT data pipelines.
