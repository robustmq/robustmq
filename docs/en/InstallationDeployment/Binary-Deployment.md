# Binary Deployment

This guide covers running RobustMQ from a binary package, in both standalone mode (development/testing) and cluster mode (production).

> No binary package available? You can [build one locally](../ContributionGuide/ContributingCode/Build-and-Package.md).

## Install

Use the one-line installer:

```bash
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

See [Quick Install](../QuickGuide/Quick-Install.md) for more installation options.

After installation, `robust-server`, `robust-ctl`, and `robust-bench` are available in your PATH.

---

## Standalone Mode

Suitable for development and testing.

### Start

```bash
robust-server start
```

Defaults to `config/server.toml`. You can also specify the config explicitly:

```bash
robust-server start config/server.toml
```

### Verify

```bash
# Cluster status
robust-ctl cluster status

# Cluster health
robust-ctl cluster healthy

# MQTT overview (connections, subscriptions, etc.)
robust-ctl mqtt overview
```

`--server` defaults to `127.0.0.1:8080`. To target a different address:

```bash
robust-ctl cluster status --server 192.168.1.10:8080
```

**MQTT pub/sub test**

```bash
# Subscribe (terminal 1)
mqttx sub -h 127.0.0.1 -p 1883 -t "test/topic"

# Publish (terminal 2)
mqttx pub -h 127.0.0.1 -p 1883 -t "test/topic" -m "Hello RobustMQ!"
```

If the subscriber receives the message, the service is running correctly. Web console: `http://localhost:3000`

### Stop

```bash
robust-server stop
```

### Default Ports

| Service | Port |
|---------|------|
| MQTT | 1883 |
| HTTP API | 8083 |
| Placement Center gRPC | 1228 |
| Dashboard | 3000 |

---

## Cluster Mode

Suitable for high-availability deployments. The installation package includes cluster config templates:

```
config/cluster/
├── server-1.toml   # Node 1 (grpc: 1128, mqtt: 1883)
├── server-2.toml   # Node 2 (grpc: 1228, mqtt: 2883)
└── server-3.toml   # Node 3 (grpc: 1328, mqtt: 3883)
```

### Option 1: Single-Machine Three-Node (Dev/Test)

All three config files default to `127.0.0.1`. Just start them directly:

```bash
# Run each in a separate terminal
robust-server start config/cluster/server-1.toml
robust-server start config/cluster/server-2.toml
robust-server start config/cluster/server-3.toml
```

### Option 2: Multi-Machine Cluster (Production)

Assuming three machines with IPs `10.0.0.1`, `10.0.0.2`, `10.0.0.3` — after installing on each, update `broker_ip` and `meta_addrs` in the corresponding config file:

```toml
# Node 1 (10.0.0.1) — edit config/cluster/server-1.toml:
broker_ip = "10.0.0.1"
meta_addrs = { 1 = "10.0.0.1:1128", 2 = "10.0.0.2:1228", 3 = "10.0.0.3:1328" }

# Node 2 (10.0.0.2) — edit config/cluster/server-2.toml:
broker_ip = "10.0.0.2"
meta_addrs = { 1 = "10.0.0.1:1128", 2 = "10.0.0.2:1228", 3 = "10.0.0.3:1328" }

# Node 3 (10.0.0.3) — edit config/cluster/server-3.toml:
broker_ip = "10.0.0.3"
meta_addrs = { 1 = "10.0.0.1:1128", 2 = "10.0.0.2:1228", 3 = "10.0.0.3:1328" }
```

Then start on each node:

```bash
# Node 1
robust-server start config/cluster/server-1.toml

# Node 2
robust-server start config/cluster/server-2.toml

# Node 3
robust-server start config/cluster/server-3.toml
```

### Verify

```bash
# Cluster status (connect to any node)
robust-ctl cluster status --server 10.0.0.1:8080

# Cluster health
robust-ctl cluster healthy --server 10.0.0.1:8080

# MQTT overview
robust-ctl mqtt overview --server 10.0.0.1:8080
```

**Cross-node MQTT test**

```bash
# Subscribe (connect to node 1)
mqttx sub -h 10.0.0.1 -p 1883 -t "test/cluster"

# Publish (connect to node 2)
mqttx pub -h 10.0.0.2 -p 1883 -t "test/cluster" -m "Hello Cluster!"
```

If node 1 receives the message, the cluster is running correctly.

### Stop

```bash
robust-server stop
```

### Default Ports (per node)

| | Node 1 | Node 2 | Node 3 |
|-|--------|--------|--------|
| **MQTT TCP** | 1883 | 2883 | 3883 |
| **gRPC** | 1128 | 1228 | 1328 |
| **HTTP API** | 8080 | 8082 | 8083 |
