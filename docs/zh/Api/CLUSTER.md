# 集群管理 HTTP API

> 本文档介绍集群配置和管理相关的 HTTP API 接口。通用信息请参考 [COMMON.md](COMMON.md)。

## 通用响应格式

所有接口均返回统一的 JSON 响应结构：

**成功响应**:
```json
{
  "code": 0,
  "data": "...",
  "error": null
}
```

**失败响应**:
```json
{
  "code": 100,
  "data": "",
  "error": "错误信息"
}
```

---

## 集群配置管理

### 1. 获取集群配置

- **接口**: `GET /api/cluster/config/get`
- **描述**: 获取当前集群的完整 `BrokerConfig` 配置信息
- **请求参数**: 无

- **响应示例**:
```json
{
  "code": 0,
  "data": {
    "cluster_name": "robust_mq_cluster_default",
    "broker_id": 1,
    "broker_ip": "192.168.1.100",
    "roles": ["broker", "meta"],
    "grpc_port": 1228,
    "http_port": 8080,
    "meta_addrs": {
      "1": "127.0.0.1:1228"
    },
    "prometheus": {
      "enable": true,
      "port": 9090
    },
    "log": {
      "log_path": "./logs",
      "log_config": "./config/broker-tracing.toml"
    },
    "runtime": {
      "runtime_worker_threads": 1,
      "server_worker_threads": 0,
      "meta_worker_threads": 0,
      "broker_worker_threads": 0,
      "channels_per_address": 10,
      "tls_cert": "./config/certs/cert.pem",
      "tls_key": "./config/certs/key.pem"
    },
    "network": {
      "accept_thread_num": 8,
      "handler_thread_num": 32,
      "queue_size": 1000
    },
    "pprof": {
      "enable": false,
      "port": 6060,
      "frequency": 100
    },
    "rocksdb": {
      "data_path": "./data",
      "max_open_files": 10000
    },
    "llm_client": null,
    "cluster_limit": {
      "max_network_connection": 100000000,
      "max_network_connection_rate": 10000,
      "max_admin_http_uri_rate": 50
    },
    "meta_runtime": {
      "heartbeat_timeout_ms": 30000,
      "heartbeat_check_time_ms": 1000,
      "raft_write_timeout_sec": 30,
      "offset_raft_group_num": 1,
      "data_raft_group_num": 1
    },
    "storage_runtime": {
      "tcp_port": 1778,
      "max_segment_size": 1073741824,
      "io_thread_num": 8,
      "data_path": [],
      "offset_enable_cache": true
    },
    "mqtt_server": {
      "tcp_port": 1883,
      "tls_port": 1885,
      "websocket_port": 8083,
      "websockets_port": 8085,
      "quic_port": 9083
    },
    "mqtt_keep_alive": {
      "enable": true,
      "default_time": 180,
      "max_time": 3600,
      "default_timeout": 2
    },
    "mqtt_runtime": {
      "default_user": "admin",
      "default_password": "robustmq",
      "durable_sessions_enable": false,
      "secret_free_login": false,
      "is_self_protection_status": false
    },
    "mqtt_offline_message": {
      "enable": true,
      "expire_ms": 0,
      "max_messages_num": 0
    },
    "mqtt_slow_subscribe": {
      "enable": false,
      "record_time": 1000,
      "delay_type": "Whole"
    },
    "mqtt_flapping_detect": {
      "enable": false,
      "window_time": 1,
      "max_client_connections": 15,
      "ban_time": 5
    },
    "mqtt_protocol": {
      "max_session_expiry_interval": 1800,
      "default_session_expiry_interval": 30,
      "topic_alias_max": 65535,
      "max_packet_size": 10485760,
      "receive_max": 65535,
      "max_message_expiry_interval": 3600,
      "client_pkid_persistent": false
    },
    "mqtt_schema": {
      "enable": true,
      "strategy": "ALL",
      "failed_operation": "Discard",
      "echo_log": true,
      "log_level": "info"
    },
    "mqtt_system_monitor": {
      "enable": false,
      "os_cpu_high_watermark": 70.0,
      "os_memory_high_watermark": 80.0,
      "system_topic_interval_ms": 60000
    },
    "mqtt_limit": {
      "cluster": {
        "max_connections_per_node": 10000000,
        "max_connection_rate": 100000,
        "max_topics": 5000000,
        "max_sessions": 50000000,
        "max_publish_rate": 10000
      },
      "tenant": {
        "max_connections_per_node": 1000000,
        "max_connection_rate": 10000,
        "max_topics": 500000,
        "max_sessions": 5000000,
        "max_publish_rate": 10000
      }
    }
  },
  "error": null
}
```

---

### 2. 设置集群配置

- **接口**: `POST /api/cluster/config/set`
- **描述**: 动态更新集群配置，修改立即生效并持久化到 Meta 存储
- **请求参数**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `config_type` | string | 是 | 配置类型，见下方可选值 |
| `config` | string | 是 | 对应类型的配置 JSON 字符串 |

`config_type` 可选值及对应 `config` 字段结构：

---

#### `MqttSlowSubscribeConfig` — 慢订阅检测

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `enable` | bool | `false` | 是否启用 |
| `record_time` | u64 | `1000` | 慢订阅阈值（ms），超过该时间的订阅推送将被记录 |
| `delay_type` | string | `"CreateTime"` | 延迟统计方式：`"CreateTime"` 或 `"PublishTime"` |

```json
{
  "config_type": "MqttSlowSubscribeConfig",
  "config": "{\"enable\":true,\"record_time\":500,\"delay_type\":\"CreateTime\"}"
}
```

---

#### `MqttFlappingDetect` — 连接抖动检测

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `enable` | bool | `false` | 是否启用 |
| `window_time` | u32 | `60` | 检测窗口时间（秒） |
| `max_client_connections` | u64 | `5` | 窗口内最大允许连接次数，超过则触发封禁 |
| `ban_time` | u32 | `300` | 封禁时长（秒） |

```json
{
  "config_type": "MqttFlappingDetect",
  "config": "{\"enable\":true,\"window_time\":60,\"max_client_connections\":5,\"ban_time\":300}"
}
```

---

#### `MqttProtocol` — MQTT 协议参数

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `max_session_expiry_interval` | u32 | `2592000` | 最大 Session 过期时间（秒） |
| `default_session_expiry_interval` | u32 | `3600` | 默认 Session 过期时间（秒） |
| `topic_alias_max` | u16 | `65535` | 最大 Topic Alias 数量 |
| `max_packet_size` | u32 | `10485760` | 最大报文大小（字节，默认 10MB） |
| `receive_max` | u16 | `65535` | 接收窗口大小 |
| `max_message_expiry_interval` | u64 | `86400` | 消息最大过期时间（秒） |
| `client_pkid_persistent` | bool | `false` | 是否持久化客户端 Packet ID |

```json
{
  "config_type": "MqttProtocol",
  "config": "{\"max_session_expiry_interval\":2592000,\"default_session_expiry_interval\":3600,\"topic_alias_max\":65535,\"max_packet_size\":10485760,\"receive_max\":65535,\"max_message_expiry_interval\":86400,\"client_pkid_persistent\":false}"
}
```

---

#### `MqttOfflineMessage` — 离线消息

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `enable` | bool | `false` | 是否启用 |
| `expire_ms` | u32 | `86400000` | 离线消息过期时间（ms） |
| `max_messages_num` | u32 | `1000` | 每个客户端最多保存的离线消息数 |

```json
{
  "config_type": "MqttOfflineMessage",
  "config": "{\"enable\":true,\"expire_ms\":86400000,\"max_messages_num\":1000}"
}
```

---

#### `MqttSystemMonitor` — 系统监控

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `enable` | bool | `false` | 是否启用系统监控 |
| `os_cpu_high_watermark` | f32 | `0.7` | CPU 使用率高水位（0~1） |
| `os_memory_high_watermark` | f32 | `0.8` | 内存使用率高水位（0~1） |
| `system_topic_interval_ms` | u64 | `60000` | 系统 Topic 上报间隔（ms） |

```json
{
  "config_type": "MqttSystemMonitor",
  "config": "{\"enable\":true,\"os_cpu_high_watermark\":0.7,\"os_memory_high_watermark\":0.8,\"system_topic_interval_ms\":60000}"
}
```

---

#### `MqttSchema` — Schema 校验

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `enable` | bool | `false` | 是否启用 Schema 校验 |
| `strategy` | string | `"Forward"` | 校验策略：`"Forward"`（透传）或 `"Strict"` |
| `failed_operation` | string | `"Disconnect"` | 校验失败操作：`"Disconnect"` 或 `"Ignore"` |
| `echo_log` | bool | `false` | 是否打印 Schema 校验日志 |
| `log_level` | string | `"info"` | 日志级别 |

```json
{
  "config_type": "MqttSchema",
  "config": "{\"enable\":true,\"strategy\":\"Strict\",\"failed_operation\":\"Disconnect\",\"echo_log\":false,\"log_level\":\"info\"}"
}
```

---

#### `MqttLimit` — MQTT 流量限制

| 字段 | 类型 | 说明 |
|------|------|------|
| `cluster` | object | 集群级别限制，见 `LimitQuota` |
| `tenant` | object | 租户默认限制，见 `LimitQuota` |

**`LimitQuota` 字段**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `max_connections_per_node` | u64 | 每节点最大连接数 |
| `max_connection_rate` | u32 | 最大连接速率（连接/秒） |
| `max_topics` | u64 | 最大 Topic 数 |
| `max_sessions` | u64 | 最大 Session 数 |
| `max_publish_rate` | u32 | 最大发布速率（消息/秒） |

```json
{
  "config_type": "MqttLimit",
  "config": "{\"cluster\":{\"max_connections_per_node\":10000000,\"max_connection_rate\":100000,\"max_topics\":5000000,\"max_sessions\":50000000,\"max_publish_rate\":10000},\"tenant\":{\"max_connections_per_node\":1000000,\"max_connection_rate\":10000,\"max_topics\":500000,\"max_sessions\":5000000,\"max_publish_rate\":10000}}"
}
```

---

#### `ClusterLimit` — 集群接入限制

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `max_network_connection` | u64 | `100000000` | 最大网络连接总数 |
| `max_network_connection_rate` | u32 | `10000` | 最大网络连接速率（连接/秒） |
| `max_admin_http_uri_rate` | u32 | `50` | Admin HTTP 接口最大请求速率（次/秒） |

```json
{
  "config_type": "ClusterLimit",
  "config": "{\"max_network_connection\":100000000,\"max_network_connection_rate\":10000,\"max_admin_http_uri_rate\":50}"
}
```

---

- **响应示例**:
```json
{
  "code": 0,
  "data": "success",
  "error": null
}
```

---

## 集群信息

### 3. 获取集群状态

- **接口**: `GET /api/cluster/status`
- **描述**: 获取集群运行状态，包括版本、节点列表、各 Raft Group 的 Meta 状态等

- **响应示例**:
```json
{
  "code": 0,
  "data": {
    "version": "0.3.0",
    "cluster_name": "robust_mq_cluster_default",
    "start_time": 1738800000,
    "broker_node_list": [],
    "nodes": ["127.0.0.1"],
    "meta": {
      "mqtt": {
        "running_state": { "Ok": null },
        "id": 1,
        "current_term": 1,
        "vote": { "leader_id": { "term": 1, "node_id": 1 }, "committed": true },
        "last_log_index": 30001,
        "last_applied": { "leader_id": { "term": 1, "node_id": 1 }, "index": 30001 },
        "snapshot": { "leader_id": { "term": 1, "node_id": 1 }, "index": 30001 },
        "purged": { "leader_id": { "term": 1, "node_id": 1 }, "index": 30001 },
        "state": "Leader",
        "current_leader": 1,
        "millis_since_quorum_ack": 0,
        "membership_config": {
          "log_id": { "leader_id": { "term": 0, "node_id": 0 }, "index": 0 },
          "membership": {
            "configs": [[1]],
            "nodes": { "1": { "node_id": 1, "rpc_addr": "127.0.0.1:1228" } }
          }
        },
        "replication": { "1": { "leader_id": { "term": 1, "node_id": 1 }, "index": 30001 } }
      },
      "offset": {
        "state": "Leader",
        "current_leader": 1,
        "last_log_index": 1,
        "..."  : "..."
      },
      "meta": {
        "state": "Leader",
        "current_leader": 1,
        "last_log_index": 42853,
        "..." : "..."
      }
    }
  },
  "error": null
}
```

**`data` 字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `version` | string | 当前 Broker 版本号 |
| `cluster_name` | string | 集群名称 |
| `start_time` | u64 | 进程启动时间戳（Unix 秒） |
| `broker_node_list` | array | 集群中所有 Broker 节点信息列表 |
| `nodes` | string[] | 集群节点 IP 列表（去重后） |
| `meta` | object | 各 Raft Group 的运行状态，key 为 group 名称 |

**`meta` 的 key 说明**：

| key | 说明 |
|-----|------|
| `mqtt` | MQTT 数据 Raft Group 状态 |
| `offset` | Offset 数据 Raft Group 状态 |
| `meta` | 元数据 Raft Group 状态 |

**每个 Raft Group 状态（`MetaStatus`）字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `running_state` | object | 运行状态，`{"Ok": null}` 表示正常 |
| `id` | u64 | 当前节点 ID |
| `current_term` | u64 | 当前 Raft term |
| `vote` | object | 当前投票信息 |
| `last_log_index` | u64 | 最新日志索引 |
| `last_applied` | object | 已应用到状态机的最新日志位置 |
| `snapshot` | object/null | 最新快照位置 |
| `purged` | object/null | 已清理的最旧日志位置 |
| `state` | string | 节点角色：`Leader`、`Follower`、`Candidate` |
| `current_leader` | u64 | 当前 Leader 节点 ID |
| `millis_since_quorum_ack` | u64 | 距上次 quorum 确认的毫秒数 |
| `membership_config` | object | 集群成员配置信息 |
| `replication` | object | 各节点的复制进度，key 为节点 ID |

---

## 返回值字段说明

### BrokerConfig 各部分说明

#### 基础配置

| 字段 | 类型 | 说明 |
|------|------|------|
| `cluster_name` | string | 集群名称 |
| `broker_id` | u64 | Broker 节点 ID |
| `broker_ip` | string/null | Broker IP 地址 |
| `roles` | string[] | 节点角色，可选 `meta`、`broker`、`engine` |
| `grpc_port` | u32 | gRPC 服务端口 |
| `http_port` | u32 | HTTP API 服务端口 |
| `meta_addrs` | object | Meta 节点地址映射（节点ID → 地址） |

#### prometheus

| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | bool | 是否启用 Prometheus 指标 |
| `port` | u16 | Prometheus 指标暴露端口 |

#### log

| 字段 | 类型 | 说明 |
|------|------|------|
| `log_path` | string | 日志输出目录 |
| `log_config` | string | 日志 tracing 配置文件路径 |

#### runtime

| 字段 | 类型 | 说明 |
|------|------|------|
| `runtime_worker_threads` | usize | 兼容旧版全局线程倍数，各运行时字段为 0 时作为回退值 |
| `server_worker_threads` | usize | Server 运行时线程数（0 = 自动，等于 CPU 核心数） |
| `meta_worker_threads` | usize | Meta 运行时线程数（0 = 自动） |
| `broker_worker_threads` | usize | Broker 运行时线程数（0 = 自动，热路径运行时） |
| `channels_per_address` | usize | 每个地址的 gRPC 连接通道数 |
| `tls_cert` | string | TLS 证书文件路径 |
| `tls_key` | string | TLS 私钥文件路径 |

#### network

| 字段 | 类型 | 说明 |
|------|------|------|
| `accept_thread_num` | usize | 连接接受线程数 |
| `handler_thread_num` | usize | 消息处理线程数 |
| `queue_size` | usize | 内部队列大小 |

#### pprof

| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | bool | 是否启用 pprof 性能分析 |
| `port` | u16 | pprof HTTP 端口 |
| `frequency` | i32 | 采样频率 |

#### meta_runtime

| 字段 | 类型 | 说明 |
|------|------|------|
| `heartbeat_timeout_ms` | u64 | 心跳超时时间（毫秒） |
| `heartbeat_check_time_ms` | u64 | 心跳检查间隔（毫秒） |
| `raft_write_timeout_sec` | u64 | Raft 写入超时（秒） |
| `offset_raft_group_num` | u32 | Offset Raft 分片组数量（默认 1） |
| `data_raft_group_num` | u32 | 数据 Raft 分片组数量（默认 1） |

#### rocksdb

| 字段 | 类型 | 说明 |
|------|------|------|
| `data_path` | string | RocksDB 数据目录 |
| `max_open_files` | i32 | 最大打开文件数 |

#### storage_runtime

| 字段 | 类型 | 说明 |
|------|------|------|
| `tcp_port` | u32 | 存储引擎 TCP 端口 |
| `max_segment_size` | u32 | 最大段文件大小（字节） |
| `io_thread_num` | u32 | IO 线程数 |
| `data_path` | string[] | 数据存储路径列表 |
| `offset_enable_cache` | bool | 是否启用 Offset 缓存 |

#### mqtt_server

| 字段 | 类型 | 说明 |
|------|------|------|
| `tcp_port` | u32 | MQTT TCP 端口 |
| `tls_port` | u32 | MQTT TLS 端口 |
| `websocket_port` | u32 | MQTT WebSocket 端口 |
| `websockets_port` | u32 | MQTT WebSocket Secure 端口 |
| `quic_port` | u32 | MQTT QUIC 端口 |

#### mqtt_keep_alive

| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | bool | 是否启用 Keep Alive 检测 |
| `default_time` | u16 | 默认 Keep Alive 时间（秒） |
| `max_time` | u16 | 最大 Keep Alive 时间（秒） |
| `default_timeout` | u16 | 超时倍数 |

#### mqtt_runtime

| 字段 | 类型 | 说明 |
|------|------|------|
| `default_user` | string | 默认用户名 |
| `default_password` | string | 默认密码 |
| `durable_sessions_enable` | bool | 是否启用持久化会话 |
| `secret_free_login` | bool | 是否允许免密登录 |
| `is_self_protection_status` | bool | 是否处于自我保护状态 |

#### mqtt_offline_message

| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | bool | 是否启用离线消息 |
| `expire_ms` | u32 | 过期时间（毫秒），0 表示不过期 |
| `max_messages_num` | u32 | 最大离线消息数量，0 表示不限制 |

#### mqtt_slow_subscribe

| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | bool | 是否启用慢订阅检测 |
| `record_time` | u64 | 记录阈值时间（毫秒） |
| `delay_type` | string | 延迟类型：`Whole`、`Internal`、`Response` |

#### mqtt_flapping_detect

| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | bool | 是否启用连接抖动检测 |
| `window_time` | u32 | 时间窗口（分钟） |
| `max_client_connections` | u64 | 窗口内最大连接次数 |
| `ban_time` | u32 | 封禁时间（分钟） |

#### mqtt_protocol

| 字段 | 类型 | 说明 |
|------|------|------|
| `max_session_expiry_interval` | u32 | 最大会话过期间隔（秒） |
| `default_session_expiry_interval` | u32 | 默认会话过期间隔（秒） |
| `topic_alias_max` | u16 | Topic Alias 最大值 |
| `max_qos_flight_message` | u8 | QoS 飞行窗口最大消息数 |
| `max_packet_size` | u32 | 最大报文大小（字节） |
| `receive_max` | u16 | Receive Maximum |
| `max_message_expiry_interval` | u64 | 最大消息过期间隔（秒） |
| `client_pkid_persistent` | bool | 客户端 Packet ID 是否持久化 |

#### mqtt_schema

| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | bool | 是否启用 Schema 校验 |
| `strategy` | string | 校验策略：`ALL`、`Any` |
| `failed_operation` | string | 校验失败处理：`Discard`、`DisconnectAndDiscard`、`Ignore` |
| `echo_log` | bool | 是否输出校验日志 |
| `log_level` | string | 日志级别 |

#### mqtt_system_monitor

| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | bool | 是否启用系统监控 |
| `os_cpu_high_watermark` | f32 | CPU 高水位线（百分比） |
| `os_memory_high_watermark` | f32 | 内存高水位线（百分比） |
| `system_topic_interval_ms` | u64 | 系统 Topic 指标发布间隔（毫秒） |

#### cluster_limit

| 字段 | 类型 | 说明 |
|------|------|------|
| `max_network_connection` | u64 | 集群最大网络连接数 |
| `max_network_connection_rate` | u32 | 集群每秒最大新建连接速率 |
| `max_admin_http_uri_rate` | u32 | Admin HTTP 接口每秒最大请求速率 |

#### mqtt_limit

`cluster` 和 `tenant` 均为 `LimitQuota` 结构：

| 字段 | 类型 | 说明 |
|------|------|------|
| `max_connections_per_node` | u64 | 每节点最大连接数 |
| `max_connection_rate` | u32 | 每秒最大新建连接速率 |
| `max_topics` | u64 | 最大主题数 |
| `max_sessions` | u64 | 最大会话数 |
| `max_publish_rate` | u32 | 每秒最大发布速率 |

#### llm_client

可选配置，不填时为 `null`。

| 字段 | 类型 | 说明 |
|------|------|------|
| `platform` | string | LLM 平台，如 `open_ai`、`anthropic`、`ollama` 等 |
| `model` | string | 模型名称 |
| `token` | string/null | API Token（Ollama 不需要） |
| `base_url` | string/null | 自定义 API 地址（需以 `http://` 或 `https://` 开头） |

---

## 使用示例

### 获取集群配置
```bash
curl -X GET http://localhost:8080/api/cluster/config/get
```

### 获取集群状态
```bash
curl -X GET http://localhost:8080/api/cluster/status
```

### 设置连接抖动检测配置
```bash
curl -X POST http://localhost:8080/api/cluster/config/set \
  -H "Content-Type: application/json" \
  -d '{
    "config_type": "FlappingDetect",
    "config": "{\"enable\":true,\"window_time\":2,\"max_client_connections\":20,\"ban_time\":10}"
  }'
```

---

---

## 租户管理

租户（Tenant）是 RobustMQ 多租户架构的核心概念，用于对集群资源进行逻辑隔离。适用于同一集群服务多个业务、多个环境（开发/测试/生产）等场景。

### 4. 获取租户列表

- **接口**: `GET /api/tenant/list`
- **描述**: 获取集群中所有租户，支持分页、排序、过滤
- **请求参数**（Query String）:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `tenant_name` | string | 否 | 按租户名称模糊查询（包含匹配） |
| `page` | u32 | 否 | 页码，从 1 开始，默认 1 |
| `limit` | u32 | 否 | 每页条数，默认 100 |
| `sort_field` | string | 否 | 排序字段，支持 `tenant_name` |
| `sort_by` | string | 否 | 排序方向：`asc` / `desc` |
| `filter_field` | string | 否 | 过滤字段名 |
| `filter_values` | string[] | 否 | 过滤值列表 |
| `exact_match` | string | 否 | 是否精确匹配：`exact` |

- **响应示例**:
```json
{
  "code": 0,
  "data": {
    "data": [
      {
        "tenant_name": "business-a",
        "desc": "业务 A 租户",
        "config": {
          "max_connections_per_node": 10000000,
          "max_create_connection_rate_per_second": 10000,
          "max_topics": 5000000,
          "max_sessions": 50000000,
          "max_publish_rate": 10000
        },
        "create_time": 1738800000
      }
    ],
    "total_count": 1
  },
  "error": null
}
```

- **curl 示例**:
```bash
# 查询所有租户
curl -X GET "http://localhost:8080/api/tenant/list"

# 模糊查询租户名包含 "business" 的租户
curl -X GET "http://localhost:8080/api/tenant/list?tenant_name=business"
```

---

### 5. 创建租户

- **接口**: `POST /api/tenant/create`
- **描述**: 创建一个新租户
- **请求体**:

| 字段 | 类型 | 必填 | 校验 | 说明 |
|------|------|------|------|------|
| `tenant_name` | string | 是 | 长度 1-128 | 租户名称，全局唯一 |
| `desc` | string | 否 | 长度 ≤ 500 | 租户描述 |
| `config` | object | 否 | - | 租户资源配额配置，不填则使用默认值 |
| `config.max_connections_per_node` | u64 | 否 | - | 每节点最大连接数（默认 10000000） |
| `config.max_create_connection_rate_per_second` | u32 | 否 | - | 每秒最大新建连接速率（默认 10000） |
| `config.max_topics` | u64 | 否 | - | 最大主题数（默认 5000000） |
| `config.max_sessions` | u64 | 否 | - | 最大会话数（默认 50000000） |
| `config.max_publish_rate` | u32 | 否 | - | 每秒最大发布消息速率（默认 10000） |

- **请求示例**:
```json
{
  "tenant_name": "business-a",
  "desc": "业务 A 租户",
  "config": {
    "max_connections_per_node": 50000,
    "max_topics": 100000,
    "max_sessions": 200000,
    "max_publish_rate": 5000
  }
}
```

- **响应示例**:
```json
{
  "code": 0,
  "data": "success",
  "error": null
}
```

- **curl 示例**:
```bash
curl -X POST http://localhost:8080/api/tenant/create \
  -H "Content-Type: application/json" \
  -d '{"tenant_name": "business-a", "desc": "业务 A 租户"}'
```

---

### 6. 删除租户

- **接口**: `POST /api/tenant/delete`
- **描述**: 删除指定租户。删除后，归属该租户的元数据将不再受该租户管控。
- **请求体**:

| 字段 | 类型 | 必填 | 校验 | 说明 |
|------|------|------|------|------|
| `tenant_name` | string | 是 | 长度 1-128 | 要删除的租户名称 |

- **请求示例**:
```json
{
  "tenant_name": "business-a"
}
```

- **响应示例**:
```json
{
  "code": 0,
  "data": "success",
  "error": null
}
```

- **curl 示例**:
```bash
curl -X POST http://localhost:8080/api/tenant/delete \
  -H "Content-Type: application/json" \
  -d '{"tenant_name": "business-a"}'
```

---

### 7. 更新租户

- **接口**: `POST /api/tenant/update`
- **描述**: 更新租户的描述和资源配额。租户必须已存在。`config` 不传时保持原有配置不变。
- **请求体**:

| 字段 | 类型 | 必填 | 校验 | 说明 |
|------|------|------|------|------|
| `tenant_name` | string | 是 | 长度 1-128 | 要更新的租户名称 |
| `desc` | string | 否 | 长度 ≤ 500 | 新的租户描述 |
| `config` | object | 否 | - | 租户资源配额配置，不填则保持原有配置不变 |
| `config.max_connections_per_node` | u64 | 否 | - | 每节点最大连接数 |
| `config.max_create_connection_rate_per_second` | u32 | 否 | - | 每秒最大新建连接速率 |
| `config.max_topics` | u64 | 否 | - | 最大主题数 |
| `config.max_sessions` | u64 | 否 | - | 最大会话数 |
| `config.max_publish_rate` | u32 | 否 | - | 每秒最大发布消息速率 |

- **请求示例**:
```json
{
  "tenant_name": "business-a",
  "desc": "业务 A 租户（已更新）",
  "config": {
    "max_connections_per_node": 100000,
    "max_publish_rate": 20000
  }
}
```

- **响应示例**:
```json
{
  "code": 0,
  "data": "success",
  "error": null
}
```

- **错误响应**（租户不存在时）:
```json
{
  "code": 1,
  "message": "Tenant business-a not found",
  "data": null
}
```

- **curl 示例**:
```bash
curl -X POST http://localhost:8080/api/tenant/update \
  -H "Content-Type: application/json" \
  -d '{"tenant_name": "business-a", "desc": "业务 A 租户（已更新）", "config": {"max_connections_per_node": 100000}}'
```

---

## 健康检查

### 8. 集群存活检查

- **接口**: `GET /cluster/healthy`
- **描述**: 检查服务是否存活，返回 `true` 表示正常
- **请求参数**: 无
- **响应示例**:
```json
{
  "code": 0,
  "message": "success",
  "data": true
}
```

---

### 9. 就绪检查

- **接口**: `GET /health/ready`
- **描述**: 检查所有配置的端口是否就绪，用于 K8s readiness probe
- **请求参数**: 无
- **响应**:
  - **200 OK** — 所有端口就绪:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "status": "ok",
    "check_type": "ready",
    "message": "all configured ports are ready"
  }
}
```
  - **503 Service Unavailable** — 端口未就绪:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "status": "not_ready",
    "check_type": "ready",
    "message": "one or more configured ports are not ready"
  }
}
```

---

### 10. 节点健康检查

- **接口**: `GET /health/node`
- **描述**: 节点级健康检查（占位实现，始终返回 ok）
- **请求参数**: 无
- **响应示例**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "status": "ok",
    "check_type": "node",
    "message": "health check placeholder"
  }
}
```

---

### 11. 集群健康检查

- **接口**: `GET /health/cluster`
- **描述**: 集群级健康检查（占位实现，始终返回 ok）
- **请求参数**: 无
- **响应示例**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "status": "ok",
    "check_type": "cluster",
    "message": "health check placeholder"
  }
}
```

---

## 注意事项

1. **响应格式**: 成功时 `code` 为 `0`，`error` 为 `null`；失败时 `code` 为 `100`，`error` 包含错误信息
2. **配置格式**: `config_set` 接口的 `config` 字段必须是有效的 JSON 字符串
3. **热更新**: 部分配置支持热更新，无需重启服务
4. **备份建议**: 修改配置前建议先通过 `config/get` 接口获取当前配置进行备份
5. **租户隔离**: 租户为逻辑隔离，适用于私有化部署场景；SaaS 多租户场景建议使用物理隔离方案
