# MySQL Connector

## Overview

The MySQL Connector is a data integration component provided by RobustMQ for bridging MQTT messages to MySQL relational database systems. This connector supports high-performance data writing and is suitable for scenarios such as IoT data storage, historical data analysis, data persistence, and business data integration.

## Configuration

### Connector Configuration

The MySQL Connector uses the `MySQLConnectorConfig` structure for configuration:

```rust
pub struct MySQLConnectorConfig {
    pub host: String,                    // MySQL server address
    pub port: u16,                       // MySQL server port
    pub database: String,                // Database name
    pub username: String,                // Username
    pub password: String,                // Password
    pub table: String,                   // Target table name
    pub sql_template: Option<String>,    // Custom SQL template
    pub pool_size: Option<u32>,          // Connection pool size
    pub enable_batch_insert: Option<bool>, // Enable batch insert
    pub enable_upsert: Option<bool>,     // Enable UPSERT operation
    pub conflict_columns: Option<String>, // Conflict column definition
}
```

### Configuration Parameters

| Parameter | Type | Required | Description | Example |
|--------|------|------|------|------|
| `host` | String | Yes | MySQL server address | `localhost` or `192.168.1.100` |
| `port` | u16 | Yes | MySQL server port | `3306` |
| `database` | String | Yes | Database name | `mqtt_data` |
| `username` | String | Yes | Database username | `root` |
| `password` | String | Yes | Database password | `password123` |
| `table` | String | Yes | Target table name | `mqtt_messages` |
| `sql_template` | String | No | Custom SQL insert template | `INSERT INTO ...` |
| `pool_size` | u32 | No | Connection pool size, default 10 | `20` |
| `enable_batch_insert` | bool | No | Enable batch insert, default false | `true` |
| `enable_upsert` | bool | No | Enable UPSERT operation, default false | `true` |
| `conflict_columns` | String | No | Conflict column definition (used in UPSERT mode) | `record_key` |

### Configuration Examples

#### JSON Configuration Format
```json
{
  "host": "localhost",
  "port": 3306,
  "database": "mqtt_data",
  "username": "root",
  "password": "password123",
  "table": "mqtt_messages",
  "pool_size": 20,
  "enable_batch_insert": true,
  "enable_upsert": false
}
```

#### Full Connector Configuration
```json
{
  "cluster_name": "default",
  "connector_name": "mysql_connector_01",
  "connector_type": "MySQL",
  "config": "{\"host\": \"localhost\", \"port\": 3306, \"database\": \"mqtt_data\", \"username\": \"root\", \"password\": \"password123\", \"table\": \"mqtt_messages\", \"enable_batch_insert\": true}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## Database Table Structure

### Table Schema Definition

The MySQL Connector requires the target table to have the following structure:

```sql
CREATE TABLE mqtt_messages (
    record_key VARCHAR(255) NOT NULL,   -- Unique record key (typically client ID)
    payload TEXT,                       -- Message payload (JSON format)
    timestamp BIGINT NOT NULL,          -- Message timestamp (milliseconds)
    PRIMARY KEY (record_key, timestamp),
    INDEX idx_timestamp (timestamp),
    INDEX idx_record_key (record_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

### Field Descriptions

| Field Name | Data Type | Description | Example |
|--------|----------|------|------|
| `record_key` | VARCHAR(255) | Unique record key, typically client ID | `sensor_001` |
| `payload` | TEXT | Message payload (JSON format) | `{"client_id":"sensor_001","topic":"sensor/temperature",...}` |
| `timestamp` | BIGINT | Message timestamp (milliseconds) | `1640995200000` |

### Index Recommendations

To improve query performance, it is recommended to create the following indexes:

```sql
-- Time range query index
CREATE INDEX idx_mqtt_messages_timestamp ON mqtt_messages (timestamp);

-- Record key query index
CREATE INDEX idx_mqtt_messages_record_key ON mqtt_messages (record_key);

-- Composite query index
CREATE INDEX idx_mqtt_messages_key_timestamp ON mqtt_messages (record_key, timestamp);
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
- Increases database write throughput
- Reduces system resource consumption

**Notes:**
- Batch size is automatically controlled by the system
- Suitable for high-frequency message scenarios
- In batch mode, custom `sql_template` is not applied

### UPSERT Operation

Enabling the `enable_upsert` option can handle duplicate data:

```json
{
  "enable_upsert": true
}
```

**How it works:**
- Uses MySQL's `ON DUPLICATE KEY UPDATE` syntax
- When a duplicate `record_key` is encountered, updates the `payload` and `timestamp` fields
- Suitable for scenarios requiring the latest state

**SQL Example:**
```sql
INSERT INTO mqtt_messages (record_key, payload, timestamp) 
VALUES (?, ?, ?) 
ON DUPLICATE KEY UPDATE 
    payload = VALUES(payload), 
    timestamp = VALUES(timestamp)
```

### Custom SQL Template

You can customize the insert statement via `sql_template` (only effective in non-batch mode):

```json
{
  "sql_template": "INSERT INTO mqtt_messages (record_key, payload, timestamp) VALUES (?, ?, ?)",
  "enable_batch_insert": false
}
```

**Notes:**
- SQL template must contain 3 placeholders `?`
- Placeholders are bound in order: `record_key`, `payload`, `timestamp`
- This option is not effective in batch insert mode

### Connection Pool Management

The connector uses a connection pool to manage database connections:

```json
{
  "pool_size": 20
}
```

**Configuration Recommendations:**
- Low concurrency scenario: 5-10 connections
- Medium concurrency scenario: 10-20 connections
- High concurrency scenario: 20-50 connections

## Using robust-ctl to Create MySQL Connector

### Basic Syntax

Use the `robust-ctl` command-line tool to easily create and manage MySQL connectors:

```bash
robust-ctl mqtt connector create \
  --connector-name <connector_name> \
  --connector-type <connector_type> \
  --config <config> \
  --topic-id <topic_id>
```

### Creating MySQL Connector

#### 1. Basic Create Command

```bash
# Create MySQL connector
robust-ctl mqtt connector create \
  --connector-name "mysql_connector_01" \
  --connector-type "MySQL" \
  --config '{"host": "localhost", "port": 3306, "database": "mqtt_data", "username": "root", "password": "password123", "table": "mqtt_messages"}' \
  --topic-id "sensor/data"
```

#### 2. Parameter Descriptions

| Parameter | Description | Example Value |
|------|------|--------|
| `--connector-name` | Connector name, must be unique | `mysql_connector_01` |
| `--connector-type` | Connector type, fixed as `MySQL` | `MySQL` |
| `--config` | Configuration information in JSON format | `{"host": "localhost", "port": 3306, ...}` |
| `--topic-id` | MQTT topic to listen to | `sensor/data` |

#### 3. High Performance Configuration Example

```bash
# Create high-performance MySQL connector
robust-ctl mqtt connector create \
  --connector-name "high_perf_mysql" \
  --connector-type "MySQL" \
  --config '{"host": "mysql.example.com", "port": 3306, "database": "iot_data", "username": "iot_user", "password": "secure_password", "table": "sensor_data", "pool_size": 30, "enable_batch_insert": true, "enable_upsert": true}' \
  --topic-id "iot/sensors/+/data"
```

### Managing Connectors

#### 1. List All Connectors
```bash
# List all connectors
robust-ctl mqtt connector list

# List specific connector
robust-ctl mqtt connector list --connector-name "mysql_connector_01"
```

#### 2. Delete Connector
```bash
# Delete specific connector
robust-ctl mqtt connector delete --connector-name "mysql_connector_01"
```

### Complete Operation Example

#### Scenario: Creating IoT Data Storage System

```bash
# 1. Create sensor data MySQL connector
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_mysql" \
  --connector-type "MySQL" \
  --config '{"host": "mysql.iot.local", "port": 3306, "database": "iot_platform", "username": "iot_writer", "password": "iot_pass_2023", "table": "sensor_readings", "pool_size": 25, "enable_batch_insert": true}' \
  --topic-id "iot/sensors/+/readings"

# 2. Create device status MySQL connector
robust-ctl mqtt connector create \
  --connector-name "device_status_mysql" \
  --connector-type "MySQL" \
  --config '{"host": "mysql.iot.local", "port": 3306, "database": "iot_platform", "username": "iot_writer", "password": "iot_pass_2023", "table": "device_status", "enable_upsert": true}' \
  --topic-id "iot/devices/+/status"

# 3. Create alarm message MySQL connector
robust-ctl mqtt connector create \
  --connector-name "alarm_mysql" \
  --connector-type "MySQL" \
  --config '{"host": "mysql.iot.local", "port": 3306, "database": "iot_platform", "username": "iot_writer", "password": "iot_pass_2023", "table": "alarm_logs", "pool_size": 15}' \
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
-- Query data from the last 1 hour
SELECT record_key, payload, FROM_UNIXTIME(timestamp/1000) as time 
FROM mqtt_messages 
WHERE timestamp > UNIX_TIMESTAMP(DATE_SUB(NOW(), INTERVAL 1 HOUR)) * 1000
ORDER BY timestamp DESC;

-- Query messages for a specific record key
SELECT payload, FROM_UNIXTIME(timestamp/1000) as time 
FROM mqtt_messages 
WHERE record_key = 'sensor_001'
ORDER BY timestamp DESC
LIMIT 100;

-- Query data within a specific time range
SELECT * FROM mqtt_messages 
WHERE timestamp BETWEEN 1640995200000 AND 1640995300000
ORDER BY timestamp;
```

### Aggregation Queries

```sql
-- Count messages per hour
SELECT 
    FROM_UNIXTIME(timestamp/1000, '%Y-%m-%d %H:00:00') as hour,
    COUNT(*) as message_count
FROM mqtt_messages 
WHERE timestamp > UNIX_TIMESTAMP(DATE_SUB(NOW(), INTERVAL 24 HOUR)) * 1000
GROUP BY hour
ORDER BY hour;

-- Count messages per record key
SELECT 
    record_key,
    COUNT(*) as message_count,
    FROM_UNIXTIME(MIN(timestamp)/1000) as first_message,
    FROM_UNIXTIME(MAX(timestamp)/1000) as last_message
FROM mqtt_messages 
GROUP BY record_key
ORDER BY message_count DESC;
```

## Performance Optimization

### Database Optimization

1. **Table Partitioning**
```sql
-- Partition by month
CREATE TABLE mqtt_messages (
    record_key VARCHAR(255) NOT NULL,
    payload TEXT,
    timestamp BIGINT NOT NULL,
    PRIMARY KEY (record_key, timestamp)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
PARTITION BY RANGE (timestamp) (
    PARTITION p202401 VALUES LESS THAN (1706745600000),
    PARTITION p202402 VALUES LESS THAN (1709251200000),
    PARTITION pmax VALUES LESS THAN MAXVALUE
);
```

2. **Regular Cleanup**
```sql
-- Delete data older than 30 days
DELETE FROM mqtt_messages 
WHERE timestamp < UNIX_TIMESTAMP(DATE_SUB(NOW(), INTERVAL 30 DAY)) * 1000;
```

3. **MySQL Configuration Optimization**
```ini
# MySQL configuration optimization
max_connections = 200
innodb_buffer_pool_size = 1G
innodb_log_file_size = 256M
innodb_flush_log_at_trx_commit = 2
```

### Connector Optimization

1. **Enable Batch Insert**
```json
{
  "enable_batch_insert": true,
  "pool_size": 30
}
```

2. **Properly Set Connection Pool Size**
- Adjust `pool_size` based on concurrent message volume
- Monitor database connection usage

3. **Use UPSERT to Handle Duplicate Data**
```json
{
  "enable_upsert": true
}
```

## Monitoring and Troubleshooting

### Log Monitoring

The connector outputs detailed operation logs:

```
INFO  Connector mysql_connector_01 successfully connected to MySQL database: mqtt_data
INFO  Connector mysql_connector_01 successfully wrote 100 records to MySQL table mqtt_messages
ERROR Connector mysql_connector_01 failed to write data to MySQL table mqtt_messages, error: connection timeout
```

### Common Issues

1. **Connection Failure**
   - Check network connectivity
   - Verify database credentials
   - Confirm firewall settings
   - Check if MySQL service is running

2. **Low Write Performance**
   - Enable batch insert
   - Increase connection pool size
   - Optimize database indexes
   - Check disk I/O performance

3. **Data Duplication**
   - Enable UPSERT functionality
   - Set appropriate primary key constraints

4. **Connection Pool Exhaustion**
   - Increase `pool_size` parameter
   - Optimize message processing speed
   - Check for slow queries

## MySQL vs PostgreSQL Comparison

| Feature | MySQL Connector | PostgreSQL Connector |
|------|-------------|-------------------|
| Data Model | Record key + Payload + Timestamp | Client ID + Topic + Payload + Data + Timestamp |
| UPSERT Syntax | `ON DUPLICATE KEY UPDATE` | `ON CONFLICT ... DO UPDATE` |
| Custom SQL | Supports `sql_template` | Not supported |
| Batch Insert | Supported | Supported |
| Connection Pool | sqlx connection pool | tokio-postgres client |
| Use Cases | General data storage | Structured data storage |

## Summary

The MySQL Connector is an important component of the RobustMQ data integration system, providing high-performance relational database bridging capabilities. With proper configuration and use, it can meet various business needs such as IoT data storage, historical data analysis, data persistence, and business data integration.

This connector fully leverages MySQL's high performance and mature stability, combined with Rust's memory safety and zero-cost abstraction advantages, to achieve efficient and reliable data storage. It is an important tool for building modern IoT data platforms and analysis systems.

**Core Features:**
- ✅ High-performance batch insert
- ✅ UPSERT operation support
- ✅ Connection pool management
- ✅ Custom SQL templates
- ✅ Flexible data model
- ✅ Comprehensive error handling

