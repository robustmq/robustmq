# Webhook Connector

## Overview

The Webhook connector is a data integration component provided by RobustMQ for forwarding MQTT messages to external web services via HTTP requests. It supports sending real-time MQTT data in JSON format via POST/PUT to specified HTTP endpoints, making it suitable for event notifications, data synchronization, and third-party system integration.

## Configuration

### Connector Configuration

The Webhook connector uses `WebhookConnectorConfig` for configuration:

```rust
pub struct WebhookConnectorConfig {
    pub url: String,                       // HTTP endpoint URL
    pub method: WebhookHttpMethod,         // HTTP method (POST/PUT)
    pub headers: HashMap<String, String>,  // Custom HTTP headers
    pub timeout_ms: u64,                   // Request timeout (milliseconds)
    pub auth_type: WebhookAuthType,        // Authentication type
    pub username: Option<String>,          // Username (Basic auth)
    pub password: Option<String>,          // Password (Basic auth)
    pub bearer_token: Option<String>,      // Bearer Token
}

pub enum WebhookHttpMethod {
    Post,   // HTTP POST
    Put,    // HTTP PUT
}

pub enum WebhookAuthType {
    None,    // No authentication
    Basic,   // Basic authentication
    Bearer,  // Bearer Token authentication
}
```

### Configuration Parameters

| Parameter | Type | Required | Default | Description | Example |
|-----------|------|----------|---------|-------------|---------|
| `url` | String | Yes | - | HTTP endpoint URL, max 2048 characters, must start with `http://` or `https://` | `https://api.example.com/webhook` |
| `method` | String | No | `post` | HTTP method: `post`, `put` | `post` |
| `headers` | Object | No | `{}` | Custom HTTP header key-value pairs | `{"X-Api-Key": "abc123"}` |
| `timeout_ms` | Number | No | `5000` | Request timeout in milliseconds, range: 1-60000 | `10000` |
| `auth_type` | String | No | `none` | Authentication type: `none`, `basic`, `bearer` | `basic` |
| `username` | String | No | - | Username, required for Basic auth | `admin` |
| `password` | String | No | - | Password, required for Basic auth | `password123` |
| `bearer_token` | String | No | - | Bearer Token, required for Bearer auth | `eyJhbGciOi...` |

### Configuration Examples

#### JSON Configuration

**No Authentication**
```json
{
  "url": "http://localhost:8080/webhook"
}
```

**Basic Authentication**
```json
{
  "url": "https://api.example.com/events",
  "method": "post",
  "auth_type": "basic",
  "username": "admin",
  "password": "password123",
  "timeout_ms": 10000
}
```

**Bearer Token Authentication**
```json
{
  "url": "https://api.example.com/events",
  "auth_type": "bearer",
  "bearer_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "headers": {
    "X-Source": "robustmq",
    "X-Event-Type": "mqtt_message"
  }
}
```

**Custom Headers**
```json
{
  "url": "https://api.example.com/ingest",
  "method": "put",
  "headers": {
    "X-Api-Key": "your-api-key",
    "X-Tenant-Id": "tenant-001"
  },
  "timeout_ms": 15000
}
```

#### Full Connector Configuration

```json
{
  "cluster_name": "default",
  "connector_name": "webhook_connector_01",
  "connector_type": "webhook",
  "config": "{\"url\": \"https://api.example.com/events\", \"auth_type\": \"bearer\", \"bearer_token\": \"your-token\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## Message Format

### Single Message

When forwarding a single message, the HTTP request body is a JSON object:

```json
{
  "payload": "{\"temperature\": 25.5}",
  "timestamp": 1640995200,
  "key": "sensor_001",
  "headers": [
    {"name": "topic", "value": "sensor/temperature"},
    {"name": "qos", "value": "1"}
  ]
}
```

### Batch Messages

When forwarding multiple messages, the HTTP request body is a JSON array:

```json
[
  {
    "payload": "{\"temperature\": 25.5}",
    "timestamp": 1640995200,
    "key": "sensor_001"
  },
  {
    "payload": "{\"temperature\": 26.0}",
    "timestamp": 1640995201,
    "key": "sensor_002"
  }
]
```

### Field Description

| Field | Type | Description |
|-------|------|-------------|
| `payload` | String | Message content |
| `timestamp` | Number | Message timestamp (seconds) |
| `key` | String | Message key (optional) |
| `headers` | Array | Message headers (optional) |

## Creating a Webhook Connector with robust-ctl

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
  --connector-name "webhook_connector_01" \
  --connector-type "webhook" \
  --config '{"url": "http://localhost:8080/webhook"}' \
  --topic-id "sensor/data"
```

#### 2. Authenticated Webhook

```bash
robust-ctl mqtt connector create \
  --connector-name "webhook_auth" \
  --connector-type "webhook" \
  --config '{"url": "https://api.example.com/events", "auth_type": "bearer", "bearer_token": "your-token", "timeout_ms": 10000}' \
  --topic-id "device/status"
```

### Managing Connectors

```bash
# List all connectors
robust-ctl mqtt connector list

# View specific connector
robust-ctl mqtt connector list --connector-name "webhook_connector_01"

# Delete connector
robust-ctl mqtt connector delete --connector-name "webhook_connector_01"
```

## Performance Tips

### 1. Timeout Settings
- Set `timeout_ms` according to your target service's response time
- Use shorter timeouts for alert scenarios
- Increase timeout for high-latency services

### 2. Target Service
- Ensure the target HTTP service has sufficient concurrent processing capacity
- Target service should return 2xx status codes for success
- Use HTTPS for secure data transmission

### 3. Security
- Use HTTPS in production environments
- Use Bearer Token or Basic Auth for authentication
- Rotate credentials regularly
- Pass additional security information via custom headers

## Troubleshooting

### Common Issues

**Connection Timeout**
- Check if the target HTTP service is running
- Verify the URL is correct
- Check network and firewall settings
- Increase `timeout_ms` value

**Authentication Failure (HTTP 401/403)**
- Verify authentication type and credentials
- Check if Bearer Token has expired
- Confirm user permissions

**Server Error (HTTP 5xx)**
- Check target service logs
- Verify request format meets target service requirements
- Check target service load

## Summary

The Webhook connector provides real-time MQTT message forwarding to external HTTP services with support for flexible authentication, custom headers, and standard HTTP/JSON format compatible with any web service.
