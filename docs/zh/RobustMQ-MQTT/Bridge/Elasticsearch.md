# Elasticsearch 连接器

## 概述

Elasticsearch 连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息桥接到 Elasticsearch 搜索和分析引擎。该连接器支持将实时 MQTT 数据流写入 Elasticsearch 索引，适合日志分析、全文搜索、数据可视化和实时监控等场景。

## 配置说明

### 连接器配置

Elasticsearch 连接器使用 `ElasticsearchConnectorConfig` 结构进行配置：

```rust
pub struct ElasticsearchConnectorConfig {
    pub url: String,                  // Elasticsearch 服务器地址
    pub index: String,                // 索引名称
    pub auth_type: ElasticsearchAuthType,  // 认证类型
    pub username: Option<String>,     // 用户名（Basic 认证）
    pub password: Option<String>,     // 密码（Basic 认证）
    pub api_key: Option<String>,      // API 密钥（ApiKey 认证）
    pub enable_tls: bool,             // 是否启用 TLS
    pub ca_cert_path: Option<String>, // CA 证书路径
    pub timeout_secs: u64,            // 请求超时时间（秒）
    pub max_retries: u32,             // 最大重试次数
}

pub enum ElasticsearchAuthType {
    None,    // 无认证
    Basic,   // 基本认证
    ApiKey,  // API 密钥认证
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 示例 |
|--------|------|------|--------|------|------|
| `url` | String | 是 | - | Elasticsearch 服务器地址，长度不超过 512 字符 | `http://localhost:9200` |
| `index` | String | 是 | - | Elasticsearch 索引名称，长度不超过 256 字符 | `mqtt_messages` |
| `auth_type` | String | 否 | `none` | 认证类型：`none`、`basic`、`apikey` | `basic` |
| `username` | String | 否 | - | 用户名，Basic 认证时必填，长度不超过 256 字符 | `elastic` |
| `password` | String | 否 | - | 密码，Basic 认证时必填，长度不超过 256 字符 | `password123` |
| `api_key` | String | 否 | - | API 密钥，ApiKey 认证时必填 | `api_key_value` |
| `enable_tls` | Boolean | 否 | `false` | 是否启用 TLS 加密连接 | `true` |
| `ca_cert_path` | String | 否 | - | CA 证书文件路径（TLS 连接时使用） | `/etc/certs/ca.crt` |
| `timeout_secs` | Number | 否 | `30` | 请求超时时间（秒），范围：1-300 | `60` |
| `max_retries` | Number | 否 | `3` | 最大重试次数，不超过 10 | `5` |

### 配置示例

#### JSON 配置格式

**无认证配置**
```json
{
  "url": "http://localhost:9200",
  "index": "mqtt_messages"
}
```

**Basic 认证配置**
```json
{
  "url": "http://localhost:9200",
  "index": "mqtt_messages",
  "auth_type": "basic",
  "username": "elastic",
  "password": "password123"
}
```

**API Key 认证配置**
```json
{
  "url": "https://elasticsearch.example.com:9200",
  "index": "mqtt_messages",
  "auth_type": "apikey",
  "api_key": "your_api_key_here",
  "enable_tls": true,
  "timeout_secs": 60,
  "max_retries": 5
}
```

**TLS 加密连接配置**
```json
{
  "url": "https://elasticsearch.example.com:9200",
  "index": "mqtt_messages",
  "auth_type": "basic",
  "username": "elastic",
  "password": "password123",
  "enable_tls": true,
  "ca_cert_path": "/etc/certs/ca.crt"
}
```

#### 完整连接器配置

```json
{
  "cluster_name": "default",
  "connector_name": "elasticsearch_connector_01",
  "connector_type": "Elasticsearch",
  "config": "{\"url\": \"http://localhost:9200\", \"index\": \"mqtt_messages\", \"auth_type\": \"basic\", \"username\": \"elastic\", \"password\": \"password123\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## 消息格式

### 传输格式
Elasticsearch 连接器将 MQTT 消息转换为 JSON 格式后批量写入 Elasticsearch 索引。

### 消息结构

```json
{
  "offset": 12345,
  "header": [
    {
      "name": "topic",
      "value": "sensor/temperature"
    },
    {
      "name": "qos",
      "value": "1"
    }
  ],
  "key": "sensor_001",
  "data": "eyJ0ZW1wZXJhdHVyZSI6IDI1LjUsICJodW1pZGl0eSI6IDYwfQ==",
  "tags": ["sensor", "temperature"],
  "timestamp": 1640995200,
  "crc_num": 1234567890
}
```

### 字段说明

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `offset` | Number | 消息偏移量 |
| `header` | Array | 消息头信息数组 |
| `key` | String | 消息键值 |
| `data` | String | 消息内容（Base64 编码） |
| `tags` | Array | 消息标签数组 |
| `timestamp` | Number | 消息时间戳（秒） |
| `crc_num` | Number | 消息 CRC 校验值 |

## 使用 robust-ctl 创建 Elasticsearch 连接器

### 基本语法

使用 `robust-ctl` 命令行工具可以方便地创建和管理 Elasticsearch 连接器：

```bash
robust-ctl mqtt connector create \
  --connector-name <连接器名称> \
  --connector-type <连接器类型> \
  --config <配置> \
  --topic-id <主题ID>
```

### 创建 Elasticsearch 连接器

#### 1. 基本创建命令

```bash
# 创建 Elasticsearch 连接器（无认证）
robust-ctl mqtt connector create \
  --connector-name "elasticsearch_connector_01" \
  --connector-type "Elasticsearch" \
  --config '{"url": "http://localhost:9200", "index": "mqtt_messages"}' \
  --topic-id "sensor/data"
```

#### 2. 参数说明

| 参数 | 说明 | 示例值 |
|------|------|--------|
| `--connector-name` | 连接器名称，必须唯一 | `elasticsearch_connector_01` |
| `--connector-type` | 连接器类型，固定为 `Elasticsearch` | `Elasticsearch` |
| `--config` | JSON 格式的配置信息 | `{"url": "http://localhost:9200", "index": "mqtt_messages"}` |
| `--topic-id` | 要监听的 MQTT 主题 | `sensor/data` |

#### 3. 配置示例

**无认证连接**
```bash
robust-ctl mqtt connector create \
  --connector-name "es_no_auth" \
  --connector-type "Elasticsearch" \
  --config '{"url": "http://localhost:9200", "index": "mqtt_logs"}' \
  --topic-id "logs/#"
```

**Basic 认证连接**
```bash
robust-ctl mqtt connector create \
  --connector-name "es_basic_auth" \
  --connector-type "Elasticsearch" \
  --config '{"url": "http://localhost:9200", "index": "mqtt_messages", "auth_type": "basic", "username": "elastic", "password": "password123"}' \
  --topic-id "sensor/+/data"
```

**API Key 认证连接（带 TLS）**
```bash
robust-ctl mqtt connector create \
  --connector-name "es_secure" \
  --connector-type "Elasticsearch" \
  --config '{"url": "https://es.example.com:9200", "index": "mqtt_secure", "auth_type": "apikey", "api_key": "your_key", "enable_tls": true, "timeout_secs": 60, "max_retries": 5}' \
  --topic-id "secure/data"
```

### 管理连接器

#### 1. 列出所有连接器
```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 列出指定名称的连接器
robust-ctl mqtt connector list --connector-name "elasticsearch_connector_01"
```

#### 2. 删除连接器
```bash
# 删除指定连接器
robust-ctl mqtt connector delete --connector-name "elasticsearch_connector_01"
```

### 完整操作示例

#### 场景：创建 IoT 数据搜索分析系统

```bash
# 1. 创建传感器数据 Elasticsearch 连接器
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_es" \
  --connector-type "Elasticsearch" \
  --config '{"url": "http://localhost:9200", "index": "iot_sensors", "auth_type": "basic", "username": "elastic", "password": "elastic123"}' \
  --topic-id "iot/sensors/+/data"

# 2. 创建设备状态 Elasticsearch 连接器
robust-ctl mqtt connector create \
  --connector-name "device_status_es" \
  --connector-type "Elasticsearch" \
  --config '{"url": "http://localhost:9200", "index": "device_status", "auth_type": "basic", "username": "elastic", "password": "elastic123"}' \
  --topic-id "iot/devices/+/status"

# 3. 创建告警消息 Elasticsearch 连接器
robust-ctl mqtt connector create \
  --connector-name "alarm_es" \
  --connector-type "Elasticsearch" \
  --config '{"url": "http://localhost:9200", "index": "alarms", "auth_type": "basic", "username": "elastic", "password": "elastic123", "timeout_secs": 60}' \
  --topic-id "iot/alarms/#"

# 4. 查看创建的连接器
robust-ctl mqtt connector list

# 5. 测试连接器（发布测试消息）
robust-ctl mqtt publish \
  --username "test_user" \
  --password "test_pass" \
  --topic "iot/sensors/temp_001/data" \
  --qos 1 \
  --message '{"temperature": 25.5, "humidity": 60, "timestamp": 1640995200}'
```

## Elasticsearch 查询示例

连接器将数据写入 Elasticsearch 后，可以使用 Elasticsearch 的 REST API 或 Kibana 进行查询和分析。

### 1. 查询所有消息

```bash
curl -X GET "localhost:9200/mqtt_messages/_search?pretty" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": {
      "match_all": {}
    }
  }'
```

### 2. 按时间范围查询

```bash
curl -X GET "localhost:9200/mqtt_messages/_search?pretty" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": {
      "range": {
        "timestamp": {
          "gte": 1640995200,
          "lte": 1641081600
        }
      }
    }
  }'
```

### 3. 全文搜索

```bash
curl -X GET "localhost:9200/mqtt_messages/_search?pretty" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": {
      "match": {
        "data": "temperature"
      }
    }
  }'
```

## 性能优化建议

### 1. 批量写入
- 连接器默认使用批量写入（Bulk API）提高性能
- 建议根据消息量调整批量大小

### 2. 索引设置
- 根据数据量合理设置索引分片数
- 使用索引生命周期管理（ILM）自动管理旧数据
- 建议使用时间序列索引模式（如：`mqtt_messages-2023.12.15`）

### 3. 连接优化
- 对于高吞吐量场景，适当增加 `timeout_secs` 和 `max_retries`
- 使用连接池减少连接开销
- 考虑使用 Elasticsearch 集群提高可用性

### 4. 安全建议
- 生产环境建议启用 TLS 加密
- 使用 API Key 认证替代 Basic 认证
- 定期轮换 API 密钥
- 使用最小权限原则配置 Elasticsearch 用户

## 监控和故障排查

### 1. 查看连接器状态

```bash
robust-ctl mqtt connector list --connector-name "elasticsearch_connector_01"
```

### 2. 查看 Elasticsearch 索引状态

```bash
curl -X GET "localhost:9200/_cat/indices/mqtt_messages?v"
```

### 3. 常见问题

**问题 1：连接失败**
- 检查 Elasticsearch 服务是否运行
- 验证 URL 地址是否正确
- 检查网络连接和防火墙设置

**问题 2：认证失败**
- 验证用户名和密码是否正确
- 检查 API Key 是否有效
- 确认用户权限是否足够

**问题 3：写入超时**
- 增加 `timeout_secs` 配置
- 检查 Elasticsearch 服务负载
- 考虑增加 Elasticsearch 集群资源

## 总结

Elasticsearch 连接器是 RobustMQ 数据集成系统的强大组件，提供了与 Elasticsearch 搜索引擎的无缝集成能力。通过合理的配置和使用，可以实现：

- **实时搜索**：MQTT 消息实时写入 Elasticsearch，支持全文搜索和聚合分析
- **数据可视化**：结合 Kibana 实现数据的可视化展示和监控
- **灵活认证**：支持多种认证方式，满足不同的安全需求
- **高性能**：批量写入和连接优化确保高吞吐量场景下的性能
- **易于集成**：简单的配置即可将 MQTT 数据桥接到 Elasticsearch

该连接器充分利用了 Rust 语言的性能优势和 Elasticsearch 的搜索能力，是构建现代化 IoT 数据分析和监控系统的理想选择。

