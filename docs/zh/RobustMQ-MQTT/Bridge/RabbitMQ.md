# RabbitMQ 连接器

## 概述

RabbitMQ 连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息桥接到 RabbitMQ 消息队列系统。该连接器支持 AMQP 协议，提供可靠的消息传递能力，适合消息路由、异步处理、微服务通信和企业集成等场景。

## 配置说明

### 连接器配置

RabbitMQ 连接器使用 `RabbitMQConnectorConfig` 结构进行配置：

```rust
pub struct RabbitMQConnectorConfig {
    pub server: String,              // RabbitMQ 服务器地址
    pub port: u16,                   // RabbitMQ 服务器端口
    pub username: String,            // 用户名
    pub password: String,            // 密码
    pub virtual_host: String,        // 虚拟主机
    pub exchange: String,            // Exchange 名称
    pub routing_key: String,         // Routing Key
    pub delivery_mode: DeliveryMode, // 投递模式
    pub enable_tls: bool,            // 启用 TLS/SSL
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 说明 | 示例 |
|--------|------|------|------|------|
| `server` | String | 是 | RabbitMQ 服务器地址 | `localhost` 或 `rabbitmq.example.com` |
| `port` | u16 | 否 | RabbitMQ 服务器端口，默认 5672 | `5672`（非TLS）或 `5671`（TLS） |
| `username` | String | 是 | 用户名 | `guest` |
| `password` | String | 是 | 密码 | `guest` |
| `virtual_host` | String | 否 | 虚拟主机，默认 `/` | `/` 或 `/production` |
| `exchange` | String | 是 | Exchange 名称 | `mqtt_messages` |
| `routing_key` | String | 是 | Routing Key，用于消息路由 | `sensor.data` |
| `delivery_mode` | String | 否 | 投递模式，默认 `NonPersistent` | `NonPersistent` 或 `Persistent` |
| `enable_tls` | bool | 否 | 启用 TLS/SSL，默认 false | `true` 或 `false` |

### 投递模式说明

| 模式 | 说明 | AMQP 值 | 持久化 |
|------|------|---------|--------|
| `NonPersistent` | 非持久化消息，性能更高 | 1 | ❌ |
| `Persistent` | 持久化消息，重启后保留 | 2 | ✅ |

### 配置示例

#### JSON 配置格式

**基本配置（非 TLS）**：
```json
{
  "server": "localhost",
  "port": 5672,
  "username": "guest",
  "password": "guest",
  "virtual_host": "/",
  "exchange": "mqtt_messages",
  "routing_key": "sensor.data",
  "delivery_mode": "Persistent",
  "enable_tls": false
}
```

**TLS 配置**：
```json
{
  "server": "rabbitmq.example.com",
  "port": 5671,
  "username": "mqtt_user",
  "password": "secure_password",
  "virtual_host": "/production",
  "exchange": "mqtt_exchange",
  "routing_key": "iot.sensor.#",
  "delivery_mode": "Persistent",
  "enable_tls": true
}
```

#### 完整连接器配置
```json
{
  "cluster_name": "default",
  "connector_name": "rabbitmq_connector_01",
  "connector_type": "RabbitMQ",
  "config": "{\"server\": \"localhost\", \"port\": 5672, \"username\": \"guest\", \"password\": \"guest\", \"virtual_host\": \"/\", \"exchange\": \"mqtt_messages\", \"routing_key\": \"sensor.data\", \"delivery_mode\": \"Persistent\", \"enable_tls\": false}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## 消息格式

### 传输格式
RabbitMQ 连接器将 MQTT 消息转换为 JSON 格式后发送到 RabbitMQ Exchange，每个消息作为一条 AMQP 消息。

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

### AMQP 消息属性

| 属性 | 说明 |
|------|------|
| `delivery_mode` | 投递模式（1=非持久化，2=持久化） |
| `exchange` | 目标 Exchange |
| `routing_key` | 路由键，用于消息路由 |
| `content_type` | `application/json` |
| `body` | 消息内容（JSON 格式的 Record） |

## RabbitMQ 配置

### Exchange 类型

RabbitMQ 连接器可以配合不同类型的 Exchange 使用：

#### 1. Direct Exchange（直接交换）
```bash
# 创建 Direct Exchange
rabbitmqadmin declare exchange name=mqtt_direct type=direct durable=true

# 绑定队列
rabbitmqadmin declare binding source=mqtt_direct destination=sensor_queue routing_key=sensor.data
```

#### 2. Topic Exchange（主题交换）
```bash
# 创建 Topic Exchange
rabbitmqadmin declare exchange name=mqtt_topic type=topic durable=true

# 绑定队列（支持通配符）
rabbitmqadmin declare binding source=mqtt_topic destination=sensor_queue routing_key="sensor.*.data"
rabbitmqadmin declare binding source=mqtt_topic destination=all_queue routing_key="sensor.#"
```

#### 3. Fanout Exchange（广播交换）
```bash
# 创建 Fanout Exchange
rabbitmqadmin declare exchange name=mqtt_fanout type=fanout durable=true

# 绑定多个队列（无需 routing key）
rabbitmqadmin declare binding source=mqtt_fanout destination=queue1
rabbitmqadmin declare binding source=mqtt_fanout destination=queue2
```

### 队列配置示例

```bash
# 创建持久化队列
rabbitmqadmin declare queue name=mqtt_messages durable=true

# 创建具有 TTL 的队列
rabbitmqadmin declare queue name=mqtt_messages_ttl \
  durable=true \
  arguments='{"x-message-ttl": 86400000}'

# 创建具有最大长度的队列
rabbitmqadmin declare queue name=mqtt_messages_limited \
  durable=true \
  arguments='{"x-max-length": 100000}'
```

## 使用 robust-ctl 创建 RabbitMQ 连接器

### 基本语法

使用 `robust-ctl` 命令行工具可以方便地创建和管理 RabbitMQ 连接器：

```bash
robust-ctl mqtt connector create \
  --connector-name <连接器名称> \
  --connector-type <连接器类型> \
  --config <配置> \
  --topic-id <主题ID>
```

### 创建 RabbitMQ 连接器

#### 1. 基本创建命令

```bash
# 创建 RabbitMQ 连接器
robust-ctl mqtt connector create \
  --connector-name "rabbitmq_connector_01" \
  --connector-type "RabbitMQ" \
  --config '{"server": "localhost", "port": 5672, "username": "guest", "password": "guest", "virtual_host": "/", "exchange": "mqtt_messages", "routing_key": "sensor.data", "delivery_mode": "Persistent", "enable_tls": false}' \
  --topic-id "sensor/data"
```

#### 2. 参数说明

| 参数 | 说明 | 示例值 |
|------|------|--------|
| `--connector-name` | 连接器名称，必须唯一 | `rabbitmq_connector_01` |
| `--connector-type` | 连接器类型，固定为 `RabbitMQ` | `RabbitMQ` |
| `--config` | JSON 格式的配置信息 | `{"server": "localhost", ...}` |
| `--topic-id` | 要监听的 MQTT 主题 | `sensor/data` |

#### 3. 高级配置示例

**持久化消息配置**：
```bash
robust-ctl mqtt connector create \
  --connector-name "rabbitmq_persistent" \
  --connector-type "RabbitMQ" \
  --config '{"server": "rabbitmq.local", "port": 5672, "username": "mqtt_user", "password": "mqtt_pass", "virtual_host": "/production", "exchange": "mqtt_persistent", "routing_key": "iot.sensor.data", "delivery_mode": "Persistent", "enable_tls": false}' \
  --topic-id "iot/sensors/+/data"
```

**TLS 安全连接配置**：
```bash
robust-ctl mqtt connector create \
  --connector-name "rabbitmq_secure" \
  --connector-type "RabbitMQ" \
  --config '{"server": "rabbitmq.example.com", "port": 5671, "username": "admin", "password": "secure_pass", "virtual_host": "/secure", "exchange": "mqtt_secure", "routing_key": "secure.messages", "delivery_mode": "Persistent", "enable_tls": true}' \
  --topic-id "secure/#"
```

### 管理连接器

#### 1. 列出所有连接器
```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 列出指定名称的连接器
robust-ctl mqtt connector list --connector-name "rabbitmq_connector_01"
```

#### 2. 删除连接器
```bash
# 删除指定连接器
robust-ctl mqtt connector delete --connector-name "rabbitmq_connector_01"
```

### 完整操作示例

#### 场景：创建 IoT 消息路由系统

```bash
# 1. 创建传感器数据 RabbitMQ 连接器（Topic Exchange）
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_rabbitmq" \
  --connector-type "RabbitMQ" \
  --config '{"server": "rabbitmq.iot.local", "port": 5672, "username": "iot_user", "password": "iot_pass", "virtual_host": "/iot", "exchange": "iot_topic_exchange", "routing_key": "sensor.temperature.data", "delivery_mode": "Persistent", "enable_tls": false}' \
  --topic-id "iot/sensors/temperature/+"

# 2. 创建设备状态 RabbitMQ 连接器
robust-ctl mqtt connector create \
  --connector-name "device_status_rabbitmq" \
  --connector-type "RabbitMQ" \
  --config '{"server": "rabbitmq.iot.local", "port": 5672, "username": "iot_user", "password": "iot_pass", "virtual_host": "/iot", "exchange": "iot_topic_exchange", "routing_key": "device.status", "delivery_mode": "Persistent", "enable_tls": false}' \
  --topic-id "iot/devices/+/status"

# 3. 创建告警消息 RabbitMQ 连接器（高优先级）
robust-ctl mqtt connector create \
  --connector-name "alarm_rabbitmq" \
  --connector-type "RabbitMQ" \
  --config '{"server": "rabbitmq.iot.local", "port": 5672, "username": "iot_user", "password": "iot_pass", "virtual_host": "/iot", "exchange": "iot_alarms", "routing_key": "alarm.critical", "delivery_mode": "Persistent", "enable_tls": false}' \
  --topic-id "iot/alarms/#"

# 4. 查看创建的连接器
robust-ctl mqtt connector list

# 5. 测试连接器（发布测试消息）
robust-ctl mqtt publish \
  --username "test_user" \
  --password "test_pass" \
  --topic "iot/sensors/temperature/001" \
  --qos 1 \
  --message '{"temperature": 25.5, "humidity": 60, "timestamp": 1640995200}'
```

## RabbitMQ 部署示例

### Docker 部署

```bash
# 启动 RabbitMQ 服务（带管理界面）
docker run -d \
  --name rabbitmq \
  -p 5672:5672 \
  -p 15672:15672 \
  -e RABBITMQ_DEFAULT_USER=guest \
  -e RABBITMQ_DEFAULT_PASS=guest \
  rabbitmq:3-management

# 等待服务启动
sleep 10

# 创建 Exchange
docker exec rabbitmq rabbitmqadmin declare exchange name=mqtt_messages type=topic durable=true

# 创建队列
docker exec rabbitmq rabbitmqadmin declare queue name=sensor_data durable=true

# 绑定队列到 Exchange
docker exec rabbitmq rabbitmqadmin declare binding \
  source=mqtt_messages \
  destination=sensor_data \
  routing_key="sensor.*.data"
```

### Docker Compose 部署

```yaml
version: '3.8'

services:
  rabbitmq:
    image: rabbitmq:3-management
    container_name: rabbitmq
    ports:
      - "5672:5672"   # AMQP 端口
      - "15672:15672" # 管理界面端口
    environment:
      RABBITMQ_DEFAULT_USER: mqtt_user
      RABBITMQ_DEFAULT_PASS: mqtt_pass
      RABBITMQ_DEFAULT_VHOST: /iot
    volumes:
      - rabbitmq_data:/var/lib/rabbitmq
    healthcheck:
      test: ["CMD", "rabbitmq-diagnostics", "ping"]
      interval: 30s
      timeout: 10s
      retries: 5

  robustmq:
    image: robustmq/robustmq:latest
    container_name: robustmq
    ports:
      - "1883:1883"
      - "8883:8883"
    depends_on:
      - rabbitmq
    environment:
      - RABBITMQ_SERVER=rabbitmq
      - RABBITMQ_PORT=5672

volumes:
  rabbitmq_data:
```

### 访问管理界面

启动后访问 RabbitMQ 管理界面：
- URL: `http://localhost:15672`
- 用户名: `guest`（或配置的用户名）
- 密码: `guest`（或配置的密码）

## 消息路由示例

### 1. 单一队列路由

```bash
# 创建 Exchange
rabbitmqadmin declare exchange name=mqtt_direct type=direct durable=true

# 创建队列
rabbitmqadmin declare queue name=sensor_queue durable=true

# 绑定
rabbitmqadmin declare binding \
  source=mqtt_direct \
  destination=sensor_queue \
  routing_key=sensor.data
```

**连接器配置**：
```json
{
  "exchange": "mqtt_direct",
  "routing_key": "sensor.data"
}
```

### 2. 主题路由（通配符）

```bash
# 创建 Topic Exchange
rabbitmqadmin declare exchange name=mqtt_topic type=topic durable=true

# 创建多个队列
rabbitmqadmin declare queue name=temperature_queue durable=true
rabbitmqadmin declare queue name=humidity_queue durable=true
rabbitmqadmin declare queue name=all_sensors_queue durable=true

# 绑定（使用通配符）
rabbitmqadmin declare binding source=mqtt_topic destination=temperature_queue routing_key="sensor.temperature.*"
rabbitmqadmin declare binding source=mqtt_topic destination=humidity_queue routing_key="sensor.humidity.*"
rabbitmqadmin declare binding source=mqtt_topic destination=all_sensors_queue routing_key="sensor.#"
```

**连接器配置**：
```json
{
  "exchange": "mqtt_topic",
  "routing_key": "sensor.temperature.room1"
}
```

### 3. 广播路由

```bash
# 创建 Fanout Exchange
rabbitmqadmin declare exchange name=mqtt_fanout type=fanout durable=true

# 创建多个队列
rabbitmqadmin declare queue name=logger_queue durable=true
rabbitmqadmin declare queue name=analytics_queue durable=true
rabbitmqadmin declare queue name=storage_queue durable=true

# 绑定（无需 routing key）
rabbitmqadmin declare binding source=mqtt_fanout destination=logger_queue
rabbitmqadmin declare binding source=mqtt_fanout destination=analytics_queue
rabbitmqadmin declare binding source=mqtt_fanout destination=storage_queue
```

**连接器配置**：
```json
{
  "exchange": "mqtt_fanout",
  "routing_key": ""
}
```

## 性能优化

### 连接器优化

1. **选择合适的投递模式**
   - 高性能场景：使用 `NonPersistent`
   - 可靠性要求高：使用 `Persistent`

2. **连接管理**
   - 连接器自动管理连接和通道
   - 支持自动重连
   - 使用 Publisher Confirms 确保消息送达

### RabbitMQ 服务器优化

```ini
# rabbitmq.conf 配置示例

# 增加文件描述符限制
vm_memory_high_watermark.relative = 0.6

# 磁盘空间告警阈值
disk_free_limit.absolute = 2GB

# 消息持久化优化
queue_index_embed_msgs_below = 4096
```

### 队列优化建议

```bash
# 创建优化的队列
rabbitmqadmin declare queue name=optimized_queue \
  durable=true \
  arguments='{
    "x-max-length": 100000,
    "x-message-ttl": 86400000,
    "x-queue-mode": "lazy"
  }'
```

## 监控和故障排除

### 日志监控

连接器会输出详细的运行日志：

```
INFO  Connecting to RabbitMQ at localhost:5672 (exchange: mqtt_messages, routing_key: sensor.data)
INFO  Successfully connected to RabbitMQ
INFO  RabbitMQ connector thread exited successfully
ERROR Connector rabbitmq_connector_01 failed to write data to RabbitMQ exchange mqtt_messages, error: connection timeout
```

### 使用 RabbitMQ 管理工具

```bash
# 查看队列状态
rabbitmqctl list_queues name messages consumers

# 查看 Exchange 状态
rabbitmqctl list_exchanges name type

# 查看绑定关系
rabbitmqctl list_bindings

# 查看连接
rabbitmqctl list_connections
```

### 常见问题

#### 1. 连接失败
```bash
# 检查 RabbitMQ 服务状态
rabbitmqctl status

# 检查端口是否开放
telnet localhost 5672

# 查看 RabbitMQ 日志
tail -f /var/log/rabbitmq/rabbit@hostname.log
```

#### 2. 认证失败
```bash
# 创建用户
rabbitmqctl add_user mqtt_user mqtt_pass

# 设置权限
rabbitmqctl set_permissions -p / mqtt_user ".*" ".*" ".*"

# 设置用户标签
rabbitmqctl set_user_tags mqtt_user administrator
```

#### 3. 消息未到达队列
```bash
# 检查 Exchange 和 Queue 绑定
rabbitmqctl list_bindings

# 检查 routing key 是否正确
# Topic Exchange: 使用 . 分隔
# * 匹配一个单词
# # 匹配零个或多个单词
```

#### 4. 性能问题
```bash
# 查看队列堆积
rabbitmqctl list_queues name messages

# 查看内存使用
rabbitmqctl status | grep memory

# 启用 lazy queue 模式（减少内存使用）
rabbitmqadmin declare queue name=lazy_queue \
  durable=true \
  arguments='{"x-queue-mode": "lazy"}'
```

## 总结

RabbitMQ 连接器是 RobustMQ 数据集成系统的重要组件，提供了强大的消息路由和分发能力。通过合理的配置和使用，可以满足消息队列、异步处理、微服务通信和企业集成等多种业务需求。

该连接器充分利用了 RabbitMQ 的 AMQP 协议和路由机制，结合 Rust 语言的内存安全和零成本抽象优势，实现了高效、可靠的消息传输。支持多种 Exchange 类型（Direct、Topic、Fanout）和投递模式（持久化/非持久化），是构建现代化消息驱动架构和企业集成平台的重要工具。

### 关键特性

✅ **AMQP 协议支持**：完全兼容 AMQP 0-9-1 协议
✅ **灵活的路由机制**：支持 Direct、Topic、Fanout Exchange
✅ **消息持久化**：可选的消息持久化保证数据安全
✅ **TLS/SSL 支持**：支持加密传输保障通信安全
✅ **Publisher Confirms**：确认机制保证消息送达
✅ **虚拟主机隔离**：支持多租户环境
