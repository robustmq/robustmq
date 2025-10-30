# Local File Connector

## Overview

The Local File connector is a data integration component provided by RobustMQ for writing MQTT messages to the local file system. This connector is simple and reliable, suitable for scenarios such as data backup, log recording, and offline analysis.

## Configuration

### Connector Configuration

The Local File connector uses the `LocalFileConnectorConfig` structure for configuration:

```rust
pub struct LocalFileConnectorConfig {
    pub local_file_path: String,       // Local file path
    pub rotation_strategy: RotationStrategy,  // File rotation strategy
    pub max_size_gb: u64,              // Maximum file size (GB)
}

pub enum RotationStrategy {
    None,    // No rotation
    Size,    // Rotate by size
    Hourly,  // Rotate hourly
    Daily,   // Rotate daily
}
```

### Configuration Parameters

| Parameter | Type | Required | Default | Description | Example |
|-----------|------|----------|---------|-------------|---------|
| `local_file_path` | String | Yes | - | Complete path to the local file | `/var/log/mqtt_messages.log` |
| `rotation_strategy` | String | No | `none` | File rotation strategy: `none` (no rotation), `size` (rotate by size), `hourly` (rotate hourly), `daily` (rotate daily) | `daily` |
| `max_size_gb` | Number | No | `1` | Maximum file size in GB, only effective when `rotation_strategy` is `size`, range: 1-10 | `5` |

### Configuration Examples

#### JSON Configuration Format

**Basic Configuration (No Rotation)**
```json
{
  "local_file_path": "/var/log/mqtt_messages.log"
}
```

**Rotate by Size**
```json
{
  "local_file_path": "/var/log/mqtt_messages.log",
  "rotation_strategy": "size",
  "max_size_gb": 5
}
```

**Rotate Hourly**
```json
{
  "local_file_path": "/var/log/mqtt_messages.log",
  "rotation_strategy": "hourly"
}
```

**Rotate Daily**
```json
{
  "local_file_path": "/var/log/mqtt_messages.log",
  "rotation_strategy": "daily"
}
```

#### Complete Connector Configuration

**Without Rotation**
```json
{
  "cluster_name": "default",
  "connector_name": "file_connector_01",
  "connector_type": "LocalFile",
  "config": "{\"local_file_path\": \"/var/log/mqtt_messages.log\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

**With Rotation Strategy**
```json
{
  "cluster_name": "default",
  "connector_name": "file_connector_01",
  "connector_type": "LocalFile",
  "config": "{\"local_file_path\": \"/var/log/mqtt_messages.log\", \"rotation_strategy\": \"daily\"}",
  "topic_name": "sensor/data",
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

## File Rotation Strategy

### Feature Description

The Local File connector supports automatic file rotation functionality, which can automatically create new files based on file size or time, preventing a single file from becoming too large.

### Rotation Strategy Types

#### 1. None (No Rotation)
- Default strategy, all data written to the same file
- Suitable for scenarios with low message volume

#### 2. Size (Rotate by Size)
- Automatically rotates when the file reaches the specified size
- Size range: 1GB - 10GB
- Rotated filename format: `original_name_YYYYMMdd_HHMMSS.extension`
- Example: `mqtt_messages_20231215_143025.log`

#### 3. Hourly (Rotate Hourly)
- Automatically creates a new file every hour
- Rotated filename format: `original_name_YYYYMMdd_HH.extension`
- Example: `mqtt_messages_20231215_14.log`

#### 4. Daily (Rotate Daily)
- Automatically creates a new file every day
- Rotated filename format: `original_name_YYYYMMdd.extension`
- Example: `mqtt_messages_20231215.log`

### Usage Examples

#### Rotate by Size (5GB)
```bash
robust-ctl mqtt connector create \
  --connector-name "file_size_rotation" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/mqtt.log", "rotation_strategy": "size", "max_size_gb": 5}' \
  --topic-id "sensor/data"
```

#### Rotate Hourly
```bash
robust-ctl mqtt connector create \
  --connector-name "file_hourly_rotation" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/mqtt.log", "rotation_strategy": "hourly"}' \
  --topic-id "sensor/data"
```

#### Rotate Daily
```bash
robust-ctl mqtt connector create \
  --connector-name "file_daily_rotation" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/mqtt.log", "rotation_strategy": "daily"}' \
  --topic-id "sensor/data"
```

### Rotation Behavior

1. **File Check**: Checks if rotation is needed before writing data
2. **File Rename**: Current file is renamed to a timestamped file
3. **Create New File**: A new file is created using the original filename
4. **Continue Writing**: New data is written to the new file

### Important Notes

- Rotation operations are atomic and will not lose data
- Old files need to be manually cleaned up or managed through log rotation tools
- When rotating by size, the actual file size may be slightly larger than the set value (because it checks after batch writes)
- When rotating by time, the precision is at the second level

## Creating File Connectors with robust-ctl

### Basic Syntax

Use the `robust-ctl` command-line tool to easily create and manage local file connectors:

```bash
robust-ctl mqtt connector create \
  --connector-name <connector_name> \
  --connector-type <connector_type> \
  --config <config> \
  --topic-id <topic_name>
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
