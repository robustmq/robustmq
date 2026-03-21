# 连接器管理 HTTP API

> 本文档介绍连接器（Connector）相关的所有 HTTP API 接口。连接器用于将 MQTT 消息桥接到外部系统。
>
> 通用信息请参考 [COMMON.md](COMMON.md)。

## API 接口列表

### 1. 连接器列表查询
- **接口**: `GET /api/mqtt/connector/list`
- **描述**: 查询连接器列表，支持按租户过滤和连接器名称模糊搜索
- **请求参数**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `tenant` | string | 否 | 按租户精确过滤，指定后仅返回该租户的连接器（性能更好） |
| `connector_name` | string | 否 | 按连接器名称模糊搜索（包含匹配） |
| `limit` | u32 | 否 | 每页大小 |
| `page` | u32 | 否 | 页码，从 1 开始 |
| `sort_field` | string | 否 | 排序字段，支持 `connector_name`、`tenant` |
| `sort_by` | string | 否 | 排序方向：`asc` / `desc` |

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant": "default",
        "connector_name": "kafka_connector",
        "connector_type": "kafka",
        "config": "{\"Kafka\":{\"bootstrap_servers\":\"localhost:9092\"}}",
        "topic_name": "topic_001",
        "status": "Running",
        "broker_id": "1",
        "create_time": "2024-01-01 10:00:00",
        "update_time": "2024-01-01 11:00:00"
      }
    ],
    "total_count": 8
  }
}
```

**字段说明**：
- `tenant`: 连接器所属租户
- `connector_name`: 连接器名称
- `connector_type`: 连接器类型
- `config`: 连接器配置（JSON 字符串）
- `topic_name`: 关联的 MQTT 主题
- `status`: 连接器状态（Idle, Running, Stopped）
- `broker_id`: 运行连接器的 Broker 节点 ID
- `create_time`: 创建时间
- `update_time`: 更新时间

---

### 2. 连接器详情查询
- **接口**: `GET /api/mqtt/connector/detail`
- **描述**: 查询指定连接器的详细运行状态
- **请求参数**:
```json
{
  "tenant": "default",
  "connector_name": "kafka_connector"
}
```

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "last_send_time": 1698765432,
    "send_success_total": 10245,
    "send_fail_total": 3,
    "last_msg": "Batch sent successfully"
  }
}
```

- **错误响应**:
```json
{
  "code": 1,
  "message": "Connector kafka_connector does not exist."
}
```

或

```json
{
  "code": 1,
  "message": "Connector thread kafka_connector does not exist."
}
```

**字段说明**：
- `last_send_time`: 最后发送时间（Unix 时间戳，秒）
- `send_success_total`: 累计发送成功消息数
- `send_fail_total`: 累计发送失败消息数
- `last_msg`: 最后一次操作的消息描述，可能为 `null`

**注意事项**：
- 连接器必须存在且当前正在运行才能查询详情
- 统计数据在连接器重启后会重置

---

### 3. 创建连接器
- **接口**: `POST /api/mqtt/connector/create`
- **描述**: 创建新的连接器
- **请求参数**:
```json
{
  "tenant": "default",
  "connector_name": "new_connector",
  "connector_type": "kafka",
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
  "topic_name": "sensor/+",
  "failure_strategy": {
    "strategy": "discard",
    "retry_total_times": null,
    "wait_time_ms": null,
    "topic_name": null
  }
}
```

**参数验证规则**：
- `tenant`: 长度 1-256 个字符，所属租户名称
- `connector_name`: 长度 1-128 个字符
- `connector_type`: 长度 1-50 个字符，必须是支持的类型（见下文枚举）
- `config`: 长度 1-4096 个字符，JSON 字符串
- `topic_name`: 长度 1-256 个字符，关联的 MQTT 主题
- `failure_strategy`: 失败处理策略（见下文）

- **响应**: 成功返回 `"success"`

---

### 4. 删除连接器
- **接口**: `POST /api/mqtt/connector/delete`
- **描述**: 删除连接器
- **请求参数**:
```json
{
  "tenant": "default",
  "connector_name": "old_connector"
}
```

- **响应**: 成功返回 `"success"`

---

## 连接器类型与配置

### 支持的连接器类型

| 类型标识 | 名称 | 分类 |
|---------|------|------|
| `kafka` | Apache Kafka | 消息队列 |
| `pulsar` | Apache Pulsar | 消息队列 |
| `rabbitmq` | RabbitMQ | 消息队列 |
| `mqtt` | MQTT Bridge | 消息队列 |
| `redis` | Redis | 缓存/数据库 |
| `mysql` | MySQL | 关系型数据库 |
| `postgres` | PostgreSQL | 关系型数据库 |
| `mongodb` | MongoDB | NoSQL 数据库 |
| `clickhouse` | ClickHouse | 分析型数据库 |
| `cassandra` | Cassandra | 分布式数据库 |
| `elasticsearch` | Elasticsearch | 搜索引擎 |
| `greptime` | GreptimeDB | 时序数据库 |
| `influxdb` | InfluxDB | 时序数据库 |
| `opentsdb` | OpenTSDB | 时序数据库 |
| `webhook` | Webhook (HTTP) | HTTP 推送 |
| `s3` | AWS S3 | 对象存储 |
| `file` | 本地文件 | 文件存储 |

---

### Kafka 连接器

```json
{
  "connector_type": "kafka",
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}"
}
```

**必填参数**：
- `bootstrap_servers`: Kafka broker 地址，格式 `host1:port1,host2:port2`
- `topic`: Kafka 主题名称

**可选参数**：
- `key`: 消息键（默认 `""`），用于分区路由，最大 256 字符
- `compression_type`: 压缩算法（默认 `"none"`），可选 `none`、`gzip`、`snappy`、`lz4`、`zstd`
- `batch_size`: 批量发送最大字节数（默认 `16384`），范围 1-1048576
- `linger_ms`: 发送批次前等待时间（默认 `5`），范围 0-60000
- `acks`: 确认级别（默认 `"1"`），可选 `"0"`、`"1"`、`"all"`
- `retries`: 最大重试次数（默认 `3`），范围 0-100
- `message_timeout_ms`: 消息投递超时（默认 `30000`），范围 1000-300000
- `cleanup_timeout_secs`: 关闭时刷新超时（默认 `10`），范围 0-300

---

### Pulsar 连接器

```json
{
  "connector_type": "pulsar",
  "config": "{\"server\":\"pulsar://localhost:6650\",\"topic\":\"mqtt-messages\"}"
}
```

**必填参数**：
- `server`: Pulsar broker 地址，格式 `pulsar://host:port` 或 `pulsar+ssl://host:port`
- `topic`: Pulsar topic 名称

**认证参数**（三选一）：
- Token 认证：`token`
- OAuth2 认证：`oauth`（JSON 字符串）
- 基本认证：`basic_name` + `basic_password`

**可选参数**：
- `connection_timeout_secs`: 连接超时（默认 `30`），范围 1-300
- `operation_timeout_secs`: 操作超时（默认 `30`），范围 1-300
- `send_timeout_secs`: 发送超时（默认 `30`），范围 1-300
- `batch_size`: 批量记录数（默认 `100`），范围 1-10000
- `max_pending_messages`: 最大待发送消息数（默认 `1000`），范围 1-100000
- `compression`: 压缩算法（默认 `"none"`），可选 `none`、`lz4`、`zlib`、`zstd`、`snappy`

---

### RabbitMQ 连接器

```json
{
  "connector_type": "rabbitmq",
  "config": "{\"server\":\"localhost\",\"username\":\"guest\",\"exchange\":\"mqtt_messages\"}"
}
```

**必填参数**：
- `server`: RabbitMQ 服务器地址
- `username`: 用户名
- `password`: 密码
- `exchange`: 交换机名称

**可选参数**：
- `port`: 端口（默认 `5672`）
- `virtual_host`: 虚拟主机（默认 `"/"`）
- `routing_key`: 路由键（默认 `""`）
- `delivery_mode`: 持久化模式（默认 `"NonPersistent"`），可选 `NonPersistent`、`Persistent`
- `enable_tls`: 是否启用 TLS（默认 `false`）
- `connection_timeout_secs`: 连接超时（默认 `30`），范围 1-300
- `heartbeat_secs`: 心跳间隔（默认 `60`），范围 0-300
- `channel_max`: 最大通道数（默认 `2047`）
- `frame_max`: 最大帧大小（默认 `131072`），范围 4096-1048576
- `batch_size`: 批量记录数（默认 `100`），范围 1-10000
- `publisher_confirms`: 发布者确认（默认 `true`）
- `confirm_timeout_secs`: 确认超时（默认 `30`），范围 1-300

---

### MQTT Bridge 连接器

```json
{
  "connector_type": "mqtt",
  "config": "{\"server\":\"mqtt://remote-broker:1883\"}"
}
```

**必填参数**：
- `server`: 远端 MQTT Broker 地址，格式 `mqtt://host:port` 或 `mqtts://host:port`

**可选参数**：
- `client_id_prefix`: 客户端 ID 前缀，最大 64 字符
- `username`: 用户名
- `password`: 密码
- `protocol_version`: 协议版本（默认 `"v5"`），可选 `v3`、`v4`、`v5`
- `keepalive_secs`: 保活间隔（默认 `60`），范围 1-65535
- `connect_timeout_secs`: 连接超时（默认 `10`），范围 1-300
- `enable_tls`: 是否启用 TLS（默认 `false`）
- `topic_prefix`: 主题前缀（可选），转发时在原始主题前添加前缀
- `qos`: QoS 级别（默认 `1`），可选 0、1、2
- `retain`: 是否保留消息（默认 `false`）
- `max_retries`: 最大重试次数（默认 `3`），最大 10

---

### Redis 连接器

```json
{
  "connector_type": "redis",
  "config": "{\"server\":\"127.0.0.1:6379\",\"command_template\":\"LPUSH mqtt_messages {payload}\"}"
}
```

**必填参数**：
- `server`: Redis 服务器地址，格式 `host:port`，集群或哨兵模式使用逗号分隔
- `command_template`: Redis 命令模板，最大 4096 字符

**可选参数**：
- `mode`: 运行模式（默认 `"single"`），可选 `single`、`cluster`、`sentinel`
- `database`: 数据库编号（默认 `0`），范围 0-15
- `username`: 用户名
- `password`: 密码
- `sentinel_master_name`: 哨兵 master 名称（`sentinel` 模式必填）
- `tls_enabled`: 是否启用 TLS（默认 `false`）
- `connect_timeout_ms`: 连接超时毫秒（默认 `5000`）
- `pool_size`: 连接池大小（默认 `10`），范围 1-100
- `max_retries`: 最大重试次数（默认 `3`）
- `retry_interval_ms`: 重试间隔毫秒（默认 `1000`）

---

### MySQL 连接器

```json
{
  "connector_type": "mysql",
  "config": "{\"host\":\"localhost\",\"port\":3306,\"database\":\"mqtt_data\",\"username\":\"root\",\"password\":\"password\",\"table\":\"mqtt_messages\"}"
}
```

**必填参数**：
- `host`: MySQL 服务器地址
- `port`: 端口（默认 `3306`）
- `database`: 数据库名称
- `username`: 用户名
- `password`: 密码
- `table`: 表名称

**可选参数**：
- `pool_size`: 连接池最大连接数（默认 `10`），范围 1-1000
- `min_pool_size`: 连接池最小连接数（默认 `2`）
- `connect_timeout_secs`: 连接超时（默认 `10`），范围 1-300
- `acquire_timeout_secs`: 获取连接超时（默认 `30`），范围 1-300
- `idle_timeout_secs`: 空闲超时（默认 `600`），范围 0-3600
- `max_lifetime_secs`: 连接最大生命周期（默认 `1800`），范围 0-7200
- `batch_size`: 批量记录数（默认 `100`），范围 1-10000
- `enable_batch_insert`: 批量插入模式（默认 `false`）
- `enable_upsert`: upsert 模式（默认 `false`），使用 `ON DUPLICATE KEY UPDATE`
- `conflict_columns`: 冲突列名（upsert 模式必填）
- `sql_template`: 自定义 SQL 模板，包含 3 个 `?` 占位符（`record_key`, `payload`, `timestamp`）

---

### PostgreSQL 连接器

```json
{
  "connector_type": "postgres",
  "config": "{\"host\":\"localhost\",\"port\":5432,\"database\":\"mqtt_data\",\"username\":\"postgres\",\"password\":\"password\",\"table\":\"mqtt_messages\"}"
}
```

**必填参数**：
- `host`: PostgreSQL 服务器地址
- `port`: 端口（默认 `5432`）
- `database`: 数据库名称
- `username`: 用户名
- `password`: 密码
- `table`: 表名称

**可选参数**：
- `pool_size`: 连接池最大连接数（默认 `10`），范围 1-1000
- `min_pool_size`: 连接池最小连接数（默认 `2`）
- `connect_timeout_secs`: 连接超时（默认 `10`），范围 1-300
- `acquire_timeout_secs`: 获取连接超时（默认 `30`），范围 1-300
- `idle_timeout_secs`: 空闲超时（默认 `600`），范围 0-3600
- `max_lifetime_secs`: 连接最大生命周期（默认 `1800`），范围 0-7200
- `batch_size`: 批量记录数（默认 `100`），范围 1-10000
- `enable_batch_insert`: 批量插入模式（默认 `false`）
- `enable_upsert`: upsert 模式（默认 `false`），使用 `ON CONFLICT ... DO UPDATE`
- `conflict_columns`: 冲突列名（upsert 模式必填）
- `sql_template`: 自定义 SQL 模板，包含 5 个 `$1`-`$5` 占位符（`client_id`, `topic`, `timestamp`, `payload`, `data`）

---

### MongoDB 连接器

```json
{
  "connector_type": "mongodb",
  "config": "{\"host\":\"localhost\",\"port\":27017,\"database\":\"mqtt_data\",\"collection\":\"mqtt_messages\"}"
}
```

**必填参数**：
- `host`: MongoDB 服务器地址
- `port`: 端口（默认 `27017`）
- `database`: 数据库名称
- `collection`: 集合名称

**可选参数**：
- `username`: 用户名
- `password`: 密码
- `auth_source`: 认证数据库（默认 `"admin"`）
- `deployment_mode`: 部署模式（默认 `"single"`），可选 `single`、`replicaset`、`sharded`
- `replica_set_name`: 副本集名称（`replicaset` 模式必填）
- `enable_tls`: 是否启用 TLS（默认 `false`）
- `max_pool_size`: 连接池最大连接数，范围 1-1000
- `min_pool_size`: 连接池最小连接数
- `connect_timeout_secs`: 连接超时（默认 `10`），范围 1-300
- `server_selection_timeout_secs`: 服务器选择超时（默认 `30`），范围 1-300
- `socket_timeout_secs`: Socket 超时（默认 `60`），范围 1-600
- `batch_size`: 批量记录数（默认 `100`），范围 1-10000
- `ordered_insert`: 顺序插入（默认 `false`）
- `w`: 写关注级别（默认 `"1"`），可选 `"0"`、`"1"`、`"majority"`

---

### ClickHouse 连接器

```json
{
  "connector_type": "clickhouse",
  "config": "{\"url\":\"http://localhost:8123\",\"database\":\"mqtt\",\"table\":\"messages\"}"
}
```

**必填参数**：
- `url`: ClickHouse HTTP 接口地址，必须以 `http://` 或 `https://` 开头
- `database`: 数据库名称
- `table`: 表名称

**可选参数**：
- `username`: 用户名（默认 `""`）
- `password`: 密码（默认 `""`）
- `pool_size`: 连接池大小（默认 `8`），范围 1-64
- `timeout_secs`: 请求超时（默认 `15`），范围 1-300

---

### Cassandra 连接器

```json
{
  "connector_type": "cassandra",
  "config": "{\"nodes\":[\"127.0.0.1\"],\"keyspace\":\"mqtt\",\"table\":\"messages\"}"
}
```

**必填参数**：
- `nodes`: Cassandra 节点地址列表
- `keyspace`: Keyspace 名称
- `table`: 表名称

**可选参数**：
- `port`: 端口（默认 `9042`）
- `username`: 用户名（默认 `""`）
- `password`: 密码（默认 `""`）
- `replication_factor`: 副本因子（默认 `1`）
- `timeout_secs`: 超时（默认 `15`），范围 1-300

---

### Elasticsearch 连接器

```json
{
  "connector_type": "elasticsearch",
  "config": "{\"url\":\"http://localhost:9200\",\"index\":\"mqtt_messages\"}"
}
```

**必填参数**：
- `url`: Elasticsearch 服务器地址
- `index`: 索引名称

**可选参数**：
- `auth_type`: 认证类型（默认 `"none"`），可选 `none`、`basic`、`apikey`
- `username`: 用户名（Basic 认证必填）
- `password`: 密码（Basic 认证必填）
- `api_key`: API 密钥（ApiKey 认证必填）
- `enable_tls`: 是否启用 TLS（默认 `false`）
- `ca_cert_path`: CA 证书路径
- `timeout_secs`: 请求超时（默认 `30`），范围 1-300
- `max_retries`: 最大重试次数（默认 `3`），最大 10

---

### GreptimeDB 连接器

```json
{
  "connector_type": "greptime",
  "config": "{\"server_addr\":\"localhost:4000\",\"database\":\"public\"}"
}
```

**必填参数**：
- `server_addr`: GreptimeDB 服务器地址
- `database`: 数据库名称
- `user`: 用户名（无认证时传空字符串）
- `password`: 密码（无认证时传空字符串）

**可选参数**：
- `precision`: 时间精度（默认 `"Second"`）

---

### InfluxDB 连接器

```json
{
  "connector_type": "influxdb",
  "config": "{\"server\":\"http://localhost:8086\",\"version\":\"v2\",\"token\":\"my-token\",\"org\":\"my-org\",\"bucket\":\"my-bucket\",\"measurement\":\"mqtt_data\"}"
}
```

**必填参数**：
- `server`: InfluxDB 服务器地址，必须以 `http://` 或 `https://` 开头
- `measurement`: 测量名称
- `token`: API Token（v2 版本使用，v1 时传空字符串）
- `org`: 组织名（v2 版本使用，v1 时传空字符串）
- `bucket`: Bucket 名（v2 版本使用，v1 时传空字符串）
- `database`: 数据库名称（v1 版本使用，v2 时传空字符串）
- `username`: 用户名（v1 版本使用，v2 时传空字符串）
- `password`: 密码（v1 版本使用，v2 时传空字符串）

**说明**：v2 填 `token`/`org`/`bucket`，其余传空字符串；v1 填 `database`/`username`/`password`，其余传空字符串。

**可选参数**：
- `version`: InfluxDB 版本（默认 `"v2"`），可选 `v1`、`v2`
- `precision`: 写入精度（默认 `"ms"`），可选 `ns`（纳秒）、`us`（微秒）、`ms`（毫秒）、`s`（秒）
- `timeout_secs`: 请求超时（默认 `15`），范围 1-300

---

### OpenTSDB 连接器

```json
{
  "connector_type": "opentsdb",
  "config": "{\"server\":\"http://localhost:4242\"}"
}
```

**必填参数**：
- `server`: OpenTSDB 服务器地址，必须以 `http://` 或 `https://` 开头

**可选参数**：
- `metric_field`: 消息中 metric 字段名（默认 `"metric"`）
- `value_field`: 消息中 value 字段名（默认 `"value"`）
- `tags_fields`: 消息中 tags 字段名列表（默认 `[]`）
- `timeout_secs`: 请求超时（默认 `30`），范围 1-300
- `max_retries`: 最大重试次数（默认 `3`），最大 10
- `summary`: 返回摘要信息（默认 `false`）
- `details`: 返回详细信息（默认 `false`）

---

### Webhook 连接器

```json
{
  "connector_type": "webhook",
  "config": "{\"url\":\"https://example.com/webhook\"}"
}
```

**必填参数**：
- `url`: Webhook 目标 URL，必须以 `http://` 或 `https://` 开头，最大 2048 字符

**可选参数**：
- `method`: HTTP 方法（默认 `"post"`），可选 `post`、`put`
- `headers`: 自定义请求头（默认 `{}`）
- `timeout_ms`: 请求超时毫秒（默认 `5000`），范围 1-60000
- `auth_type`: 认证类型（默认 `"none"`），可选 `none`、`basic`、`bearer`
- `username`: 用户名（Basic 认证必填）
- `password`: 密码（Basic 认证必填）
- `bearer_token`: Bearer Token（Bearer 认证必填）

---

### AWS S3 连接器

```json
{
  "connector_type": "s3",
  "config": "{\"bucket\":\"my-mqtt-bucket\",\"region\":\"us-east-1\",\"object_key_prefix\":\"mqtt\"}"
}
```

**必填参数**：
- `bucket`: S3 Bucket 名称
- `region`: AWS 区域（例如 `us-east-1`）
- `access_key_id`: Access Key（无认证时传空字符串）
- `secret_access_key`: Secret Key（无认证时传空字符串）
- `session_token`: 临时凭证 Session Token（不使用时传空字符串）

**可选参数**：
- `endpoint`: S3 Endpoint（兼容 MinIO 等对象存储时可配置，默认 `""`）
- `root`: 对象存储根路径前缀（默认 `""`）
- `object_key_prefix`: 对象 key 前缀（默认 `"mqtt"`）
- `file_extension`: 对象后缀名（默认 `"json"`，仅允许字母数字）

> 写入说明：S3 连接器按批次将消息序列化为 JSON 数组后写入单个对象。

---

### 本地文件连接器

```json
{
  "connector_type": "file",
  "config": "{\"local_file_path\":\"/tmp/mqtt_messages.log\"}"
}
```

**必填参数**：
- `local_file_path`: 文件路径

**可选参数**：
- `rotation_strategy`: 文件滚动策略（默认 `"none"`），可选 `none`、`size`、`hourly`、`daily`
- `max_size_gb`: 文件最大大小 GB（默认 `1`，范围 1-10，仅 `size` 策略生效）

---

## 失败处理策略

`failure_strategy` 参数定义连接器消息投递失败时的处理方式。

### 策略类型

#### 1. 丢弃策略（Discard）

立即丢弃失败消息，不重试。

```json
{
  "strategy": "discard"
}
```

#### 2. 重试后丢弃策略（DiscardAfterRetry）

在指定次数内重试，超过次数后丢弃。

```json
{
  "strategy": "discard_after_retry",
  "retry_total_times": 3,
  "wait_time_ms": 1000
}
```

- `retry_total_times`: 最大重试次数（不包含首次发送，必须 > 0）
- `wait_time_ms`: 重试间隔毫秒（必须 > 0）

#### 3. 死信队列策略（DeadMessageQueue）

先进行重试，当重试次数耗尽后，将失败消息写入指定的死信队列主题，便于后续恢复和分析。

```json
{
  "strategy": "dead_message_queue",
  "topic_name": "dead_letter_queue",
  "retry_total_times": 3,
  "wait_time_ms": 1000
}
```

- `topic_name`: 死信队列主题名称（默认 `"dead_letter_queue"`）
- `retry_total_times`: 写入死信队列前的最大重试次数（不包含首次发送，默认 `3`，必须 > 0）
- `wait_time_ms`: 重试间隔毫秒（默认 `1000`，必须 > 0）

> 参数校验：`dead_message_queue` 的 `topic_name`（若传入）不能为空，且长度不能超过 256。

**死信消息格式**：

写入死信队列的消息为 JSON 格式的 `DeadLetterRecord`：

```json
{
  "connector_name": "kafka_bridge",
  "source_topic": "sensor/+",
  "error_message": "Connection refused",
  "retry_times": 3,
  "original_key": "sensor-001",
  "original_data": [98, 121, 116, 101, 115],
  "original_timestamp": 1640995200,
  "dead_letter_timestamp": 1640995260
}
```

| 字段 | 类型 | 说明 |
|-----|------|------|
| `connector_name` | String | 失败的连接器名称 |
| `source_topic` | String | 原始消息来源主题 |
| `error_message` | String | 失败原因 |
| `retry_times` | u32 | 已重试次数 |
| `original_key` | String? | 原始消息 key（可为 null） |
| `original_data` | [u8] | 原始消息数据（字节数组） |
| `original_timestamp` | u64 | 原始消息时间戳 |
| `dead_letter_timestamp` | u64 | 进入死信队列的时间戳 |

---

## 枚举值参考

### 连接器类型 (connector_type)

| 值 | 名称 | 分类 |
|---|------|------|
| `kafka` | Apache Kafka | 消息队列 |
| `pulsar` | Apache Pulsar | 消息队列 |
| `rabbitmq` | RabbitMQ | 消息队列 |
| `mqtt` | MQTT Bridge | 消息桥接 |
| `redis` | Redis | 缓存/数据库 |
| `mysql` | MySQL | 关系型数据库 |
| `postgres` | PostgreSQL | 关系型数据库 |
| `mongodb` | MongoDB | NoSQL 数据库 |
| `clickhouse` | ClickHouse | 分析型数据库 |
| `cassandra` | Cassandra | 分布式数据库 |
| `elasticsearch` | Elasticsearch | 搜索引擎 |
| `greptime` | GreptimeDB | 时序数据库 |
| `influxdb` | InfluxDB | 时序数据库 |
| `opentsdb` | OpenTSDB | 时序数据库 |
| `webhook` | Webhook (HTTP) | HTTP 推送 |
| `s3` | AWS S3 | 对象存储 |
| `file` | 本地文件 | 文件存储 |

### 连接器状态 (status)

| 值 | 说明 |
|---|------|
| `Idle` | 空闲，未分配到 Broker |
| `Running` | 运行中 |
| `Stopped` | 已停止 |

### 失败处理策略 (strategy)

| 值 | 说明 |
|---|------|
| `discard` | 直接丢弃 |
| `discard_after_retry` | 重试后丢弃 |
| `dead_message_queue` | 写入死信队列 |

### Redis 模式 (mode)

| 值 | 说明 |
|---|------|
| `single` | 单节点模式 |
| `cluster` | 集群模式 |
| `sentinel` | 哨兵模式 |

### Webhook HTTP 方法 (method)

| 值 | 说明 |
|---|------|
| `post` | HTTP POST |
| `put` | HTTP PUT |

### Webhook 认证类型 (auth_type)

| 值 | 说明 |
|---|------|
| `none` | 无认证 |
| `basic` | Basic 认证 |
| `bearer` | Bearer Token 认证 |

### MQTT Bridge 协议版本 (protocol_version)

| 值 | 说明 |
|---|------|
| `v3` | MQTT 3.1 |
| `v4` | MQTT 3.1.1 |
| `v5` | MQTT 5.0 |

### InfluxDB 版本 (version)

| 值 | 说明 |
|---|------|
| `v1` | InfluxDB 1.x |
| `v2` | InfluxDB 2.x |

### InfluxDB 写入精度 (precision)

| 值 | 说明 |
|---|------|
| `ns` | 纳秒 |
| `us` | 微秒 |
| `ms` | 毫秒 |
| `s` | 秒 |

### MongoDB 部署模式 (deployment_mode)

| 值 | 说明 |
|---|------|
| `single` | 单节点 |
| `replicaset` | 副本集 |
| `sharded` | 分片集群 |

---

## 使用示例

### 查询连接器列表
```bash
curl "http://localhost:8080/api/mqtt/connector/list?limit=10&page=1"
```

### 按租户查询连接器列表
```bash
curl "http://localhost:8080/api/mqtt/connector/list?tenant=default&limit=10&page=1"
```

### 查询连接器详情
```bash
curl "http://localhost:8080/api/mqtt/connector/detail?tenant=default&connector_name=kafka_bridge"
```

### 创建 Kafka 连接器
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "connector_name": "kafka_bridge",
    "connector_type": "kafka",
    "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "discard"
    }
  }'
```

### 创建带重试策略的连接器
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "connector_name": "kafka_bridge_retry",
    "connector_type": "kafka",
    "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "discard_after_retry",
      "retry_total_times": 5,
      "wait_time_ms": 2000
    }
  }'
```

### 创建带死信队列的连接器
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "connector_name": "kafka_bridge_dlq",
    "connector_type": "kafka",
    "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "dead_message_queue",
      "topic_name": "dead_letter_queue",
      "retry_total_times": 5,
      "wait_time_ms": 2000
    }
  }'
```

### 创建 Redis 连接器
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "connector_name": "redis_bridge",
    "connector_type": "redis",
    "config": "{\"server\":\"127.0.0.1:6379\",\"command_template\":\"LPUSH mqtt_messages {payload}\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "discard"
    }
  }'
```

### 创建 Webhook 连接器
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "connector_name": "webhook_bridge",
    "connector_type": "webhook",
    "config": "{\"url\":\"https://example.com/webhook\",\"method\":\"post\",\"auth_type\":\"bearer\",\"bearer_token\":\"my-token\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "discard"
    }
  }'
```

### 创建 InfluxDB 连接器（v2）
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "connector_name": "influxdb_bridge",
    "connector_type": "influxdb",
    "config": "{\"server\":\"http://localhost:8086\",\"version\":\"v2\",\"token\":\"my-token\",\"org\":\"my-org\",\"bucket\":\"mqtt\",\"measurement\":\"sensor_data\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "discard"
    }
  }'
```

### 创建 AWS S3 连接器
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "connector_name": "s3_bridge",
    "connector_type": "s3",
    "config": "{\"bucket\":\"my-mqtt-bucket\",\"region\":\"us-east-1\",\"object_key_prefix\":\"mqtt\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "discard"
    }
  }'
```

### 删除连接器
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/delete \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "connector_name": "old_connector"
  }'
```

---

*文档版本: v2.0*
*最后更新: 2026-03-15*
*基于代码版本: RobustMQ v0.3.2*
