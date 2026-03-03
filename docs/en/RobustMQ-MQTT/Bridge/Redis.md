# Redis Connector

## Overview

The Redis connector is a data integration component provided by RobustMQ for bridging MQTT messages to Redis databases. It supports Single, Cluster, and Sentinel deployment modes, and uses command templates for flexible data writing, making it suitable for real-time caching, message queuing, session storage, and counter scenarios.

## Configuration

### Connector Configuration

The Redis connector uses `RedisConnectorConfig` for configuration:

```rust
pub struct RedisConnectorConfig {
    pub server: String,                       // Redis server address
    pub mode: RedisMode,                      // Deployment mode
    pub database: u8,                         // Database number
    pub username: Option<String>,             // Username
    pub password: Option<String>,             // Password
    pub sentinel_master_name: Option<String>, // Sentinel master name
    pub command_template: String,             // Command template
    pub tls_enabled: bool,                    // Enable TLS
    pub connect_timeout_ms: u64,              // Connection timeout (ms)
    pub pool_size: u32,                       // Connection pool size
    pub max_retries: u32,                     // Max retries
    pub retry_interval_ms: u64,              // Retry interval (ms)
}

pub enum RedisMode {
    Single,    // Single node
    Cluster,   // Cluster mode
    Sentinel,  // Sentinel mode
}
```

### Configuration Parameters

| Parameter | Type | Required | Default | Description | Example |
|-----------|------|----------|---------|-------------|---------|
| `server` | String | Yes | - | Redis server address, max 1024 characters | `127.0.0.1:6379` |
| `mode` | String | No | `single` | Deployment mode: `single`, `cluster`, `sentinel` | `single` |
| `database` | Number | No | `0` | Database number, range: 0-15 | `0` |
| `username` | String | No | - | Username (Redis 6.0+ ACL) | `default` |
| `password` | String | No | - | Password | `password123` |
| `sentinel_master_name` | String | No | - | Sentinel master name, required in sentinel mode | `mymaster` |
| `command_template` | String | Yes | - | Command template with `${key}`, `${payload}` placeholders, max 4096 characters | `SET ${key} ${payload}` |
| `tls_enabled` | Boolean | No | `false` | Enable TLS encryption | `true` |
| `connect_timeout_ms` | Number | No | `5000` | Connection timeout in milliseconds | `10000` |
| `pool_size` | Number | No | `10` | Connection pool size, range: 1-100 | `20` |
| `max_retries` | Number | No | `3` | Maximum retry count | `5` |
| `retry_interval_ms` | Number | No | `1000` | Retry interval in milliseconds | `2000` |

### Command Templates

Command templates support the following placeholders:

| Placeholder | Description |
|-------------|-------------|
| `${key}` | Message key |
| `${payload}` | Message content |
| `${timestamp}` | Message timestamp |

#### Command Template Examples

```
SET ${key} ${payload}
LPUSH mqtt:messages ${payload}
HSET mqtt:data ${key} ${payload}
PUBLISH mqtt:channel ${payload}
SETEX ${key} 3600 ${payload}
```

### Configuration Examples

#### JSON Configuration

**Single Mode**
```json
{
  "server": "127.0.0.1:6379",
  "command_template": "LPUSH mqtt:messages ${payload}"
}
```

**With Password**
```json
{
  "server": "127.0.0.1:6379",
  "password": "your_password",
  "database": 1,
  "command_template": "SET mqtt:${key} ${payload}",
  "pool_size": 20
}
```

**Cluster Mode**
```json
{
  "server": "redis-1:6379,redis-2:6379,redis-3:6379",
  "mode": "cluster",
  "password": "cluster_password",
  "command_template": "SET mqtt:${key} ${payload}",
  "pool_size": 30,
  "connect_timeout_ms": 10000
}
```

**Sentinel Mode**
```json
{
  "server": "sentinel-1:26379,sentinel-2:26379,sentinel-3:26379",
  "mode": "sentinel",
  "sentinel_master_name": "mymaster",
  "password": "sentinel_password",
  "command_template": "LPUSH mqtt:events ${payload}",
  "pool_size": 15
}
```

## Creating a Redis Connector with robust-ctl

### Basic Syntax

```bash
robust-ctl mqtt connector create \
  --connector-name <name> \
  --connector-type <type> \
  --config <config> \
  --topic-id <topic>
```

### Examples

#### 1. Basic Creation

```bash
robust-ctl mqtt connector create \
  --connector-name "redis_connector_01" \
  --connector-type "redis" \
  --config '{"server": "127.0.0.1:6379", "command_template": "LPUSH mqtt:messages ${payload}"}' \
  --topic-id "sensor/data"
```

#### 2. Authenticated Redis

```bash
robust-ctl mqtt connector create \
  --connector-name "redis_auth" \
  --connector-type "redis" \
  --config '{"server": "127.0.0.1:6379", "password": "your_password", "database": 1, "command_template": "SET mqtt:${key} ${payload}", "pool_size": 20}' \
  --topic-id "device/status"
```

### Managing Connectors

```bash
# List all connectors
robust-ctl mqtt connector list

# View specific connector
robust-ctl mqtt connector list --connector-name "redis_connector_01"

# Delete connector
robust-ctl mqtt connector delete --connector-name "redis_connector_01"
```

## Performance Tips

### 1. Connection Pool
- Set `pool_size` based on concurrency requirements
- Increase pool size for high-throughput scenarios

### 2. Command Selection
- Use batch commands (e.g., `LPUSH`) to reduce network round trips
- Use `SETEX` or `EXPIRE` for data with TTL requirements
- Avoid blocking commands

### 3. Deployment Mode
- Single mode for development and testing
- Use Cluster or Sentinel mode in production
- Sentinel mode provides automatic failover

### 4. Security
- Enable TLS in production environments
- Use ACL (Redis 6.0+) for fine-grained access control
- Rotate passwords regularly

## Troubleshooting

### Common Issues

**Connection Failure**
- Check if Redis service is running
- Verify server address and port
- Check network and firewall settings

**Authentication Failure**
- Verify password is correct
- Check username and ACL configuration (Redis 6.0+)

**Write Timeout**
- Increase `connect_timeout_ms`
- Check Redis service load
- Consider increasing pool size

**Sentinel Connection Issues**
- Confirm `sentinel_master_name` is correct
- Check Sentinel nodes are running
- Verify connectivity to all Sentinel nodes

## Summary

The Redis connector provides seamless integration with Redis databases, supporting Single, Cluster, and Sentinel deployment modes with flexible command templates for customizable data writing patterns.
