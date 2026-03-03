# ClickHouse Connector

## Overview

The ClickHouse connector is a data integration component provided by RobustMQ for writing MQTT messages to the ClickHouse columnar database. ClickHouse excels at processing and analyzing massive volumes of data with minimal latency, making it ideal for real-time IoT data analytics, time-series data storage, and log processing scenarios.

This connector uses the [official ClickHouse Rust client](https://github.com/ClickHouse/clickhouse-rs) with efficient RowBinary format for data transfer, supporting LZ4 compression and schema validation.

## Features

- Based on the official ClickHouse Rust client with RowBinary format
- LZ4 compression for efficient data transfer
- Schema validation to prevent type mismatches
- Username/password authentication
- Batch inserts for high throughput

## Table Schema

The connector writes data using a fixed row format. Users must create a matching table in ClickHouse beforehand:

```sql
CREATE TABLE mqtt_messages (
    data String,
    key String,
    timestamp UInt64
) ENGINE = MergeTree()
ORDER BY timestamp;
```

### Field Description

| Field | Type | Description |
|-------|------|-------------|
| `data` | String | MQTT message content (payload) |
| `key` | String | Message key (typically source identifier) |
| `timestamp` | UInt64 | Message timestamp (Unix seconds) |

> For more complex data transformations, use ClickHouse's [Materialized View](https://clickhouse.com/docs/en/sql-reference/statements/create/view#materialized-view) to transform data from the raw table to the target table.

## Configuration

### Connector Configuration

```rust
pub struct ClickHouseConnectorConfig {
    pub url: String,           // ClickHouse HTTP URL
    pub database: String,      // Database name
    pub table: String,         // Table name
    pub username: String,      // Username
    pub password: String,      // Password
    pub pool_size: u32,        // Connection pool size
    pub timeout_secs: u64,     // Timeout (seconds)
}
```

### Configuration Parameters

| Parameter | Type | Required | Default | Description | Example |
|-----------|------|----------|---------|-------------|---------|
| `url` | String | Yes | - | ClickHouse HTTP endpoint, must start with `http://` or `https://` | `http://localhost:8123` |
| `database` | String | Yes | - | Database name, max 256 characters | `mqtt_data` |
| `table` | String | Yes | - | Table name, max 256 characters | `mqtt_messages` |
| `username` | String | No | empty | Connection username | `default` |
| `password` | String | No | empty | Connection password | `password123` |
| `pool_size` | Number | No | `8` | Connection pool size, range: 1-64 | `8` |
| `timeout_secs` | Number | No | `15` | Request timeout (seconds), range: 1-300 | `15` |

### Configuration Examples

#### JSON Configuration

**Basic Configuration**
```json
{
  "url": "http://localhost:8123",
  "database": "mqtt_data",
  "table": "mqtt_messages"
}
```

**With Authentication**
```json
{
  "url": "http://clickhouse-server:8123",
  "database": "mqtt_data",
  "table": "mqtt_messages",
  "username": "emqx",
  "password": "public",
  "pool_size": 16,
  "timeout_secs": 30
}
```

#### Full Connector Configuration

```json
{
  "cluster_name": "default",
  "connector_name": "clickhouse_connector_01",
  "connector_type": "clickhouse",
  "config": "{\"url\": \"http://localhost:8123\", \"database\": \"mqtt_data\", \"table\": \"mqtt_messages\", \"username\": \"default\", \"password\": \"\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## Using robust-ctl to Create ClickHouse Connector

### Basic Syntax

```bash
robust-ctl mqtt connector create \
  --connector-name <connector-name> \
  --connector-type <connector-type> \
  --config <config> \
  --topic-id <topic-id>
```

### Creation Examples

#### 1. Basic Creation

```bash
robust-ctl mqtt connector create \
  --connector-name "clickhouse_connector_01" \
  --connector-type "clickhouse" \
  --config '{"url": "http://localhost:8123", "database": "mqtt_data", "table": "mqtt_messages"}' \
  --topic-id "sensor/data"
```

#### 2. With Authentication

```bash
robust-ctl mqtt connector create \
  --connector-name "clickhouse_auth" \
  --connector-type "clickhouse" \
  --config '{"url": "http://clickhouse-server:8123", "database": "iot_db", "table": "device_logs", "username": "admin", "password": "secret", "pool_size": 16}' \
  --topic-id "device/#"
```

### Managing Connectors

```bash
# List all connectors
robust-ctl mqtt connector list

# View specific connector
robust-ctl mqtt connector list --connector-name "clickhouse_connector_01"

# Delete connector
robust-ctl mqtt connector delete --connector-name "clickhouse_connector_01"
```

### Full Operation Example

#### Scenario: IoT Sensor Data Analytics

```bash
# 1. Create table in ClickHouse
# clickhouse-client --query "CREATE TABLE mqtt_data.sensor_messages (data String, key String, timestamp UInt64) ENGINE = MergeTree() ORDER BY timestamp"

# 2. Create connector
robust-ctl mqtt connector create \
  --connector-name "sensor_to_clickhouse" \
  --connector-type "clickhouse" \
  --config '{"url": "http://localhost:8123", "database": "mqtt_data", "table": "sensor_messages", "username": "default"}' \
  --topic-id "sensor/+"

# 3. View created connectors
robust-ctl mqtt connector list
```

## Performance Optimization

### 1. Connection Pool
- Set `pool_size` based on concurrency requirements
- Recommended 16-32 for high-throughput scenarios

### 2. Table Engine Selection
- Use `MergeTree` or `ReplacingMergeTree` for time-series data
- Add TTL for automatic data cleanup: `TTL timestamp + INTERVAL 30 DAY`

### 3. Batch Writes
- The connector uses built-in batch writes (up to 100 records per batch)
- ClickHouse is optimized for bulk inserts

### 4. Security Recommendations
- Use a dedicated database user in production
- Restrict user permissions to INSERT only
- Configure TLS on ClickHouse for HTTPS access

## Monitoring and Troubleshooting

### 1. Check Connector Status

```bash
robust-ctl mqtt connector list --connector-name "clickhouse_connector_01"
```

### 2. Common Issues

**Issue 1: Connection Failure**
- Check if ClickHouse is running
- Verify URL and port (default HTTP port is 8123)
- Confirm username/password credentials
- Check network connectivity and firewall rules

**Issue 2: Write Failure (Schema Mismatch)**
- Verify the ClickHouse table has the required columns: `data String, key String, timestamp UInt64`
- Check database and table name spelling

**Issue 3: Write Latency**
- Monitor ClickHouse server load
- Increase `pool_size` if needed
- Check network latency

## Summary

The ClickHouse connector is an important component of the RobustMQ data integration system, leveraging ClickHouse's columnar storage and high-performance analytics capabilities for efficient MQTT data storage. With simple configuration, you can achieve:

- **Real-time Analytics**: MQTT messages written to ClickHouse in real-time for instant query analysis
- **High Throughput**: Batch writes with RowBinary format, suitable for large-scale IoT data
- **Official Support**: Uses the official ClickHouse Rust client for long-term stability
- **Flexible Extension**: Use Materialized Views for data transformation and processing
