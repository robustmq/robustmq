# Experience RobustMQ MQTT

This guide will take you through a quick experience of RobustMQ's MQTT functionality, including starting the Broker, viewing cluster configuration, sending and consuming MQTT messages.

## Table of Contents

- [Running the Broker](#running-the-broker)
- [Send MQTT Messages](#send-mqtt-messages)
- [Consume MQTT Messages](#consume-mqtt-messages)
- [Advanced Features](#advanced-features)

## Running the Broker

### 1. Download and Extract Binary Package

First, we need to download and extract the RobustMQ binary package:

```bash
# Download the latest version binary package (using v1.0.0 as an example)
wget https://github.com/robustmq/robustmq/releases/download/v0.1.33/robustmq-v0.1.33-linux-amd64.tar.gz

# Extract the binary package
tar -xzf robustmq-v0.1.33-linux-amd64.tar.gz

# Enter the extracted directory
cd robustmq-v0.1.33-linux-amd64
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

### 4. View Cluster Status

RobustMQ provides a powerful command line management tool `robust-ctl`, let's view the cluster running status:

```bash
# View cluster running status
$ ./bin/robust-ctl status

🚀 Checking RobustMQ status...
✅ RobustMQ Status: Online
📋 Version: RobustMQ 0.1.33
🌐 Server: 127.0.0.1:8080
```
Displaying the above information indicates that the node has started successfully.

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
# Terminal 2: View cluster status
./bin/robust-ctl status
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