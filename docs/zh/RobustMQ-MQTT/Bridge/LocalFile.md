# 本地文件连接器

## 概述

本地文件连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息写入本地文件系统。该连接器简单可靠，适合数据备份、日志记录和离线分析等场景。

## 配置说明

### 连接器配置

本地文件连接器使用 `LocalFileConnectorConfig` 结构进行配置：

```rust
pub struct LocalFileConnectorConfig {
    pub local_file_path: String,       // 本地文件路径
    pub rotation_strategy: RotationStrategy,  // 文件滚动策略
    pub max_size_gb: u64,              // 最大文件大小（GB）
}

pub enum RotationStrategy {
    None,    // 不滚动
    Size,    // 按大小滚动
    Hourly,  // 按小时滚动
    Daily,   // 按天滚动
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 示例 |
|--------|------|------|--------|------|------|
| `local_file_path` | String | 是 | - | 本地文件的完整路径 | `/var/log/mqtt_messages.log` |
| `rotation_strategy` | String | 否 | `none` | 文件滚动策略：`none`（不滚动）、`size`（按大小滚动）、`hourly`（按小时滚动）、`daily`（按天滚动） | `daily` |
| `max_size_gb` | Number | 否 | `1` | 文件最大大小（GB），仅在 `rotation_strategy` 为 `size` 时生效，范围：1-10 | `5` |

### 配置示例

#### JSON 配置格式

**基本配置（无滚动）**
```json
{
  "local_file_path": "/var/log/mqtt_messages.log"
}
```

**按大小滚动**
```json
{
  "local_file_path": "/var/log/mqtt_messages.log",
  "rotation_strategy": "size",
  "max_size_gb": 5
}
```

**按小时滚动**
```json
{
  "local_file_path": "/var/log/mqtt_messages.log",
  "rotation_strategy": "hourly"
}
```

**按天滚动**
```json
{
  "local_file_path": "/var/log/mqtt_messages.log",
  "rotation_strategy": "daily"
}
```

#### 完整连接器配置

**无滚动配置**
```json
{
  "cluster_name": "my_cluster",
  "connector_name": "file_connector",
  "connector_type": "LocalFile",
  "config": "{\"local_file_path\": \"/var/log/mqtt_messages.log\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": 1,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

**带滚动策略配置**
```json
{
  "cluster_name": "my_cluster",
  "connector_name": "file_connector",
  "connector_type": "LocalFile",
  "config": "{\"local_file_path\": \"/var/log/mqtt_messages.log\", \"rotation_strategy\": \"daily\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": 1,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## 消息格式

### 存储格式
本地文件连接器将 MQTT 消息转换为 JSON 格式存储，每个消息占一行。

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

## 文件滚动策略

### 功能说明

本地文件连接器支持自动文件滚动（rotation）功能，可以根据文件大小或时间自动创建新文件，避免单个文件过大。

### 滚动策略类型

#### 1. None（不滚动）
- 默认策略，所有数据写入同一个文件
- 适合消息量较小的场景

#### 2. Size（按大小滚动）
- 当文件达到指定大小时自动滚动
- 大小范围：1GB - 10GB
- 滚动后的文件名格式：`原文件名_YYYYMMdd_HHMMSS.扩展名`
- 示例：`mqtt_messages_20231215_143025.log`

#### 3. Hourly（按小时滚动）
- 每小时自动创建新文件
- 滚动后的文件名格式：`原文件名_YYYYMMdd_HH.扩展名`
- 示例：`mqtt_messages_20231215_14.log`

#### 4. Daily（按天滚动）
- 每天自动创建新文件
- 滚动后的文件名格式：`原文件名_YYYYMMdd.扩展名`
- 示例：`mqtt_messages_20231215.log`

### 使用示例

#### 按大小滚动（5GB）
```bash
robust-ctl mqtt connector create \
  --connector-name "file_size_rotation" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/mqtt.log", "rotation_strategy": "size", "max_size_gb": 5}' \
  --topic-id "sensor/data"
```

#### 按小时滚动
```bash
robust-ctl mqtt connector create \
  --connector-name "file_hourly_rotation" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/mqtt.log", "rotation_strategy": "hourly"}' \
  --topic-id "sensor/data"
```

#### 按天滚动
```bash
robust-ctl mqtt connector create \
  --connector-name "file_daily_rotation" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/mqtt.log", "rotation_strategy": "daily"}' \
  --topic-id "sensor/data"
```

### 滚动行为

1. **文件检查**：在写入数据前检查是否需要滚动
2. **文件重命名**：当前文件被重命名为带时间戳的文件
3. **创建新文件**：使用原始文件名创建新文件
4. **继续写入**：新数据写入新文件

### 注意事项

- 滚动操作是原子性的，不会丢失数据
- 旧文件需要手动清理或通过日志轮转工具管理
- 按大小滚动时，实际文件大小可能略大于设置值（因为是批量写入后检查）
- 按时间滚动时，精确度为秒级

## 使用 robust-ctl 创建文件连接器

### 基本语法

使用 `robust-ctl` 命令行工具可以方便地创建和管理本地文件连接器：

```bash
robust-ctl mqtt connector create \
  --connector-name <连接器名称> \
  --connector-type <连接器类型> \
  --config <配置> \
  --topic-id <主题ID>
```

### 创建本地文件连接器

#### 1. 基本创建命令

```bash
# 创建本地文件连接器
robust-ctl mqtt connector create \
  --connector-name "file_connector_01" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/mqtt_messages.log"}' \
  --topic-id "sensor/data"
```

#### 2. 参数说明

| 参数 | 说明 | 示例值 |
|------|------|--------|
| `--connector-name` | 连接器名称，必须唯一 | `file_connector_01` |
| `--connector-type` | 连接器类型，固定为 `LocalFile` | `LocalFile` |
| `--config` | JSON 格式的配置信息 | `{"local_file_path": "/path/to/file.log"}` |
| `--topic-id` | 要监听的 MQTT 主题 | `sensor/data` |

#### 3. 配置示例

```bash
# 创建传感器数据记录连接器
robust-ctl mqtt connector create \
  --connector-name "sensor_logger" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/sensor_data.log"}' \
  --topic-id "sensors/+/data"
```

### 管理连接器

#### 1. 列出所有连接器
```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 列出指定名称的连接器
robust-ctl mqtt connector list --connector-name "file_connector_01"
```

#### 2. 删除连接器
```bash
# 删除指定连接器
robust-ctl mqtt connector delete --connector-name "file_connector_01"
```

### 完整操作示例

#### 场景：创建 IoT 传感器数据记录系统

```bash
# 1. 创建传感器数据连接器
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_logger" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/iot_sensors.log"}' \
  --topic-id "iot/sensors/+/data"

# 2. 创建设备状态连接器
robust-ctl mqtt connector create \
  --connector-name "device_status_logger" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/device_status.log"}' \
  --topic-id "iot/devices/+/status"

# 3. 创建告警消息连接器
robust-ctl mqtt connector create \
  --connector-name "alarm_logger" \
  --connector-type "LocalFile" \
  --config '{"local_file_path": "/var/log/robustmq/alarms.log"}' \
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


## 总结

本地文件连接器是 RobustMQ 数据集成系统的重要组成部分，提供了简单可靠的消息持久化能力。通过合理的配置和使用，可以满足数据备份、日志记录和离线分析等多种业务需求。

该连接器充分发挥了 Rust 语言的优势，在保证高性能的同时，提供了内存安全和并发安全的特性，是构建可靠 IoT 数据管道的重要工具。
