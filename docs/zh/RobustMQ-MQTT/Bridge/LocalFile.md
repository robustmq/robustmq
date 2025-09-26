# 本地文件连接器

## 概述

本地文件连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息写入本地文件系统。该连接器简单可靠，适合数据备份、日志记录和离线分析等场景。

## 配置说明

### 连接器配置

本地文件连接器使用 `LocalFileConnectorConfig` 结构进行配置：

```rust
pub struct LocalFileConnectorConfig {
    pub local_file_path: String,  // 本地文件路径
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 说明 | 示例 |
|--------|------|------|------|------|
| `local_file_path` | String | 是 | 本地文件的完整路径 | `/var/log/mqtt_messages.log` |

### 配置示例

#### JSON 配置格式
```json
{
  "local_file_path": "/var/log/mqtt_messages.log"
}
```

#### 完整连接器配置
```json
{
  "cluster_name": "my_cluster",
  "connector_name": "file_connector",
  "connector_type": "LocalFile",
  "config": "{\"local_file_path\": \"/var/log/mqtt_messages.log\"}",
  "topic_id": "sensor/data",
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
