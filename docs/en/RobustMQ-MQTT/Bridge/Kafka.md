# Kafka Connector

## Overview

The Kafka connector is a data integration component provided by RobustMQ for bridging MQTT messages to Apache Kafka message queue systems. This connector supports high-throughput message transmission, suitable for scenarios such as real-time data stream processing, event-driven architecture, and big data analysis.

## Configuration

### Connector Configuration

The Kafka connector uses the `KafkaConnectorConfig` structure for configuration:

```rust
pub struct KafkaConnectorConfig {
    pub bootstrap_servers: String,  // Kafka server addresses
    pub topic: String,              // Kafka topic name
    pub key: String,                // Message key
}
```

### Configuration Parameters

| Parameter | Type | Required | Description | Example |
|-----------|------|----------|-------------|---------|
| `bootstrap_servers` | String | Yes | Kafka server address list | `localhost:9092` or `kafka1:9092,kafka2:9092` |
| `topic` | String | Yes | Kafka topic name | `mqtt_messages` |
| `key` | String | Yes | Message key for partition routing | `sensor_data` |

### Configuration Examples

#### JSON Configuration Format
```json
{
  "bootstrap_servers": "localhost:9092",
  "topic": "mqtt_messages",
  "key": "sensor_data"
}
```

#### Complete Connector Configuration
```json
{
  "cluster_name": "default",
  "connector_name": "kafka_connector_01",
  "connector_type": "Kafka",
  "config": "{\"bootstrap_servers\": \"localhost:9092\", \"topic\": \"mqtt_messages\", \"key\": \"sensor_data\"}",
  "topic_id": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## Message Format

### Transmission Format
The Kafka connector converts MQTT messages to JSON format and sends them to Kafka topics, with each message as a Kafka record.

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

## Creating Kafka Connectors with robust-ctl

### Basic Syntax

Use the `robust-ctl` command-line tool to easily create and manage Kafka connectors:

```bash
robust-ctl mqtt connector create \
  --connector-name <connector_name> \
  --connector-type <connector_type> \
  --config <config> \
  --topic-id <topic_id>
```

### Creating Kafka Connectors

#### 1. Basic Create Command

```bash
# Create Kafka connector
robust-ctl mqtt connector create \
  --connector-name "kafka_connector_01" \
  --connector-type "Kafka" \
  --config '{"bootstrap_servers": "localhost:9092", "topic": "mqtt_messages", "key": "sensor_data"}' \
  --topic-id "sensor/data"
```

#### 2. Parameter Description

| Parameter | Description | Example Value |
|-----------|-------------|---------------|
| `--connector-name` | Connector name, must be unique | `kafka_connector_01` |
| `--connector-type` | Connector type, fixed as `Kafka` | `Kafka` |
| `--config` | Configuration information in JSON format | `{"bootstrap_servers": "localhost:9092", "topic": "mqtt_messages", "key": "sensor_data"}` |
| `--topic-id` | MQTT topic to monitor | `sensor/data` |

#### 3. Configuration Example

```bash
# Create sensor data Kafka connector
robust-ctl mqtt connector create \
  --connector-name "sensor_kafka_logger" \
  --connector-type "Kafka" \
  --config '{"bootstrap_servers": "kafka1:9092,kafka2:9092", "topic": "sensor_data", "key": "sensor_key"}' \
  --topic-id "sensors/+/data"
```

### Managing Connectors

#### 1. List All Connectors
```bash
# List all connectors
robust-ctl mqtt connector list

# List connector with specific name
robust-ctl mqtt connector list --connector-name "kafka_connector_01"
```

#### 2. Delete Connector
```bash
# Delete specific connector
robust-ctl mqtt connector delete --connector-name "kafka_connector_01"
```

### Complete Operation Example

#### Scenario: Creating IoT Data Stream Processing System

```bash
# 1. Create sensor data Kafka connector
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_kafka" \
  --connector-type "Kafka" \
  --config '{"bootstrap_servers": "kafka1:9092,kafka2:9092", "topic": "iot_sensors", "key": "sensor_key"}' \
  --topic-id "iot/sensors/+/data"

# 2. Create device status Kafka connector
robust-ctl mqtt connector create \
  --connector-name "device_status_kafka" \
  --connector-type "Kafka" \
  --config '{"bootstrap_servers": "kafka1:9092,kafka2:9092", "topic": "device_status", "key": "device_key"}' \
  --topic-id "iot/devices/+/status"

# 3. Create alarm message Kafka connector
robust-ctl mqtt connector create \
  --connector-name "alarm_kafka" \
  --connector-type "Kafka" \
  --config '{"bootstrap_servers": "kafka1:9092,kafka2:9092", "topic": "alarms", "key": "alarm_key"}' \
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

The Kafka connector is an important component of RobustMQ's data integration system, providing high-performance message queue bridging capabilities. Through reasonable configuration and usage, it can meet various business requirements such as real-time data stream processing, event-driven architecture, and big data analysis.

This connector fully leverages Kafka's high-throughput characteristics, combined with Rust's memory safety and zero-cost abstraction advantages, achieving efficient and reliable message transmission, making it an important tool for building modern data pipelines and stream processing systems.
