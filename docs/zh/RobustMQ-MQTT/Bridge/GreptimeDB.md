# GreptimeDB 连接器

## 概述

GreptimeDB 连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息桥接到 GreptimeDB 时序数据库系统。该连接器支持高并发的时序数据写入，适合 IoT 数据存储、实时监控、时间序列分析和数据可视化等场景。

## 配置说明

### 连接器配置

GreptimeDB 连接器使用 `GreptimeDBConnectorConfig` 结构进行配置：

```rust
pub struct GreptimeDBConnectorConfig {
    pub server_addr: String,        // GreptimeDB 服务器地址
    pub database: String,           // 数据库名称
    pub user: String,               // 用户名
    pub password: String,           // 密码
    pub precision: TimePrecision,   // 时间精度
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 说明 | 示例 |
|--------|------|------|------|------|
| `server_addr` | String | 是 | GreptimeDB 服务器地址 | `localhost:4000` |
| `database` | String | 是 | 数据库名称 | `public` |
| `user` | String | 是 | 用户名 | `greptime_user` |
| `password` | String | 是 | 密码 | `greptime_pwd` |
| `precision` | String | 否 | 时间精度，默认为秒 | `s`、`ms`、`us`、`ns` |

### 时间精度说明

| 精度值 | 说明 | 示例 |
|--------|------|------|
| `s` | 秒级精度 | `1640995200` |
| `ms` | 毫秒级精度 | `1640995200000` |
| `us` | 微秒级精度 | `1640995200000000` |
| `ns` | 纳秒级精度 | `1640995200000000000` |

### 配置示例

#### JSON 配置格式
```json
{
  "server_addr": "localhost:4000",
  "database": "public",
  "user": "greptime_user",
  "password": "greptime_pwd",
  "precision": "s"
}
```

#### 完整连接器配置
```json
{
  "cluster_name": "default",
  "connector_name": "greptimedb_connector_01",
  "connector_type": "GreptimeDB",
  "config": "{\"server_addr\": \"localhost:4000\", \"database\": \"public\", \"user\": \"greptime_user\", \"password\": \"greptime_pwd\", \"precision\": \"s\"}",
  "topic_id": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## 消息格式

### 传输格式
GreptimeDB 连接器将 MQTT 消息转换为 InfluxDB Line Protocol 格式后发送到 GreptimeDB，支持时序数据的结构化存储。

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

### InfluxDB Line Protocol 格式

GreptimeDB 连接器将消息转换为以下格式：

```
measurement,tag1=value1,tag2=value2 field1=value1,field2=value2 timestamp
```

#### 转换示例

**输入消息**：
```json
{
  "key": "sensor_001",
  "header": [{"name": "device_type", "value": "temperature"}],
  "data": {"temperature": 25.5, "humidity": 60},
  "tags": ["sensor", "iot"],
  "timestamp": 1640995200,
  "crc_num": 1234567890,
  "offset": 100
}
```

**转换后的 Line Protocol**：
```
sensor_001,device_type=temperature data="{\"temperature\":25.5,\"humidity\":60}",tags="[\"sensor\",\"iot\"]",crc_num=1234567890i,offset=100i 1640995200
```

### 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `measurement` | String | 测量名称（来自 `key` 字段） |
| `tags` | String | 标签（来自 `header` 和 `tags` 字段） |
| `fields` | String | 字段（包含 `data`、`tags`、`crc_num`、`offset`） |
| `timestamp` | Number | 时间戳 |

## 使用 robust-ctl 创建 GreptimeDB 连接器

### 基本语法

使用 `robust-ctl` 命令行工具可以方便地创建和管理 GreptimeDB 连接器：

```bash
robust-ctl mqtt connector create \
  --connector-name <连接器名称> \
  --connector-type <连接器类型> \
  --config <配置> \
  --topic-id <主题ID>
```

### 创建 GreptimeDB 连接器

#### 1. 基本创建命令

```bash
# 创建 GreptimeDB 连接器
robust-ctl mqtt connector create \
  --connector-name "greptimedb_connector_01" \
  --connector-type "GreptimeDB" \
  --config '{"server_addr": "localhost:4000", "database": "public", "user": "greptime_user", "password": "greptime_pwd", "precision": "s"}' \
  --topic-id "sensor/data"
```

#### 2. 参数说明

| 参数 | 说明 | 示例值 |
|------|------|--------|
| `--connector-name` | 连接器名称，必须唯一 | `greptimedb_connector_01` |
| `--connector-type` | 连接器类型，固定为 `GreptimeDB` | `GreptimeDB` |
| `--config` | JSON 格式的配置信息 | `{"server_addr": "localhost:4000", "database": "public", "user": "greptime_user", "password": "greptime_pwd", "precision": "s"}` |
| `--topic-id` | 要监听的 MQTT 主题 | `sensor/data` |

#### 3. 配置示例

```bash
# 创建传感器数据 GreptimeDB 连接器
robust-ctl mqtt connector create \
  --connector-name "sensor_greptimedb" \
  --connector-type "GreptimeDB" \
  --config '{"server_addr": "greptimedb:4000", "database": "iot_data", "user": "iot_user", "password": "iot_pass", "precision": "ms"}' \
  --topic-id "sensors/+/data"
```

### 管理连接器

#### 1. 列出所有连接器
```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 列出指定名称的连接器
robust-ctl mqtt connector list --connector-name "greptimedb_connector_01"
```

#### 2. 删除连接器
```bash
# 删除指定连接器
robust-ctl mqtt connector delete --connector-name "greptimedb_connector_01"
```

### 完整操作示例

#### 场景：创建 IoT 时序数据存储系统

```bash
# 1. 创建传感器数据 GreptimeDB 连接器
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_greptimedb" \
  --connector-type "GreptimeDB" \
  --config '{"server_addr": "greptimedb:4000", "database": "iot_sensors", "user": "sensor_user", "password": "sensor_pass", "precision": "ms"}' \
  --topic-id "iot/sensors/+/data"

# 2. 创建设备状态 GreptimeDB 连接器
robust-ctl mqtt connector create \
  --connector-name "device_status_greptimedb" \
  --connector-type "GreptimeDB" \
  --config '{"server_addr": "greptimedb:4000", "database": "device_status", "user": "device_user", "password": "device_pass", "precision": "s"}' \
  --topic-id "iot/devices/+/status"

# 3. 创建告警消息 GreptimeDB 连接器
robust-ctl mqtt connector create \
  --connector-name "alarm_greptimedb" \
  --connector-type "GreptimeDB" \
  --config '{"server_addr": "greptimedb:4000", "database": "alarms", "user": "alarm_user", "password": "alarm_pass", "precision": "s"}' \
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

### GreptimeDB 部署示例

#### Docker 部署
```bash
# 启动 GreptimeDB 服务
docker run -p 4000-4004:4000-4004 \
  -p 4242:4242 \
  -v "$(pwd)/greptimedb:/tmp/greptimedb" \
  --name greptime --rm \
  greptime/greptimedb standalone start \
  --http-addr 0.0.0.0:4000 \
  --rpc-addr 0.0.0.0:4001 \
  --mysql-addr 0.0.0.0:4002 \
  --user-provider=static_user_provider:cmd:greptime_user=greptime_pwd
```

#### 连接配置
- **HTTP 地址**: `http://localhost:4000`
- **用户名**: `greptime_user`
- **密码**: `greptime_pwd`
- **默认数据库**: `public`


## 总结

GreptimeDB 连接器是 RobustMQ 数据集成系统的重要组件，提供了高效的时序数据存储能力。通过合理的配置和使用，可以满足 IoT 数据存储、实时监控、时间序列分析和数据可视化等多种业务需求。

该连接器充分利用了 GreptimeDB 的时序数据库特性，结合 Rust 语言的内存安全和零成本抽象优势，实现了高性能、高可靠性的时序数据存储，是构建现代化 IoT 数据平台和实时分析系统的重要工具。
