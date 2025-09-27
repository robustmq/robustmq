# Apache Pulsar Connector

RobustMQ supports bridging MQTT messages to Apache Pulsar messaging system, enabling high-throughput, low-latency message transmission and persistent storage.

## Features

- **High-Performance Message Transmission**: Based on Pulsar's distributed architecture, supports high-concurrent message processing
- **Multiple Authentication Methods**: Supports Token, OAuth2, and Basic authentication
- **Message Persistence**: Leverages Pulsar's persistence mechanism to ensure no message loss
- **Automatic Retry Mechanism**: Built-in error retry and fault recovery mechanisms
- **Asynchronous Non-blocking**: Uses asynchronous sending mode to improve system throughput

## Configuration

### Connector Configuration Structure

```json
{
  "server": "pulsar://localhost:6650",
  "topic": "mqtt-messages",
  "token": "your-auth-token",
  "oauth": "{\"issuer_url\":\"https://auth.example.com\",\"client_id\":\"client\",\"client_secret\":\"secret\"}",
  "basic_name": "username",
  "basic_password": "password"
}
```

### Configuration Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `server` | String | ✅ | Pulsar server address, format: `pulsar://host:port` |
| `topic` | String | ✅ | Target Pulsar topic name |
| `token` | String | ❌ | Token authentication token |
| `oauth` | String | ❌ | OAuth2 authentication configuration (JSON format) |
| `basic_name` | String | ❌ | Basic authentication username |
| `basic_password` | String | ❌ | Basic authentication password |

### Authentication Methods

#### 1. Token Authentication
```json
{
  "server": "pulsar://localhost:6650",
  "topic": "mqtt-messages",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

#### 2. OAuth2 Authentication
```json
{
  "server": "pulsar://localhost:6650",
  "topic": "mqtt-messages",
  "oauth": "{\"issuer_url\":\"https://auth.example.com\",\"client_id\":\"mqtt-client\",\"client_secret\":\"secret123\"}"
}
```

#### 3. Basic Authentication
```json
{
  "server": "pulsar://localhost:6650",
  "topic": "mqtt-messages",
  "basic_name": "mqtt-user",
  "basic_password": "mqtt-password"
}
```

#### 4. No Authentication (Development Environment)
```json
{
  "server": "pulsar://localhost:6650",
  "topic": "mqtt-messages"
}
```

## Usage Examples

### 1. Create Pulsar Connector

Use RobustMQ management API to create a Pulsar connector:

```bash
curl -X POST http://localhost:8080/api/v1/connector \
  -H "Content-Type: application/json" \
  -d '{
    "cluster_name": "default-cluster",
    "connector_name": "pulsar-connector-1",
    "connector_type": "pulsar",
    "topic_id": "mqtt/sensor/+/data",
    "config": "{\"server\":\"pulsar://localhost:6650\",\"topic\":\"sensor-data\"}"
  }'
```

### 2. Check Connector Status

```bash
curl -X GET http://localhost:8080/api/v1/connector/pulsar-connector-1
```

### 3. Update Connector Configuration

```bash
curl -X PUT http://localhost:8080/api/v1/connector/pulsar-connector-1 \
  -H "Content-Type: application/json" \
  -d '{
    "config": "{\"server\":\"pulsar://localhost:6650\",\"topic\":\"sensor-data-v2\",\"token\":\"new-token\"}"
  }'
```

### 4. Delete Connector

```bash
curl -X DELETE http://localhost:8080/api/v1/connector/pulsar-connector-1
```

## Message Format

### MQTT Message Conversion

When RobustMQ converts MQTT messages to Pulsar messages, it maintains the following format:

```json
{
  "topic": "mqtt/sensor/temperature/data",
  "payload": "25.6",
  "qos": 1,
  "retain": false,
  "timestamp": 1703123456789,
  "client_id": "sensor-001",
  "headers": {
    "content-type": "application/json",
    "device-id": "sensor-001"
  }
}
```

### Pulsar Message Properties

| Property | Description |
|----------|-------------|
| `key` | Uses MQTT client ID as message key |
| `properties` | Contains MQTT message metadata information |
| `payload` | Original payload data of MQTT message |
| `event_time` | Publish timestamp of MQTT message |

## Deployment Configuration

### Docker Compose Example

```yaml
version: '3.8'
services:
  pulsar:
    image: apachepulsar/pulsar:3.1.0
    command: bin/pulsar standalone
    ports:
      - "6650:6650"
      - "8080:8080"
    environment:
      - PULSAR_MEM="-Xms512m -Xmx512m -XX:MaxDirectMemorySize=256m"

  robustmq:
    image: robustmq/robustmq:latest
    ports:
      - "1883:1883"
      - "8883:8883"
      - "8080:8080"
    depends_on:
      - pulsar
    environment:
      - ROBUSTMQ_PULSAR_SERVER=pulsar://pulsar:6650
```

### Kubernetes Deployment

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: robustmq-pulsar-config
data:
  connector.json: |
    {
      "server": "pulsar://pulsar-broker:6650",
      "topic": "mqtt-messages",
      "token": "your-token-here"
    }
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: robustmq-with-pulsar
spec:
  replicas: 3
  selector:
    matchLabels:
      app: robustmq
  template:
    metadata:
      labels:
        app: robustmq
    spec:
      containers:
      - name: robustmq
        image: robustmq/robustmq:latest
        ports:
        - containerPort: 1883
        - containerPort: 8080
        volumeMounts:
        - name: config
          mountPath: /app/config
      volumes:
      - name: config
        configMap:
          name: robustmq-pulsar-config
```

## Performance Tuning

### Connector Configuration Optimization

```json
{
  "server": "pulsar://localhost:6650",
  "topic": "mqtt-messages",
  "batch_size": 1000,
  "batch_timeout_ms": 100,
  "compression_type": "LZ4",
  "send_timeout_ms": 30000
}
```

### Pulsar Client Optimization

- **Batching**: Enable message batching to improve throughput
- **Compression**: Use LZ4 or ZSTD compression to reduce network transmission
- **Connection Pooling**: Reuse Pulsar client connections
- **Async Sending**: Use asynchronous mode to avoid blocking

## Monitoring Metrics

RobustMQ provides the following monitoring metrics for Pulsar connector:

### Message Metrics
- `pulsar_messages_sent_total`: Total number of messages sent to Pulsar
- `pulsar_messages_failed_total`: Total number of failed message sends
- `pulsar_send_duration_ms`: Message sending duration distribution

### Connection Metrics
- `pulsar_connections_active`: Number of active Pulsar connections
- `pulsar_connection_errors_total`: Total connection errors
- `pulsar_reconnections_total`: Number of reconnections

### Performance Metrics
- `pulsar_throughput_messages_per_sec`: Message throughput (messages/sec)
- `pulsar_throughput_bytes_per_sec`: Data throughput (bytes/sec)

## Troubleshooting

### Common Issues

#### 1. Connection Failure
```bash
# Check Pulsar service status
curl http://localhost:8080/admin/v2/clusters

# Verify network connectivity
telnet localhost 6650
```

#### 2. Authentication Failure
```bash
# Verify Token validity
curl -H "Authorization: Bearer YOUR_TOKEN" \
     http://localhost:8080/admin/v2/tenants
```

#### 3. Message Send Failure
```bash
# Check if topic exists
curl http://localhost:8080/admin/v2/persistent/public/default/mqtt-messages
```

### Log Analysis

```bash
# View connector logs
grep "PulsarBridgePlugin" /var/log/robustmq/robustmq.log

# View Pulsar-related errors
grep "pulsar" /var/log/robustmq/robustmq.log | grep ERROR
```

### Performance Diagnosis

```bash
# Check message backlog
curl http://localhost:8080/admin/v2/persistent/public/default/mqtt-messages/stats

# Monitor connector performance
curl http://localhost:9090/metrics | grep pulsar_
```

## Best Practices

### 1. Topic Design
- Use meaningful topic names
- Consider partitioning strategy to improve parallelism
- Set appropriate message retention policies

### 2. Security Configuration
- Authentication must be enabled in production environments
- Use TLS encryption for transmission
- Regularly rotate authentication credentials

### 3. Performance Optimization
- Adjust batch size based on message volume
- Enable message compression to save bandwidth
- Monitor and adjust timeout parameters

### 4. Operations Management
- Set up comprehensive monitoring and alerting
- Regularly backup connector configurations
- Establish fault recovery procedures

Through the Apache Pulsar connector, RobustMQ can efficiently bridge MQTT messages to the Pulsar ecosystem, providing strong support for building large-scale, highly reliable IoT data pipelines.
