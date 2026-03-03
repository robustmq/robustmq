# InfluxDB 连接器

## 概述

InfluxDB 连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息写入 InfluxDB 时序数据库。InfluxDB 专为时序数据设计，广泛应用于 IoT 监控、基础设施指标采集和实时分析等场景。

该连接器使用 HTTP API + Line Protocol 直接写入数据，无需额外客户端库，兼容 InfluxDB v1.x 和 v2.x。

## 功能特性

- 同时支持 InfluxDB v1 和 v2
- 基于 HTTP API + Line Protocol，兼容性好
- 支持 Token 认证（v2）和用户名/密码认证（v1）
- 可配置写入精度（ns/us/ms/s）
- 批量写入，高吞吐

## 数据映射

连接器将 MQTT 消息转换为 InfluxDB Line Protocol 格式：

```
measurement,key=<message_key> payload="<message_payload>" <timestamp>
```

| Line Protocol 组件 | 来源 | 说明 |
|-------------------|------|------|
| `measurement` | 配置项 | 由用户在连接器配置中指定 |
| `key` (tag) | `record.key` | 消息键值（通常为来源标识），作为 tag 存储 |
| `payload` (field) | `record.data` | MQTT 消息内容，作为字符串 field 存储 |
| `timestamp` | `record.timestamp` | 消息时间戳 |

## 配置说明

### 连接器配置

```rust
pub struct InfluxDBConnectorConfig {
    pub server: String,          // InfluxDB HTTP 地址
    pub version: InfluxDBVersion, // v1 或 v2
    pub token: String,           // v2 认证 Token
    pub org: String,             // v2 组织
    pub bucket: String,          // v2 Bucket
    pub database: String,        // v1 数据库
    pub username: String,        // v1 用户名
    pub password: String,        // v1 密码
    pub measurement: String,     // measurement 名称
    pub precision: InfluxDBPrecision, // 写入精度
    pub timeout_secs: u64,       // 超时时间（秒）
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 示例 |
|--------|------|------|--------|------|------|
| `server` | String | 是 | - | InfluxDB HTTP 端点，必须以 `http://` 或 `https://` 开头 | `http://localhost:8086` |
| `version` | String | 否 | `v2` | InfluxDB 版本：`v1` 或 `v2` | `v2` |
| `measurement` | String | 是 | - | InfluxDB measurement 名称 | `mqtt_messages` |
| `precision` | String | 否 | `ms` | 写入精度：`ns`/`us`/`ms`/`s` | `ms` |
| `timeout_secs` | Number | 否 | `15` | 请求超时时间（秒），范围：1-300 | `15` |

**InfluxDB v2 专属参数：**

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 示例 |
|--------|------|------|--------|------|------|
| `token` | String | 是 | - | 认证 Token | `my-token` |
| `org` | String | 是 | - | 组织名称 | `my-org` |
| `bucket` | String | 是 | - | Bucket 名称 | `mqtt-bucket` |

**InfluxDB v1 专属参数：**

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 示例 |
|--------|------|------|--------|------|------|
| `database` | String | 是 | - | 数据库名称 | `mqtt_db` |
| `username` | String | 否 | 空 | 用户名 | `admin` |
| `password` | String | 否 | 空 | 密码 | `password123` |

### 配置示例

#### InfluxDB v2 配置

```json
{
  "server": "http://localhost:8086",
  "version": "v2",
  "token": "my-super-secret-token",
  "org": "my-org",
  "bucket": "iot-data",
  "measurement": "sensor_readings",
  "precision": "ms"
}
```

#### InfluxDB v1 配置

```json
{
  "server": "http://localhost:8086",
  "version": "v1",
  "database": "mqtt_db",
  "username": "admin",
  "password": "password",
  "measurement": "mqtt_messages",
  "precision": "s"
}
```

#### 完整连接器配置

```json
{
  "cluster_name": "default",
  "connector_name": "influxdb_connector_01",
  "connector_type": "influxdb",
  "config": "{\"server\": \"http://localhost:8086\", \"version\": \"v2\", \"token\": \"my-token\", \"org\": \"my-org\", \"bucket\": \"mqtt-data\", \"measurement\": \"sensor\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## 使用 robust-ctl 创建 InfluxDB 连接器

### 基本语法

```bash
robust-ctl mqtt connector create \
  --connector-name <连接器名称> \
  --connector-type <连接器类型> \
  --config <配置> \
  --topic-id <主题ID>
```

### 创建示例

#### 1. InfluxDB v2

```bash
robust-ctl mqtt connector create \
  --connector-name "influxdb_v2_connector" \
  --connector-type "influxdb" \
  --config '{"server": "http://localhost:8086", "version": "v2", "token": "my-token", "org": "my-org", "bucket": "iot-data", "measurement": "sensor_readings"}' \
  --topic-id "sensor/data"
```

#### 2. InfluxDB v1

```bash
robust-ctl mqtt connector create \
  --connector-name "influxdb_v1_connector" \
  --connector-type "influxdb" \
  --config '{"server": "http://localhost:8086", "version": "v1", "database": "mqtt_db", "measurement": "mqtt_messages"}' \
  --topic-id "device/#"
```

### 管理连接器

```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 查看指定连接器
robust-ctl mqtt connector list --connector-name "influxdb_v2_connector"

# 删除连接器
robust-ctl mqtt connector delete --connector-name "influxdb_v2_connector"
```

### 完整操作示例

#### 场景：IoT 传感器数据存储

```bash
# 1. 确保 InfluxDB v2 已启动，并创建 bucket
# influx bucket create -n iot-data -o my-org

# 2. 创建连接器
robust-ctl mqtt connector create \
  --connector-name "sensor_to_influxdb" \
  --connector-type "influxdb" \
  --config '{"server": "http://localhost:8086", "version": "v2", "token": "my-token", "org": "my-org", "bucket": "iot-data", "measurement": "sensor"}' \
  --topic-id "sensor/+"

# 3. 查看创建的连接器
robust-ctl mqtt connector list
```

## 性能优化建议

### 1. 写入精度
- 如果不需要毫秒级精度，使用 `s`（秒）可减少存储开销
- IoT 场景通常 `ms` 或 `s` 即可满足

### 2. 批量写入
- 连接器内置批量写入，每批最多 100 条记录
- InfluxDB 对大批量 Line Protocol 写入非常友好

### 3. Bucket 策略（v2）
- 合理设置 Retention Policy / Bucket 的数据保留时间
- 对于高频数据，考虑使用 Downsampling Task 聚合历史数据

### 4. 安全建议
- v2 使用有限权限的 Token（仅 Write 权限）
- v1 创建专用写入用户
- 生产环境使用 HTTPS

## 监控和故障排查

### 1. 查看连接器状态

```bash
robust-ctl mqtt connector list --connector-name "influxdb_v2_connector"
```

### 2. 常见问题

**问题 1：连接失败**
- 检查 InfluxDB 服务是否运行
- 验证 server 地址和端口（默认 8086）
- 确认网络连通性

**问题 2：认证失败（v2）**
- 检查 Token 是否正确且有效
- 确认 org 和 bucket 名称与 InfluxDB 配置一致
- Token 需要有目标 bucket 的写入权限

**问题 3：认证失败（v1）**
- 检查用户名/密码是否正确
- 确认数据库已创建

**问题 4：写入延迟**
- 检查 InfluxDB 服务器负载
- 适当降低写入精度
- 检查网络延迟

## 总结

InfluxDB 连接器为 RobustMQ 提供了与时序数据库的原生集成能力。通过直接使用 HTTP + Line Protocol，实现了：

- **广泛兼容**：同时支持 InfluxDB v1 和 v2，无需额外客户端库
- **高性能**：批量 Line Protocol 写入，原生 HTTP API 无额外序列化开销
- **易维护**：无第三方 crate 依赖，基于稳定的 Line Protocol 规范
- **灵活配置**：支持多种认证方式和写入精度
