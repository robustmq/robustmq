# OpenTSDB Connector

## Overview

The OpenTSDB connector is a data integration component provided by RobustMQ for bridging MQTT messages to OpenTSDB time-series database. It automatically converts MQTT message payloads into OpenTSDB data point format (metric + tags + value + timestamp), making it suitable for IoT device monitoring, sensor data collection, and industrial energy management scenarios.

## Configuration

### Connector Configuration

The OpenTSDB connector uses `OpenTSDBConnectorConfig` for configuration:

```rust
pub struct OpenTSDBConnectorConfig {
    pub server: String,          // OpenTSDB server URL
    pub metric_field: String,    // Field name for metric in payload
    pub value_field: String,     // Field name for value in payload
    pub tags_fields: Vec<String>,// Fields to extract as tags
    pub timeout_secs: u64,       // Request timeout (seconds)
    pub max_retries: u32,        // Max retries
    pub summary: bool,           // Request summary response
    pub details: bool,           // Request detailed response
}
```

### Configuration Parameters

| Parameter | Type | Required | Default | Description | Example |
|-----------|------|----------|---------|-------------|---------|
| `server` | String | Yes | - | OpenTSDB server URL, must start with `http://` or `https://`, max 512 characters | `http://localhost:4242` |
| `metric_field` | String | No | `metric` | Field name for metric in payload JSON | `metric` |
| `value_field` | String | No | `value` | Field name for value in payload JSON | `value` |
| `tags_fields` | Array | No | `[]` | Fields to extract as tags; when empty, reads the `tags` object from payload | `["host", "region"]` |
| `timeout_secs` | Number | No | `30` | Request timeout in seconds, range: 1-300 | `60` |
| `max_retries` | Number | No | `3` | Max retries, up to 10 | `5` |
| `summary` | Boolean | No | `false` | Append `?summary` to `/api/put` requests | `true` |
| `details` | Boolean | No | `false` | Append `?details` to `/api/put` requests | `true` |

### Data Mapping

The connector extracts fields from MQTT message payload (JSON) and converts them to OpenTSDB data points.

#### Mode 1: Using tags object (default)

When `tags_fields` is empty, the connector reads the `tags` object from the payload:

```json
{
  "metric": "cpu",
  "tags": {
    "host": "serverA",
    "region": "us-east"
  },
  "value": 85.5,
  "timestamp": 1640995200
}
```

Converts to OpenTSDB data point:

```json
{
  "metric": "cpu",
  "timestamp": 1640995200,
  "value": 85.5,
  "tags": {
    "host": "serverA",
    "region": "us-east"
  }
}
```

#### Mode 2: Using tags_fields

When configured with `tags_fields: ["host", "region"]`, the connector extracts tags from top-level fields:

```json
{
  "metric": "temperature",
  "host": "sensor_001",
  "region": "factory_A",
  "value": 25.5
}
```

Converts to:

```json
{
  "metric": "temperature",
  "timestamp": 1640995200,
  "value": 25.5,
  "tags": {
    "host": "sensor_001",
    "region": "factory_A"
  }
}
```

### Configuration Examples

#### JSON Configuration

**Basic Configuration**
```json
{
  "server": "http://localhost:4242"
}
```

**Custom Field Mapping**
```json
{
  "server": "http://localhost:4242",
  "metric_field": "metric",
  "value_field": "value",
  "tags_fields": ["host", "region", "device"],
  "timeout_secs": 60,
  "max_retries": 5
}
```

**With Response Summary**
```json
{
  "server": "http://opentsdb.example.com:4242",
  "summary": true,
  "details": true,
  "timeout_secs": 30
}
```

## Creating an OpenTSDB Connector with robust-ctl

### Basic Syntax

```bash
robust-ctl mqtt connector create \
  --connector-name <name> \
  --connector-type <type> \
  --config <config> \
  --topic-id <topic>
```

### Examples

#### 1. Basic Creation

```bash
robust-ctl mqtt connector create \
  --connector-name "opentsdb_connector_01" \
  --connector-type "opentsdb" \
  --config '{"server": "http://localhost:4242"}' \
  --topic-id "sensor/data"
```

#### 2. Custom Field Mapping

```bash
robust-ctl mqtt connector create \
  --connector-name "opentsdb_custom" \
  --connector-type "opentsdb" \
  --config '{"server": "http://localhost:4242", "metric_field": "metric", "value_field": "value", "tags_fields": ["host", "region"], "timeout_secs": 60}' \
  --topic-id "iot/sensors/temperature"
```

### Managing Connectors

```bash
# List all connectors
robust-ctl mqtt connector list

# View specific connector
robust-ctl mqtt connector list --connector-name "opentsdb_connector_01"

# Delete connector
robust-ctl mqtt connector delete --connector-name "opentsdb_connector_01"
```

## Querying OpenTSDB Data

After data is written, you can query via the OpenTSDB HTTP API:

```bash
curl -X POST http://localhost:4242/api/query -H "Content-Type: application/json" -d '{
  "start": "1h-ago",
  "queries": [
    {
      "aggregator": "last",
      "metric": "cpu.usage",
      "tags": {
        "host": "*"
      }
    }
  ]
}'
```

## Performance Tips

### 1. Batch Writing
- The connector supports batch writing by default, combining multiple messages into a single `/api/put` request
- Effectively reduces HTTP request count in high-throughput scenarios

### 2. Timeout Settings
- Set `timeout_secs` based on your OpenTSDB server performance
- Increase timeout for remote deployments

### 3. Data Modeling
- Use dot-separated hierarchical metric names, e.g., `cpu.usage`, `disk.io.read`
- Tags identify data dimensions, e.g., host, region, device
- Avoid high-cardinality tag values (e.g., UUIDs) to prevent performance degradation

## Troubleshooting

### Common Issues

**Connection Failure**
- Check if OpenTSDB service is running
- Verify server address and port
- Check network and firewall settings

**Data Write Failure**
- Verify payload format contains required metric and value fields
- Check metric naming follows OpenTSDB conventions
- Enable `summary` and `details` for detailed error information

**Write Timeout**
- Increase `timeout_secs`
- Check OpenTSDB service load
- Consider scaling OpenTSDB cluster resources

## Summary

The OpenTSDB connector automatically converts MQTT messages to OpenTSDB time-series data points with flexible field mapping, batch writing, and IoT-optimized data modeling support.
