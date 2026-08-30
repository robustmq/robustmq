# MQTT 配置说明

> 本文档描述 RobustMQ MQTT 协议的所有配置项。全局/基础配置请参考 [Broker 配置说明](BROKER.md)；其他协议配置见 [Kafka 配置](KafkaConfig.md)、[AMQP 配置](AMQPConfig.md)、[NATS 配置](NATSConfig.md)。

## 概述

所有 MQTT 相关配置都汇总在 TOML 的 `[mqtt_runtime]` 表下（对应 Rust 侧的 `MqttRuntime` struct，与 `[kafka_runtime]` 是同一种组织方式），本文档按子表逐一说明。环境变量覆盖规则与 [Broker 配置说明](BROKER.md#环境变量覆盖) 一致，例如：

```bash
export ROBUST_MQ_SERVER_MQTT_RUNTIME_SERVER_TCP_PORT=1883
```

`[mqtt_runtime]` 表本身还直接挂着一个顶层字段（不属于任何子表）：

```toml
[mqtt_runtime]
broker_worker_threads = 0  # 0 = 自动：CPU 核数
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `broker_worker_threads` | `usize` | `0`（自动） | broker-runtime（MQTT 连接处理、消息投递热路径）工作线程数，`0` = 自动取 CPU 核数 |

---

## 1. MQTT 服务器配置

### [mqtt_runtime.server]

MQTT 协议监听端口配置。

```toml
[mqtt_runtime.server]
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

## 2. MQTT 认证与运行时配置

### [mqtt_runtime.auth]

```toml
[mqtt_runtime.auth]
default_user = "admin"
default_password = "robustmq"
durable_sessions_enable = false
secret_free_login = false
is_self_protection_status = false
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `default_user` | `string` | `"admin"` | 系统默认用户名 |
| `default_password` | `string` | `"robustmq"` | 系统默认密码 |
| `durable_sessions_enable` | `bool` | `false` | 是否启用持久会话（`false` 为临时会话，性能更好） |
| `secret_free_login` | `bool` | `false` | 是否允许免密登录 |
| `is_self_protection_status` | `bool` | `false` | 是否处于自我保护状态（连接过载时拒绝新连接） |

> MQTT 的网络线程复用统一的 `[broker_network]` 配置（见 [Broker 配置说明](BROKER.md)），不再有独立的 `[mqtt_runtime.auth.network]`。

---

## 3. MQTT Keep Alive 配置

### [mqtt_runtime.keep_alive]

```toml
[mqtt_runtime.keep_alive]
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

## 4. MQTT 协议配置

### [mqtt_runtime.protocol]

```toml
[mqtt_runtime.protocol]
max_session_expiry_interval = 1800
default_session_expiry_interval = 30
topic_alias_max = 65535
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
| `max_packet_size` | `u32` | `10485760` (10 MB) | 单个 MQTT 数据包最大大小（字节） |
| `receive_max` | `u16` | `65535` | 未确认的 PUBLISH 数据包最大数量 |
| `max_message_expiry_interval` | `u64` | `3600` | 消息最大过期时间（秒） |
| `client_pkid_persistent` | `bool` | `false` | 是否持久化客户端 Packet ID |

---

## 5. MQTT 限流配置

### [mqtt_runtime.limit]

集群和租户级别的连接/流量限流配置。

```toml
[mqtt_runtime.limit.cluster]
max_connections_per_node = 10000000
max_connection_rate = 100000
max_topics = 5000000
max_sessions = 50000000
max_publish_rate = 10000

[mqtt_runtime.limit.tenant]
max_connections_per_node = 1000000
max_connection_rate = 10000
max_topics = 500000
max_sessions = 5000000
max_publish_rate = 10000
```

| 配置项 | 类型 | 说明 |
|--------|------|------|
| `max_connections_per_node` | `u64` | 每节点最大连接数 |
| `max_connection_rate` | `u32` | 每秒最大新建连接速率 |
| `max_topics` | `u64` | 最大 Topic 数量 |
| `max_sessions` | `u64` | 最大 Session 数量 |
| `max_publish_rate` | `u32` | 每秒最大 Publish 消息速率 |

---

## 6. MQTT 离线消息配置

### [mqtt_runtime.offline_message]

客户端离线期间的消息存储配置。

```toml
[mqtt_runtime.offline_message]
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

## 7. MQTT 连接抖动检测配置

### [mqtt_runtime.flapping_detect]

检测客户端频繁连接/断开（抖动）并自动封禁。

```toml
[mqtt_runtime.flapping_detect]
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

## 8. MQTT 慢订阅检测配置

### [mqtt_runtime.slow_subscribe]

慢订阅监控配置，用于检测消息分发延迟。

```toml
[mqtt_runtime.slow_subscribe]
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

## 9. MQTT Schema 验证配置

### [mqtt_runtime.schema]

消息 Schema 验证配置。

```toml
[mqtt_runtime.schema]
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

## 10. MQTT 系统监控配置

### [mqtt_runtime.system_monitor]

系统资源监控配置。

```toml
[mqtt_runtime.system_monitor]
enable = false
os_cpu_high_watermark = 70.0
os_memory_high_watermark = 80.0
system_topic_interval_ms = 60000
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable` | `bool` | `false` | 是否启用系统资源监控 |
| `os_cpu_high_watermark` | `f32` | `70.0` | CPU 使用率高水位线（%） |
| `os_memory_high_watermark` | `f32` | `80.0` | 内存使用率高水位线（%） |
| `system_topic_interval_ms` | `u64` | `60000` | 系统 Topic 指标发布间隔（毫秒） |

---

## 完整示例

```toml
[mqtt_runtime]
broker_worker_threads = 0

[mqtt_runtime.server]
tcp_port = 1883
tls_port = 1885
websocket_port = 8083
websockets_port = 8085
quic_port = 9083

[mqtt_runtime.auth]
default_user = "admin"
default_password = "your_secure_password"
durable_sessions_enable = false
secret_free_login = false
is_self_protection_status = false

[mqtt_runtime.keep_alive]
enable = true
default_time = 180
max_time = 3600
default_timeout = 2

[mqtt_runtime.protocol]
max_session_expiry_interval = 1800
default_session_expiry_interval = 30
topic_alias_max = 65535
max_packet_size = 10485760
receive_max = 65535
max_message_expiry_interval = 3600
client_pkid_persistent = false

[mqtt_runtime.offline_message]
enable = true
expire_ms = 0
max_messages_num = 0

[mqtt_runtime.flapping_detect]
enable = false
window_time = 1
max_client_connections = 15
ban_time = 5

[mqtt_runtime.slow_subscribe]
enable = false
record_time = 1000
delay_type = "Whole"

[mqtt_runtime.schema]
enable = true
strategy = "ALL"
failed_operation = "Discard"
echo_log = true
log_level = "info"

[mqtt_runtime.system_monitor]
enable = false
os_cpu_high_watermark = 70.0
os_memory_high_watermark = 80.0
system_topic_interval_ms = 60000
```

## 延伸阅读

- [Broker 配置说明](BROKER.md) — 全局/基础配置
- [RobustMQ MQTT 核心概念](../RobustMQ-MQTT/MQTTCoreConcepts.md)
