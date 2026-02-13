# Broker 配置说明

> 本文档描述 RobustMQ Broker 服务的所有配置项。日志配置请参考 [Logging.md](Logging.md)。

## 概述

RobustMQ 使用 TOML 格式的配置文件来管理系统配置。主配置文件为 `config/server.toml`。

### 配置加载优先级

1. 环境变量（最高）
2. 配置文件
3. 默认值（最低）

### 环境变量覆盖

支持通过环境变量覆盖配置文件中的设置。命名规则：

```
ROBUST_MQ_SERVER_{SECTION}_{KEY}
```

- 顶层配置项：`ROBUST_MQ_SERVER_{KEY}`
- Section 内配置项：`ROBUST_MQ_SERVER_{SECTION}_{KEY}`
- 所有字母大写，`.` 替换为 `_`

示例：

```bash
export ROBUST_MQ_SERVER_CLUSTER_NAME="my-cluster"
export ROBUST_MQ_SERVER_MQTT_SERVER_TCP_PORT=1883
export ROBUST_MQ_SERVER_PROMETHEUS_PORT=9091
```

---

## 1. 基础配置

顶层配置项，定义集群和节点的基本信息。

```toml
cluster_name = "robust_mq_cluster_default"
broker_id = 1
broker_ip = "127.0.0.1"
roles = ["broker", "meta"]
grpc_port = 1228
http_port = 8080

[meta_addrs]
1 = "127.0.0.1:1228"
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `cluster_name` | `string` | `"robust_mq_cluster_default"` | 集群名称，同一集群内所有节点必须一致 |
| `broker_id` | `u64` | `1` | 节点唯一标识 |
| `broker_ip` | `string` | 自动获取本机 IP | 节点 IP 地址 |
| `roles` | `array` | `["broker", "meta"]` | 节点角色列表，可选值：`meta`、`broker`、`engine` |
| `grpc_port` | `u32` | `1228` | gRPC 服务端口 |
| `http_port` | `u32` | `8080` | HTTP API 服务端口 |
| `meta_addrs` | `table` | `{1 = "127.0.0.1:1228"}` | Meta 节点地址映射，键为节点 ID，值为 `IP:端口` |

### 部署模式

- **一体化部署**：`roles = ["meta", "broker", "engine"]`
- **分离式部署**：
  - Meta 节点：`roles = ["meta"]`
  - Broker 节点：`roles = ["broker"]`
  - Engine 节点：`roles = ["engine"]`

---

## 2. 运行时配置

### [runtime]

Tokio 运行时与 TLS 配置。

```toml
[runtime]
runtime_worker_threads = 8
tls_cert = "./config/certs/cert.pem"
tls_key = "./config/certs/key.pem"
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `runtime_worker_threads` | `usize` | 自动检测 CPU 核数 | Tokio 运行时工作线程数 |
| `tls_cert` | `string` | `"./config/certs/cert.pem"` | TLS 证书文件路径 |
| `tls_key` | `string` | `"./config/certs/key.pem"` | TLS 私钥文件路径 |

---

## 3. 网络配置

### [network]

网络层线程与队列配置。

```toml
[network]
accept_thread_num = 8
handler_thread_num = 32
response_thread_num = 8
queue_size = 1000
lock_max_try_mut_times = 30
lock_try_mut_sleep_time_ms = 50
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `accept_thread_num` | `usize` | `8` | 接受新连接的线程数 |
| `handler_thread_num` | `usize` | `32` | 请求处理线程数 |
| `response_thread_num` | `usize` | `8` | 响应发送线程数 |
| `queue_size` | `usize` | `1000` | 内部消息队列大小 |
| `lock_max_try_mut_times` | `u64` | `30` | 锁获取最大重试次数 |
| `lock_try_mut_sleep_time_ms` | `u64` | `50` | 锁重试间隔时间（毫秒） |

---

## 4. Meta 运行时配置

### [meta_runtime]

元数据服务心跳与 Raft 配置。

```toml
[meta_runtime]
heartbeat_timeout_ms = 30000
heartbeat_check_time_ms = 1000
raft_write_timeout_sec = 30
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `heartbeat_timeout_ms` | `u64` | `30000` | 节点心跳超时时间（毫秒），超时后标记节点不可用 |
| `heartbeat_check_time_ms` | `u64` | `1000` | 心跳检查间隔（毫秒） |
| `raft_write_timeout_sec` | `u64` | `30` | Raft 写操作超时时间（秒） |

---

## 5. RocksDB 配置

### [rocksdb]

本地 RocksDB 存储配置。

```toml
[rocksdb]
data_path = "./data"
max_open_files = 10000
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `data_path` | `string` | `"./data"` | RocksDB 数据存储目录 |
| `max_open_files` | `i32` | `10000` | 最大同时打开文件数 |

---

## 6. 存储引擎运行时配置

### [storage_runtime]

Journal 存储引擎运行时配置。

```toml
[storage_runtime]
tcp_port = 1778
max_segment_size = 1073741824
io_thread_num = 8
data_path = []
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `tcp_port` | `u32` | `1778` | 存储引擎 TCP 端口 |
| `max_segment_size` | `u32` | `1073741824` (1 GB) | 单个 Segment 文件最大大小（字节） |
| `io_thread_num` | `u32` | `8` | IO 处理线程数 |
| `data_path` | `array` | `[]` | 数据存储路径列表 |

---

## 7. 消息存储配置

### [message_storage]

消息持久化存储后端配置。

```toml
[message_storage]
storage_type = "EngineMemory"
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `storage_type` | `string` | `"EngineMemory"` | 存储类型 |

**存储类型可选值：**

| 值 | 说明 |
|----|------|
| `EngineMemory` | 内存存储（重启后数据丢失，适用于测试） |
| `EngineSegment` | 基于 Segment 的存储引擎 |
| `EngineRocksDB` | 基于 RocksDB 的本地存储 |
| `Mysql` | MySQL 数据库存储 |
| `MinIO` | MinIO 对象存储 |
| `S3` | AWS S3 对象存储 |

根据所选 `storage_type`，需配置对应子项：

**memory_config（EngineMemory 时可选）：**

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `max_records_per_shard` | `usize` | `1000` | 每个 Shard 的最大记录数 |
| `max_shard_size_limit` | `usize` | `10000000` | 每个 Shard 的最大总大小 |

**mysql_config（Mysql 时）：**

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `mysql_addr` | `string` | `""` | MySQL 数据库地址 |

**minio_config（MinIO 时）：**

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `data_dir` | `string` | `""` | MinIO 数据目录 |
| `bucket` | `string` | `""` | MinIO Bucket 名称 |

**s3_config（S3 时）：**

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `endpoint` | `string` | `""` | S3 端点地址 |
| `bucket` | `string` | `""` | S3 Bucket 名称 |
| `region` | `string` | `""` | S3 Region |
| `access_key` | `string` | `""` | 访问密钥 |
| `secret_key` | `string` | `""` | 密钥 |
| `enable_virtual_host_style` | `bool` | `false` | 是否使用虚拟主机风格访问 |

---

## 8. Offset 存储配置

### [storage_offset]

消息消费 Offset 缓存配置。

```toml
[storage_offset]
enable_cache = true
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable_cache` | `bool` | `true` | 是否启用 Offset 缓存 |

---

## 9. MQTT 服务器配置

### [mqtt_server]

MQTT 协议监听端口配置。

```toml
[mqtt_server]
tcp_port = 1883
tls_port = 1885
websocket_port = 8083
websockets_port = 8085
quic_port = 9083
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `tcp_port` | `u32` | `1883` | MQTT over TCP 端口 |
| `tls_port` | `u32` | `1885` | MQTT over TLS 端口 |
| `websocket_port` | `u32` | `8083` | MQTT over WebSocket 端口 |
| `websockets_port` | `u32` | `8085` | MQTT over WebSocket Secure 端口 |
| `quic_port` | `u32` | `9083` | MQTT over QUIC 端口 |

---

## 10. MQTT 运行时配置

### [mqtt_runtime]

MQTT 运行时基本参数。

```toml
[mqtt_runtime]
default_user = "admin"
default_password = "robustmq"
max_connection_num = 1000000
durable_sessions_enable = false
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `default_user` | `string` | `"admin"` | 系统默认用户名 |
| `default_password` | `string` | `"robustmq"` | 系统默认密码 |
| `max_connection_num` | `usize` | `1000000` | 单节点最大连接数 |
| `durable_sessions_enable` | `bool` | `false` | 是否启用持久会话（`false` 为临时会话，性能更好） |

---

## 11. MQTT Keep Alive 配置

### [mqtt_keep_alive]

MQTT 心跳保活配置。

```toml
[mqtt_keep_alive]
enable = true
default_time = 180
max_time = 3600
default_timeout = 2
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `true` | 是否启用 Keep Alive 心跳检测 |
| `default_time` | `u16` | `180` | 默认心跳间隔（秒） |
| `max_time` | `u16` | `3600` | 最大心跳间隔（秒） |
| `default_timeout` | `u16` | `2` | 连续超时次数后断开连接 |

---

## 12. MQTT 协议配置

### [mqtt_protocol_config]

MQTT 协议参数配置。

```toml
[mqtt_protocol_config]
max_session_expiry_interval = 1800
default_session_expiry_interval = 30
topic_alias_max = 65535
max_qos_flight_message = 2
max_packet_size = 10485760
receive_max = 65535
max_message_expiry_interval = 3600
client_pkid_persistent = false
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `max_session_expiry_interval` | `u32` | `1800` | 会话最大过期时间（秒） |
| `default_session_expiry_interval` | `u32` | `30` | 会话默认过期时间（秒） |
| `topic_alias_max` | `u16` | `65535` | 主题别名最大数量 |
| `max_qos_flight_message` | `u8` | `2` | QoS 飞行窗口最大消息数 |
| `max_packet_size` | `u32` | `10485760` (10 MB) | 单个 MQTT 数据包最大大小（字节） |
| `receive_max` | `u16` | `65535` | 未确认的 PUBLISH 数据包最大数量 |
| `max_message_expiry_interval` | `u64` | `3600` | 消息最大过期时间（秒） |
| `client_pkid_persistent` | `bool` | `false` | 是否持久化客户端 Packet ID |

---

## 13. MQTT 安全配置

### [mqtt_security]

MQTT 集群安全相关动态配置。

```toml
[mqtt_security]
is_self_protection_status = false
secret_free_login = false
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `is_self_protection_status` | `bool` | `false` | 是否启用自我保护模式 |
| `secret_free_login` | `bool` | `false` | 是否允许免密登录 |

---

## 14. MQTT 认证与鉴权配置

### [mqtt_auth_config]

MQTT 认证（AuthN）和鉴权（AuthZ）配置。

```toml
[mqtt_auth_config.authn_config]
authn_type = "password_based"

[mqtt_auth_config.authn_config.password_based_config]
# 密码认证相关配置

[mqtt_auth_config.authn_config.password_based_config.storage_config]
storage_type = "placement"

[mqtt_auth_config.authn_config.password_based_config.password_config]
credential_type = "username"
algorithm = "plain"
salt_position = "disable"

[mqtt_auth_config.authz_config.storage_config]
storage_type = "placement"
```

**认证类型（authn_type）可选值：** `password_based`、`JWT`

**密码算法（algorithm）可选值：** `plain`、`md5`、`sha`、`sha256`、`sha512`、`bcrypt`、`pbkdf2`

**认证/鉴权数据存储（storage_type）可选值：** `placement`、`mysql`、`postgresql`、`redis`、`http`、`file`（仅鉴权）

---

## 15. MQTT 离线消息配置

### [mqtt_offline_message]

客户端离线期间的消息存储配置。

```toml
[mqtt_offline_message]
enable = true
expire_ms = 0
max_messages_num = 0
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `true` | 是否启用离线消息 |
| `expire_ms` | `u32` | `0` | 离线消息过期时间（毫秒），`0` 表示不过期 |
| `max_messages_num` | `u32` | `0` | 每个客户端最大离线消息数，`0` 表示无限制 |

---

## 16. MQTT 连接抖动检测配置

### [mqtt_flapping_detect]

检测客户端频繁连接/断开（抖动）并自动封禁。

```toml
[mqtt_flapping_detect]
enable = false
window_time = 1
max_client_connections = 15
ban_time = 5
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `false` | 是否启用连接抖动检测 |
| `window_time` | `u32` | `1` | 检测时间窗口（秒） |
| `max_client_connections` | `u64` | `15` | 时间窗口内最大连接次数 |
| `ban_time` | `u32` | `5` | 触发抖动后封禁时间（秒） |

---

## 17. MQTT 慢订阅检测配置

### [mqtt_slow_subscribe_config]

慢订阅监控配置，用于检测消息分发延迟。

```toml
[mqtt_slow_subscribe_config]
enable = false
record_time = 1000
delay_type = "Whole"
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `false` | 是否启用慢订阅检测 |
| `record_time` | `u64` | `1000` | 慢订阅记录阈值（毫秒） |
| `delay_type` | `string` | `"Whole"` | 延迟计算类型：`Whole`（全链路）、`Partial`（部分） |

---

## 18. MQTT Schema 验证配置

### [mqtt_schema]

消息 Schema 验证配置。

```toml
[mqtt_schema]
enable = true
strategy = "ALL"
failed_operation = "Discard"
echo_log = true
log_level = "info"
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `true` | 是否启用 Schema 验证 |
| `strategy` | `string` | `"ALL"` | 验证策略 |
| `failed_operation` | `string` | `"Discard"` | 验证失败时的操作 |
| `echo_log` | `bool` | `true` | 是否输出 Schema 验证日志 |
| `log_level` | `string` | `"info"` | Schema 验证日志级别 |

**验证策略（strategy）：**
- `ALL`：消息必须通过所有绑定的 Schema 验证
- `Any`：消息只需通过任一 Schema 验证

**失败操作（failed_operation）：**
- `Discard`：丢弃验证失败的消息
- `DisconnectAndDiscard`：断开连接并丢弃消息
- `Ignore`：忽略验证失败，继续处理

---

## 19. MQTT 系统监控配置

### [mqtt_system_monitor]

系统资源监控配置。

```toml
[mqtt_system_monitor]
enable = false
os_cpu_high_watermark = 70.0
os_memory_high_watermark = 80.0
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `false` | 是否启用系统资源监控 |
| `os_cpu_high_watermark` | `f32` | `70.0` | CPU 使用率高水位线（%） |
| `os_memory_high_watermark` | `f32` | `80.0` | 内存使用率高水位线（%） |

---

## 20. 监控配置

### [prometheus]

Prometheus 指标暴露配置。

```toml
[prometheus]
enable = true
port = 9090
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `true` | 是否启用 Prometheus 指标收集 |
| `port` | `u32` | `9090` | Prometheus 指标暴露端口 |

### [p_prof]

PProf 性能分析配置。

```toml
[p_prof]
enable = false
port = 6060
frequency = 100
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `false` | 是否启用 PProf 性能分析 |
| `port` | `u16` | `6060` | PProf 服务端口 |
| `frequency` | `i32` | `100` | 采样频率 |

---

## 完整配置示例

```toml
# ========== 基础配置 ==========
cluster_name = "production-cluster"
broker_id = 1
roles = ["meta", "broker", "engine"]
grpc_port = 1228
http_port = 8080

[meta_addrs]
1 = "192.168.1.10:1228"
2 = "192.168.1.11:1228"
3 = "192.168.1.12:1228"

# ========== 运行时 ==========
[runtime]
runtime_worker_threads = 8
tls_cert = "./config/certs/cert.pem"
tls_key = "./config/certs/key.pem"

# ========== 网络 ==========
[network]
accept_thread_num = 8
handler_thread_num = 32
response_thread_num = 8
queue_size = 2000

# ========== Meta ==========
[meta_runtime]
heartbeat_timeout_ms = 30000
heartbeat_check_time_ms = 1000
raft_write_timeout_sec = 30

# ========== RocksDB ==========
[rocksdb]
data_path = "/data/robustmq"
max_open_files = 20000

# ========== 存储引擎 ==========
[storage_runtime]
tcp_port = 1778
max_segment_size = 1073741824
io_thread_num = 8

# ========== 消息存储 ==========
[message_storage]
storage_type = "EngineMemory"

# ========== Offset 缓存 ==========
[storage_offset]
enable_cache = true

# ========== MQTT 服务器 ==========
[mqtt_server]
tcp_port = 1883
tls_port = 1885
websocket_port = 8083
websockets_port = 8085
quic_port = 9083

# ========== MQTT 运行时 ==========
[mqtt_runtime]
default_user = "admin"
default_password = "your_secure_password"
max_connection_num = 1000000
durable_sessions_enable = false

# ========== MQTT Keep Alive ==========
[mqtt_keep_alive]
enable = true
default_time = 180
max_time = 3600
default_timeout = 2

# ========== MQTT 协议 ==========
[mqtt_protocol_config]
max_session_expiry_interval = 1800
default_session_expiry_interval = 30
topic_alias_max = 65535
max_qos_flight_message = 2
max_packet_size = 10485760
receive_max = 65535
max_message_expiry_interval = 3600
client_pkid_persistent = false

# ========== MQTT 安全 ==========
[mqtt_security]
is_self_protection_status = false
secret_free_login = false

# ========== MQTT 认证 ==========
[mqtt_auth_config.authn_config]
authn_type = "password_based"

[mqtt_auth_config.authz_config.storage_config]
storage_type = "placement"

# ========== MQTT 离线消息 ==========
[mqtt_offline_message]
enable = true
expire_ms = 0
max_messages_num = 0

# ========== MQTT 抖动检测 ==========
[mqtt_flapping_detect]
enable = false
window_time = 1
max_client_connections = 15
ban_time = 5

# ========== MQTT 慢订阅 ==========
[mqtt_slow_subscribe_config]
enable = false
record_time = 1000
delay_type = "Whole"

# ========== MQTT Schema ==========
[mqtt_schema]
enable = true
strategy = "ALL"
failed_operation = "Discard"
echo_log = true
log_level = "info"

# ========== MQTT 系统监控 ==========
[mqtt_system_monitor]
enable = false
os_cpu_high_watermark = 70.0
os_memory_high_watermark = 80.0

# ========== 监控 ==========
[prometheus]
enable = true
port = 9090

[p_prof]
enable = false
port = 6060
frequency = 100

# ========== 日志 ==========
[log]
log_config = "./config/broker-tracing.toml"
log_path = "./logs"
```
