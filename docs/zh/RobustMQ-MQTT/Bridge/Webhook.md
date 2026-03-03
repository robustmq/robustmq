# Webhook 连接器

## 概述

Webhook 连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息通过 HTTP 请求转发到外部 Web 服务。该连接器支持将实时 MQTT 数据流以 JSON 格式 POST/PUT 到指定的 HTTP 端点，适合事件通知、数据同步、第三方系统集成等场景。

## 配置说明

### 连接器配置

Webhook 连接器使用 `WebhookConnectorConfig` 结构进行配置：

```rust
pub struct WebhookConnectorConfig {
    pub url: String,                       // HTTP 端点地址
    pub method: WebhookHttpMethod,         // HTTP 方法（POST/PUT）
    pub headers: HashMap<String, String>,  // 自定义 HTTP 请求头
    pub timeout_ms: u64,                   // 请求超时时间（毫秒）
    pub auth_type: WebhookAuthType,        // 认证类型
    pub username: Option<String>,          // 用户名（Basic 认证）
    pub password: Option<String>,          // 密码（Basic 认证）
    pub bearer_token: Option<String>,      // Bearer Token
}

pub enum WebhookHttpMethod {
    Post,   // HTTP POST
    Put,    // HTTP PUT
}

pub enum WebhookAuthType {
    None,    // 无认证
    Basic,   // Basic 认证
    Bearer,  // Bearer Token 认证
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 示例 |
|--------|------|------|--------|------|------|
| `url` | String | 是 | - | HTTP 端点地址，长度不超过 2048 字符，必须以 `http://` 或 `https://` 开头 | `https://api.example.com/webhook` |
| `method` | String | 否 | `post` | HTTP 方法：`post`、`put` | `post` |
| `headers` | Object | 否 | `{}` | 自定义 HTTP 请求头键值对 | `{"X-Api-Key": "abc123"}` |
| `timeout_ms` | Number | 否 | `5000` | 请求超时时间（毫秒），范围：1-60000 | `10000` |
| `auth_type` | String | 否 | `none` | 认证类型：`none`、`basic`、`bearer` | `basic` |
| `username` | String | 否 | - | 用户名，Basic 认证时必填 | `admin` |
| `password` | String | 否 | - | 密码，Basic 认证时必填 | `password123` |
| `bearer_token` | String | 否 | - | Bearer Token，Bearer 认证时必填 | `eyJhbGciOi...` |

### 配置示例

#### JSON 配置格式

**无认证配置**
```json
{
  "url": "http://localhost:8080/webhook"
}
```

**Basic 认证配置**
```json
{
  "url": "https://api.example.com/events",
  "method": "post",
  "auth_type": "basic",
  "username": "admin",
  "password": "password123",
  "timeout_ms": 10000
}
```

**Bearer Token 认证配置**
```json
{
  "url": "https://api.example.com/events",
  "auth_type": "bearer",
  "bearer_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "headers": {
    "X-Source": "robustmq",
    "X-Event-Type": "mqtt_message"
  }
}
```

**自定义请求头配置**
```json
{
  "url": "https://api.example.com/ingest",
  "method": "put",
  "headers": {
    "X-Api-Key": "your-api-key",
    "X-Tenant-Id": "tenant-001"
  },
  "timeout_ms": 15000
}
```

#### 完整连接器配置

```json
{
  "cluster_name": "default",
  "connector_name": "webhook_connector_01",
  "connector_type": "webhook",
  "config": "{\"url\": \"https://api.example.com/events\", \"auth_type\": \"bearer\", \"bearer_token\": \"your-token\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## 消息格式

### 单条消息

当连接器转发单条消息时，HTTP 请求体为单个 JSON 对象：

```json
{
  "payload": "{\"temperature\": 25.5}",
  "timestamp": 1640995200,
  "key": "sensor_001",
  "headers": [
    {"name": "topic", "value": "sensor/temperature"},
    {"name": "qos", "value": "1"}
  ]
}
```

### 批量消息

当连接器批量转发消息时，HTTP 请求体为 JSON 数组：

```json
[
  {
    "payload": "{\"temperature\": 25.5}",
    "timestamp": 1640995200,
    "key": "sensor_001"
  },
  {
    "payload": "{\"temperature\": 26.0}",
    "timestamp": 1640995201,
    "key": "sensor_002"
  }
]
```

### 字段说明

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `payload` | String | 消息内容 |
| `timestamp` | Number | 消息时间戳（秒） |
| `key` | String | 消息键值（可选） |
| `headers` | Array | 消息头信息数组（可选） |

## 使用 robust-ctl 创建 Webhook 连接器

### 基本语法

```bash
robust-ctl mqtt connector create \
  --connector-name <连接器名称> \
  --connector-type <连接器类型> \
  --config <配置> \
  --topic-id <主题ID>
```

### 创建示例

#### 1. 基本创建命令

```bash
robust-ctl mqtt connector create \
  --connector-name "webhook_connector_01" \
  --connector-type "webhook" \
  --config '{"url": "http://localhost:8080/webhook"}' \
  --topic-id "sensor/data"
```

#### 2. 带认证的 Webhook

```bash
robust-ctl mqtt connector create \
  --connector-name "webhook_auth" \
  --connector-type "webhook" \
  --config '{"url": "https://api.example.com/events", "auth_type": "bearer", "bearer_token": "your-token", "timeout_ms": 10000}' \
  --topic-id "device/status"
```

### 管理连接器

```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 查看指定连接器
robust-ctl mqtt connector list --connector-name "webhook_connector_01"

# 删除连接器
robust-ctl mqtt connector delete --connector-name "webhook_connector_01"
```

### 完整操作示例

#### 场景：IoT 设备事件通知

```bash
# 1. 创建传感器数据推送
robust-ctl mqtt connector create \
  --connector-name "sensor_webhook" \
  --connector-type "webhook" \
  --config '{"url": "https://api.example.com/iot/sensor-data", "auth_type": "bearer", "bearer_token": "your-token", "headers": {"X-Source": "robustmq"}}' \
  --topic-id "iot/sensors/temperature"

# 2. 创建设备告警推送
robust-ctl mqtt connector create \
  --connector-name "alarm_webhook" \
  --connector-type "webhook" \
  --config '{"url": "https://api.example.com/iot/alarms", "auth_type": "basic", "username": "admin", "password": "secret", "timeout_ms": 3000}' \
  --topic-id "iot/alarms"

# 3. 查看创建的连接器
robust-ctl mqtt connector list
```

## 性能优化建议

### 1. 超时设置
- 根据目标服务的响应时间合理设置 `timeout_ms`
- 对于高延迟服务，适当增加超时时间
- 对于告警类场景，建议使用较短的超时时间

### 2. 目标服务
- 确保目标 HTTP 服务具有足够的并发处理能力
- 建议目标服务返回 2xx 状态码表示成功
- 使用 HTTPS 确保数据传输安全

### 3. 安全建议
- 生产环境建议使用 HTTPS
- 使用 Bearer Token 或 Basic Auth 进行认证
- 定期轮换认证凭证
- 通过自定义 Headers 传递额外的安全信息

## 监控和故障排查

### 1. 查看连接器状态

```bash
robust-ctl mqtt connector list --connector-name "webhook_connector_01"
```

### 2. 常见问题

**问题 1：连接超时**
- 检查目标 HTTP 服务是否正常运行
- 验证 URL 地址是否正确
- 检查网络连接和防火墙设置
- 适当增加 `timeout_ms` 值

**问题 2：认证失败（HTTP 401/403）**
- 验证认证类型和凭证是否正确
- 检查 Bearer Token 是否过期
- 确认用户权限是否足够

**问题 3：服务端错误（HTTP 5xx）**
- 检查目标服务日志
- 确认请求格式是否满足目标服务要求
- 检查目标服务负载情况

## 总结

Webhook 连接器是 RobustMQ 数据集成系统的重要组件，提供了将 MQTT 消息实时推送到外部 HTTP 服务的能力。通过简单的配置即可实现：

- **事件通知**：将 MQTT 消息实时推送到 Web 服务
- **灵活认证**：支持 Basic Auth 和 Bearer Token 认证
- **自定义请求**：支持自定义 HTTP 方法和请求头
- **易于集成**：标准 HTTP/JSON 格式，与任何 Web 服务兼容
