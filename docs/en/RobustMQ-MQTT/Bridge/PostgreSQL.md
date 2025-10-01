# PostgreSQL Connector

## Overview

The PostgreSQL connector is a data integration component provided by RobustMQ for bridging MQTT messages to PostgreSQL relational database systems. This connector supports high-performance data writing, suitable for scenarios such as IoT data storage, historical data analysis, data persistence, and business data integration.

## Configuration

### Connector Configuration

The PostgreSQL connector uses the `PostgresConnectorConfig` structure for configuration:

```rust
pub struct PostgresConnectorConfig {
    pub host: String,                    // PostgreSQL server address
    pub port: u16,                       // PostgreSQL server port
    pub database: String,                // Database name
    pub username: String,                // Username
    pub password: String,                // Password
    pub table: String,                   // Target table name
    pub pool_size: Option<u32>,          // Connection pool size
    pub enable_batch_insert: Option<bool>, // Enable batch insert
    pub enable_upsert: Option<bool>,     // Enable UPSERT operation
    pub conflict_columns: Option<String>, // Conflict columns definition
}
```

### Configuration Parameters

| Parameter | Type | Required | Description | Example |
|-----------|------|----------|-------------|---------|
| `host` | String | Yes | PostgreSQL server address | `localhost` or `192.168.1.100` |
| `port` | u16 | Yes | PostgreSQL server port | `5432` |
| `database` | String | Yes | Database name | `mqtt_data` |
| `username` | String | Yes | Database username | `postgres` |
| `password` | String | Yes | Database password | `password123` |
| `table` | String | Yes | Target table name | `mqtt_messages` |
| `pool_size` | u32 | No | Connection pool size, default is 10 | `20` |
| `enable_batch_insert` | bool | No | Enable batch insert, default is false | `true` |
| `enable_upsert` | bool | No | Enable UPSERT operation, default is false | `true` |
| `conflict_columns` | String | No | Conflict columns definition, default is "client_id, topic" | `"client_id, topic"` |

### Configuration Examples

#### JSON Configuration Format
```json
{
  "host": "localhost",
  "port": 5432,
  "database": "mqtt_data",
  "username": "postgres",
  "password": "password123",
  "table": "mqtt_messages",
  "pool_size": 20,
  "enable_batch_insert": true,
  "enable_upsert": false,
  "conflict_columns": "client_id, topic"
}
```

#### Complete Connector Configuration
```json
{
  "cluster_name": "default",
  "connector_name": "postgres_connector_01",
  "connector_type": "Postgres",
  "config": "{\"host\": \"localhost\", \"port\": 5432, \"database\": \"mqtt_data\", \"username\": \"postgres\", \"password\": \"password123\", \"table\": \"mqtt_messages\", \"enable_batch_insert\": true}",
  "topic_id": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## Database Table Schema

### Table Structure Definition

The PostgreSQL connector requires the target table to have the following structure:

```sql
CREATE TABLE mqtt_messages (
    client_id VARCHAR(255) NOT NULL,     -- MQTT client ID
    topic VARCHAR(500) NOT NULL,         -- MQTT topic
    timestamp BIGINT NOT NULL,           -- Message timestamp (seconds)
    payload TEXT,                        -- Message payload (string format)
    data BYTEA,                          -- Message raw data (binary format)
    PRIMARY KEY (client_id, topic, timestamp)
);
```

### Field Description

| Field Name | Data Type | Description | Example |
|------------|-----------|-------------|---------|
| `client_id` | VARCHAR(255) | MQTT client ID | `sensor_001` |
| `topic` | VARCHAR(500) | MQTT topic name | `sensor/temperature` |
| `timestamp` | BIGINT | Message timestamp (seconds) | `1640995200` |
| `payload` | TEXT | Message payload (UTF-8 string) | `{"temperature": 25.5}` |
| `data` | BYTEA | Message raw data (binary) | `\x7b2274656d70657261747572...` |

### Index Recommendations

To improve query performance, it is recommended to create the following indexes:

```sql
-- Time range query index
CREATE INDEX idx_mqtt_messages_timestamp ON mqtt_messages (timestamp);

-- Topic query index
CREATE INDEX idx_mqtt_messages_topic ON mqtt_messages (topic);

-- Client query index
CREATE INDEX idx_mqtt_messages_client_id ON mqtt_messages (client_id);

-- Composite query index
CREATE INDEX idx_mqtt_messages_topic_timestamp ON mqtt_messages (topic, timestamp);
```

## Advanced Features

### Batch Insert

Enabling the `enable_batch_insert` option can significantly improve write performance:

```json
{
  "enable_batch_insert": true
}
```

**Advantages:**
- Reduces network round trips
- Improves database write throughput
- Reduces system resource consumption

**Notes:**
- Batch size is automatically controlled by the system (default 100 records)
- Suitable for high-frequency message scenarios

### UPSERT Operation

Enabling the `enable_upsert` option can handle duplicate data:

```json
{
  "enable_upsert": true,
  "conflict_columns": "client_id, topic"
}
```

**How it works:**
- When conflicts are encountered, updates the existing record's `timestamp`, `payload`, and `data` fields
- Conflict detection is based on the column combination specified by `conflict_columns`

**SQL Example:**
```sql
INSERT INTO mqtt_messages (client_id, topic, timestamp, payload, data) 
VALUES ($1, $2, $3, $4, $5) 
ON CONFLICT (client_id, topic) 
DO UPDATE SET 
    timestamp = EXCLUDED.timestamp, 
    payload = EXCLUDED.payload, 
    data = EXCLUDED.data
```

### Connection Pool Management

The connector uses connection pooling to manage database connections:

```json
{
  "pool_size": 20
}
```

**Configuration Recommendations:**
- Low concurrency scenarios: 5-10 connections
- Medium concurrency scenarios: 10-20 connections
- High concurrency scenarios: 20-50 connections

## Creating PostgreSQL Connectors with robust-ctl

### Basic Syntax

Use the `robust-ctl` command-line tool to easily create and manage PostgreSQL connectors:

```bash
robust-ctl mqtt connector create \
  --connector-name <connector_name> \
  --connector-type <connector_type> \
  --config <config> \
  --topic-id <topic_id>
```

### Creating PostgreSQL Connectors

#### 1. Basic Create Command

```bash
# Create PostgreSQL connector
robust-ctl mqtt connector create \
  --connector-name "postgres_connector_01" \
  --connector-type "Postgres" \
  --config '{"host": "localhost", "port": 5432, "database": "mqtt_data", "username": "postgres", "password": "password123", "table": "mqtt_messages"}' \
  --topic-id "sensor/data"
```

#### 2. Parameter Description

| Parameter | Description | Example Value |
|-----------|-------------|---------------|
| `--connector-name` | Connector name, must be unique | `postgres_connector_01` |
| `--connector-type` | Connector type, fixed as `Postgres` | `Postgres` |
| `--config` | Configuration information in JSON format | `{"host": "localhost", "port": 5432, ...}` |
| `--topic-id` | MQTT topic to monitor | `sensor/data` |

#### 3. High-Performance Configuration Example

```bash
# Create high-performance PostgreSQL connector
robust-ctl mqtt connector create \
  --connector-name "high_perf_postgres" \
  --connector-type "Postgres" \
  --config '{"host": "postgres.example.com", "port": 5432, "database": "iot_data", "username": "iot_user", "password": "secure_password", "table": "sensor_data", "pool_size": 30, "enable_batch_insert": true, "enable_upsert": true, "conflict_columns": "client_id, topic"}' \
  --topic-id "iot/sensors/+/data"
```

### Managing Connectors

#### 1. List All Connectors
```bash
# List all connectors
robust-ctl mqtt connector list

# List connector with specific name
robust-ctl mqtt connector list --connector-name "postgres_connector_01"
```

#### 2. Delete Connector
```bash
# Delete specific connector
robust-ctl mqtt connector delete --connector-name "postgres_connector_01"
```

### Complete Operation Example

#### Scenario: Creating IoT Data Storage System

```bash
# 1. Create sensor data PostgreSQL connector
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_postgres" \
  --connector-type "Postgres" \
  --config '{"host": "postgres.iot.local", "port": 5432, "database": "iot_platform", "username": "iot_writer", "password": "iot_pass_2023", "table": "sensor_readings", "pool_size": 25, "enable_batch_insert": true}' \
  --topic-id "iot/sensors/+/readings"

# 2. Create device status PostgreSQL connector
robust-ctl mqtt connector create \
  --connector-name "device_status_postgres" \
  --connector-type "Postgres" \
  --config '{"host": "postgres.iot.local", "port": 5432, "database": "iot_platform", "username": "iot_writer", "password": "iot_pass_2023", "table": "device_status", "enable_upsert": true, "conflict_columns": "client_id"}' \
  --topic-id "iot/devices/+/status"

# 3. Create alarm message PostgreSQL connector
robust-ctl mqtt connector create \
  --connector-name "alarm_postgres" \
  --connector-type "Postgres" \
  --config '{"host": "postgres.iot.local", "port": 5432, "database": "iot_platform", "username": "iot_writer", "password": "iot_pass_2023", "table": "alarm_logs", "pool_size": 15}' \
  --topic-id "iot/alarms/#"

# 4. View created connectors
robust-ctl mqtt connector list

# 5. Test connector (publish test message)
robust-ctl mqtt publish \
  --username "test_user" \
  --password "test_pass" \
  --topic "iot/sensors/temp_001/readings" \
  --qos 1 \
  --message '{"temperature": 25.5, "humidity": 60, "timestamp": 1640995200}'
```

## Data Query Examples

### Basic Queries

```sql
-- Query sensor data from the last hour
SELECT client_id, topic, timestamp, payload 
FROM mqtt_messages 
WHERE timestamp > EXTRACT(EPOCH FROM NOW() - INTERVAL '1 hour')
ORDER BY timestamp DESC;

-- Query messages from a specific client
SELECT topic, timestamp, payload 
FROM mqtt_messages 
WHERE client_id = 'sensor_001'
ORDER BY timestamp DESC
LIMIT 100;

-- Query messages from a specific topic
SELECT client_id, timestamp, payload 
FROM mqtt_messages 
WHERE topic LIKE 'sensor/temperature%'
ORDER BY timestamp DESC;
```

### Aggregate Queries

```sql
-- Count messages per hour
SELECT 
    DATE_TRUNC('hour', TO_TIMESTAMP(timestamp)) as hour,
    COUNT(*) as message_count
FROM mqtt_messages 
WHERE timestamp > EXTRACT(EPOCH FROM NOW() - INTERVAL '24 hours')
GROUP BY hour
ORDER BY hour;

-- Count messages by topic
SELECT 
    topic,
    COUNT(*) as message_count,
    MIN(TO_TIMESTAMP(timestamp)) as first_message,
    MAX(TO_TIMESTAMP(timestamp)) as last_message
FROM mqtt_messages 
GROUP BY topic
ORDER BY message_count DESC;
```

## Performance Optimization

### Database Optimization

1. **Table Partitioning**
```sql
-- Partition by time
CREATE TABLE mqtt_messages_2024_01 PARTITION OF mqtt_messages
FOR VALUES FROM (1704067200) TO (1706745600);
```

2. **Regular Cleanup**
```sql
-- Delete data older than 30 days
DELETE FROM mqtt_messages 
WHERE timestamp < EXTRACT(EPOCH FROM NOW() - INTERVAL '30 days');
```

3. **Connection Pool Tuning**
```sql
-- PostgreSQL configuration optimization
max_connections = 200
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 4MB
```

### Connector Optimization

1. **Enable Batch Insert**
```json
{
  "enable_batch_insert": true,
  "pool_size": 30
}
```

2. **Set Appropriate Connection Pool Size**
- Adjust `pool_size` based on concurrent message volume
- Monitor database connection usage

3. **Use UPSERT for Duplicate Data Handling**
```json
{
  "enable_upsert": true,
  "conflict_columns": "client_id, topic"
}
```

## Monitoring and Troubleshooting

### Log Monitoring

The connector outputs detailed runtime logs:

```
INFO  Connector postgres_connector_01 successfully connected to PostgreSQL database: mqtt_data
INFO  Connector postgres_connector_01 successfully wrote 100 records to PostgreSQL table mqtt_messages
ERROR Connector postgres_connector_01 failed to write data to PostgreSQL table mqtt_messages, error: connection timeout
```

### Common Issues

1. **Connection Failure**
   - Check network connectivity
   - Verify database credentials
   - Confirm firewall settings

2. **Low Write Performance**
   - Enable batch insert
   - Increase connection pool size
   - Optimize database indexes

3. **Data Duplication**
   - Enable UPSERT functionality
   - Configure appropriate conflict columns

## Summary

The PostgreSQL connector is an important component of RobustMQ's data integration system, providing high-performance relational database bridging capabilities. Through reasonable configuration and usage, it can meet various business requirements such as IoT data storage, historical data analysis, data persistence, and business data integration.

This connector fully leverages PostgreSQL's ACID properties and rich query capabilities, combined with Rust's memory safety and zero-cost abstraction advantages, achieving efficient and reliable data storage. It is an important tool for building modern IoT data platforms and analysis systems.
