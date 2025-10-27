# MQTT Broker Configuration

> This document describes all configuration items related to MQTT Broker. For general configuration information, please refer to [COMMON.md](COMMON.md).

## MQTT Server Configuration

### Network Port Configuration
```toml
[mqtt_server]
tcp_port = 1883              # MQTT TCP port
tls_port = 1885              # MQTT TLS port
websocket_port = 8083        # WebSocket port
websockets_port = 8085       # WebSocket over TLS port
quic_port = 9083            # QUIC protocol port
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `tcp_port` | `u32` | `1883` | MQTT over TCP protocol port |
| `tls_port` | `u32` | `1885` | MQTT over TLS protocol port |
| `websocket_port` | `u32` | `8083` | MQTT over WebSocket port |
| `websockets_port` | `u32` | `8085` | MQTT over WebSocket Secure port |
| `quic_port` | `u32` | `9083` | MQTT over QUIC protocol port |

---

## MQTT Keep Alive Configuration

### Keep Alive Configuration
```toml
[mqtt_keep_alive]
enable = true                # Enable Keep Alive detection
default_time = 180          # Default Keep Alive interval (seconds)
max_time = 3600            # Maximum Keep Alive interval (seconds)
default_timeout = 2        # Default timeout count
```

### Configuration Description

| Configuration | Type | Default | Description |
|--------------|------|---------|-------------|
| `enable` | `bool` | `true` | Enable Keep Alive heartbeat detection |
| `default_time` | `u16` | `180` | Default Keep Alive interval (seconds) |
| `max_time` | `u16` | `3600` | Maximum Keep Alive interval (seconds) |
| `default_timeout` | `u16` | `2` | Disconnect after consecutive timeout count |

---

## MQTT Authentication Storage Configuration

### Authentication Storage Configuration
```toml
[mqtt_auth_config]
storage_type = "placement"    # Storage type
journal_addr = ""            # Journal address
mysql_addr = ""              # MySQL address
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `storage_type` | `string` | `"placement"` | Authentication data storage type: placement, journal, mysql |
| `journal_addr` | `string` | `""` | Journal engine address (used when storage_type is journal) |
| `mysql_addr` | `string` | `""` | MySQL database address (used when storage_type is mysql) |

### Storage Type Description
- **placement**: Use metadata service to store authentication information
- **journal**: Use Journal engine to store authentication information
- **mysql**: Use MySQL database to store authentication information

---

## MQTT Message Storage Configuration

### Message Storage Configuration
```toml
[mqtt_message_storage]
storage_type = "memory"       # Storage type
journal_addr = ""            # Journal address
mysql_addr = ""              # MySQL address
rocksdb_data_path = ""       # RocksDB data path
rocksdb_max_open_files = 10000  # RocksDB max open files
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `storage_type` | `string` | `"memory"` | Message storage type: memory, journal, mysql, rocksdb |
| `journal_addr` | `string` | `""` | Journal engine address |
| `mysql_addr` | `string` | `""` | MySQL database address |
| `rocksdb_data_path` | `string` | `""` | RocksDB data storage path |
| `rocksdb_max_open_files` | `i32` | `10000` | RocksDB maximum open files |

### Storage Type Description
- **memory**: Memory storage (data lost after restart, suitable for testing)
- **journal**: Use Journal engine for persistent storage
- **mysql**: Use MySQL database storage
- **rocksdb**: Use RocksDB local storage

---

## MQTT Runtime Configuration

### Runtime Configuration
```toml
[mqtt_runtime]
default_user = "admin"        # Default username
default_password = "robustmq" # Default password
max_connection_num = 1000000  # Maximum connection count
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `default_user` | `string` | `"admin"` | System default username |
| `default_password` | `string` | `"robustmq"` | System default password |
| `max_connection_num` | `usize` | `1000000` | Maximum connections per node |

---

## MQTT Protocol Configuration

### Protocol Parameters Configuration
```toml
[mqtt_protocol_config]
max_session_expiry_interval = 1800      # Maximum session expiry interval (seconds)
default_session_expiry_interval = 30    # Default session expiry interval (seconds)
topic_alias_max = 65535                 # Topic alias maximum
max_qos = 2                            # Maximum QoS level
max_packet_size = 10485760             # Maximum packet size (bytes)
receive_max = 65535                    # Receive maximum
client_pkid_persistent = false        # Client packet ID persistence
max_message_expiry_interval = 3600     # Maximum message expiry interval (seconds)
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `max_session_expiry_interval` | `u32` | `1800` | Session maximum expiry time (seconds) |
| `default_session_expiry_interval` | `u32` | `30` | Session default expiry time (seconds) |
| `topic_alias_max` | `u16` | `65535` | Maximum number of topic aliases |
| `max_qos` | `u8` | `2` | Maximum supported QoS level |
| `max_packet_size` | `u32` | `10485760` | Maximum MQTT packet size (bytes) |
| `receive_max` | `u16` | `65535` | Maximum number of unacknowledged PUBLISH packets |
| `max_message_expiry_interval` | `u64` | `3600` | Maximum message expiry time (seconds) |
| `client_pkid_persistent` | `bool` | `false` | Whether to persist client packet identifiers |

---

## MQTT Security Configuration

### Security Configuration
```toml
[mqtt_security]
secret_free_login = false            # Allow password-free login
is_self_protection_status = false   # Enable self-protection mode
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `secret_free_login` | `bool` | `false` | Whether to allow password-free login |
| `is_self_protection_status` | `bool` | `false` | Whether to enable self-protection mode |

---

## MQTT Offline Message Configuration

### Offline Message Configuration
```toml
[mqtt_offline_message]
enable = true                # Enable offline messages
expire_ms = 3600000         # Message expiry time (milliseconds)
max_messages_num = 1000     # Maximum offline message count
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `true` | Whether to enable offline message functionality |
| `expire_ms` | `u32` | `0` | Offline message expiry time (milliseconds), 0 means no expiry |
| `max_messages_num` | `u32` | `0` | Maximum offline messages per client, 0 means unlimited |

---

## MQTT System Monitor Configuration

### System Monitor Configuration
```toml
[mqtt_system_monitor]
enable = false                        # Enable system monitoring
os_cpu_high_watermark = 70.0         # CPU high watermark (%)
os_memory_high_watermark = 80.0      # Memory high watermark (%)
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `false` | Whether to enable system resource monitoring |
| `os_cpu_high_watermark` | `f32` | `70.0` | CPU usage high watermark (percentage) |
| `os_memory_high_watermark` | `f32` | `80.0` | Memory usage high watermark (percentage) |

---

## MQTT Slow Subscribe Configuration

### Slow Subscribe Configuration
```toml
[mqtt_slow_subscribe_config]
enable = false               # Enable slow subscribe detection
record_time = 1000          # Record time threshold (milliseconds)
delay_type = "Whole"        # Delay type
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `false` | Whether to enable slow subscribe detection |
| `record_time` | `u64` | `1000` | Slow subscribe record time threshold (milliseconds) |
| `delay_type` | `DelayType` | `Whole` | Delay calculation type: Whole, Partial |

---

## MQTT Flapping Detection Configuration

### Flapping Detection Configuration
```toml
[mqtt_flapping_detect]
enable = false                    # Enable flapping detection
window_time = 60                 # Time window (seconds)
max_client_connections = 15      # Maximum connection count
ban_time = 300                   # Ban duration (seconds)
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `false` | Whether to enable connection flapping detection |
| `window_time` | `u32` | `60` | Detection time window (seconds) |
| `max_client_connections` | `u64` | `15` | Maximum connection attempts within time window |
| `ban_time` | `u32` | `300` | Ban duration after triggering flapping (seconds) |

---

## MQTT Schema Configuration

### Schema Validation Configuration
```toml
[mqtt_schema]
enable = true                        # Enable Schema validation
strategy = "ALL"                     # Validation strategy
failed_operation = "Discard"         # Failed validation operation
echo_log = true                      # Output logs
log_level = "info"                   # Log level
```

### Configuration Description

| Configuration | Type | Default | Description |
|---------------|------|---------|-------------|
| `enable` | `bool` | `true` | Whether to enable Schema validation functionality |
| `strategy` | `SchemaStrategy` | `ALL` | Validation strategy: ALL (validate all), Any (validate any) |
| `failed_operation` | `SchemaFailedOperation` | `Discard` | Operation when validation fails |
| `echo_log` | `bool` | `true` | Whether to output Schema validation logs |
| `log_level` | `string` | `"info"` | Schema validation log level |

### Validation Strategy Description
- **ALL**: Message must pass all bound Schema validations
- **Any**: Message only needs to pass any bound Schema validation

### Failed Operation Description
- **Discard**: Discard messages that fail validation
- **DisconnectAndDiscard**: Disconnect and discard messages
- **Ignore**: Ignore validation failures and continue processing

---

## Complete MQTT Configuration Example

### Production Environment Configuration
```toml
# MQTT server port configuration
[mqtt_server]
tcp_port = 1883
tls_port = 1885
websocket_port = 8083
websockets_port = 8085
quic_port = 9083

# Authentication storage configuration
[mqtt_auth_config]
storage_type = "placement"

# Message storage configuration
[mqtt_message_storage]
storage_type = "journal"
journal_addr = "127.0.0.1:1778"

# Runtime configuration
[mqtt_runtime]
default_user = "admin"
default_password = "your_secure_password"
max_connection_num = 5000000

# System monitor configuration
[mqtt_system_monitor]
enable = true
os_cpu_high_watermark = 80.0
os_memory_high_watermark = 85.0

# Offline message configuration
[mqtt_offline_message]
enable = true
expire_ms = 86400000  # 24 hours
max_messages_num = 10000

# Slow subscribe detection configuration
[mqtt_slow_subscribe_config]
enable = true
delay_type = "Whole"

# Flapping detection configuration
[mqtt_flapping_detect]
enable = true
window_time = 120
max_client_connections = 10
ban_time = 600

# Protocol configuration
[mqtt_protocol_config]
max_session_expiry_interval = 7200
default_session_expiry_interval = 300
topic_alias_max = 1000
max_qos = 2
max_packet_size = 10485760
receive_max = 1000
client_pkid_persistent = true
max_message_expiry_interval = 86400

# Security configuration
[mqtt_security]
secret_free_login = false
is_self_protection_status = true

# Schema validation configuration
[mqtt_schema]
enable = true
strategy = "ALL"
failed_operation = "Discard"
echo_log = true
log_level = "warn"
```

---

## Environment Variable Override Examples

### MQTT Related Environment Variables
```bash
# MQTT server ports
export ROBUSTMQ_MQTT_SERVER_TCP_PORT=1883
export ROBUSTMQ_MQTT_SERVER_TLS_PORT=1885

# Authentication configuration
export ROBUSTMQ_MQTT_AUTH_STORAGE_STORAGE_TYPE="mysql"
export ROBUSTMQ_MQTT_AUTH_STORAGE_MYSQL_ADDR="localhost:3306"

# Runtime configuration
export ROBUSTMQ_MQTT_RUNTIME_DEFAULT_USER="admin"
export ROBUSTMQ_MQTT_RUNTIME_DEFAULT_PASSWORD="secure_password"
export ROBUSTMQ_MQTT_RUNTIME_MAX_CONNECTION_NUM=2000000

# System monitor configuration
export ROBUSTMQ_MQTT_SYSTEM_MONITOR_ENABLE=true
export ROBUSTMQ_MQTT_SYSTEM_MONITOR_OS_CPU_HIGH_WATERMARK=85.0

# Offline message configuration
export ROBUSTMQ_MQTT_OFFLINE_MESSAGE_ENABLE=true
export ROBUSTMQ_MQTT_OFFLINE_MESSAGE_MAX_MESSAGES_NUM=20000
```

---

## Performance Tuning Recommendations

### High Concurrency Scenarios
```toml
[mqtt_runtime]
max_connection_num = 10000000

[mqtt_protocol_config]
max_packet_size = 1048576      # 1MB
receive_max = 100

[network]
accept_thread_num = 8
handler_thread_num = 16
response_thread_num = 8
queue_size = 5000
```

### Low Latency Scenarios
```toml
[mqtt_system_monitor]
enable = true

[mqtt_slow_subscribe_config]
enable = true
delay_type = "Partial"

[network]
lock_max_try_mut_times = 10
lock_try_mut_sleep_time_ms = 10
```

### High Reliability Scenarios
```toml
[mqtt_message_storage]
storage_type = "journal"

[mqtt_offline_message]
enable = true
expire_ms = 604800000  # 7 days
max_messages_num = 100000

[mqtt_protocol_config]
client_pkid_persistent = true
max_session_expiry_interval = 86400  # 24 hours
```

---

## Troubleshooting

### Common Issues
1. **Connection Limit** - Adjust `max_connection_num` and system file descriptor limits
2. **High Memory Usage** - Adjust offline message configuration and storage type
3. **Performance Issues** - Optimize thread configuration and queue sizes
4. **Security Issues** - Check authentication configuration and security settings

### Debug Configuration
```toml
# Enable verbose logging
[mqtt_schema]
echo_log = true
log_level = "debug"

# Enable system monitoring
[mqtt_system_monitor]
enable = true

# Enable slow subscribe detection
[mqtt_slow_subscribe_config]
enable = true
```

---

*Documentation Version: v1.0*  
*Last Updated: 2024-01-01*  
*Based on Code Version: RobustMQ v0.1.31*
