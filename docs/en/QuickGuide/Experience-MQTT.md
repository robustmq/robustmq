# Experience RobustMQ MQTT

This guide will help you quickly experience RobustMQ's MQTT functionality, including starting the Broker, viewing cluster configuration, and sending and consuming MQTT messages.

## Table of Contents

- [Run Broker](#run-broker)
- [View Cluster Configuration](#view-cluster-configuration)
- [Send MQTT Messages](#send-mqtt-messages)
- [Consume MQTT Messages](#consume-mqtt-messages)
- [Advanced Features](#advanced-features)

## Run Broker

### 1. Download and Extract Binary Package

First, we need to download and extract the RobustMQ binary package:

```bash
# Download the latest version binary package (using v1.0.0 as example)
wget https://github.com/robustmq/robustmq/releases/download/v1.0.0/robustmq-v1.0.0-linux-amd64.tar.gz

# Extract the binary package
tar -xzf robustmq-v1.0.0-linux-amd64.tar.gz

# Enter the extracted directory
cd robustmq-v1.0.0-linux-amd64
```

### 2. Start RobustMQ Broker

```bash
# Start Broker (using default configuration)
./bin/broker-server start

# Or start with configuration file
./bin/broker-server start config/server.toml

# Start in background
nohup ./bin/broker-server start > broker.log 2>&1 &
```

### 3. Verify Broker Startup Status

After the Broker starts successfully, you should see output similar to the following:

```bash
[INFO] RobustMQ Broker starting...
[INFO] MQTT server listening on 0.0.0.0:1883
[INFO] Admin server listening on 0.0.0.0:8080
[INFO] Broker started successfully
```

### 4. Check Service Status

```bash
# Check if MQTT port is listening
netstat -tlnp | grep 1883

# Check if admin port is listening
netstat -tlnp | grep 8080

# Or use ss command
ss -tlnp | grep :1883
ss -tlnp | grep :8080
```

## View Cluster Configuration

### Using robust-ctl Command Line Tool

RobustMQ provides a powerful command-line management tool `robust-ctl`. Let's view the cluster configuration:

```bash
# View cluster configuration
./bin/robust-ctl cluster config get
```

### Configuration Information Interpretation

After executing `robust-ctl cluster config get`, you will see configuration information similar to the following:

```json
{
  "cluster": {
    "name": "robustmq-cluster",
    "nodes": [
      {
        "id": "node-1",
        "address": "127.0.0.1:9090",
        "role": "leader",
        "status": "active"
      }
    ],
    "replication_factor": 1,
    "consensus": "raft"
  },
  "mqtt": {
    "port": 1883,
    "max_connections": 10000,
    "keep_alive": 60,
    "retain_available": true,
    "wildcard_subscription_available": true
  }
}
```

## Send MQTT Messages

### Using MQTTX to Send Messages

```bash
# Send simple message
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"

# Send QoS 1 message
mqttx pub -h localhost -p 1883 -t "test/qos1" -m "QoS 1 message" -q 1

# Send retained message
mqttx pub -h localhost -p 1883 -t "test/retained" -m "Retained message" -r

# Send JSON format message
mqttx pub -h localhost -p 1883 -t "sensors/temperature" -m '{"value": 25.5, "unit": "celsius", "timestamp": "2024-01-01T12:00:00Z"}'
```

## Consume MQTT Messages

### Using MQTTX to Subscribe to Messages

```bash
# Subscribe to single topic
mqttx sub -h localhost -p 1883 -t "test/topic"

# Subscribe to wildcard topics
mqttx sub -h localhost -p 1883 -t "test/+"  # Single-level wildcard
mqttx sub -h localhost -p 1883 -t "test/#"  # Multi-level wildcard

# Subscribe to QoS 1 messages
mqttx sub -h localhost -p 1883 -t "test/qos1" -q 1

# Subscribe and display detailed information
mqttx sub -h localhost -p 1883 -t "test/topic" --verbose
```

## Advanced Features

### Performance Testing

```bash
# Use MQTTX for performance testing
mqttx bench pub -h localhost -p 1883 -t "test/bench" -c 10 -C 100

# Test subscription performance
mqttx bench sub -h localhost -p 1883 -t "test/bench" -c 50
```

## Complete Example

Let's experience RobustMQ MQTT functionality through a complete example:

### Step 1: Start Broker

```bash
# Terminal 1: Start Broker
./bin/broker-server start
```

### Step 2: View Cluster Configuration

```bash
# Terminal 2: View configuration
./bin/robust-ctl cluster config get
```

### Step 3: Subscribe to Messages

```bash
# Terminal 3: Subscribe to messages
mqttx sub -h localhost -p 1883 -t "demo/temperature" --verbose
```

### Step 4: Send Messages

```bash
# Terminal 4: Send messages
mqttx pub -h localhost -p 1883 -t "demo/temperature" -m '{"sensor": "temp-001", "value": 23.5, "unit": "celsius"}'
```
