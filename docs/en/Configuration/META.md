# Meta Service Configuration

> This document describes all configuration items related to Meta service (Metadata Service/Meta Service). For general configuration information, please refer to [COMMON.md](COMMON.md).

## Meta Service Overview

Meta service (also known as Meta Service) is RobustMQ's metadata management service, responsible for:
- Cluster node management
- Metadata storage and synchronization
- Service discovery
- Configuration management

---

## Meta Runtime Configuration

### Runtime Configuration
```toml
[meta_runtime]
heartbeat_timeout_ms = 30000      # Heartbeat timeout (milliseconds)
heartbeat_check_time_ms = 1000    # Heartbeat check interval (milliseconds)
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `heartbeat_timeout_ms` | `u64` | `30000` | Node heartbeat timeout (milliseconds) |
| `heartbeat_check_time_ms` | `u64` | `1000` | Heartbeat check interval (milliseconds) |

### Heartbeat Mechanism Description
- **Heartbeat Timeout**: Nodes that don't send heartbeat within specified time will be marked as unavailable
- **Heartbeat Check**: Periodically check heartbeat status of all nodes
- **Failover**: Automatically elect new primary node when primary node fails

---

## Meta Storage Configuration

### RocksDB Storage Configuration
```toml
[rocksdb]
data_path = "./data/meta-service/data"  # Data storage path
max_open_files = 10000                      # Maximum open files
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `data_path` | `string` | `"./data"` | Meta service data storage directory |
| `max_open_files` | `i32` | `10000` | RocksDB maximum simultaneously open files |

---

## Meta Cluster Configuration

### Cluster Node Configuration
```toml
# Metadata center node address configuration
[meta_service]
1 = "127.0.0.1:1228"
2 = "127.0.0.1:1229"
3 = "127.0.0.1:1230"
```

### Configuration Description

| Configuration | Type | Description |
|---------------|------|-------------|
| `meta_service` | `table` | Metadata center node address mapping table |
| Key | `string` | Node ID (usually corresponds to broker_id) |
| Value | `string` | Node address in format "IP:port" |

### Cluster Deployment Modes
1. **Single Node Mode**: Configure only one node, suitable for testing and development
2. **Multi-Node Mode**: Configure multiple nodes for high availability
3. **Recommended Configuration**: Production environment recommends odd number of nodes (3, 5, 7)

---

## Meta Service Role Configuration

### Role Configuration
```toml
roles = ["meta", "broker"]    # Node role list
```

### Role Description
- **meta**: Metadata service role
- **broker**: MQTT Broker role
- **journal**: Journal engine role

### Deployment Modes
1. **Integrated Deployment**: `roles = ["meta", "broker", "journal"]`
2. **Separated Deployment**:
   - Meta nodes: `roles = ["meta"]`
   - Broker nodes: `roles = ["broker"]`
   - Journal nodes: `roles = ["journal"]`

---

## Complete Meta Service Configuration Examples

### Single Node Configuration
```toml
# Basic configuration
cluster_name = "dev-cluster"
broker_id = 1
roles = ["meta", "broker"]
grpc_port = 1228

# Metadata center configuration
[meta_service]
1 = "127.0.0.1:1228"

# Meta runtime configuration
[meta_runtime]
heartbeat_timeout_ms = 10000
heartbeat_check_time_ms = 2000

# Storage configuration
[rocksdb]
data_path = "./data/meta"
max_open_files = 5000
```

### High Availability Cluster Configuration
```toml
# Basic configuration
cluster_name = "production-cluster"
broker_id = 1
roles = ["meta"]
grpc_port = 1228

# Metadata center cluster configuration
[meta_service]
1 = "192.168.1.10:1228"
2 = "192.168.1.11:1228"
3 = "192.168.1.12:1228"

# Meta runtime configuration
[meta_runtime]
heartbeat_timeout_ms = 30000
heartbeat_check_time_ms = 5000

# Storage configuration
[rocksdb]
data_path = "/data/robustmq/meta"
max_open_files = 20000
```

---

## Environment Variable Override Examples

### Meta Related Environment Variables
```bash
# Cluster basic configuration
export ROBUSTMQ_CLUSTER_NAME="production-cluster"
export ROBUSTMQ_BROKER_ID=1
export ROBUSTMQ_GRPC_PORT=1228

# Meta runtime configuration
export ROBUSTMQ_PLACE_RUNTIME_HEARTBEAT_TIMEOUT_MS=30000
export ROBUSTMQ_PLACE_RUNTIME_HEARTBEAT_CHECK_TIME_MS=5000

# Storage configuration
export ROBUSTMQ_ROCKSDB_DATA_PATH="/data/robustmq/meta"
export ROBUSTMQ_ROCKSDB_MAX_OPEN_FILES=20000
```

---

## Performance Tuning Recommendations

### High Concurrency Scenarios
```toml
[meta_runtime]
heartbeat_timeout_ms = 15000      # Reduce heartbeat timeout
heartbeat_check_time_ms = 3000    # Increase heartbeat check frequency

[rocksdb]
max_open_files = 50000           # Increase file handles

[network]
handler_thread_num = 16          # Increase handler threads
queue_size = 10000              # Increase queue size
```

### Low Latency Scenarios
```toml
[meta_runtime]
heartbeat_check_time_ms = 500    # More frequent heartbeat checks

[network]
lock_max_try_mut_times = 10      # Reduce lock retry times
lock_try_mut_sleep_time_ms = 5   # Reduce lock sleep time
```

### High Reliability Scenarios
```toml
[meta_runtime]
heartbeat_timeout_ms = 60000     # Increase heartbeat tolerance
heartbeat_check_time_ms = 10000  # Moderate check frequency

[rocksdb]
data_path = "/data/robustmq/meta"  # Use persistent storage path
max_open_files = 30000             # Sufficient file handles
```

---

## Cluster Management

### Node Discovery
Meta service uses `meta_service` configuration for node discovery:
1. Each node connects to all configured Meta nodes on startup
2. Maintain node status through heartbeat mechanism
3. Automatically handle node failures and recovery

### Data Consistency
- Use Raft protocol to ensure data consistency
- Support multi-replica data storage
- Automatically handle split-brain and network partitions

### Failure Recovery
- Automatically detect node failures
- Support dynamic node joining and leaving
- Automatic data synchronization and recovery

---

## Troubleshooting

### Common Issues
1. **Node Cannot Join Cluster**
   - Check `meta_service` configuration
   - Verify network connectivity
   - Confirm ports are not occupied

2. **Heartbeat Timeout**
   - Adjust `heartbeat_timeout_ms`
   - Check network latency
   - Optimize system load

3. **Data Storage Issues**
   - Check `data_path` directory permissions
   - Verify disk space
   - Adjust `max_open_files`

4. **Election Failure**
   - Ensure odd number of cluster nodes
   - Check node time synchronization
   - Verify network partition situations

### Debugging Methods
```toml
# Enable verbose logging
[log]
log_config = "./config/debug-tracing.toml"

# Shorten heartbeat interval for debugging
[meta_runtime]
heartbeat_check_time_ms = 1000
```

---

## Monitoring Metrics

Meta service provides the following monitoring metrics:
- Node health status
- Heartbeat latency statistics
- Data synchronization status
- Election status changes
- Storage usage

---

*Documentation Version: v1.0*
*Last Updated: 2024-01-01*
*Based on Code Version: RobustMQ v0.1.31*
