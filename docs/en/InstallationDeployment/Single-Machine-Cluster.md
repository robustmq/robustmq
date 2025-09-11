# Single Machine RobustMQ

This guide describes how to run RobustMQ on a single machine, suitable for development and testing environments.

## Prerequisites

### Download Binary Package

```bash
# Download the latest version
wget https://github.com/robustmq/robustmq/releases/download/v1.0.0/robustmq-v1.0.0-linux-amd64.tar.gz

# Extract
tar -xzf robustmq-v1.0.0-linux-amd64.tar.gz
cd robustmq-v1.0.0-linux-amd64
```

## Starting RobustMQ

### Start with Default Configuration

```bash
# Start RobustMQ
./bin/broker-server start
```

### Start with Configuration File

```bash
# Start with configuration file
./bin/broker-server start config/server.toml
```

### Start in Background

```bash
# Start in background
nohup ./bin/broker-server start > robustmq.log 2>&1 &
```

## Verifying Running Status

### Check Service Status

```bash
# Check MQTT port
netstat -tlnp | grep 1883

# Check admin port
netstat -tlnp | grep 8080
```

### Test MQTT Connection

```bash
# Send message
mqttx pub -h 127.0.0.1 -p 1883 -t "test/topic" -m "Hello RobustMQ!"

# Subscribe to messages
mqttx sub -h 127.0.0.1 -p 1883 -t "test/topic"
```

## Stopping Service

```bash
# Stop RobustMQ
pkill -f "broker-server"

# Or find process ID and stop
ps aux | grep broker-server
kill <PID>
```

## Default Ports

| Service | Port | Description |
|---------|------|-------------|
| MQTT | 1883 | MQTT protocol port |
| Admin | 8080 | Admin interface port |
| Cluster | 9090 | Cluster communication port |
| Meta | 9091 | Metadata service port |

## Important Notes

1. **Port Usage**: Ensure default ports are not occupied
2. **Firewall**: Ensure firewall allows communication on relevant ports
3. **Resource Requirements**: Recommend at least 2GB memory
4. **Data Directory**: RobustMQ will create data files in the current directory
