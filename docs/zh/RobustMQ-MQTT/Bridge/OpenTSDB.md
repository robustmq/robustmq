# OpenTSDB 连接器

## 概述

OpenTSDB 连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息桥接到 OpenTSDB 时序数据库。该连接器自动将 MQTT 消息 payload 转换为 OpenTSDB 数据点格式（metric + tags + value + timestamp），适合 IoT 设备监控、传感器数据采集、工业能耗管理等时序数据场景。

## 配置说明

### 连接器配置

OpenTSDB 连接器使用 `OpenTSDBConnectorConfig` 结构进行配置：

```rust
pub struct OpenTSDBConnectorConfig {
    pub server: String,          // OpenTSDB 服务器地址
    pub metric_field: String,    // payload 中表示 metric 的字段名
    pub value_field: String,     // payload 中表示 value 的字段名
    pub tags_fields: Vec<String>,// payload 中作为 tags 的字段列表
    pub timeout_secs: u64,       // 请求超时时间（秒）
    pub max_retries: u32,        // 最大重试次数
    pub summary: bool,           // 请求 summary 响应
    pub details: bool,           // 请求 details 响应
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 示例 |
|--------|------|------|--------|------|------|
| `server` | String | 是 | - | OpenTSDB 服务器地址，必须以 `http://` 或 `https://` 开头，长度不超过 512 字符 | `http://localhost:4242` |
| `metric_field` | String | 否 | `metric` | payload JSON 中表示 metric 名称的字段 | `metric` |
| `value_field` | String | 否 | `value` | payload JSON 中表示数值的字段 | `value` |
| `tags_fields` | Array | 否 | `[]` | payload 中作为 tags 的字段列表；为空时自动读取 `tags` 对象 | `["host", "region"]` |
| `timeout_secs` | Number | 否 | `30` | 请求超时时间（秒），范围：1-300 | `60` |
| `max_retries` | Number | 否 | `3` | 最大重试次数，不超过 10 | `5` |
| `summary` | Boolean | 否 | `false` | 是否在 `/api/put` 请求中附加 `?summary` 参数 | `true` |
| `details` | Boolean | 否 | `false` | 是否在 `/api/put` 请求中附加 `?details` 参数 | `true` |

### 数据映射

连接器从 MQTT 消息 payload（JSON 格式）中提取字段，转换为 OpenTSDB 数据点。

#### 方式一：使用 tags 对象（默认）

当 `tags_fields` 为空时，连接器从 payload 中读取 `tags` 对象：

```json
{
  "metric": "cpu",
  "tags": {
    "host": "serverA",
    "region": "us-east"
  },
  "value": 85.5,
  "timestamp": 1640995200
}
```

转换为 OpenTSDB 数据点：

```json
{
  "metric": "cpu",
  "timestamp": 1640995200,
  "value": 85.5,
  "tags": {
    "host": "serverA",
    "region": "us-east"
  }
}
```

#### 方式二：使用 tags_fields 指定字段

当配置 `tags_fields: ["host", "region"]` 时，连接器从 payload 顶层字段中提取 tags：

```json
{
  "metric": "temperature",
  "host": "sensor_001",
  "region": "factory_A",
  "value": 25.5
}
```

转换为：

```json
{
  "metric": "temperature",
  "timestamp": 1640995200,
  "value": 25.5,
  "tags": {
    "host": "sensor_001",
    "region": "factory_A"
  }
}
```

### 配置示例

#### JSON 配置格式

**基本配置**
```json
{
  "server": "http://localhost:4242"
}
```

**自定义字段映射**
```json
{
  "server": "http://localhost:4242",
  "metric_field": "metric",
  "value_field": "value",
  "tags_fields": ["host", "region", "device"],
  "timeout_secs": 60,
  "max_retries": 5
}
```

**启用响应摘要**
```json
{
  "server": "http://opentsdb.example.com:4242",
  "summary": true,
  "details": true,
  "timeout_secs": 30
}
```

## 使用 robust-ctl 创建 OpenTSDB 连接器

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
  --connector-name "opentsdb_connector_01" \
  --connector-type "opentsdb" \
  --config '{"server": "http://localhost:4242"}' \
  --topic-id "sensor/data"
```

#### 2. 自定义字段映射

```bash
robust-ctl mqtt connector create \
  --connector-name "opentsdb_custom" \
  --connector-type "opentsdb" \
  --config '{"server": "http://localhost:4242", "metric_field": "metric", "value_field": "value", "tags_fields": ["host", "region"], "timeout_secs": 60}' \
  --topic-id "iot/sensors/temperature"
```

### 管理连接器

```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 查看指定连接器
robust-ctl mqtt connector list --connector-name "opentsdb_connector_01"

# 删除连接器
robust-ctl mqtt connector delete --connector-name "opentsdb_connector_01"
```

### 完整操作示例

#### 场景：工业设备监控

```bash
# 1. 创建 CPU 监控连接器
robust-ctl mqtt connector create \
  --connector-name "cpu_monitor" \
  --connector-type "opentsdb" \
  --config '{"server": "http://localhost:4242", "tags_fields": ["host"]}' \
  --topic-id "monitor/cpu"

# 2. 创建温度传感器连接器
robust-ctl mqtt connector create \
  --connector-name "temp_sensor" \
  --connector-type "opentsdb" \
  --config '{"server": "http://localhost:4242", "tags_fields": ["device", "location"]}' \
  --topic-id "sensor/temperature"

# 3. 查看连接器
robust-ctl mqtt connector list

# 4. 发布测试消息
robust-ctl mqtt publish \
  --username "test_user" \
  --password "test_pass" \
  --topic "monitor/cpu" \
  --qos 1 \
  --message '{"metric": "cpu.usage", "host": "serverA", "value": 85.5}'
```

## 查询 OpenTSDB 数据

连接器将数据写入后，可通过 OpenTSDB HTTP API 查询：

```bash
curl -X POST http://localhost:4242/api/query -H "Content-Type: application/json" -d '{
  "start": "1h-ago",
  "queries": [
    {
      "aggregator": "last",
      "metric": "cpu.usage",
      "tags": {
        "host": "*"
      }
    }
  ]
}'
```

## 性能优化建议

### 1. 批量写入
- 连接器默认支持批量写入，将多条消息合并为一次 `/api/put` 请求
- 高吞吐量场景下可有效减少 HTTP 请求次数

### 2. 超时设置
- 根据 OpenTSDB 服务器性能合理设置 `timeout_secs`
- 对于远程部署适当增加超时时间

### 3. 数据建模
- metric 名称建议使用点号分隔的层级结构，如 `cpu.usage`、`disk.io.read`
- tags 用于标识数据维度，如 host、region、device
- 避免 tag 值的基数过高（如 UUID），以免影响 OpenTSDB 性能

## 监控和故障排查

### 1. 查看连接器状态

```bash
robust-ctl mqtt connector list --connector-name "opentsdb_connector_01"
```

### 2. 常见问题

**问题 1：连接失败**
- 检查 OpenTSDB 服务是否正常运行
- 验证服务器地址和端口是否正确
- 检查网络连接和防火墙设置

**问题 2：数据写入失败**
- 确认 payload 格式正确，包含必需的 metric 和 value 字段
- 检查 metric 命名是否符合 OpenTSDB 规范
- 启用 `summary` 和 `details` 查看详细错误信息

**问题 3：写入超时**
- 增加 `timeout_secs` 配置
- 检查 OpenTSDB 服务负载
- 考虑增加 OpenTSDB 集群资源

## 总结

OpenTSDB 连接器将 MQTT 消息自动转换为 OpenTSDB 时序数据点，提供了：

- **自动格式转换**：MQTT JSON payload 自动映射为 metric + tags + value + timestamp
- **灵活字段映射**：支持 tags 对象和 tags_fields 两种方式
- **批量写入**：多条消息合并写入，提升吞吐量
- **适配 IoT 场景**：设备监控、传感器采集、工业数据分析
