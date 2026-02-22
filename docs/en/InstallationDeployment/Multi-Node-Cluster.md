# Multi-Node Cluster Deployment

This guide describes how to deploy a RobustMQ multi-node cluster, suitable for production environments.

## Prerequisites

### System Requirements

- **Number of Nodes**: Recommend at least 3 nodes (odd number of nodes)
- **Operating System**: Linux (recommend Ubuntu 20.04+ or CentOS 8+)
- **Memory**: At least 4GB per node
- **Disk**: At least 50GB available space per node
- **Network**: Inter-node network latency < 10ms

### Download Binary Package

Execute on each node:

```bash
# Download the latest version
wget https://github.com/robustmq/robustmq/releases/download/v1.0.0/robustmq-v1.0.0-linux-amd64.tar.gz

# Extract
tar -xzf robustmq-v1.0.0-linux-amd64.tar.gz
cd robustmq-v1.0.0-linux-amd64
```

## Cluster Configuration

### Node Planning

Assuming deployment of a 3-node cluster:

| Node | IP Address | Role | gRPC Port |
|------|------------|------|-----------|
| node1 | 192.168.1.10 | meta, broker | 1228 |
| node2 | 192.168.1.11 | meta, broker | 1228 |
| node3 | 192.168.1.12 | meta, broker | 1228 |

### Node 1 Configuration (config/node1.toml)

```toml
cluster_name = "robustmq-cluster"
broker_id = 1
roles = ["meta", "broker"]
grpc_port = 1228
meta_addrs = { 
    1 = "192.168.1.10:1228",
    2 = "192.168.1.11:1228", 
    3 = "192.168.1.12:1228"
}

[rocksdb]
data_path = "./data/broker/data"
max_open_files = 10000

[p_prof]
enable = false
port = 6777
frequency = 1000

[log]
log_config = "./config/logger.toml"
log_path = "./data/broker/logs"
```

### Node 2 Configuration (config/node2.toml)

```toml
cluster_name = "robustmq-cluster"
broker_id = 2
roles = ["meta", "broker"]
grpc_port = 1228
meta_addrs = { 
    1 = "192.168.1.10:1228",
    2 = "192.168.1.11:1228", 
    3 = "192.168.1.12:1228"
}

[rocksdb]
data_path = "./data/broker/data"
max_open_files = 10000

[p_prof]
enable = false
port = 6777
frequency = 1000

[log]
log_config = "./config/logger.toml"
log_path = "./data/broker/logs"
```

### Node 3 Configuration (config/node3.toml)

```toml
cluster_name = "robustmq-cluster"
broker_id = 3
roles = ["meta", "broker"]
grpc_port = 1228
meta_addrs = { 
    1 = "192.168.1.10:1228",
    2 = "192.168.1.11:1228", 
    3 = "192.168.1.12:1228"
}

[rocksdb]
data_path = "./data/broker/data"
max_open_files = 10000

[p_prof]
enable = false
port = 6777
frequency = 1000

[log]
log_config = "./config/logger.toml"
log_path = "./data/broker/logs"
```

## Starting Cluster

### Start Nodes in Order

**Important**: Must start nodes in order, start all meta nodes first.

```bash
# Start on node 1
./bin/broker-server start config/node1.toml

# Start on node 2
./bin/broker-server start config/node2.toml

# Start on node 3
./bin/broker-server start config/node3.toml
```

### Start in Background

```bash
# Start in background on node 1
nohup ./bin/broker-server start config/node1.toml > node1.log 2>&1 &

# Start in background on node 2
nohup ./bin/broker-server start config/node2.toml > node2.log 2>&1 &

# Start in background on node 3
nohup ./bin/broker-server start config/node3.toml > node3.log 2>&1 &
```

## Verifying Cluster

### Check Node Status

Check service status on each node:

```bash
# Check gRPC port
netstat -tlnp | grep 1228

# Check process
ps aux | grep broker-server
```

### Test MQTT Connection

```bash
# Connect to any node to send message
mqttx pub -h 192.168.1.10 -p 1883 -t "test/cluster" -m "Message to cluster"

# Subscribe to messages on another node
mqttx sub -h 192.168.1.11 -p 1883 -t "test/cluster"
```

### Verify Cluster Consistency

```bash
# Send message on node 1
mqttx pub -h 192.168.1.10 -p 1883 -t "test/consistency" -m "Test message"

# Both node 2 and node 3 should receive the message
mqttx sub -h 192.168.1.11 -p 1883 -t "test/consistency"
mqttx sub -h 192.168.1.12 -p 1883 -t "test/consistency"
```

## Cluster Management

### View Cluster Status

```bash
# View cluster configuration
./bin/robust-ctl cluster config get --server 192.168.1.10:8080
```

### Stop Cluster

```bash
# Stop all nodes
pkill -f "broker-server"

# Or stop each node separately
kill $(ps aux | grep "node1.toml" | grep -v grep | awk '{print $2}')
kill $(ps aux | grep "node2.toml" | grep -v grep | awk '{print $2}')
kill $(ps aux | grep "node3.toml" | grep -v grep | awk '{print $2}')
```

## Port Description

| Service | Port | Description |
|---------|------|-------------|
| MQTT | 1883 | MQTT protocol port |
| Admin | 8080 | Admin interface port |
| gRPC | 1228 | Cluster communication port |
| pprof | 6777 | Performance analysis port |
