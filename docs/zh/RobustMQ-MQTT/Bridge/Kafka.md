# Kafka 连接器

## 概述

Kafka 连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息桥接到 Apache Kafka 消息队列系统。该连接器支持高吞吐量的消息传输，适合实时数据流处理、事件驱动架构和大数据分析等场景。

## 配置说明

### 连接器配置

Kafka 连接器使用 `KafkaConnectorConfig` 结构进行配置：

```rust
pub struct KafkaConnectorConfig {
    pub bootstrap_servers: String,  // Kafka 服务器地址
    pub topic: String,              // Kafka 主题名称
    pub key: String,                // 消息键值
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 说明 | 示例 |
|--------|------|------|------|------|
| `bootstrap_servers` | String | 是 | Kafka 服务器地址列表 | `localhost:9092` 或 `kafka1:9092,kafka2:9092` |
| `topic` | String | 是 | Kafka 主题名称 | `mqtt_messages` |
| `key` | String | 是 | 消息键值，用于分区路由 | `sensor_data` |

### 配置示例

#### JSON 配置格式
```json
{
  "bootstrap_servers": "localhost:9092",
  "topic": "mqtt_messages",
  "key": "sensor_data"
}
```

#### 完整连接器配置
```json
{
  "cluster_name": "default",
  "connector_name": "kafka_connector_01",
  "connector_type": "Kafka",
  "config": "{\"bootstrap_servers\": \"localhost:9092\", \"topic\": \"mqtt_messages\", \"key\": \"sensor_data\"}",
  "topic_id": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## 消息格式

### 传输格式
Kafka 连接器将 MQTT 消息转换为 JSON 格式后发送到 Kafka 主题，每个消息作为一条 Kafka 记录。

### 消息结构

```json
{
  "topic": "sensor/temperature",
  "qos": 1,
  "retain": false,
  "payload": "eyJ0ZW1wZXJhdHVyZSI6IDI1LjUsICJodW1pZGl0eSI6IDYwfQ==",
  "client_id": "sensor_001",
  "username": "sensor_user",
  "timestamp": 1640995200,
  "message_id": 12345,
  "header": [
    {
      "key": "content-type",
      "value": "application/json"
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

| 字段 | 类型 | 说明 |
|------|------|------|
| `topic` | String | MQTT 主题名称 |
| `qos` | Number | QoS 级别 (0, 1, 2) |
| `retain` | Boolean | 保留标志 |
| `payload` | String | 消息载荷（Base64 编码） |
| `client_id` | String | 客户端 ID |
| `username` | String | 用户名 |
| `timestamp` | Number | 消息时间戳（秒） |
| `message_id` | Number | 消息 ID |
| `header` | Array | 消息头信息数组 |
| `key` | String | 消息键值 |
| `data` | String | 消息内容（Base64 编码） |
| `tags` | Array | 消息标签数组 |
| `timestamp` | Number | 消息时间戳（秒） |
| `crc_num` | Number | 消息 CRC 校验值 |

## 使用 robust-ctl 创建 Kafka 连接器

### 基本语法

使用 `robust-ctl` 命令行工具可以方便地创建和管理 Kafka 连接器：

```bash
robust-ctl mqtt connector create \
  --connector-name <连接器名称> \
  --connector-type <连接器类型> \
  --config <配置> \
  --topic-id <主题ID>
```

### 创建 Kafka 连接器

#### 1. 基本创建命令

```bash
# 创建 Kafka 连接器
robust-ctl mqtt connector create \
  --connector-name "kafka_connector_01" \
  --connector-type "Kafka" \
  --config '{"bootstrap_servers": "localhost:9092", "topic": "mqtt_messages", "key": "sensor_data"}' \
  --topic-id "sensor/data"
```

#### 2. 参数说明

| 参数 | 说明 | 示例值 |
|------|------|--------|
| `--connector-name` | 连接器名称，必须唯一 | `kafka_connector_01` |
| `--connector-type` | 连接器类型，固定为 `Kafka` | `Kafka` |
| `--config` | JSON 格式的配置信息 | `{"bootstrap_servers": "localhost:9092", "topic": "mqtt_messages", "key": "sensor_data"}` |
| `--topic-id` | 要监听的 MQTT 主题 | `sensor/data` |

#### 3. 配置示例

```bash
# 创建传感器数据 Kafka 连接器
robust-ctl mqtt connector create \
  --connector-name "sensor_kafka_logger" \
  --connector-type "Kafka" \
  --config '{"bootstrap_servers": "kafka1:9092,kafka2:9092", "topic": "sensor_data", "key": "sensor_key"}' \
  --topic-id "sensors/+/data"
```

### 管理连接器

#### 1. 列出所有连接器
```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 列出指定名称的连接器
robust-ctl mqtt connector list --connector-name "kafka_connector_01"
```

#### 2. 删除连接器
```bash
# 删除指定连接器
robust-ctl mqtt connector delete --connector-name "kafka_connector_01"
```

### 完整操作示例

#### 场景：创建 IoT 数据流处理系统

```bash
# 1. 创建传感器数据 Kafka 连接器
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_kafka" \
  --connector-type "Kafka" \
  --config '{"bootstrap_servers": "kafka1:9092,kafka2:9092", "topic": "iot_sensors", "key": "sensor_key"}' \
  --topic-id "iot/sensors/+/data"

# 2. 创建设备状态 Kafka 连接器
robust-ctl mqtt connector create \
  --connector-name "device_status_kafka" \
  --connector-type "Kafka" \
  --config '{"bootstrap_servers": "kafka1:9092,kafka2:9092", "topic": "device_status", "key": "device_key"}' \
  --topic-id "iot/devices/+/status"

# 3. 创建告警消息 Kafka 连接器
robust-ctl mqtt connector create \
  --connector-name "alarm_kafka" \
  --connector-type "Kafka" \
  --config '{"bootstrap_servers": "kafka1:9092,kafka2:9092", "topic": "alarms", "key": "alarm_key"}' \
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

Kafka 连接器是 RobustMQ 数据集成系统的重要组件，提供了高性能的消息队列桥接能力。通过合理的配置和使用，可以满足实时数据流处理、事件驱动架构和大数据分析等多种业务需求。

该连接器充分利用了 Kafka 的高吞吐量特性，结合 Rust 语言的内存安全和零成本抽象优势，实现了高效、可靠的消息传输，是构建现代化数据管道和流处理系统的重要工具。
