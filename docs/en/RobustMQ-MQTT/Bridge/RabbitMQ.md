# RabbitMQ Connector

## Overview

The RabbitMQ connector is a data integration component provided by RobustMQ for bridging MQTT messages to RabbitMQ message queue systems. This connector supports the AMQP protocol and provides reliable message delivery capabilities, suitable for message routing, asynchronous processing, microservice communication, and enterprise integration scenarios.

## Configuration

### Connector Configuration

The RabbitMQ connector uses the `RabbitMQConnectorConfig` structure for configuration:

```rust
pub struct RabbitMQConnectorConfig {
    pub server: String,              // RabbitMQ server address
    pub port: u16,                   // RabbitMQ server port
    pub username: String,            // Username
    pub password: String,            // Password
    pub virtual_host: String,        // Virtual host
    pub exchange: String,            // Exchange name
    pub routing_key: String,         // Routing key
    pub delivery_mode: DeliveryMode, // Delivery mode
    pub enable_tls: bool,            // Enable TLS/SSL
}
```

### Configuration Parameters

| Parameter | Type | Required | Description | Example |
|-----------|------|----------|-------------|---------|
| `server` | String | Yes | RabbitMQ server address | `localhost` or `rabbitmq.example.com` |
| `port` | u16 | No | RabbitMQ server port, default 5672 | `5672` (non-TLS) or `5671` (TLS) |
| `username` | String | Yes | Username | `guest` |
| `password` | String | Yes | Password | `guest` |
| `virtual_host` | String | No | Virtual host, default `/` | `/` or `/production` |
| `exchange` | String | Yes | Exchange name | `mqtt_messages` |
| `routing_key` | String | Yes | Routing key for message routing | `sensor.data` |
| `delivery_mode` | String | No | Delivery mode, default `NonPersistent` | `NonPersistent` or `Persistent` |
| `enable_tls` | bool | No | Enable TLS/SSL, default false | `true` or `false` |

### Delivery Mode

| Mode | Description | AMQP Value | Persistence |
|------|-------------|------------|-------------|
| `NonPersistent` | Non-persistent messages, higher performance | 1 | ❌ |
| `Persistent` | Persistent messages, survive restarts | 2 | ✅ |

### Configuration Examples

#### JSON Configuration Format

**Basic Configuration (Non-TLS)**:
```json
{
  "server": "localhost",
  "port": 5672,
  "username": "guest",
  "password": "guest",
  "virtual_host": "/",
  "exchange": "mqtt_messages",
  "routing_key": "sensor.data",
  "delivery_mode": "Persistent",
  "enable_tls": false
}
```

**TLS Configuration**:
```json
{
  "server": "rabbitmq.example.com",
  "port": 5671,
  "username": "mqtt_user",
  "password": "secure_password",
  "virtual_host": "/production",
  "exchange": "mqtt_exchange",
  "routing_key": "iot.sensor.#",
  "delivery_mode": "Persistent",
  "enable_tls": true
}
```

#### Complete Connector Configuration
```json
{
  "cluster_name": "default",
  "connector_name": "rabbitmq_connector_01",
  "connector_type": "RabbitMQ",
  "config": "{\"server\": \"localhost\", \"port\": 5672, \"username\": \"guest\", \"password\": \"guest\", \"virtual_host\": \"/\", \"exchange\": \"mqtt_messages\", \"routing_key\": \"sensor.data\", \"delivery_mode\": \"Persistent\", \"enable_tls\": false}",
  "topic_id": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## Message Format

### Transport Format
The RabbitMQ connector converts MQTT messages to JSON format before sending them to RabbitMQ Exchange, with each message as an AMQP message.

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

### AMQP Message Properties

| Property | Description |
|----------|-------------|
| `delivery_mode` | Delivery mode (1=non-persistent, 2=persistent) |
| `exchange` | Target exchange |
| `routing_key` | Routing key for message routing |
| `content_type` | `application/json` |
| `body` | Message content (Record in JSON format) |

## RabbitMQ Configuration

### Exchange Types

The RabbitMQ connector works with different exchange types:

#### 1. Direct Exchange
```bash
# Create Direct Exchange
rabbitmqadmin declare exchange name=mqtt_direct type=direct durable=true

# Bind queue
rabbitmqadmin declare binding source=mqtt_direct destination=sensor_queue routing_key=sensor.data
```

#### 2. Topic Exchange
```bash
# Create Topic Exchange
rabbitmqadmin declare exchange name=mqtt_topic type=topic durable=true

# Bind queues (with wildcards)
rabbitmqadmin declare binding source=mqtt_topic destination=sensor_queue routing_key="sensor.*.data"
rabbitmqadmin declare binding source=mqtt_topic destination=all_queue routing_key="sensor.#"
```

#### 3. Fanout Exchange
```bash
# Create Fanout Exchange
rabbitmqadmin declare exchange name=mqtt_fanout type=fanout durable=true

# Bind multiple queues (no routing key needed)
rabbitmqadmin declare binding source=mqtt_fanout destination=queue1
rabbitmqadmin declare binding source=mqtt_fanout destination=queue2
```

### Queue Configuration Examples

```bash
# Create durable queue
rabbitmqadmin declare queue name=mqtt_messages durable=true

# Create queue with TTL
rabbitmqadmin declare queue name=mqtt_messages_ttl \
  durable=true \
  arguments='{"x-message-ttl": 86400000}'

# Create queue with max length
rabbitmqadmin declare queue name=mqtt_messages_limited \
  durable=true \
  arguments='{"x-max-length": 100000}'
```

## Using robust-ctl to Create RabbitMQ Connector

### Basic Syntax

Use the `robust-ctl` command-line tool to create and manage RabbitMQ connectors:

```bash
robust-ctl mqtt connector create \
  --connector-name <connector-name> \
  --connector-type <connector-type> \
  --config <configuration> \
  --topic-id <topic-id>
```

### Creating RabbitMQ Connector

#### 1. Basic Creation Command

```bash
# Create RabbitMQ connector
robust-ctl mqtt connector create \
  --connector-name "rabbitmq_connector_01" \
  --connector-type "RabbitMQ" \
  --config '{"server": "localhost", "port": 5672, "username": "guest", "password": "guest", "virtual_host": "/", "exchange": "mqtt_messages", "routing_key": "sensor.data", "delivery_mode": "Persistent", "enable_tls": false}' \
  --topic-id "sensor/data"
```

#### 2. Parameter Description

| Parameter | Description | Example |
|-----------|-------------|---------|
| `--connector-name` | Connector name, must be unique | `rabbitmq_connector_01` |
| `--connector-type` | Connector type, must be `RabbitMQ` | `RabbitMQ` |
| `--config` | JSON format configuration | `{"server": "localhost", ...}` |
| `--topic-id` | MQTT topic to monitor | `sensor/data` |

#### 3. Advanced Configuration Examples

**Persistent Message Configuration**:
```bash
robust-ctl mqtt connector create \
  --connector-name "rabbitmq_persistent" \
  --connector-type "RabbitMQ" \
  --config '{"server": "rabbitmq.local", "port": 5672, "username": "mqtt_user", "password": "mqtt_pass", "virtual_host": "/production", "exchange": "mqtt_persistent", "routing_key": "iot.sensor.data", "delivery_mode": "Persistent", "enable_tls": false}' \
  --topic-id "iot/sensors/+/data"
```

**TLS Secure Connection Configuration**:
```bash
robust-ctl mqtt connector create \
  --connector-name "rabbitmq_secure" \
  --connector-type "RabbitMQ" \
  --config '{"server": "rabbitmq.example.com", "port": 5671, "username": "admin", "password": "secure_pass", "virtual_host": "/secure", "exchange": "mqtt_secure", "routing_key": "secure.messages", "delivery_mode": "Persistent", "enable_tls": true}' \
  --topic-id "secure/#"
```

### Managing Connectors

#### 1. List All Connectors
```bash
# List all connectors
robust-ctl mqtt connector list

# List specific connector
robust-ctl mqtt connector list --connector-name "rabbitmq_connector_01"
```

#### 2. Delete Connector
```bash
# Delete specific connector
robust-ctl mqtt connector delete --connector-name "rabbitmq_connector_01"
```

### Complete Operation Example

#### Scenario: Creating IoT Message Routing System

```bash
# 1. Create sensor data RabbitMQ connector (Topic Exchange)
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_rabbitmq" \
  --connector-type "RabbitMQ" \
  --config '{"server": "rabbitmq.iot.local", "port": 5672, "username": "iot_user", "password": "iot_pass", "virtual_host": "/iot", "exchange": "iot_topic_exchange", "routing_key": "sensor.temperature.data", "delivery_mode": "Persistent", "enable_tls": false}' \
  --topic-id "iot/sensors/temperature/+"

# 2. Create device status RabbitMQ connector
robust-ctl mqtt connector create \
  --connector-name "device_status_rabbitmq" \
  --connector-type "RabbitMQ" \
  --config '{"server": "rabbitmq.iot.local", "port": 5672, "username": "iot_user", "password": "iot_pass", "virtual_host": "/iot", "exchange": "iot_topic_exchange", "routing_key": "device.status", "delivery_mode": "Persistent", "enable_tls": false}' \
  --topic-id "iot/devices/+/status"

# 3. Create alarm message RabbitMQ connector (high priority)
robust-ctl mqtt connector create \
  --connector-name "alarm_rabbitmq" \
  --connector-type "RabbitMQ" \
  --config '{"server": "rabbitmq.iot.local", "port": 5672, "username": "iot_user", "password": "iot_pass", "virtual_host": "/iot", "exchange": "iot_alarms", "routing_key": "alarm.critical", "delivery_mode": "Persistent", "enable_tls": false}' \
  --topic-id "iot/alarms/#"

# 4. View created connectors
robust-ctl mqtt connector list

# 5. Test connector (publish test message)
robust-ctl mqtt publish \
  --username "test_user" \
  --password "test_pass" \
  --topic "iot/sensors/temperature/001" \
  --qos 1 \
  --message '{"temperature": 25.5, "humidity": 60, "timestamp": 1640995200}'
```

## RabbitMQ Deployment Examples

### Docker Deployment

```bash
# Start RabbitMQ service (with management UI)
docker run -d \
  --name rabbitmq \
  -p 5672:5672 \
  -p 15672:15672 \
  -e RABBITMQ_DEFAULT_USER=guest \
  -e RABBITMQ_DEFAULT_PASS=guest \
  rabbitmq:3-management

# Wait for service to start
sleep 10

# Create Exchange
docker exec rabbitmq rabbitmqadmin declare exchange name=mqtt_messages type=topic durable=true

# Create queue
docker exec rabbitmq rabbitmqadmin declare queue name=sensor_data durable=true

# Bind queue to Exchange
docker exec rabbitmq rabbitmqadmin declare binding \
  source=mqtt_messages \
  destination=sensor_data \
  routing_key="sensor.*.data"
```

### Docker Compose Deployment

```yaml
version: '3.8'

services:
  rabbitmq:
    image: rabbitmq:3-management
    container_name: rabbitmq
    ports:
      - "5672:5672"   # AMQP port
      - "15672:15672" # Management UI port
    environment:
      RABBITMQ_DEFAULT_USER: mqtt_user
      RABBITMQ_DEFAULT_PASS: mqtt_pass
      RABBITMQ_DEFAULT_VHOST: /iot
    volumes:
      - rabbitmq_data:/var/lib/rabbitmq
    healthcheck:
      test: ["CMD", "rabbitmq-diagnostics", "ping"]
      interval: 30s
      timeout: 10s
      retries: 5

  robustmq:
    image: robustmq/robustmq:latest
    container_name: robustmq
    ports:
      - "1883:1883"
      - "8883:8883"
    depends_on:
      - rabbitmq
    environment:
      - RABBITMQ_SERVER=rabbitmq
      - RABBITMQ_PORT=5672

volumes:
  rabbitmq_data:
```

### Access Management UI

After startup, access RabbitMQ management UI:
- URL: `http://localhost:15672`
- Username: `guest` (or configured username)
- Password: `guest` (or configured password)

## Message Routing Examples

### 1. Single Queue Routing

```bash
# Create Exchange
rabbitmqadmin declare exchange name=mqtt_direct type=direct durable=true

# Create queue
rabbitmqadmin declare queue name=sensor_queue durable=true

# Bind
rabbitmqadmin declare binding \
  source=mqtt_direct \
  destination=sensor_queue \
  routing_key=sensor.data
```

**Connector Configuration**:
```json
{
  "exchange": "mqtt_direct",
  "routing_key": "sensor.data"
}
```

### 2. Topic Routing (Wildcards)

```bash
# Create Topic Exchange
rabbitmqadmin declare exchange name=mqtt_topic type=topic durable=true

# Create multiple queues
rabbitmqadmin declare queue name=temperature_queue durable=true
rabbitmqadmin declare queue name=humidity_queue durable=true
rabbitmqadmin declare queue name=all_sensors_queue durable=true

# Bind (using wildcards)
rabbitmqadmin declare binding source=mqtt_topic destination=temperature_queue routing_key="sensor.temperature.*"
rabbitmqadmin declare binding source=mqtt_topic destination=humidity_queue routing_key="sensor.humidity.*"
rabbitmqadmin declare binding source=mqtt_topic destination=all_sensors_queue routing_key="sensor.#"
```

**Connector Configuration**:
```json
{
  "exchange": "mqtt_topic",
  "routing_key": "sensor.temperature.room1"
}
```

### 3. Fanout Routing

```bash
# Create Fanout Exchange
rabbitmqadmin declare exchange name=mqtt_fanout type=fanout durable=true

# Create multiple queues
rabbitmqadmin declare queue name=logger_queue durable=true
rabbitmqadmin declare queue name=analytics_queue durable=true
rabbitmqadmin declare queue name=storage_queue durable=true

# Bind (no routing key needed)
rabbitmqadmin declare binding source=mqtt_fanout destination=logger_queue
rabbitmqadmin declare binding source=mqtt_fanout destination=analytics_queue
rabbitmqadmin declare binding source=mqtt_fanout destination=storage_queue
```

**Connector Configuration**:
```json
{
  "exchange": "mqtt_fanout",
  "routing_key": ""
}
```

## Performance Optimization

### Connector Optimization

1. **Choose Appropriate Delivery Mode**
   - High performance scenario: Use `NonPersistent`
   - High reliability requirement: Use `Persistent`

2. **Connection Management**
   - Connector automatically manages connections and channels
   - Supports automatic reconnection
   - Uses Publisher Confirms to ensure message delivery

### RabbitMQ Server Optimization

```ini
# rabbitmq.conf configuration example

# Increase file descriptor limit
vm_memory_high_watermark.relative = 0.6

# Disk space alarm threshold
disk_free_limit.absolute = 2GB

# Message persistence optimization
queue_index_embed_msgs_below = 4096
```

### Queue Optimization Recommendations

```bash
# Create optimized queue
rabbitmqadmin declare queue name=optimized_queue \
  durable=true \
  arguments='{
    "x-max-length": 100000,
    "x-message-ttl": 86400000,
    "x-queue-mode": "lazy"
  }'
```

## Monitoring and Troubleshooting

### Log Monitoring

The connector outputs detailed operation logs:

```
INFO  Connecting to RabbitMQ at localhost:5672 (exchange: mqtt_messages, routing_key: sensor.data)
INFO  Successfully connected to RabbitMQ
INFO  RabbitMQ connector thread exited successfully
ERROR Connector rabbitmq_connector_01 failed to write data to RabbitMQ exchange mqtt_messages, error: connection timeout
```

### Using RabbitMQ Management Tools

```bash
# View queue status
rabbitmqctl list_queues name messages consumers

# View Exchange status
rabbitmqctl list_exchanges name type

# View bindings
rabbitmqctl list_bindings

# View connections
rabbitmqctl list_connections
```

### Common Issues

#### 1. Connection Failure
```bash
# Check RabbitMQ service status
rabbitmqctl status

# Check if port is open
telnet localhost 5672

# View RabbitMQ logs
tail -f /var/log/rabbitmq/rabbit@hostname.log
```

#### 2. Authentication Failure
```bash
# Create user
rabbitmqctl add_user mqtt_user mqtt_pass

# Set permissions
rabbitmqctl set_permissions -p / mqtt_user ".*" ".*" ".*"

# Set user tags
rabbitmqctl set_user_tags mqtt_user administrator
```

#### 3. Messages Not Reaching Queue
```bash
# Check Exchange and Queue bindings
rabbitmqctl list_bindings

# Check if routing key is correct
# Topic Exchange: use . as separator
# * matches one word
# # matches zero or more words
```

#### 4. Performance Issues
```bash
# View queue backlog
rabbitmqctl list_queues name messages

# View memory usage
rabbitmqctl status | grep memory

# Enable lazy queue mode (reduce memory usage)
rabbitmqadmin declare queue name=lazy_queue \
  durable=true \
  arguments='{"x-queue-mode": "lazy"}'
```

## Summary

The RabbitMQ connector is an important component of the RobustMQ data integration system, providing powerful message routing and distribution capabilities. With proper configuration and usage, it can meet various business requirements such as message queuing, asynchronous processing, microservice communication, and enterprise integration.

This connector fully leverages RabbitMQ's AMQP protocol and routing mechanisms, combined with Rust's memory safety and zero-cost abstraction advantages, to achieve efficient and reliable message transmission. It supports multiple Exchange types (Direct, Topic, Fanout) and delivery modes (persistent/non-persistent), making it an essential tool for building modern message-driven architectures and enterprise integration platforms.

### Key Features

✅ **AMQP Protocol Support**: Fully compatible with AMQP 0-9-1 protocol
✅ **Flexible Routing Mechanism**: Supports Direct, Topic, Fanout Exchange
✅ **Message Persistence**: Optional message persistence for data safety
✅ **TLS/SSL Support**: Supports encrypted transmission for communication security
✅ **Publisher Confirms**: Confirmation mechanism ensures message delivery
✅ **Virtual Host Isolation**: Supports multi-tenant environments
