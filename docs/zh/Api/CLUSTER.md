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
      "runtime_worker_threads": 16,
      "tls_cert": "./config/certs/cert.pem",
      "tls_key": "./config/certs/key.pem"
    },
    "network": {
      "accept_thread_num": 8,
      "handler_thread_num": 32,
      "response_thread_num": 8,
      "queue_size": 1000,
      "lock_max_try_mut_times": 30,
      "lock_try_mut_sleep_time_ms": 50
    },
    "p_prof": {
      "enable": false,
      "port": 6060,
      "frequency": 100
    },
    "message_storage": {
      "storage_type": "EngineMemory",
      "engine_config": null,
      "memory_config": null,
      "minio_config": null,
      "mysql_config": null,
      "rocksdb_config": null,
      "s3_config": null
    },
    "meta_runtime": {
      "heartbeat_timeout_ms": 30000,
      "heartbeat_check_time_ms": 1000,
      "raft_write_timeout_sec": 30
    },
    "rocksdb": {
      "data_path": "./data",
      "max_open_files": 10000
    },
    "storage_runtime": {
      "tcp_port": 1778,
      "max_segment_size": 1073741824,
      "io_thread_num": 8,
      "data_path": []
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
    "mqtt_auth_config": {
      "authn_config": {
        "authn_type": "password_based",
        "jwt_config": null,
        "password_based_config": {
          "storage_config": {
            "storage_type": "placement",
            "placement_config": { "journal_addr": "" },
            "mysql_config": null,
            "postgres_config": null,
            "redis_config": null,
            "http_config": null
          },
          "password_config": {
            "credential_type": "username",
            "algorithm": "plain",
            "salt_position": "disable",
            "salt_rounds": null,
            "mac_fun": null,
            "iterations": null,
            "dk_length": null
          }
        }
      },
      "authz_config": {
        "storage_config": {
          "storage_type": "placement",
          "placement_config": { "journal_addr": "" },
          "mysql_config": null,
          "postgres_config": null,
          "redis_config": null,
          "http_config": null
        }
      }
    },
    "mqtt_runtime": {
      "default_user": "admin",
      "default_password": "robustmq",
      "max_connection_num": 1000000,
      "durable_sessions_enable": false
    },
    "mqtt_offline_message": {
      "enable": true,
      "expire_ms": 0,
      "max_messages_num": 0
    },
    "mqtt_slow_subscribe_config": {
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
    "mqtt_protocol_config": {
      "max_session_expiry_interval": 1800,
      "default_session_expiry_interval": 30,
      "topic_alias_max": 65535,
      "max_qos_flight_message": 2,
      "max_packet_size": 10485760,
      "receive_max": 65535,
      "max_message_expiry_interval": 3600,
      "client_pkid_persistent": false
    },
    "mqtt_security": {
      "is_self_protection_status": false,
      "secret_free_login": false
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
      "os_memory_high_watermark": 80.0
    },
    "storage_offset": {
      "enable_cache": true
    }
  },
  "error": null
}
```

---

### 2. 设置集群配置

- **接口**: `POST /api/cluster/config/set`
- **描述**: 动态更新集群的部分配置
- **请求参数**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `config_type` | string | 是 | 功能类型，见下方可选值 |
| `config` | string | 是 | 对应功能的配置 JSON 字符串 |

`config_type` 可选值：

| 值 | 说明 |
|----|------|
| `SlowSubscribe` | 慢订阅检测 |
| `OfflineMessage` | 离线消息 |
| `SystemAlarm` | 系统告警 |
| `FlappingDetect` | 连接抖动检测 |

- **请求示例**:
```json
{
  "config_type": "OfflineMessage",
  "config": "{\"enable\":true,\"expire_ms\":60000,\"max_messages_num\":5000}"
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

> **注意**: 此接口目前部分功能尚在开发中，`SlowSubscribe` 和 `OfflineMessage` 的实际更新逻辑暂未启用。

---

## 集群信息

### 3. 获取集群状态

- **接口**: `GET /api/cluster/status`
- **描述**: 获取集群运行状态，包括版本、节点列表、Meta 状态等

- **响应示例**:
```json
{
  "code": 0,
  "data": {
    "version": "0.1.34",
    "cluster_name": "robust_mq_cluster_default",
    "start_time": 1738800000,
    "broker_node_list": [],
    "meta": { ... },
    "nodes": ["192.168.1.100", "192.168.1.101"]
  },
  "error": null
}
```

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
| `runtime_worker_threads` | usize | Tokio 运行时工作线程数 |
| `tls_cert` | string | TLS 证书文件路径 |
| `tls_key` | string | TLS 私钥文件路径 |

#### network

| 字段 | 类型 | 说明 |
|------|------|------|
| `accept_thread_num` | usize | 连接接受线程数 |
| `handler_thread_num` | usize | 消息处理线程数 |
| `response_thread_num` | usize | 响应线程数 |
| `queue_size` | usize | 内部队列大小 |
| `lock_max_try_mut_times` | u64 | 锁最大尝试次数 |
| `lock_try_mut_sleep_time_ms` | u64 | 锁重试等待时间（毫秒） |

#### p_prof

| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | bool | 是否启用 pprof 性能分析 |
| `port` | u16 | pprof HTTP 端口 |
| `frequency` | i32 | 采样频率 |

#### message_storage

| 字段 | 类型 | 说明 |
|------|------|------|
| `storage_type` | string | 存储类型：`EngineMemory`、`EngineSegment`、`EngineRocksDB`、`Mysql`、`MinIO`、`S3` |
| `engine_config` | object/null | Engine 存储配置 |
| `memory_config` | object/null | 内存存储配置 |
| `minio_config` | object/null | MinIO 存储配置 |
| `mysql_config` | object/null | MySQL 存储配置 |
| `rocksdb_config` | object/null | RocksDB 存储配置 |
| `s3_config` | object/null | S3 存储配置 |

#### meta_runtime

| 字段 | 类型 | 说明 |
|------|------|------|
| `heartbeat_timeout_ms` | u64 | 心跳超时时间（毫秒） |
| `heartbeat_check_time_ms` | u64 | 心跳检查间隔（毫秒） |
| `raft_write_timeout_sec` | u64 | Raft 写入超时（秒） |

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
| `max_connection_num` | usize | 最大连接数 |
| `durable_sessions_enable` | bool | 是否启用持久化会话 |

#### mqtt_offline_message

| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | bool | 是否启用离线消息 |
| `expire_ms` | u32 | 过期时间（毫秒），0 表示不过期 |
| `max_messages_num` | u32 | 最大离线消息数量，0 表示不限制 |

#### mqtt_slow_subscribe_config

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

#### mqtt_protocol_config

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

#### mqtt_security

| 字段 | 类型 | 说明 |
|------|------|------|
| `is_self_protection_status` | bool | 是否处于自我保护状态 |
| `secret_free_login` | bool | 是否允许免密登录 |

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

#### storage_offset

| 字段 | 类型 | 说明 |
|------|------|------|
| `enable_cache` | bool | 是否启用偏移量缓存 |

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

## 注意事项

1. **响应格式**: 成功时 `code` 为 `0`，`error` 为 `null`；失败时 `code` 为 `100`，`error` 包含错误信息
2. **配置格式**: `config_set` 接口的 `config` 字段必须是有效的 JSON 字符串
3. **热更新**: 部分配置支持热更新，无需重启服务
4. **备份建议**: 修改配置前建议先通过 `config/get` 接口获取当前配置进行备份
