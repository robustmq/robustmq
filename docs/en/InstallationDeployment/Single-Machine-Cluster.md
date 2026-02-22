# Standalone Mode

This guide covers running RobustMQ on a single machine using the binary package, suitable for development and testing.

## Install

See [Quick Install](../QuickGuide/Quick-Install.md) for full instructions, or use the one-line installer:

```bash
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

After installation, `robust-server`, `robust-ctl`, and `robust-bench` are available in your PATH.

## Start Services

```bash
robust-server start
```

Defaults to `config/server.toml`. You can also specify the config explicitly:

```bash
robust-server start config/server.toml
```

## Verify

**Check cluster status**

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

## Stop Services

```bash
robust-server stop
```

## Default Ports

| Service | Port |
|---------|------|
| MQTT | 1883 |
| HTTP API | 8083 |
| Placement Center gRPC | 1228 |
| Dashboard | 3000 |
