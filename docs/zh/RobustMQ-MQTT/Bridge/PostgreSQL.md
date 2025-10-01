# PostgreSQL 连接器

## 概述

PostgreSQL 连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息桥接到 PostgreSQL 关系型数据库系统。该连接器支持高性能的数据写入，适合 IoT 数据存储、历史数据分析、数据持久化和业务数据集成等场景。

## 配置说明

### 连接器配置

PostgreSQL 连接器使用 `PostgresConnectorConfig` 结构进行配置：

```rust
pub struct PostgresConnectorConfig {
    pub host: String,                    // PostgreSQL 服务器地址
    pub port: u16,                       // PostgreSQL 服务器端口
    pub database: String,                // 数据库名称
    pub username: String,                // 用户名
    pub password: String,                // 密码
    pub table: String,                   // 目标表名
    pub pool_size: Option<u32>,          // 连接池大小
    pub enable_batch_insert: Option<bool>, // 启用批量插入
    pub enable_upsert: Option<bool>,     // 启用 UPSERT 操作
    pub conflict_columns: Option<String>, // 冲突列定义
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 说明 | 示例 |
|--------|------|------|------|------|
| `host` | String | 是 | PostgreSQL 服务器地址 | `localhost` 或 `192.168.1.100` |
| `port` | u16 | 是 | PostgreSQL 服务器端口 | `5432` |
| `database` | String | 是 | 数据库名称 | `mqtt_data` |
| `username` | String | 是 | 数据库用户名 | `postgres` |
| `password` | String | 是 | 数据库密码 | `password123` |
| `table` | String | 是 | 目标表名 | `mqtt_messages` |
| `pool_size` | u32 | 否 | 连接池大小，默认为 10 | `20` |
| `enable_batch_insert` | bool | 否 | 启用批量插入，默认为 false | `true` |
| `enable_upsert` | bool | 否 | 启用 UPSERT 操作，默认为 false | `true` |
| `conflict_columns` | String | 否 | 冲突列定义，默认为 "client_id, topic" | `"client_id, topic"` |

### 配置示例

#### JSON 配置格式
```json
{
  "host": "localhost",
  "port": 5432,
  "database": "mqtt_data",
  "username": "postgres",
  "password": "password123",
  "table": "mqtt_messages",
  "pool_size": 20,
  "enable_batch_insert": true,
  "enable_upsert": false,
  "conflict_columns": "client_id, topic"
}
```

#### 完整连接器配置
```json
{
  "cluster_name": "default",
  "connector_name": "postgres_connector_01",
  "connector_type": "Postgres",
  "config": "{\"host\": \"localhost\", \"port\": 5432, \"database\": \"mqtt_data\", \"username\": \"postgres\", \"password\": \"password123\", \"table\": \"mqtt_messages\", \"enable_batch_insert\": true}",
  "topic_id": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## 数据库表结构

### 表结构定义

PostgreSQL 连接器要求目标表具有以下结构：

```sql
CREATE TABLE mqtt_messages (
    client_id VARCHAR(255) NOT NULL,     -- MQTT 客户端 ID
    topic VARCHAR(500) NOT NULL,         -- MQTT 主题
    timestamp BIGINT NOT NULL,           -- 消息时间戳（秒）
    payload TEXT,                        -- 消息载荷（字符串格式）
    data BYTEA,                          -- 消息原始数据（二进制格式）
    PRIMARY KEY (client_id, topic, timestamp)
);
```

### 字段说明

| 字段名 | 数据类型 | 说明 | 示例 |
|--------|----------|------|------|
| `client_id` | VARCHAR(255) | MQTT 客户端 ID | `sensor_001` |
| `topic` | VARCHAR(500) | MQTT 主题名称 | `sensor/temperature` |
| `timestamp` | BIGINT | 消息时间戳（秒级） | `1640995200` |
| `payload` | TEXT | 消息载荷（UTF-8 字符串） | `{"temperature": 25.5}` |
| `data` | BYTEA | 消息原始数据（二进制） | `\x7b2274656d70657261747572...` |

### 索引建议

为了提高查询性能，建议创建以下索引：

```sql
-- 时间范围查询索引
CREATE INDEX idx_mqtt_messages_timestamp ON mqtt_messages (timestamp);

-- 主题查询索引
CREATE INDEX idx_mqtt_messages_topic ON mqtt_messages (topic);

-- 客户端查询索引
CREATE INDEX idx_mqtt_messages_client_id ON mqtt_messages (client_id);

-- 复合查询索引
CREATE INDEX idx_mqtt_messages_topic_timestamp ON mqtt_messages (topic, timestamp);
```

## 高级特性

### 批量插入

启用 `enable_batch_insert` 选项可以显著提高写入性能：

```json
{
  "enable_batch_insert": true
}
```

**优势：**
- 减少网络往返次数
- 提高数据库写入吞吐量
- 降低系统资源消耗

**注意事项：**
- 批量大小由系统自动控制（默认 100 条记录）
- 适合高频率消息场景

### UPSERT 操作

启用 `enable_upsert` 选项可以处理重复数据：

```json
{
  "enable_upsert": true,
  "conflict_columns": "client_id, topic"
}
```

**工作原理：**
- 当遇到冲突时，更新现有记录的 `timestamp`、`payload` 和 `data` 字段
- 冲突检测基于 `conflict_columns` 指定的列组合

**SQL 示例：**
```sql
INSERT INTO mqtt_messages (client_id, topic, timestamp, payload, data) 
VALUES ($1, $2, $3, $4, $5) 
ON CONFLICT (client_id, topic) 
DO UPDATE SET 
    timestamp = EXCLUDED.timestamp, 
    payload = EXCLUDED.payload, 
    data = EXCLUDED.data
```

### 连接池管理

连接器使用连接池来管理数据库连接：

```json
{
  "pool_size": 20
}
```

**配置建议：**
- 低并发场景：5-10 个连接
- 中等并发场景：10-20 个连接
- 高并发场景：20-50 个连接

## 使用 robust-ctl 创建 PostgreSQL 连接器

### 基本语法

使用 `robust-ctl` 命令行工具可以方便地创建和管理 PostgreSQL 连接器：

```bash
robust-ctl mqtt connector create \
  --connector-name <连接器名称> \
  --connector-type <连接器类型> \
  --config <配置> \
  --topic-id <主题ID>
```

### 创建 PostgreSQL 连接器

#### 1. 基本创建命令

```bash
# 创建 PostgreSQL 连接器
robust-ctl mqtt connector create \
  --connector-name "postgres_connector_01" \
  --connector-type "Postgres" \
  --config '{"host": "localhost", "port": 5432, "database": "mqtt_data", "username": "postgres", "password": "password123", "table": "mqtt_messages"}' \
  --topic-id "sensor/data"
```

#### 2. 参数说明

| 参数 | 说明 | 示例值 |
|------|------|--------|
| `--connector-name` | 连接器名称，必须唯一 | `postgres_connector_01` |
| `--connector-type` | 连接器类型，固定为 `Postgres` | `Postgres` |
| `--config` | JSON 格式的配置信息 | `{"host": "localhost", "port": 5432, ...}` |
| `--topic-id` | 要监听的 MQTT 主题 | `sensor/data` |

#### 3. 高性能配置示例

```bash
# 创建高性能 PostgreSQL 连接器
robust-ctl mqtt connector create \
  --connector-name "high_perf_postgres" \
  --connector-type "Postgres" \
  --config '{"host": "postgres.example.com", "port": 5432, "database": "iot_data", "username": "iot_user", "password": "secure_password", "table": "sensor_data", "pool_size": 30, "enable_batch_insert": true, "enable_upsert": true, "conflict_columns": "client_id, topic"}' \
  --topic-id "iot/sensors/+/data"
```

### 管理连接器

#### 1. 列出所有连接器
```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 列出指定名称的连接器
robust-ctl mqtt connector list --connector-name "postgres_connector_01"
```

#### 2. 删除连接器
```bash
# 删除指定连接器
robust-ctl mqtt connector delete --connector-name "postgres_connector_01"
```

### 完整操作示例

#### 场景：创建 IoT 数据存储系统

```bash
# 1. 创建传感器数据 PostgreSQL 连接器
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_postgres" \
  --connector-type "Postgres" \
  --config '{"host": "postgres.iot.local", "port": 5432, "database": "iot_platform", "username": "iot_writer", "password": "iot_pass_2023", "table": "sensor_readings", "pool_size": 25, "enable_batch_insert": true}' \
  --topic-id "iot/sensors/+/readings"

# 2. 创建设备状态 PostgreSQL 连接器
robust-ctl mqtt connector create \
  --connector-name "device_status_postgres" \
  --connector-type "Postgres" \
  --config '{"host": "postgres.iot.local", "port": 5432, "database": "iot_platform", "username": "iot_writer", "password": "iot_pass_2023", "table": "device_status", "enable_upsert": true, "conflict_columns": "client_id"}' \
  --topic-id "iot/devices/+/status"

# 3. 创建告警消息 PostgreSQL 连接器
robust-ctl mqtt connector create \
  --connector-name "alarm_postgres" \
  --connector-type "Postgres" \
  --config '{"host": "postgres.iot.local", "port": 5432, "database": "iot_platform", "username": "iot_writer", "password": "iot_pass_2023", "table": "alarm_logs", "pool_size": 15}' \
  --topic-id "iot/alarms/#"

# 4. 查看创建的连接器
robust-ctl mqtt connector list

# 5. 测试连接器（发布测试消息）
robust-ctl mqtt publish \
  --username "test_user" \
  --password "test_pass" \
  --topic "iot/sensors/temp_001/readings" \
  --qos 1 \
  --message '{"temperature": 25.5, "humidity": 60, "timestamp": 1640995200}'
```

## 数据查询示例

### 基本查询

```sql
-- 查询最近 1 小时的传感器数据
SELECT client_id, topic, timestamp, payload 
FROM mqtt_messages 
WHERE timestamp > EXTRACT(EPOCH FROM NOW() - INTERVAL '1 hour')
ORDER BY timestamp DESC;

-- 查询特定客户端的消息
SELECT topic, timestamp, payload 
FROM mqtt_messages 
WHERE client_id = 'sensor_001'
ORDER BY timestamp DESC
LIMIT 100;

-- 查询特定主题的消息
SELECT client_id, timestamp, payload 
FROM mqtt_messages 
WHERE topic LIKE 'sensor/temperature%'
ORDER BY timestamp DESC;
```

### 聚合查询

```sql
-- 统计每小时的消息数量
SELECT 
    DATE_TRUNC('hour', TO_TIMESTAMP(timestamp)) as hour,
    COUNT(*) as message_count
FROM mqtt_messages 
WHERE timestamp > EXTRACT(EPOCH FROM NOW() - INTERVAL '24 hours')
GROUP BY hour
ORDER BY hour;

-- 统计各主题的消息数量
SELECT 
    topic,
    COUNT(*) as message_count,
    MIN(TO_TIMESTAMP(timestamp)) as first_message,
    MAX(TO_TIMESTAMP(timestamp)) as last_message
FROM mqtt_messages 
GROUP BY topic
ORDER BY message_count DESC;
```

## 性能优化

### 数据库优化

1. **表分区**
```sql
-- 按时间分区
CREATE TABLE mqtt_messages_2024_01 PARTITION OF mqtt_messages
FOR VALUES FROM (1704067200) TO (1706745600);
```

2. **定期清理**
```sql
-- 删除 30 天前的数据
DELETE FROM mqtt_messages 
WHERE timestamp < EXTRACT(EPOCH FROM NOW() - INTERVAL '30 days');
```

3. **连接池调优**
```sql
-- PostgreSQL 配置优化
max_connections = 200
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 4MB
```

### 连接器优化

1. **启用批量插入**
```json
{
  "enable_batch_insert": true,
  "pool_size": 30
}
```

2. **合理设置连接池大小**
- 根据并发消息量调整 `pool_size`
- 监控数据库连接使用情况

3. **使用 UPSERT 处理重复数据**
```json
{
  "enable_upsert": true,
  "conflict_columns": "client_id, topic"
}
```

## 监控和故障排除

### 日志监控

连接器会输出详细的运行日志：

```
INFO  Connector postgres_connector_01 successfully connected to PostgreSQL database: mqtt_data
INFO  Connector postgres_connector_01 successfully wrote 100 records to PostgreSQL table mqtt_messages
ERROR Connector postgres_connector_01 failed to write data to PostgreSQL table mqtt_messages, error: connection timeout
```

### 常见问题

1. **连接失败**
   - 检查网络连通性
   - 验证数据库凭据
   - 确认防火墙设置

2. **写入性能低**
   - 启用批量插入
   - 增加连接池大小
   - 优化数据库索引

3. **数据重复**
   - 启用 UPSERT 功能
   - 配置合适的冲突列

## 总结

PostgreSQL 连接器是 RobustMQ 数据集成系统的重要组件，提供了高性能的关系型数据库桥接能力。通过合理的配置和使用，可以满足 IoT 数据存储、历史数据分析、数据持久化和业务数据集成等多种业务需求。

该连接器充分利用了 PostgreSQL 的 ACID 特性和丰富的查询能力，结合 Rust 语言的内存安全和零成本抽象优势，实现了高效、可靠的数据存储，是构建现代化 IoT 数据平台和分析系统的重要工具。
