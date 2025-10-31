# Elasticsearch Connector

## Overview

The Elasticsearch connector is a data integration component provided by RobustMQ for bridging MQTT messages to the Elasticsearch search and analytics engine. This connector supports writing real-time MQTT data streams to Elasticsearch indices, making it suitable for scenarios such as log analysis, full-text search, data visualization, and real-time monitoring.

## Configuration

### Connector Configuration

The Elasticsearch connector uses the `ElasticsearchConnectorConfig` structure for configuration:

```rust
pub struct ElasticsearchConnectorConfig {
    pub url: String,                  // Elasticsearch server address
    pub index: String,                // Index name
    pub auth_type: ElasticsearchAuthType,  // Authentication type
    pub username: Option<String>,     // Username (Basic auth)
    pub password: Option<String>,     // Password (Basic auth)
    pub api_key: Option<String>,      // API key (ApiKey auth)
    pub enable_tls: bool,             // Enable TLS
    pub ca_cert_path: Option<String>, // CA certificate path
    pub timeout_secs: u64,            // Request timeout (seconds)
    pub max_retries: u32,             // Maximum retry attempts
}

pub enum ElasticsearchAuthType {
    None,    // No authentication
    Basic,   // Basic authentication
    ApiKey,  // API key authentication
}
```

### Configuration Parameters

| Parameter | Type | Required | Default | Description | Example |
|-----------|------|----------|---------|-------------|---------|
| `url` | String | Yes | - | Elasticsearch server address, max length 512 characters | `http://localhost:9200` |
| `index` | String | Yes | - | Elasticsearch index name, max length 256 characters | `mqtt_messages` |
| `auth_type` | String | No | `none` | Authentication type: `none`, `basic`, `apikey` | `basic` |
| `username` | String | No | - | Username, required for Basic auth, max length 256 characters | `elastic` |
| `password` | String | No | - | Password, required for Basic auth, max length 256 characters | `password123` |
| `api_key` | String | No | - | API key, required for ApiKey auth | `api_key_value` |
| `enable_tls` | Boolean | No | `false` | Enable TLS encrypted connection | `true` |
| `ca_cert_path` | String | No | - | CA certificate file path (for TLS connections) | `/etc/certs/ca.crt` |
| `timeout_secs` | Number | No | `30` | Request timeout in seconds, range: 1-300 | `60` |
| `max_retries` | Number | No | `3` | Maximum retry attempts, max 10 | `5` |

### Configuration Examples

#### JSON Configuration Format

**No Authentication**
```json
{
  "url": "http://localhost:9200",
  "index": "mqtt_messages"
}
```

**Basic Authentication**
```json
{
  "url": "http://localhost:9200",
  "index": "mqtt_messages",
  "auth_type": "basic",
  "username": "elastic",
  "password": "password123"
}
```

**API Key Authentication**
```json
{
  "url": "https://elasticsearch.example.com:9200",
  "index": "mqtt_messages",
  "auth_type": "apikey",
  "api_key": "your_api_key_here",
  "enable_tls": true,
  "timeout_secs": 60,
  "max_retries": 5
}
```

**TLS Encrypted Connection**
```json
{
  "url": "https://elasticsearch.example.com:9200",
  "index": "mqtt_messages",
  "auth_type": "basic",
  "username": "elastic",
  "password": "password123",
  "enable_tls": true,
  "ca_cert_path": "/etc/certs/ca.crt"
}
```

#### Complete Connector Configuration

```json
{
  "cluster_name": "default",
  "connector_name": "elasticsearch_connector_01",
  "connector_type": "Elasticsearch",
  "config": "{\"url\": \"http://localhost:9200\", \"index\": \"mqtt_messages\", \"auth_type\": \"basic\", \"username\": \"elastic\", \"password\": \"password123\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## Message Format

### Transfer Format
The Elasticsearch connector converts MQTT messages to JSON format and writes them to Elasticsearch indices in bulk.

### Message Structure

```json
{
  "offset": 12345,
  "header": [
    {
      "name": "topic",
      "value": "sensor/temperature"
    },
    {
      "name": "qos",
      "value": "1"
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
| `offset` | Number | Message offset |
| `header` | Array | Message header information array |
| `key` | String | Message key value |
| `data` | String | Message content (Base64 encoded) |
| `tags` | Array | Message tags array |
| `timestamp` | Number | Message timestamp (seconds) |
| `crc_num` | Number | Message CRC checksum value |

## Creating Elasticsearch Connectors with robust-ctl

### Basic Syntax

Use the `robust-ctl` command-line tool to easily create and manage Elasticsearch connectors:

```bash
robust-ctl mqtt connector create \
  --connector-name <connector_name> \
  --connector-type <connector_type> \
  --config <config> \
  --topic-id <topic_name>
```

### Creating Elasticsearch Connectors

#### 1. Basic Create Command

```bash
# Create Elasticsearch connector (no authentication)
robust-ctl mqtt connector create \
  --connector-name "elasticsearch_connector_01" \
  --connector-type "Elasticsearch" \
  --config '{"url": "http://localhost:9200", "index": "mqtt_messages"}' \
  --topic-id "sensor/data"
```

#### 2. Parameter Description

| Parameter | Description | Example Value |
|-----------|-------------|---------------|
| `--connector-name` | Connector name, must be unique | `elasticsearch_connector_01` |
| `--connector-type` | Connector type, fixed as `Elasticsearch` | `Elasticsearch` |
| `--config` | Configuration information in JSON format | `{"url": "http://localhost:9200", "index": "mqtt_messages"}` |
| `--topic-id` | MQTT topic to monitor | `sensor/data` |

#### 3. Configuration Examples

**No Authentication Connection**
```bash
robust-ctl mqtt connector create \
  --connector-name "es_no_auth" \
  --connector-type "Elasticsearch" \
  --config '{"url": "http://localhost:9200", "index": "mqtt_logs"}' \
  --topic-id "logs/#"
```

**Basic Authentication Connection**
```bash
robust-ctl mqtt connector create \
  --connector-name "es_basic_auth" \
  --connector-type "Elasticsearch" \
  --config '{"url": "http://localhost:9200", "index": "mqtt_messages", "auth_type": "basic", "username": "elastic", "password": "password123"}' \
  --topic-id "sensor/+/data"
```

**API Key Authentication Connection (with TLS)**
```bash
robust-ctl mqtt connector create \
  --connector-name "es_secure" \
  --connector-type "Elasticsearch" \
  --config '{"url": "https://es.example.com:9200", "index": "mqtt_secure", "auth_type": "apikey", "api_key": "your_key", "enable_tls": true, "timeout_secs": 60, "max_retries": 5}' \
  --topic-id "secure/data"
```

### Managing Connectors

#### 1. List All Connectors
```bash
# List all connectors
robust-ctl mqtt connector list

# List connector with specific name
robust-ctl mqtt connector list --connector-name "elasticsearch_connector_01"
```

#### 2. Delete Connector
```bash
# Delete specific connector
robust-ctl mqtt connector delete --connector-name "elasticsearch_connector_01"
```

### Complete Operation Example

#### Scenario: Creating IoT Data Search and Analysis System

```bash
# 1. Create sensor data Elasticsearch connector
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_es" \
  --connector-type "Elasticsearch" \
  --config '{"url": "http://localhost:9200", "index": "iot_sensors", "auth_type": "basic", "username": "elastic", "password": "elastic123"}' \
  --topic-id "iot/sensors/+/data"

# 2. Create device status Elasticsearch connector
robust-ctl mqtt connector create \
  --connector-name "device_status_es" \
  --connector-type "Elasticsearch" \
  --config '{"url": "http://localhost:9200", "index": "device_status", "auth_type": "basic", "username": "elastic", "password": "elastic123"}' \
  --topic-id "iot/devices/+/status"

# 3. Create alarm message Elasticsearch connector
robust-ctl mqtt connector create \
  --connector-name "alarm_es" \
  --connector-type "Elasticsearch" \
  --config '{"url": "http://localhost:9200", "index": "alarms", "auth_type": "basic", "username": "elastic", "password": "elastic123", "timeout_secs": 60}' \
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

## Elasticsearch Query Examples

After the connector writes data to Elasticsearch, you can use Elasticsearch's REST API or Kibana for queries and analysis.

### 1. Query All Messages

```bash
curl -X GET "localhost:9200/mqtt_messages/_search?pretty" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": {
      "match_all": {}
    }
  }'
```

### 2. Query by Time Range

```bash
curl -X GET "localhost:9200/mqtt_messages/_search?pretty" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": {
      "range": {
        "timestamp": {
          "gte": 1640995200,
          "lte": 1641081600
        }
      }
    }
  }'
```

### 3. Full-Text Search

```bash
curl -X GET "localhost:9200/mqtt_messages/_search?pretty" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": {
      "match": {
        "data": "temperature"
      }
    }
  }'
```

## Performance Optimization Recommendations

### 1. Bulk Writes
- The connector uses bulk writes (Bulk API) by default for better performance
- Adjust batch size based on message volume

### 2. Index Settings
- Set appropriate number of shards based on data volume
- Use Index Lifecycle Management (ILM) to automatically manage old data
- Consider using time-series index patterns (e.g., `mqtt_messages-2023.12.15`)

### 3. Connection Optimization
- For high-throughput scenarios, increase `timeout_secs` and `max_retries`
- Use connection pooling to reduce connection overhead
- Consider using an Elasticsearch cluster for high availability

### 4. Security Recommendations
- Enable TLS encryption in production environments
- Use API Key authentication instead of Basic authentication
- Rotate API keys regularly
- Configure Elasticsearch users with principle of least privilege

## Monitoring and Troubleshooting

### 1. Check Connector Status

```bash
robust-ctl mqtt connector list --connector-name "elasticsearch_connector_01"
```

### 2. Check Elasticsearch Index Status

```bash
curl -X GET "localhost:9200/_cat/indices/mqtt_messages?v"
```

### 3. Common Issues

**Issue 1: Connection Failed**
- Check if Elasticsearch service is running
- Verify URL address is correct
- Check network connection and firewall settings

**Issue 2: Authentication Failed**
- Verify username and password are correct
- Check if API Key is valid
- Confirm user has sufficient permissions

**Issue 3: Write Timeout**
- Increase `timeout_secs` configuration
- Check Elasticsearch service load
- Consider increasing Elasticsearch cluster resources

## Summary

The Elasticsearch connector is a powerful component of RobustMQ's data integration system, providing seamless integration with the Elasticsearch search engine. With proper configuration and usage, it enables:

- **Real-time Search**: MQTT messages are written to Elasticsearch in real-time, supporting full-text search and aggregation analysis
- **Data Visualization**: Combined with Kibana for data visualization and monitoring
- **Flexible Authentication**: Supports multiple authentication methods to meet different security requirements
- **High Performance**: Bulk writes and connection optimization ensure performance in high-throughput scenarios
- **Easy Integration**: Simple configuration to bridge MQTT data to Elasticsearch

This connector fully leverages Rust's performance advantages and Elasticsearch's search capabilities, making it an ideal choice for building modern IoT data analysis and monitoring systems.


