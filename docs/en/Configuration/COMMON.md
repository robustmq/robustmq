# RobustMQ Configuration Common Guide

## Overview

RobustMQ uses TOML format configuration files to manage system configuration. Configuration files are located in the `config/` directory, with the main configuration file being `server.toml`.

## Configuration Documentation Navigation

- 🔧 **[MQTT Configuration](MQTT.md)** - MQTT Broker related configuration
- 🏗️ **[Meta Configuration](META.md)** - Metadata service configuration
- 📝 **[Journal Configuration](JOURNAL.md)** - Journal engine configuration

---

## Configuration File Structure

### Main Configuration Files
- **`config/server.toml`** - Main configuration file
- **`config/server.toml.template`** - Configuration template file
- **`config/server-tracing.toml`** - Log tracing configuration

### Configuration Loading Priority
1. Command line arguments
2. Environment variables
3. Configuration files
4. Default values

---

## Basic Configuration

### Cluster Basic Information
```toml
# Cluster name
cluster_name = "robust_mq_cluster_default"

# Node ID
broker_id = 1

# Node roles
roles = ["meta", "broker", "journal"]

# gRPC port
grpc_port = 1228

# HTTP port
http_port = 8080

# Metadata center addresses
[meta_addrs]
1 = "127.0.0.1:1228"
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `cluster_name` | `string` | `"robust_mq_cluster_default"` | Cluster name, must be the same for all nodes in the cluster |
| `broker_id` | `u64` | `1` | Unique node identifier |
| `roles` | `array` | `["meta", "broker"]` | Node roles: meta(metadata), broker(MQTT), journal(log) |
| `grpc_port` | `u32` | `1228` | gRPC service port |
| `http_port` | `u32` | `8080` | HTTP API service port |
| `meta_addrs` | `table` | `{1 = "127.0.0.1:1228"}` | Metadata center node address mapping |

---

## Runtime Configuration

### Runtime Configuration
```toml
[runtime]
runtime_worker_threads = 4    # Runtime worker thread count
tls_cert = "./config/certs/cert.pem"  # TLS certificate path
tls_key = "./config/certs/key.pem"    # TLS private key path
```

### Network Configuration
```toml
[network]
accept_thread_num = 1         # Accept connection thread count
handler_thread_num = 1        # Handler thread count
response_thread_num = 1       # Response thread count
queue_size = 1000            # Queue size
lock_max_try_mut_times = 30   # Lock maximum retry times
lock_try_mut_sleep_time_ms = 50  # Lock retry sleep time (ms)
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `runtime_worker_threads` | `usize` | Auto-detect | Tokio runtime worker thread count |
| `tls_cert` | `string` | `"./config/certs/cert.pem"` | TLS certificate file path |
| `tls_key` | `string` | `"./config/certs/key.pem"` | TLS private key file path |
| `accept_thread_num` | `usize` | `1` | Thread count for accepting new connections |
| `handler_thread_num` | `usize` | `1` | Thread count for handling requests |
| `response_thread_num` | `usize` | `1` | Thread count for sending responses |
| `queue_size` | `usize` | `1000` | Internal queue size |

---

## Storage Configuration

### RocksDB Configuration
```toml
[rocksdb]
data_path = "./data"          # Data storage path
max_open_files = 10000       # Maximum open files
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `data_path` | `string` | `"./data"` | RocksDB data storage directory |
| `max_open_files` | `i32` | `10000` | RocksDB maximum open files |

---

## Monitoring Configuration

### Prometheus Configuration
```toml
[prometheus]
enable = true                 # Enable Prometheus metrics
port = 9090                  # Prometheus metrics port
```

### PProf Configuration
```toml
[p_prof]
enable = false               # Enable performance profiling
port = 6060                 # PProf service port
frequency = 100             # Sampling frequency
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `prometheus.enable` | `bool` | `true` | Whether to enable Prometheus metrics collection |
| `prometheus.port` | `u32` | `9090` | Prometheus metrics exposure port |
| `p_prof.enable` | `bool` | `false` | Whether to enable PProf performance analysis |
| `p_prof.port` | `u16` | `6060` | PProf service port |
| `p_prof.frequency` | `i32` | `100` | PProf sampling frequency |

---

## Logging Configuration

### Log Configuration
```toml
[log]
log_config = "./config/server-tracing.toml"  # Log configuration file path
log_path = "./data/broker/logs"              # Log output directory
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `log_config` | `string` | `"./config/broker-tracing.toml"` | Log tracing configuration file path |
| `log_path` | `string` | `"./logs"` | Log file output directory |

---

## Environment Variable Override

RobustMQ supports using environment variables to override configuration file settings. Environment variable naming rule:

### Naming Rule
```
{PREFIX}_{SECTION}_{KEY}
```

### Examples
```bash
# Override cluster name
export ROBUSTMQ_CLUSTER_NAME="my-cluster"

# Override MQTT TCP port
export ROBUSTMQ_MQTT_SERVER_TCP_PORT=1884

# Override log path
export ROBUSTMQ_LOG_LOG_PATH="/var/log/robustmq"

# Override Prometheus port
export ROBUSTMQ_PROMETHEUS_PORT=9091
```

### Environment Variable Priority
Environment variable values will override corresponding settings in configuration files, with higher priority than configuration files.

---

## Configuration File Examples

### Basic Configuration Example
```toml
# Basic configuration
cluster_name = "production-cluster"
broker_id = 1
roles = ["meta", "broker"]
grpc_port = 1228

# Metadata center addresses
[meta_service]
1 = "192.168.1.10:1228"
2 = "192.168.1.11:1228"
3 = "192.168.1.12:1228"

# Runtime configuration
[runtime]
runtime_worker_threads = 8
tls_cert = "./config/certs/prod-cert.pem"
tls_key = "./config/certs/prod-key.pem"

# Network configuration
[network]
accept_thread_num = 4
handler_thread_num = 8
response_thread_num = 4
queue_size = 2000

# Storage configuration
[rocksdb]
data_path = "/data/robustmq"
max_open_files = 20000

# Monitoring configuration
[prometheus]
enable = true
port = 9090

# Log configuration
[log]
log_config = "./config/production-tracing.toml"
log_path = "/var/log/robustmq"
```

---

## Configuration Validation

### Configuration File Syntax Check
```bash
# Check TOML syntax
cargo run --bin config-validator config/server.toml
```

### Common Configuration Errors
1. **TOML Syntax Errors** - Missing quotes, unmatched brackets, etc.
2. **Port Conflicts** - Multiple services using the same port
3. **Path Not Found** - Certificate files, log directories don't exist
4. **Permission Issues** - No write permission for data directories

---

## Best Practices

### Production Environment Configuration Recommendations
1. **Security**:
   - Use strong passwords and certificates
   - Limit network access permissions
   - Enable TLS encryption

2. **Performance Optimization**:
   - Adjust thread count based on hardware configuration
   - Set reasonable queue sizes
   - Optimize storage path configuration

3. **Monitoring and Logging**:
   - Enable Prometheus monitoring
   - Configure appropriate log levels
   - Set up log rotation policies

4. **High Availability**:
   - Configure multiple metadata center nodes
   - Set appropriate heartbeat timeout
   - Configure data backup strategies

---

## Troubleshooting

### Common Issues
1. **Configuration Loading Failed** - Check TOML syntax and file paths
2. **Port Binding Failed** - Check if ports are occupied
3. **Certificate Loading Failed** - Check certificate file paths and format
4. **Storage Initialization Failed** - Check data directory permissions and disk space

### Debugging Methods
1. Enable verbose log output
2. Check configuration loading logs
3. Use configuration validation tools
4. Monitor system resource usage

---

*Documentation Version: v1.0*
*Last Updated: 2024-01-01*
*Based on Code Version: RobustMQ v0.1.31*
