# MQTT Bridge Connector

## Overview

The MQTT Bridge connector is a data integration component provided by RobustMQ for forwarding local MQTT messages to a remote MQTT Broker. This connector implements the Sink (outbound) direction of MQTT bridging, supporting MQTT 3.1, 3.1.1, and 5.0 protocols. It is suitable for cross-cluster message synchronization, multi-tier IoT data reporting, and edge-to-cloud message forwarding scenarios.

## Features

- MQTT 3.1 / 3.1.1 / 5.0 protocol support
- TLS encrypted connections
- Username/password authentication
- QoS 0/1/2 support
- Custom topic prefix
- Retain flag support
- Batch message forwarding

## Configuration

### Connector Configuration

The MQTT Bridge connector uses the `MqttBridgeConnectorConfig` struct:

```rust
pub struct MqttBridgeConnectorConfig {
    pub server: String,                       // Remote MQTT Broker address
    pub client_id_prefix: Option<String>,     // Client ID prefix
    pub username: Option<String>,             // Username
    pub password: Option<String>,             // Password
    pub protocol_version: MqttProtocolVersion, // Protocol version (V3/V4/V5)
    pub keepalive_secs: u64,                  // Keep-alive interval (seconds)
    pub connect_timeout_secs: u64,            // Connection timeout (seconds)
    pub enable_tls: bool,                     // Enable TLS
    pub topic_prefix: Option<String>,         // Topic prefix
    pub qos: i32,                             // QoS level (0/1/2)
    pub retain: bool,                         // Retain flag
    pub max_retries: u32,                     // Max retries
}
```

### Configuration Parameters

| Parameter | Type | Required | Default | Description | Example |
|-----------|------|----------|---------|-------------|---------|
| `server` | String | Yes | - | Remote MQTT Broker address, max 512 characters | `tcp://broker.example.com:1883` |
| `client_id_prefix` | String | No | `robustmq-bridge` | Client ID prefix, max 64 characters | `my-bridge` |
| `username` | String | No | - | Connection username | `admin` |
| `password` | String | No | - | Connection password | `password123` |
| `protocol_version` | String | No | `v5` | MQTT protocol version: `v3`, `v4`, `v5` | `v5` |
| `keepalive_secs` | Number | No | `60` | Keep-alive interval (seconds), range: 1-65535 | `60` |
| `connect_timeout_secs` | Number | No | `10` | Connection timeout (seconds), range: 1-300 | `10` |
| `enable_tls` | Boolean | No | `false` | Enable TLS encryption | `true` |
| `topic_prefix` | String | No | - | Topic prefix for forwarded messages | `remote/` |
| `qos` | Number | No | `1` | Message QoS level: 0, 1, 2 | `1` |
| `retain` | Boolean | No | `false` | Set retain flag on messages | `false` |
| `max_retries` | Number | No | `3` | Max retry attempts on failure, range: 0-10 | `3` |

### Configuration Examples

#### JSON Configuration

**Basic Configuration**
```json
{
  "server": "tcp://remote-broker:1883"
}
```

**With Authentication and TLS**
```json
{
  "server": "ssl://remote-broker:8883",
  "username": "bridge_user",
  "password": "bridge_pass",
  "enable_tls": true,
  "protocol_version": "v5",
  "keepalive_secs": 30,
  "connect_timeout_secs": 15
}
```

**With Topic Prefix and QoS**
```json
{
  "server": "tcp://remote-broker:1883",
  "client_id_prefix": "edge-bridge",
  "topic_prefix": "cloud/edge01",
  "qos": 2,
  "retain": true,
  "max_retries": 5
}
```

#### Full Connector Configuration

```json
{
  "cluster_name": "default",
  "connector_name": "mqtt_bridge_01",
  "connector_type": "mqtt",
  "config": "{\"server\": \"tcp://remote-broker:1883\", \"username\": \"admin\", \"password\": \"secret\", \"topic_prefix\": \"remote/\", \"qos\": 1}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## Topic Mapping Rules

The MQTT Bridge connector supports topic prefix functionality to rewrite target topics when forwarding messages:

| Source Topic | topic_prefix | Target Topic |
|-------------|-------------|-------------|
| `sensor/temperature` | None | `sensor/temperature` |
| `sensor/temperature` | `remote/` | `remote/sensor/temperature` |
| `device/status` | `cloud/edge01` | `cloud/edge01/device/status` |

## Using robust-ctl to Create MQTT Bridge Connector

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
  --connector-name "mqtt_bridge_01" \
  --connector-type "mqtt" \
  --config '{"server": "tcp://remote-broker:1883"}' \
  --topic-id "sensor/data"
```

#### 2. With Authentication

```bash
robust-ctl mqtt connector create \
  --connector-name "mqtt_bridge_auth" \
  --connector-type "mqtt" \
  --config '{"server": "ssl://remote-broker:8883", "username": "bridge_user", "password": "bridge_pass", "enable_tls": true, "qos": 2}' \
  --topic-id "device/status"
```

#### 3. With Topic Prefix

```bash
robust-ctl mqtt connector create \
  --connector-name "mqtt_bridge_prefix" \
  --connector-type "mqtt" \
  --config '{"server": "tcp://remote-broker:1883", "topic_prefix": "cloud/edge01", "client_id_prefix": "edge-bridge"}' \
  --topic-id "sensor/#"
```

### Managing Connectors

```bash
# List all connectors
robust-ctl mqtt connector list

# View specific connector
robust-ctl mqtt connector list --connector-name "mqtt_bridge_01"

# Delete connector
robust-ctl mqtt connector delete --connector-name "mqtt_bridge_01"
```

### Full Operation Example

#### Scenario: Edge-to-Cloud Message Forwarding

```bash
# 1. Create sensor data bridge
robust-ctl mqtt connector create \
  --connector-name "edge_to_cloud_sensors" \
  --connector-type "mqtt" \
  --config '{"server": "ssl://cloud-broker:8883", "username": "edge01", "password": "secret", "enable_tls": true, "topic_prefix": "cloud/edge01", "qos": 1}' \
  --topic-id "sensor/+"

# 2. Create device status bridge
robust-ctl mqtt connector create \
  --connector-name "edge_to_cloud_status" \
  --connector-type "mqtt" \
  --config '{"server": "ssl://cloud-broker:8883", "username": "edge01", "password": "secret", "enable_tls": true, "topic_prefix": "cloud/edge01", "qos": 1}' \
  --topic-id "device/status"

# 3. View created connectors
robust-ctl mqtt connector list
```

## Performance Optimization

### 1. Connection Settings
- Set `connect_timeout_secs` based on network latency
- Increase `keepalive_secs` for WAN bridging
- Use a unique `client_id_prefix` to avoid client ID conflicts

### 2. QoS Selection
- Use QoS 0 for high-throughput scenarios where message loss is acceptable
- Use QoS 1 for reliable delivery
- Use QoS 2 only when exactly-once semantics are required (higher overhead)

### 3. Security Recommendations
- Enable TLS in production (`enable_tls: true`)
- Use dedicated bridge credentials
- Rotate credentials regularly

## Monitoring and Troubleshooting

### 1. Check Connector Status

```bash
robust-ctl mqtt connector list --connector-name "mqtt_bridge_01"
```

### 2. Common Issues

**Issue 1: Connection Failure**
- Check if the remote MQTT Broker is running
- Verify the `server` address and port
- Check network connectivity and firewall rules
- Confirm username/password credentials

**Issue 2: Message Loss**
- Review QoS settings
- Ensure the remote Broker's message queue is not full
- Check connector logs for send errors

**Issue 3: High Latency**
- Check network latency
- Consider lowering QoS level for better throughput
- Monitor remote Broker load

## Current Limitations

- Only Sink (outbound) direction is supported. Source (inbound) will be added in future releases.
- Topic wildcard mapping rules are not yet supported.

## Summary

The MQTT Bridge connector is an important component of the RobustMQ data integration system, providing cross-cluster MQTT message forwarding capabilities. With simple configuration, you can achieve:

- **Cross-Cluster Sync**: Synchronize messages between MQTT clusters
- **Edge-to-Cloud**: Forward edge device messages to cloud Brokers
- **Protocol Compatibility**: Supports MQTT 3.1 / 3.1.1 / 5.0, compatible with various Brokers
- **Secure Transport**: TLS encryption and username/password authentication
