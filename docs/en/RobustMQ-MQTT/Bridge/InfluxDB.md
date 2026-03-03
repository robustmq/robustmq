# InfluxDB Connector

## Overview

The InfluxDB connector is a data integration component provided by RobustMQ for writing MQTT messages to InfluxDB time-series databases. InfluxDB is purpose-built for time-series data and is widely used in IoT monitoring, infrastructure metrics collection, and real-time analytics.

This connector uses the HTTP API + Line Protocol to write data directly, requiring no additional client libraries. It is compatible with both InfluxDB v1.x and v2.x.

## Features

- Supports both InfluxDB v1 and v2
- Based on HTTP API + Line Protocol for maximum compatibility
- Token authentication (v2) and username/password authentication (v1)
- Configurable write precision (ns/us/ms/s)
- Batch writes for high throughput

## Data Mapping

The connector converts MQTT messages to InfluxDB Line Protocol format:

```
measurement,key=<message_key> payload="<message_payload>" <timestamp>
```

| Line Protocol Component | Source | Description |
|------------------------|--------|-------------|
| `measurement` | Config | Specified by the user in connector configuration |
| `key` (tag) | `record.key` | Message key (typically source identifier), stored as tag |
| `payload` (field) | `record.data` | MQTT message content, stored as string field |
| `timestamp` | `record.timestamp` | Message timestamp |

## Configuration

### Connector Config

```rust
pub struct InfluxDBConnectorConfig {
    pub server: String,          // InfluxDB HTTP address
    pub version: InfluxDBVersion, // v1 or v2
    pub token: String,           // v2 auth token
    pub org: String,             // v2 organization
    pub bucket: String,          // v2 bucket
    pub database: String,        // v1 database
    pub username: String,        // v1 username
    pub password: String,        // v1 password
    pub measurement: String,     // measurement name
    pub precision: InfluxDBPrecision, // write precision
    pub timeout_secs: u64,       // timeout in seconds
}
```

### Configuration Parameters

| Parameter | Type | Required | Default | Description | Example |
|-----------|------|----------|---------|-------------|---------|
| `server` | String | Yes | - | InfluxDB HTTP endpoint, must start with `http://` or `https://` | `http://localhost:8086` |
| `version` | String | No | `v2` | InfluxDB version: `v1` or `v2` | `v2` |
| `measurement` | String | Yes | - | InfluxDB measurement name | `mqtt_messages` |
| `precision` | String | No | `ms` | Write precision: `ns`/`us`/`ms`/`s` | `ms` |
| `timeout_secs` | Number | No | `15` | Request timeout in seconds, range: 1-300 | `15` |

**InfluxDB v2 Parameters:**

| Parameter | Type | Required | Default | Description | Example |
|-----------|------|----------|---------|-------------|---------|
| `token` | String | Yes | - | Authentication token | `my-token` |
| `org` | String | Yes | - | Organization name | `my-org` |
| `bucket` | String | Yes | - | Bucket name | `mqtt-bucket` |

**InfluxDB v1 Parameters:**

| Parameter | Type | Required | Default | Description | Example |
|-----------|------|----------|---------|-------------|---------|
| `database` | String | Yes | - | Database name | `mqtt_db` |
| `username` | String | No | empty | Username | `admin` |
| `password` | String | No | empty | Password | `password123` |

### Configuration Examples

#### InfluxDB v2

```json
{
  "server": "http://localhost:8086",
  "version": "v2",
  "token": "my-super-secret-token",
  "org": "my-org",
  "bucket": "iot-data",
  "measurement": "sensor_readings",
  "precision": "ms"
}
```

#### InfluxDB v1

```json
{
  "server": "http://localhost:8086",
  "version": "v1",
  "database": "mqtt_db",
  "username": "admin",
  "password": "password",
  "measurement": "mqtt_messages",
  "precision": "s"
}
```

#### Full Connector Configuration

```json
{
  "cluster_name": "default",
  "connector_name": "influxdb_connector_01",
  "connector_type": "influxdb",
  "config": "{\"server\": \"http://localhost:8086\", \"version\": \"v2\", \"token\": \"my-token\", \"org\": \"my-org\", \"bucket\": \"mqtt-data\", \"measurement\": \"sensor\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## Creating an InfluxDB Connector with robust-ctl

### Basic Syntax

```bash
robust-ctl mqtt connector create \
  --connector-name <connector_name> \
  --connector-type <connector_type> \
  --config <config> \
  --topic-id <topic_id>
```

### Examples

#### 1. InfluxDB v2

```bash
robust-ctl mqtt connector create \
  --connector-name "influxdb_v2_connector" \
  --connector-type "influxdb" \
  --config '{"server": "http://localhost:8086", "version": "v2", "token": "my-token", "org": "my-org", "bucket": "iot-data", "measurement": "sensor_readings"}' \
  --topic-id "sensor/data"
```

#### 2. InfluxDB v1

```bash
robust-ctl mqtt connector create \
  --connector-name "influxdb_v1_connector" \
  --connector-type "influxdb" \
  --config '{"server": "http://localhost:8086", "version": "v1", "database": "mqtt_db", "measurement": "mqtt_messages"}' \
  --topic-id "device/#"
```

### Managing Connectors

```bash
# List all connectors
robust-ctl mqtt connector list

# View specific connector
robust-ctl mqtt connector list --connector-name "influxdb_v2_connector"

# Delete connector
robust-ctl mqtt connector delete --connector-name "influxdb_v2_connector"
```

### Full Example

#### Scenario: IoT Sensor Data Storage

```bash
# 1. Ensure InfluxDB v2 is running and create a bucket
# influx bucket create -n iot-data -o my-org

# 2. Create the connector
robust-ctl mqtt connector create \
  --connector-name "sensor_to_influxdb" \
  --connector-type "influxdb" \
  --config '{"server": "http://localhost:8086", "version": "v2", "token": "my-token", "org": "my-org", "bucket": "iot-data", "measurement": "sensor"}' \
  --topic-id "sensor/+"

# 3. View created connectors
robust-ctl mqtt connector list
```

## Performance Optimization

### 1. Write Precision
- Use `s` (seconds) if millisecond precision is not needed to reduce storage overhead
- IoT scenarios typically work well with `ms` or `s`

### 2. Batch Writes
- The connector has built-in batch writing, up to 100 records per batch
- InfluxDB handles bulk Line Protocol writes very efficiently

### 3. Bucket Strategy (v2)
- Set appropriate retention policies for your data
- For high-frequency data, consider using Downsampling Tasks to aggregate historical data

### 4. Security
- v2: Use tokens with limited permissions (write-only)
- v1: Create dedicated write-only users
- Use HTTPS in production

## Monitoring and Troubleshooting

### 1. Check Connector Status

```bash
robust-ctl mqtt connector list --connector-name "influxdb_v2_connector"
```

### 2. Common Issues

**Issue 1: Connection Failed**
- Verify InfluxDB service is running
- Check server address and port (default 8086)
- Confirm network connectivity

**Issue 2: Authentication Failed (v2)**
- Verify token is correct and valid
- Confirm org and bucket names match InfluxDB configuration
- Token must have write permissions for the target bucket

**Issue 3: Authentication Failed (v1)**
- Check username/password
- Confirm the database exists

**Issue 4: Write Latency**
- Check InfluxDB server load
- Consider reducing write precision
- Check network latency

## Summary

The InfluxDB connector provides RobustMQ with native integration to time-series databases. By using HTTP + Line Protocol directly, it achieves:

- **Broad Compatibility**: Supports both InfluxDB v1 and v2 without additional client libraries
- **High Performance**: Batch Line Protocol writes with native HTTP API, no extra serialization overhead
- **Easy Maintenance**: No third-party crate dependencies, based on stable Line Protocol specification
- **Flexible Configuration**: Supports multiple authentication methods and write precisions
