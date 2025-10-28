# MQTT Broker 配置说明

> 本文档介绍 MQTT Broker 相关的所有配置项。通用配置信息请参考 [COMMON.md](COMMON.md)。

## MQTT 服务器配置

### 网络端口配置
```toml
[mqtt_server]
tcp_port = 1883              # MQTT TCP 端口
tls_port = 1885              # MQTT TLS 端口
websocket_port = 8083        # WebSocket 端口
websockets_port = 8085       # WebSocket over TLS 端口
quic_port = 9083            # QUIC 协议端口
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `tcp_port` | `u32` | `1883` | MQTT over TCP 协议端口 |
| `tls_port` | `u32` | `1885` | MQTT over TLS 协议端口 |
| `websocket_port` | `u32` | `8083` | MQTT over WebSocket 端口 |
| `websockets_port` | `u32` | `8085` | MQTT over WebSocket Secure 端口 |
| `quic_port` | `u32` | `9083` | MQTT over QUIC 协议端口 |

---

## MQTT 心跳保持配置

### Keep Alive 配置
```toml
[mqtt_keep_alive]
enable = true                # 是否启用心跳检测
default_time = 180          # 默认心跳间隔(秒)
max_time = 3600            # 最大心跳间隔(秒)
default_timeout = 2        # 默认超时次数
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `true` | 是否启用 Keep Alive 心跳检测 |
| `default_time` | `u16` | `180` | 默认心跳间隔时间（秒） |
| `max_time` | `u16` | `3600` | 最大心跳间隔时间（秒） |
| `default_timeout` | `u16` | `2` | 连续超时次数后断开连接 |

---

## MQTT 认证存储配置

### 认证存储配置
```toml
[mqtt_auth_config]
# 认证配置
[mqtt_auth_config.authn_config]
storage_type = "placement"    # 存储类型

# 授权配置
[mqtt_auth_config.authz_config]
# 授权相关配置
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `storage_type` | `string` | `"placement"` | 认证数据存储类型：placement, journal, mysql |
| `journal_addr` | `string` | `""` | Journal 引擎地址（当 storage_type 为 journal 时使用） |
| `mysql_addr` | `string` | `""` | MySQL 数据库地址（当 storage_type 为 mysql 时使用） |

### 存储类型说明
- **placement**: 使用元数据服务存储认证信息
- **journal**: 使用 Journal 引擎存储认证信息
- **mysql**: 使用 MySQL 数据库存储认证信息

---

## MQTT 消息存储配置

### 消息存储配置
```toml
[mqtt_message_storage]
storage_type = "memory"       # 存储类型
journal_addr = ""            # Journal 地址
mysql_addr = ""              # MySQL 地址
rocksdb_data_path = ""       # RocksDB 数据路径
rocksdb_max_open_files = 10000  # RocksDB 最大打开文件数
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `storage_type` | `string` | `"memory"` | 消息存储类型：memory, journal, mysql, rocksdb |
| `journal_addr` | `string` | `""` | Journal 引擎地址 |
| `mysql_addr` | `string` | `""` | MySQL 数据库地址 |
| `rocksdb_data_path` | `string` | `""` | RocksDB 数据存储路径 |
| `rocksdb_max_open_files` | `i32` | `10000` | RocksDB 最大打开文件数 |

### 存储类型说明
- **memory**: 内存存储（重启后数据丢失，适用于测试）
- **journal**: 使用 Journal 引擎持久化存储
- **mysql**: 使用 MySQL 数据库存储
- **rocksdb**: 使用 RocksDB 本地存储

---

## MQTT 运行时配置

### 运行时配置
```toml
[mqtt_runtime]
default_user = "admin"        # 默认用户名
default_password = "robustmq" # 默认密码
max_connection_num = 1000000  # 最大连接数
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `default_user` | `string` | `"admin"` | 系统默认用户名 |
| `default_password` | `string` | `"robustmq"` | 系统默认密码 |
| `max_connection_num` | `usize` | `1000000` | 单个节点最大连接数 |

---

## MQTT 协议配置

### 协议参数配置
```toml
[mqtt_protocol_config]
max_session_expiry_interval = 1800      # 最大会话过期间隔(秒)
default_session_expiry_interval = 30    # 默认会话过期间隔(秒)
topic_alias_max = 65535                 # 主题别名最大值
max_qos = 2                            # 最大 QoS 级别
max_packet_size = 10485760             # 最大数据包大小(字节)
receive_max = 65535                    # 接收最大值
client_pkid_persistent = false        # 客户端包ID持久化
max_message_expiry_interval = 3600     # 最大消息过期间隔(秒)
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `max_session_expiry_interval` | `u32` | `1800` | 会话最大过期时间（秒） |
| `default_session_expiry_interval` | `u32` | `30` | 会话默认过期时间（秒） |
| `topic_alias_max` | `u16` | `65535` | 主题别名最大数量 |
| `max_qos` | `u8` | `2` | 支持的最大 QoS 级别 |
| `max_packet_size` | `u32` | `10485760` | 单个 MQTT 数据包最大大小（字节） |
| `receive_max` | `u16` | `65535` | 未确认的 PUBLISH 数据包最大数量 |
| `max_message_expiry_interval` | `u64` | `3600` | 消息最大过期时间（秒） |
| `client_pkid_persistent` | `bool` | `false` | 是否持久化客户端包标识符 |

---

## MQTT 安全配置

### 安全配置
```toml
[mqtt_security]
secret_free_login = false            # 是否允许免密登录
is_self_protection_status = false   # 是否启用自我保护模式
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `secret_free_login` | `bool` | `false` | 是否允许无密码登录 |
| `is_self_protection_status` | `bool` | `false` | 是否启用自我保护模式 |

---

## MQTT 离线消息配置

### 离线消息配置
```toml
[mqtt_offline_message]
enable = true                # 是否启用离线消息
expire_ms = 3600000         # 消息过期时间(毫秒)
max_messages_num = 1000     # 最大离线消息数量
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `true` | 是否启用离线消息功能 |
| `expire_ms` | `u32` | `0` | 离线消息过期时间（毫秒），0表示不过期 |
| `max_messages_num` | `u32` | `0` | 每个客户端最大离线消息数，0表示无限制 |

---

## MQTT 系统监控配置

### 系统监控配置
```toml
[mqtt_system_monitor]
enable = false                  # 是否启用系统监控
os_cpu_high_watermark = 70.0   # CPU 高水位线(%)
os_memory_high_watermark = 80.0 # 内存高水位线(%)
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `false` | 是否启用系统资源监控 |
| `os_cpu_high_watermark` | `f32` | `70.0` | CPU 使用率高水位线（百分比） |
| `os_memory_high_watermark` | `f32` | `80.0` | 内存使用率高水位线（百分比） |

---

## MQTT 慢订阅配置

### 慢订阅配置
```toml
[mqtt_slow_subscribe_config]
enable = false               # 是否启用慢订阅检测
record_time = 1000          # 记录时间阈值(毫秒)
delay_type = "Whole"        # 延迟类型
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `false` | 是否启用慢订阅检测 |
| `record_time` | `u64` | `1000` | 慢订阅记录时间阈值（毫秒） |
| `delay_type` | `DelayType` | `Whole` | 延迟计算类型：Whole, Partial |

---

## MQTT 连接抖动检测配置

### 抖动检测配置
```toml
[mqtt_flapping_detect]
enable = false                    # 是否启用连接抖动检测
window_time = 60                 # 时间窗口(秒)
max_client_connections = 15      # 最大连接次数
ban_time = 300                   # 封禁时间(秒)
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `false` | 是否启用连接抖动检测 |
| `window_time` | `u32` | `60` | 检测时间窗口（秒） |
| `max_client_connections` | `u64` | `15` | 时间窗口内最大连接次数 |
| `ban_time` | `u32` | `300` | 触发抖动后的封禁时间（秒） |

---

## MQTT Schema 配置

### Schema 验证配置
```toml
[mqtt_schema]
enable = true                        # 是否启用 Schema 验证
strategy = "ALL"                     # 验证策略
failed_operation = "Discard"         # 验证失败操作
echo_log = true                      # 是否输出日志
log_level = "info"                   # 日志级别
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `true` | 是否启用 Schema 验证功能 |
| `strategy` | `SchemaStrategy` | `ALL` | 验证策略：ALL（全部验证）, Any（任一验证） |
| `failed_operation` | `SchemaFailedOperation` | `Discard` | 验证失败时的操作 |
| `echo_log` | `bool` | `true` | 是否输出 Schema 验证日志 |
| `log_level` | `string` | `"info"` | Schema 验证日志级别 |

### 验证策略说明
- **ALL**: 消息必须通过所有绑定的 Schema 验证
- **Any**: 消息只需通过任一绑定的 Schema 验证

### 失败操作说明
- **Discard**: 丢弃验证失败的消息
- **DisconnectAndDiscard**: 断开连接并丢弃消息
- **Ignore**: 忽略验证失败，继续处理消息

---

## MQTT 完整配置示例

### 生产环境配置
```toml
# MQTT 服务器端口配置
[mqtt.server]
tcp_port = 1883
tls_port = 1885
websocket_port = 8083
websockets_port = 8085
quic_port = 9083

# 认证存储配置
[mqtt.auth.storage]
storage_type = "placement"

# 消息存储配置
[mqtt.message.storage]
storage_type = "journal"
journal_addr = "127.0.0.1:1778"

# 运行时配置
[mqtt.runtime]
default_user = "admin"
default_password = "your_secure_password"
max_connection_num = 5000000

# 系统监控配置
[mqtt.system_monitor]
enable = true
os_cpu_check_interval_ms = 30000
os_cpu_high_watermark = 80.0
os_cpu_low_watermark = 40.0
os_memory_check_interval_ms = 30000
os_memory_high_watermark = 85.0

# 离线消息配置
[mqtt.offline_message]
enable = true
expire_ms = 86400000  # 24小时
max_messages_num = 10000

# 慢订阅检测配置
[mqtt.slow_subscribe]
enable = true
max_store_num = 5000
delay_type = "Whole"

# 连接抖动检测配置
[mqtt.flapping_detect]
enable = true
window_time = 120
max_client_connections = 10
ban_time = 600

# 协议配置
[mqtt.protocol]
max_session_expiry_interval = 7200
default_session_expiry_interval = 300
topic_alias_max = 1000
max_qos = 2
max_packet_size = 10485760
max_server_keep_alive = 7200
default_server_keep_alive = 300
receive_max = 1000
client_pkid_persistent = true
max_message_expiry_interval = 86400

# 安全配置
[mqtt.security]
secret_free_login = false
is_self_protection_status = true

# Schema 验证配置
[mqtt.schema]
enable = true
strategy = "ALL"
failed_operation = "Discard"
echo_log = true
log_level = "warn"
```

---

## 环境变量覆盖示例

### MQTT 相关环境变量
```bash
# MQTT 服务器端口
export ROBUSTMQ_MQTT_SERVER_TCP_PORT=1883
export ROBUSTMQ_MQTT_SERVER_TLS_PORT=1885

# 认证配置
export ROBUSTMQ_MQTT_AUTH_STORAGE_STORAGE_TYPE="mysql"
export ROBUSTMQ_MQTT_AUTH_STORAGE_MYSQL_ADDR="localhost:3306"

# 运行时配置
export ROBUSTMQ_MQTT_RUNTIME_DEFAULT_USER="admin"
export ROBUSTMQ_MQTT_RUNTIME_DEFAULT_PASSWORD="secure_password"
export ROBUSTMQ_MQTT_RUNTIME_MAX_CONNECTION_NUM=2000000

# 系统监控配置
export ROBUSTMQ_MQTT_SYSTEM_MONITOR_ENABLE=true
export ROBUSTMQ_MQTT_SYSTEM_MONITOR_OS_CPU_HIGH_WATERMARK=85.0

# 离线消息配置
export ROBUSTMQ_MQTT_OFFLINE_MESSAGE_ENABLE=true
export ROBUSTMQ_MQTT_OFFLINE_MESSAGE_MAX_MESSAGES_NUM=20000
```

---

## 性能调优建议

### 高并发场景
```toml
[mqtt.runtime]
max_connection_num = 10000000

[mqtt.protocol]
max_packet_size = 1048576      # 1MB
receive_max = 100
max_server_keep_alive = 300

[network]
accept_thread_num = 8
handler_thread_num = 16
response_thread_num = 8
queue_size = 5000
```

### 低延迟场景
```toml
[mqtt.system_monitor]
enable = true
os_cpu_check_interval_ms = 10000
os_memory_check_interval_ms = 10000

[mqtt.slow_subscribe]
enable = true
delay_type = "Partial"

[network]
lock_max_try_mut_times = 10
lock_try_mut_sleep_time_ms = 10
```

### 高可靠性场景
```toml
[mqtt.message.storage]
storage_type = "journal"

[mqtt.offline_message]
enable = true
expire_ms = 604800000  # 7天
max_messages_num = 100000

[mqtt.protocol]
client_pkid_persistent = true
max_session_expiry_interval = 86400  # 24小时
```

---

## 故障排除

### 常见问题
1. **连接数限制** - 调整 `max_connection_num` 和系统文件描述符限制
2. **内存使用过高** - 调整离线消息配置和存储类型
3. **性能问题** - 优化线程配置和队列大小
4. **安全问题** - 检查认证配置和安全设置

### 调试配置
```toml
# 开启详细日志
[mqtt.schema]
echo_log = true
log_level = "debug"

# 启用系统监控
[mqtt.system_monitor]
enable = true
os_cpu_check_interval_ms = 10000

# 启用慢订阅检测
[mqtt.slow_subscribe]
enable = true
```

---

*文档版本: v1.0*
*最后更新: 2024-01-01*
*基于代码版本: RobustMQ v0.1.31*
