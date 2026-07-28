# Broker 配置说明

> 本文档描述 RobustMQ 的全局/基础配置项。日志配置请参考 [Logging.md](Logging.md)；各协议自己的配置已经拆分为独立文档：[MQTT 配置](MQTTConfig.md)、[Kafka 配置](KafkaConfig.md)、[AMQP 配置](AMQPConfig.md)、[NATS 配置](NATSConfig.md)。

## 概述

RobustMQ 使用 TOML 格式的配置文件来管理系统配置。主配置文件为 `config/server.toml`。

### 配置加载优先级

1. 环境变量（最高）
2. 配置文件
3. 默认值（最低）

### 环境变量覆盖

支持通过环境变量覆盖配置文件中的设置。命名规则：

```text
ROBUST_MQ_SERVER_{SECTION}_{KEY}
```

- 顶层配置项：`ROBUST_MQ_SERVER_{KEY}`
- Section 内配置项：`ROBUST_MQ_SERVER_{SECTION}_{KEY}`
- 所有字母大写，`.` 替换为 `_`

示例：

```bash
export ROBUST_MQ_SERVER_CLUSTER_NAME="my-cluster"
export ROBUST_MQ_SERVER_MQTT_RUNTIME_SERVER_TCP_PORT=1883
export ROBUST_MQ_SERVER_RUNTIME_CHANNELS_PER_ADDRESS=8
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
http_port = 58080

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
| `http_port` | `u32` | `58080` | HTTP API 服务端口 |
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

Tokio 运行时、gRPC 客户端连接池、TLS 与 pprof 采集配置。RobustMQ 内部划分了三个独立的 Tokio 运行时，分别承担不同职责，可以独立调优。

```toml
[runtime]
tls_cert = "./config/certs/cert.pem"
tls_key = "./config/certs/key.pem"
channels_per_address = 4
# 各运行时工作线程数，0 = 自动（推荐）
# server_worker_threads = 0
# meta_worker_threads = 0
# broker_worker_threads = 0
# runtime_worker_threads = 1  # 兼容旧版，新版请用各运行时独立配置
# pprof_enable = false
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `tls_cert` | `string` | `"./config/certs/cert.pem"` | TLS 证书文件路径 |
| `tls_key` | `string` | `"./config/certs/key.pem"` | TLS 私钥文件路径 |
| `channels_per_address` | `usize` | `4` | 每个 gRPC 服务地址维护的 HTTP/2 Channel（TCP 连接）数量 |
| `server_worker_threads` | `usize` | `0`（自动） | server-runtime 工作线程数，自动值 = `max(4, CPU核数 / 2)` |
| `meta_worker_threads` | `usize` | `0`（自动） | meta-runtime 工作线程数，自动值 = `max(4, CPU核数 / 2)` |
| `broker_worker_threads` | `usize` | `0`（自动） | broker-runtime 工作线程数，自动值 = `CPU核数` |
| `runtime_worker_threads` | `usize` | `1` | 兼容旧版全局线程倍数，各运行时字段为 0 时作为回退值，新版建议保持默认 |
| `pprof_enable` | `bool` | `false` | 是否启用内置 pprof 性能分析采集；采集到的火焰图通过 Admin HTTP API（复用 `http_port`）暴露，没有独立端口 |

**三个运行时说明：**

| 运行时 | 职责 | 默认线程数 |
|--------|------|-----------|
| `server-runtime` | gRPC 服务、HTTP Admin API、Prometheus 指标暴露 | `max(4, CPU/2)` |
| `meta-runtime` | Raft 状态机、RocksDB 写入 | `max(4, CPU/2)` |
| `broker-runtime` | MQTT 连接处理、消息投递热路径 | `CPU核数` |

> **调优建议：** 保持默认值 `0` 即可。通过 Grafana 的 `tokio_runtime_busy_ratio` 指标判断是否需要调整：某个运行时繁忙比持续 > 80% 时，可适当增加其线程数。

**gRPC 客户端连接池调优：** 每个 HTTP/2 Channel 支持约 200 个并发 Stream（即并发 RPC 请求），默认值 `4` 可支撑约 800 个并发 gRPC 请求，覆盖绝大多数生产场景。

| 场景 | 建议值 |
|------|--------|
| 默认 / 常规生产 | `4` |
| 高并发（万级 MQTT 连接） | `8` ~ `16` |
| 极高并发压测 | `32` |

> **注意：** 该值过大会导致系统打开的 TCP 文件描述符数量暴增（每个 Channel 占用一个 fd），在 `ulimit -n` 较小的环境下可能引发 `Too many open files` 错误。

---

## 3. Meta 运行时配置

### [meta_runtime]

元数据服务心跳与 Raft 配置。

```toml
[meta_runtime]
heartbeat_timeout_ms = 30000
heartbeat_check_time_ms = 1000
raft_write_timeout_sec = 30
offset_raft_group_num = 1
data_raft_group_num = 1
group_offset_expire_sec = 604800
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `heartbeat_timeout_ms` | `u64` | `30000` | 节点心跳超时时间（毫秒），超时后标记节点不可用 |
| `heartbeat_check_time_ms` | `u64` | `1000` | 心跳检查间隔（毫秒） |
| `raft_write_timeout_sec` | `u64` | `30` | Raft 写操作超时时间（秒） |
| `offset_raft_group_num` | `u32` | `1` | Offset Raft 分组数量 |
| `data_raft_group_num` | `u32` | `1` | 数据 Raft 分组数量 |
| `group_offset_expire_sec` | `u64` | `604800` | 消费组 Offset 过期时间（秒），默认 7 天 |

---

## 4. RocksDB 配置

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

## 5. 存储引擎运行时配置

### [storage_runtime]

Journal 存储引擎运行时配置。

```toml
[storage_runtime]
tcp_port = 1778
max_segment_size = 1073741824
io_thread_num = 8
data_path = []
expire_scan_task_num = 10
offset_enable_cache = true
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `tcp_port` | `u32` | `1778` | 存储引擎 TCP 端口 |
| `max_segment_size` | `u32` | `1073741824` (1 GB) | 单个 Segment 文件最大大小（字节） |
| `io_thread_num` | `u32` | `8` | IO 处理线程数 |
| `data_path` | `array` | `[]` | 数据存储路径列表 |
| `expire_scan_task_num` | `usize` | `10` | 过期数据扫描并发任务数 |
| `offset_enable_cache` | `bool` | `true` | 是否启用消费 Offset 缓存 |

> 存储引擎的网络线程复用统一的 [`[broker_network]`](#7-broker-网络配置) 配置，不再有独立的 `[storage_runtime.network]`。

---

## 6. 延迟任务配置

### [delay_task]

延迟消息处理任务队列配置。

```toml
[delay_task]
delay_task_queue_num = 100
delay_task_handler_concurrency = 100
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `delay_task_queue_num` | `usize` | `100` | 延迟任务队列数量 |
| `delay_task_handler_concurrency` | `usize` | `100` | 延迟任务处理并发数 |

---

## 7. Broker 网络配置

### [broker_network]

Broker 内部通用网络线程配置。

```toml
[broker_network]
accept_thread_num = 2
handler_thread_num = 16
queue_size = 1000
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `accept_thread_num` | `usize` | `2` | 接受连接的线程数 |
| `handler_thread_num` | `usize` | `16` | 请求处理线程数 |
| `queue_size` | `usize` | `1000` | 内部处理队列大小 |

---

## 8. LLM 客户端配置

### [llm_client]

用于配置 Broker 内部统一的 LLM 调用客户端（`LLMClient`）。该配置为可选项，不配置时不会启用 LLM 客户端。

```toml
[llm_client]
platform = "open_ai"
model = "gpt-4o-mini"
token = "your_api_token"
# 可选：用于 OpenAI 兼容网关或私有部署
# base_url = "https://api.openai.com/v1/"
# embedding = "text-embedding-3-small"
# embedding_model_path = "./models/embedding"
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `platform` | `string` | 无 | LLM 平台标识 |
| `model` | `string` | 无 | 模型名称，如 `gpt-4o-mini`、`claude-3-5-sonnet`、`gemini-2.0-flash` |
| `token` | `string` | 无 | 访问令牌。除 `ollama` 外其余平台必填 |
| `base_url` | `string` | 无 | 自定义 API 基地址（可选） |
| `embedding` | `string` | 无 | Embedding 模型名称（可选） |
| `embedding_model_path` | `string` | 无 | 本地 Embedding 模型文件路径（可选） |

**`base_url` 说明（重点）：**

- 不填 `base_url` 时，会使用 `genai` 的默认官方 endpoint。
- 只有在以下场景建议填写：使用代理网关、OpenAI 兼容服务、私有化部署、内网转发。
- `ollama` 不填时默认走本机：`http://localhost:11434/v1/`。

**常见平台不填 `base_url` 的默认行为：**

| `platform` | `base_url` 可否省略 | 默认 endpoint |
|------------|---------------------|---------------|
| `open_ai` / `open_ai_resp` | 可以 | OpenAI 官方 |
| `gemini` | 可以 | Google Gemini 官方 |
| `anthropic` | 可以 | Anthropic 官方 |
| `cohere` | 可以 | Cohere 官方 |
| `xai` | 可以 | xAI 官方 |
| `deep_seek` | 可以 | DeepSeek 官方 |
| `groq` / `together` / `fireworks` / `nebius` / `mimo` / `zai` / `big_model` | 可以 | 各平台官方 |
| `ollama` | 可以 | `http://localhost:11434/v1/` |

**`platform` 可选值：**

- `open_ai`
- `open_ai_resp`
- `gemini`
- `anthropic`
- `fireworks`
- `together`
- `groq`
- `mimo`
- `nebius`
- `xai`
- `deep_seek`
- `zai`
- `big_model`
- `cohere`
- `ollama`

**环境变量示例：**

```bash
export ROBUST_MQ_SERVER_LLM_CLIENT_PLATFORM=open_ai
export ROBUST_MQ_SERVER_LLM_CLIENT_MODEL=gpt-4o-mini
export ROBUST_MQ_SERVER_LLM_CLIENT_TOKEN=your_api_token
# export ROBUST_MQ_SERVER_LLM_CLIENT_BASE_URL=https://api.openai.com/v1/
```

---

## 9. Admin HTTP API 鉴权配置

### [admin]

Admin HTTP API 的登录认证配置。详细说明请参考 [API 鉴权文档](../Api/AUTH.md)。

```toml
[admin]
username = "admin"
password = "admin"
jwt_secret = "robustmq-change-me-in-production"
token_ttl_hours = 8
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `username` | `string` | `"admin"` | 管理员用户名 |
| `password` | `string` | `"admin"` | 管理员密码，生产环境务必修改 |
| `jwt_secret` | `string` | `"robustmq-change-me-in-production"` | JWT 签名密钥（HMAC-SHA256），生产环境务必修改为随机字符串（建议 32 位以上） |
| `token_ttl_hours` | `u64` | `8` | Token 有效期（小时） |

> ⚠️ **安全提示**：`password` 和 `jwt_secret` 使用默认值存在安全风险，生产部署前请务必修改。

**鉴权规则：**
- 来自 `127.0.0.1` / `::1` 的本地请求：**无需 token**，直接放行
- 来自其他 IP 的远程请求：需携带 `Authorization: Bearer <token>`
- `/api/v1/login`、`/health/*`、`/metrics` 路径：始终公开，无需鉴权

---

## 10. 监控与性能分析

RobustMQ 没有独立的 `[prometheus]` 或 `[pprof]` 配置 section——两者都复用 Admin HTTP API 的 `http_port`，没有独立可配置端口：

- **Prometheus 指标**：始终通过 `GET /metrics`（Admin HTTP API，见 [基础配置](#1-基础配置) 的 `http_port`）暴露，无需单独开关或端口配置。
- **pprof 性能分析**：由 [2. 运行时配置](#2-运行时配置) 中的 `runtime.pprof_enable` 控制是否采集，采集到的火焰图同样通过 Admin HTTP API 暴露，没有独立的 `port`/`frequency` 配置项。

---

## 完整配置示例

以下示例包含全部基础配置项；`[mqtt_runtime]`/`[kafka_runtime]`/`[amqp_runtime]`/`[nats_runtime]` 各协议自己的完整示例请见对应文档（[MQTT](MQTTConfig.md#完整示例)、[Kafka](KafkaConfig.md)、[AMQP](AMQPConfig.md)、[NATS](NATSConfig.md)）。

```toml
# ========== 基础配置 ==========
cluster_name = "production-cluster"
broker_id = 1
roles = ["meta", "broker", "engine"]
grpc_port = 1228
http_port = 58080

[meta_addrs]
1 = "192.168.1.10:1228"
2 = "192.168.1.11:1228"
3 = "192.168.1.12:1228"

# ========== 运行时 ==========
[runtime]
tls_cert = "./config/certs/cert.pem"
tls_key = "./config/certs/key.pem"
channels_per_address = 4
# server_worker_threads = 0
# meta_worker_threads = 0
# broker_worker_threads = 0
# pprof_enable = false

# ========== Meta ==========
[meta_runtime]
heartbeat_timeout_ms = 30000
heartbeat_check_time_ms = 1000
raft_write_timeout_sec = 30
offset_raft_group_num = 1
data_raft_group_num = 1
group_offset_expire_sec = 604800

# ========== RocksDB ==========
[rocksdb]
data_path = "/data/robustmq"
max_open_files = 20000

# ========== 存储引擎 ==========
[storage_runtime]
tcp_port = 1778
max_segment_size = 1073741824
io_thread_num = 8
expire_scan_task_num = 10
offset_enable_cache = true

# ========== 延迟任务 ==========
[delay_task]
delay_task_queue_num = 100
delay_task_handler_concurrency = 100

# ========== Broker 网络 ==========
[broker_network]
accept_thread_num = 2
handler_thread_num = 16
queue_size = 1000

# ========== LLM 客户端（可选） ==========
[llm_client]
platform = "open_ai"
model = "gpt-4o-mini"
token = "your_api_token"
# base_url = "https://api.openai.com/v1/"
# embedding = "text-embedding-3-small"
# embedding_model_path = "./models/embedding"

# ========== Admin 鉴权 ==========
[admin]
username = "admin"
password = "your_secure_password"
jwt_secret = "your-random-jwt-secret-32-chars-min"
token_ttl_hours = 8

# ========== 日志 ==========
[log]
log_config = "./config/broker-tracing.toml"
log_path = "./logs"
```
