# Journal Engine Configuration

> This document describes all configuration items related to Journal engine. For general configuration information, please refer to [COMMON.md](COMMON.md).

## Journal Engine Overview

Journal engine is RobustMQ's persistent storage layer, responsible for:
- Message persistent storage
- High-performance log writing
- Data sharding and replica management
- Storage space management

---

## Journal Server Configuration

### Server Port Configuration
```toml
[journal_server]
tcp_port = 1778              # Journal service TCP port
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `tcp_port` | `u32` | `1778` | Journal engine service listening port |

---

## Journal Runtime Configuration

### Runtime Configuration
```toml
[journal_runtime]
enable_auto_create_shard = true   # Enable automatic shard creation
shard_replica_num = 2            # Shard replica count
max_segment_size = 1073741824    # Maximum segment file size (bytes)
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable_auto_create_shard` | `bool` | `true` | Whether to automatically create new data shards |
| `shard_replica_num` | `u32` | `2` | Number of replicas per shard |
| `max_segment_size` | `u32` | `1073741824` | Maximum size of single segment file (bytes, default 1GB) |

### Shard Management Description
- **Auto Sharding**: When enabled, system automatically creates new shards based on data volume
- **Replica Count**: Recommend setting to odd numbers for failure recovery voting
- **Segment Size**: Affects I/O performance and storage efficiency, recommend adjusting based on hardware configuration

---

## Journal Storage Configuration

### Storage Configuration
```toml
[journal_storage]
data_path = [
    "./data/journal/data1",
    "./data/journal/data2",
    "./data/journal/data3"
]
rocksdb_max_open_files = 10000   # RocksDB maximum open files
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `data_path` | `array` | `["./data/journal/"]` | Data storage path list, supports multiple paths |
| `rocksdb_max_open_files` | `i32` | `10000` | RocksDB maximum simultaneously open files |

### Multi-Path Storage Description
- **Load Balancing**: Data is evenly distributed across multiple paths
- **Performance Improvement**: Can utilize I/O capability of multiple disks
- **Fault Tolerance**: Single disk failure won't affect the entire system

---

## Complete Journal Configuration Examples

### Development Environment Configuration
```toml
# Basic configuration
cluster_name = "dev-cluster"
broker_id = 1
roles = ["journal"]
grpc_port = 1228

# Journal server configuration
[journal_server]
tcp_port = 1778

# Journal runtime configuration
[journal_runtime]
enable_auto_create_shard = true
shard_replica_num = 1          # Single replica for development
max_segment_size = 104857600   # 100MB, suitable for development testing

# Journal storage configuration
[journal_storage]
data_path = ["./data/journal"]
rocksdb_max_open_files = 5000
```

### Production Environment Configuration
```toml
# Basic configuration
cluster_name = "production-cluster"
broker_id = 1
roles = ["journal"]
grpc_port = 1228

# Journal server configuration
[journal_server]
tcp_port = 1778

# Journal runtime configuration
[journal_runtime]
enable_auto_create_shard = true
shard_replica_num = 3          # Three replicas for production
max_segment_size = 2147483648  # 2GB, suitable for high throughput

# Journal storage configuration
[journal_storage]
data_path = [
    "/data1/robustmq/journal",
    "/data2/robustmq/journal",
    "/data3/robustmq/journal",
    "/data4/robustmq/journal"
]
rocksdb_max_open_files = 50000
```

### High Performance Configuration
```toml
# High performance Journal configuration
[journal_runtime]
enable_auto_create_shard = true
shard_replica_num = 3
max_segment_size = 4294967296  # 4GB, reduce file switching frequency

[journal_storage]
data_path = [
    "/nvme1/robustmq/journal",  # Use NVMe SSD
    "/nvme2/robustmq/journal",
    "/nvme3/robustmq/journal",
    "/nvme4/robustmq/journal"
]
rocksdb_max_open_files = 100000

# Network optimization
[network]
handler_thread_num = 32        # Increase handler threads
response_thread_num = 16       # Increase response threads
queue_size = 20000            # Increase queue capacity
```

---

## Environment Variable Override Examples

### Journal Related Environment Variables
```bash
# Journal server configuration
export ROBUSTMQ_JOURNAL_SERVER_TCP_PORT=1778

# Journal runtime configuration
export ROBUSTMQ_JOURNAL_RUNTIME_ENABLE_AUTO_CREATE_SHARD=true
export ROBUSTMQ_JOURNAL_RUNTIME_SHARD_REPLICA_NUM=3
export ROBUSTMQ_JOURNAL_RUNTIME_MAX_SEGMENT_SIZE=2147483648

# Journal storage configuration
export ROBUSTMQ_JOURNAL_STORAGE_ROCKSDB_MAX_OPEN_FILES=50000
```

---

## Performance Tuning Recommendations

### Write Performance Optimization
```toml
[journal_runtime]
max_segment_size = 4294967296    # Increase segment file size, reduce file switching

[journal_storage]
rocksdb_max_open_files = 100000  # Increase file handles
data_path = [
    "/ssd1/journal",             # Use high-speed SSD
    "/ssd2/journal"
]

[network]
handler_thread_num = 16          # Increase handler threads
queue_size = 50000              # Increase queue capacity
```

### Storage Space Optimization
```toml
[journal_runtime]
max_segment_size = 536870912     # 512MB, moderate segment size
shard_replica_num = 2           # Appropriate replica count

[journal_storage]
data_path = ["/data/journal"]    # Single path, save space
```

### Reliability Optimization
```toml
[journal_runtime]
enable_auto_create_shard = true
shard_replica_num = 3           # Three replicas ensure reliability
max_segment_size = 1073741824   # 1GB segment size

[journal_storage]
data_path = [
    "/raid1/journal",           # Use RAID storage
    "/raid2/journal"
]
rocksdb_max_open_files = 30000
```

---

## Data Management

### Sharding Strategy
1. **Auto Sharding**: System automatically creates new shards based on data volume
2. **Manual Sharding**: Can manually create shards through API
3. **Shard Size**: Controlled by `max_segment_size`

### Replica Management
1. **Replica Count**: Controlled by `shard_replica_num`
2. **Replica Distribution**: System automatically distributes replicas across different nodes
3. **Data Synchronization**: Use asynchronous replication to ensure data consistency

### Data Cleanup
- Automatically clean expired data
- Support manual trigger compression
- Regular garbage collection

---

## Monitoring and Operations

### Key Monitoring Metrics
- Write throughput (messages/second)
- Storage usage (GB)
- Shard count and distribution
- Replica synchronization latency
- Disk I/O utilization

### Operations
1. **Data Backup**: Regularly backup data directories
2. **Scaling Operations**: Add new storage paths
3. **Performance Monitoring**: Monitor disk and network usage
4. **Failure Recovery**: Recover data from replicas

---

## Troubleshooting

### Common Issues
1. **Poor Write Performance**
   - Check disk I/O performance
   - Adjust segment file size
   - Increase handler thread count

2. **Insufficient Storage Space**
   - Clean expired data
   - Add new storage paths
   - Adjust data retention policies

3. **Replica Synchronization Delay**
   - Check network bandwidth
   - Adjust replica count
   - Optimize network configuration

4. **Service Startup Failure**
   - Check port occupation
   - Verify storage path permissions
   - Confirm configuration file syntax

### Debug Configuration
```toml
# Enable verbose logging
[log]
log_config = "./config/debug-tracing.toml"

# Reduce segment file size for debugging
[journal_runtime]
max_segment_size = 10485760  # 10MB

# Enable performance profiling
[p_prof]
enable = true
port = 6060
```

---

## Monitoring Metrics

Journal service provides the following monitoring metrics:
- Write/read throughput
- Storage usage and distribution
- Shard status and health
- Replica synchronization status
- Performance statistics

---

*Documentation Version: v1.0*  
*Last Updated: 2024-01-01*  
*Based on Code Version: RobustMQ v0.1.31*
