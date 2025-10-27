# MQTTX Testing RobustMQ User Guide

## Overview

MQTTX CLI is a powerful MQTT 5.0 command-line client tool that can be used to test and validate RobustMQ MQTT Broker functionality and performance. This guide explains how to use MQTTX CLI for publishing, subscribing, and performance testing with RobustMQ.

## Installing MQTTX CLI

### macOS

```bash
# Using Homebrew
brew install emqx/mqttx/mqttx-cli

# Or using npm
npm install -g @emqx/mqttx-cli
```

### Linux

```bash
# Using npm
npm install -g @emqx/mqttx-cli

# Or download binary
wget https://github.com/emqx/MQTTX/releases/latest/download/mqttx-cli-linux-x64
chmod +x mqttx-cli-linux-x64
sudo mv mqttx-cli-linux-x64 /usr/local/bin/mqttx
```

### Windows

```bash
# Using npm
npm install -g @emqx/mqttx-cli

# Or using Chocolatey
choco install mqttx-cli
```

### Verify Installation

```bash
mqttx --version
```

## Basic Connection Testing

### Connect to RobustMQ

```bash
# Basic connection test
mqttx conn -h localhost -p 1883 --client-id robustmq_test_client

# Authenticated connection
mqttx conn -h localhost -p 1883 -u username -P password --client-id robustmq_auth_client

# SSL connection test
mqttx conn -h localhost -p 1885 --client-id robustmq_ssl_client --protocol mqtts
```

## Message Publishing Tests

### Basic Publishing

```bash
# Publish single message
mqttx pub -h localhost -p 1883 -t 'robustmq/test/hello' -m 'Hello RobustMQ!'

# Publish with specific QoS
mqttx pub -h localhost -p 1883 -t 'robustmq/test/qos1' -m 'QoS 1 message' -q 1

# Publish retained message
mqttx pub -h localhost -p 1883 -t 'robustmq/test/retain' -m 'Retained message' -r
```

### Batch Publishing

```bash
# Publish multiple messages
mqttx pub -h localhost -p 1883 -t 'robustmq/test/batch' -m 'Message 1,Message 2,Message 3' -s ','

# Read message content from file
echo "Hello from file" > message.txt
mqttx pub -h localhost -p 1883 -t 'robustmq/test/file' -M message.txt

# JSON format message
mqttx pub -h localhost -p 1883 -t 'robustmq/test/json' -m '{"sensor":"temp","value":25.5,"unit":"celsius"}'
```

### Scheduled Publishing

```bash
# Publish every 5 seconds, total 10 times
mqttx pub -h localhost -p 1883 -t 'robustmq/test/interval' -m 'Interval message' -i 5 -c 10

# Infinite loop publishing (stop with Ctrl+C)
mqttx pub -h localhost -p 1883 -t 'robustmq/test/loop' -m 'Loop message' -i 2
```

## Message Subscription Tests

### Basic Subscription

```bash
# Subscribe to single topic
mqttx sub -h localhost -p 1883 -t 'robustmq/test/hello'

# Subscribe to wildcard topic
mqttx sub -h localhost -p 1883 -t 'robustmq/test/+' -q 1

# Subscribe to multi-level wildcard
mqttx sub -h localhost -p 1883 -t 'robustmq/test/#'
```

### Advanced Subscription

```bash
# Subscribe to multiple topics
mqttx sub -h localhost -p 1883 -t 'robustmq/sensors/+/temperature,robustmq/sensors/+/humidity'

# Authenticated subscription
mqttx sub -h localhost -p 1883 -t 'robustmq/secure/+' -u username -P password

# Show detailed information
mqttx sub -h localhost -p 1883 -t 'robustmq/test/#' --verbose

# Output to file
mqttx sub -h localhost -p 1883 -t 'robustmq/logs/#' --output-mode clean > mqtt_logs.txt
```

### Formatted Output

```bash
# JSON format output
mqttx sub -h localhost -p 1883 -t 'robustmq/test/+' --format json

# Custom format output
mqttx sub -h localhost -p 1883 -t 'robustmq/test/+' --format '[%Y-%m-%d %H:%M:%S] Topic: %t, Message: %p'
```

## Performance Testing

### Publishing Performance Tests

```bash
# Basic publish test: 1000 messages, 10 concurrent connections
mqttx bench pub -h localhost -p 1883 -t 'robustmq/bench/pub' -c 10 -C 1000

# High concurrency publish test: 100 connections, 100 messages each
mqttx bench pub -h localhost -p 1883 -t 'robustmq/bench/high' -c 100 -C 100

# Specify message size and QoS
mqttx bench pub -h localhost -p 1883 -t 'robustmq/bench/large' -c 50 -C 500 -s 1024 -q 1

# Continuous stress test (specify duration)
mqttx bench pub -h localhost -p 1883 -t 'robustmq/bench/duration' -c 20 -t 60s
```

### Subscription Performance Tests

```bash
# Basic subscription test: 50 subscriber clients
mqttx bench sub -h localhost -p 1883 -t 'robustmq/bench/sub' -c 50

# Wildcard topic subscription test
mqttx bench sub -h localhost -p 1883 -t 'robustmq/bench/+' -c 100

# QoS-specific subscription test
mqttx bench sub -h localhost -p 1883 -t 'robustmq/bench/qos2' -c 30 -q 2
```

### Comprehensive Performance Tests

```bash
# Combined publish and subscribe test
# Terminal 1: Start subscriber clients
mqttx bench sub -h localhost -p 1883 -t 'robustmq/perf/+' -c 50

# Terminal 2: Start publisher clients
mqttx bench pub -h localhost -p 1883 -t 'robustmq/perf/test' -c 10 -C 1000 -i 10
```

## SSL/TLS Connection Tests

### SSL Connection

```bash
# SSL publish test
mqttx pub -h localhost -p 1885 -t 'robustmq/ssl/test' -m 'SSL message' --protocol mqtts

# SSL subscription test
mqttx sub -h localhost -p 1885 -t 'robustmq/ssl/+' --protocol mqtts

# Specify certificate files
mqttx pub -h localhost -p 1885 -t 'robustmq/ssl/cert' -m 'Cert message' \
  --ca ca.crt --cert client.crt --key client.key
```

### WebSocket Connection Tests

```bash
# WebSocket publish
mqttx pub -h localhost -p 8083 -t 'robustmq/ws/test' -m 'WebSocket message' --protocol ws

# WebSocket subscription
mqttx sub -h localhost -p 8083 -t 'robustmq/ws/+' --protocol ws

# WebSocket SSL connection
mqttx pub -h localhost -p 8085 -t 'robustmq/wss/test' -m 'WSS message' --protocol wss
```

## MQTT 5.0 Feature Tests

### User Properties Test

```bash
# Publish message with user properties
mqttx pub -h localhost -p 1883 -t 'robustmq/mqtt5/props' -m 'MQTT 5.0 message' \
  --mqtt-version 5 \
  --user-properties 'region:beijing,sensor:temperature'

# Set message expiry interval
mqttx pub -h localhost -p 1883 -t 'robustmq/mqtt5/expire' -m 'Expiring message' \
  --mqtt-version 5 \
  --message-expiry-interval 60
```

### Topic Alias Test

```bash
# Use topic alias
mqttx pub -h localhost -p 1883 -t 'robustmq/mqtt5/very/long/topic/name' -m 'Alias message' \
  --mqtt-version 5 \
  --topic-alias 1
```

## Common Test Scenarios

### IoT Device Simulation

```bash
# Simulate temperature sensor data publishing
mqttx pub -h localhost -p 1883 \
  -t 'robustmq/sensors/temp001/data' \
  -m '{"temperature":25.5,"humidity":60,"timestamp":"2024-01-01T10:00:00Z"}' \
  -i 5 -c 100

# Simulate multiple devices
for i in {1..10}; do
  mqttx pub -h localhost -p 1883 \
    -t "robustmq/sensors/device${i}/status" \
    -m "online" \
    --client-id "device${i}" &
done
```

### Message Routing Tests

```bash
# Test topic filtering
mqttx sub -h localhost -p 1883 -t 'robustmq/+/temperature' &
mqttx sub -h localhost -p 1883 -t 'robustmq/room1/+' &

# Publish to different topics to verify routing
mqttx pub -h localhost -p 1883 -t 'robustmq/room1/temperature' -m '22.5'
mqttx pub -h localhost -p 1883 -t 'robustmq/room2/temperature' -m '24.0'
mqttx pub -h localhost -p 1883 -t 'robustmq/room1/humidity' -m '65'
```

## Performance Test Metrics

### Throughput Testing

```bash
# High throughput publish test
mqttx bench pub \
  -h localhost -p 1883 \
  -t 'robustmq/throughput/test' \
  -c 100 \
  -C 1000 \
  -i 1 \
  --verbose

# High throughput subscription test
mqttx bench sub \
  -h localhost -p 1883 \
  -t 'robustmq/throughput/+' \
  -c 200 \
  --verbose
```

### Latency Testing

```bash
# Test message latency
mqttx bench pub \
  -h localhost -p 1883 \
  -t 'robustmq/latency/test' \
  -c 1 \
  -C 100 \
  -i 100 \
  --verbose
```

### Connection Count Testing

```bash
# Large number of connections test
mqttx bench conn \
  -h localhost -p 1883 \
  -c 1000 \
  --interval 10
```

## Automated Testing Scripts

### Functional Test Script

```bash
#!/bin/bash

# RobustMQ functional test script

BROKER_HOST="localhost"
BROKER_PORT="1883"
TEST_TOPIC_PREFIX="robustmq/test"

echo "Starting RobustMQ functional tests..."

# 1. Connection test
echo "1. Testing basic connection..."
mqttx conn -h $BROKER_HOST -p $BROKER_PORT --client-id robustmq_conn_test
if [ $? -eq 0 ]; then
    echo "✅ Connection test passed"
else
    echo "❌ Connection test failed"
    exit 1
fi

# 2. Publish/Subscribe test
echo "2. Testing publish/subscribe functionality..."
mqttx sub -h $BROKER_HOST -p $BROKER_PORT -t "${TEST_TOPIC_PREFIX}/pubsub" &
SUB_PID=$!
sleep 2

mqttx pub -h $BROKER_HOST -p $BROKER_PORT -t "${TEST_TOPIC_PREFIX}/pubsub" -m "Test message"
sleep 2
kill $SUB_PID
echo "✅ Publish/Subscribe test completed"

# 3. QoS test
echo "3. Testing QoS functionality..."
for qos in 0 1 2; do
    echo "Testing QoS $qos..."
    mqttx pub -h $BROKER_HOST -p $BROKER_PORT -t "${TEST_TOPIC_PREFIX}/qos$qos" -m "QoS $qos test" -q $qos
done
echo "✅ QoS test completed"

# 4. Retained message test
echo "4. Testing retained messages..."
mqttx pub -h $BROKER_HOST -p $BROKER_PORT -t "${TEST_TOPIC_PREFIX}/retain" -m "Retained message" -r
mqttx sub -h $BROKER_HOST -p $BROKER_PORT -t "${TEST_TOPIC_PREFIX}/retain" --timeout 3
echo "✅ Retained message test completed"

# 5. Performance test
echo "5. Running performance test..."
mqttx bench pub -h $BROKER_HOST -p $BROKER_PORT -t "${TEST_TOPIC_PREFIX}/perf" -c 10 -C 100 --verbose
echo "✅ Performance test completed"

echo "All RobustMQ functional tests completed!"
```

## Common Parameters

### Connection Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `-h, --hostname` | Broker host address | `-h localhost` |
| `-p, --port` | Broker port | `-p 1883` |
| `-u, --username` | Username | `-u admin` |
| `-P, --password` | Password | `-P password` |
| `--client-id` | Client ID | `--client-id test_client` |
| `--protocol` | Protocol type | `--protocol mqtts` |

### Message Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `-t, --topic` | Topic | `-t robustmq/test` |
| `-m, --message` | Message content | `-m 'Hello World'` |
| `-q, --qos` | QoS level | `-q 1` |
| `-r, --retain` | Retained message | `-r` |
| `-i, --interval` | Send interval (seconds) | `-i 5` |
| `-c, --count` | Message count | `-c 100` |

### Benchmark Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `-c, --count` | Client count | `-c 100` |
| `-C, --message-count` | Messages per client | `-C 1000` |
| `-i, --interval` | Send interval (ms) | `-i 1000` |
| `-s, --message-size` | Message size (bytes) | `-s 1024` |
| `--verbose` | Verbose output | `--verbose` |

## RobustMQ Feature Tests

### Multi-Protocol Port Tests

```bash
# Test different protocol ports
echo "Testing MQTT TCP port..."
mqttx pub -h localhost -p 1883 -t 'robustmq/tcp/test' -m 'TCP message'

echo "Testing MQTT SSL port..."
mqttx pub -h localhost -p 1885 -t 'robustmq/ssl/test' -m 'SSL message' --protocol mqtts

echo "Testing WebSocket port..."
mqttx pub -h localhost -p 8083 -t 'robustmq/ws/test' -m 'WebSocket message' --protocol ws

echo "Testing WebSocket SSL port..."
mqttx pub -h localhost -p 8085 -t 'robustmq/wss/test' -m 'WSS message' --protocol wss
```

### Cluster Testing

```bash
# Test RobustMQ cluster multiple nodes
NODES=("localhost:1883" "node2:1883" "node3:1883")

for node in "${NODES[@]}"; do
    IFS=':' read -r host port <<< "$node"
    echo "Testing node: $host:$port"
    
    mqttx pub -h $host -p $port -t 'robustmq/cluster/test' -m "Message from $host" \
      --client-id "test_client_$host"
done
```

## Monitoring and Logging

### Real-time Monitoring

```bash
# Monitor all topic message flows
mqttx sub -h localhost -p 1883 -t '#' --verbose

# Monitor specific pattern messages
mqttx sub -h localhost -p 1883 -t 'robustmq/+/+/data' --format '[%Y-%m-%d %H:%M:%S] %t: %p'

# Count messages
mqttx sub -h localhost -p 1883 -t 'robustmq/stats/+' --output-mode clean | wc -l
```

### Logging

```bash
# Log all received messages to file
mqttx sub -h localhost -p 1883 -t 'robustmq/logs/#' --verbose > robustmq_messages.log

# Log performance test results
mqttx bench pub -h localhost -p 1883 -t 'robustmq/perf/test' -c 50 -C 1000 --verbose > perf_test.log
```

## Troubleshooting

### Connection Issue Diagnosis

```bash
# Detailed connection diagnosis
mqttx conn -h localhost -p 1883 --client-id debug_client --verbose

# Test network connectivity
mqttx conn -h localhost -p 1883 --timeout 5

# Test authentication issues
mqttx conn -h localhost -p 1883 -u test_user -P wrong_password --verbose
```

### Message Delivery Issues

```bash
# Test if messages are delivered correctly
mqttx sub -h localhost -p 1883 -t 'robustmq/debug/+' --verbose &
mqttx pub -h localhost -p 1883 -t 'robustmq/debug/test' -m 'Debug message' --verbose

# Test message delivery with different QoS
for qos in 0 1 2; do
    mqttx pub -h localhost -p 1883 -t "robustmq/debug/qos$qos" -m "QoS $qos test" -q $qos --verbose
done
```

## Best Practices

### 1. Pre-test Preparation

```bash
# Check RobustMQ service status
curl -f http://localhost:8080/health || echo "RobustMQ service may not be running"

# Clean test topics (if needed)
mqttx pub -h localhost -p 1883 -t 'robustmq/test/cleanup' -m '' -r
```

### 2. Staged Testing

```bash
# Stage 1: Basic functionality test
mqttx pub -h localhost -p 1883 -t 'robustmq/test/basic' -m 'Basic test'

# Stage 2: Medium load test
mqttx bench pub -h localhost -p 1883 -t 'robustmq/test/medium' -c 10 -C 100

# Stage 3: High load test
mqttx bench pub -h localhost -p 1883 -t 'robustmq/test/heavy' -c 100 -C 1000
```

### 3. Result Analysis

```bash
# Use verbose mode for detailed statistics
mqttx bench pub -h localhost -p 1883 -t 'robustmq/analysis/test' -c 50 -C 500 --verbose | grep -E "(Published|Failed|Rate)"

# Save test results
mqttx bench pub -h localhost -p 1883 -t 'robustmq/results/test' -c 100 -C 1000 --verbose > test_results_$(date +%Y%m%d_%H%M%S).log
```

## Summary

MQTTX CLI is an ideal tool for testing RobustMQ MQTT Broker, providing complete functionality from basic connections to advanced performance testing. Through the examples and scripts in this guide, you can:

1. **Verify basic functionality**: Connection, publishing, subscribing, authentication, etc.
2. **Test advanced features**: SSL/TLS, WebSocket, MQTT 5.0, etc.
3. **Perform performance evaluation**: Throughput, latency, concurrent connections, etc.
4. **Troubleshoot issues**: Quickly locate and resolve problems
5. **Automate testing**: Integrate into CI/CD pipelines

It's recommended to use MQTTX CLI for comprehensive functional and performance testing before deploying RobustMQ to production environments, ensuring system stability and reliability.
