# MySQL 连接器

## 概述

MySQL 连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息桥接到 MySQL 关系型数据库系统。该连接器支持高性能的数据写入，适合 IoT 数据存储、历史数据分析、数据持久化和业务数据集成等场景。

## 配置说明

### 连接器配置

MySQL 连接器使用 `MySQLConnectorConfig` 结构进行配置：

```rust
pub struct MySQLConnectorConfig {
    pub host: String,                    // MySQL 服务器地址
    pub port: u16,                       // MySQL 服务器端口
    pub database: String,                // 数据库名称
    pub username: String,                // 用户名
    pub password: String,                // 密码
    pub table: String,                   // 目标表名
    pub sql_template: Option<String>,    // 自定义 SQL 模板
    pub pool_size: Option<u32>,          // 连接池大小
    pub enable_batch_insert: Option<bool>, // 启用批量插入
    pub enable_upsert: Option<bool>,     // 启用 UPSERT 操作
    pub conflict_columns: Option<String>, // 冲突列定义
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 说明 | 示例 |
|--------|------|------|------|------|
| `host` | String | 是 | MySQL 服务器地址 | `localhost` 或 `192.168.1.100` |
| `port` | u16 | 是 | MySQL 服务器端口 | `3306` |
| `database` | String | 是 | 数据库名称 | `mqtt_data` |
| `username` | String | 是 | 数据库用户名 | `root` |
| `password` | String | 是 | 数据库密码 | `password123` |
| `table` | String | 是 | 目标表名 | `mqtt_messages` |
| `sql_template` | String | 否 | 自定义 SQL 插入模板 | `INSERT INTO ...` |
| `pool_size` | u32 | 否 | 连接池大小，默认为 10 | `20` |
| `enable_batch_insert` | bool | 否 | 启用批量插入，默认为 false | `true` |
| `enable_upsert` | bool | 否 | 启用 UPSERT 操作，默认为 false | `true` |
| `conflict_columns` | String | 否 | 冲突列定义（在 UPSERT 模式使用） | `record_key` |

### 配置示例

#### JSON 配置格式
```json
{
  "host": "localhost",
  "port": 3306,
  "database": "mqtt_data",
  "username": "root",
  "password": "password123",
  "table": "mqtt_messages",
  "pool_size": 20,
  "enable_batch_insert": true,
  "enable_upsert": false
}
```

#### 完整连接器配置
```json
{
  "cluster_name": "default",
  "connector_name": "mysql_connector_01",
  "connector_type": "MySQL",
  "config": "{\"host\": \"localhost\", \"port\": 3306, \"database\": \"mqtt_data\", \"username\": \"root\", \"password\": \"password123\", \"table\": \"mqtt_messages\", \"enable_batch_insert\": true}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## 数据库表结构

### 表结构定义

MySQL 连接器要求目标表具有以下结构：

```sql
CREATE TABLE mqtt_messages (
    record_key VARCHAR(255) NOT NULL,   -- 记录唯一键（通常是客户端 ID）
    payload TEXT,                       -- 消息载荷（JSON 格式）
    timestamp BIGINT NOT NULL,          -- 消息时间戳（毫秒）
    PRIMARY KEY (record_key, timestamp),
    INDEX idx_timestamp (timestamp),
    INDEX idx_record_key (record_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

### 字段说明

| 字段名 | 数据类型 | 说明 | 示例 |
|--------|----------|------|------|
| `record_key` | VARCHAR(255) | 记录唯一键，通常是客户端 ID | `sensor_001` |
| `payload` | TEXT | 消息载荷（JSON 格式） | `{"client_id":"sensor_001","topic":"sensor/temperature",...}` |
| `timestamp` | BIGINT | 消息时间戳（毫秒级） | `1640995200000` |

### 索引建议

为了提高查询性能，建议创建以下索引：

```sql
-- 时间范围查询索引
CREATE INDEX idx_mqtt_messages_timestamp ON mqtt_messages (timestamp);

-- 记录键查询索引
CREATE INDEX idx_mqtt_messages_record_key ON mqtt_messages (record_key);

-- 复合查询索引
CREATE INDEX idx_mqtt_messages_key_timestamp ON mqtt_messages (record_key, timestamp);
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
- 批量大小由系统自动控制
- 适合高频率消息场景
- 在批量模式下，自定义 `sql_template` 不会生效

### UPSERT 操作

启用 `enable_upsert` 选项可以处理重复数据：

```json
{
  "enable_upsert": true
}
```

**工作原理：**
- 使用 MySQL 的 `ON DUPLICATE KEY UPDATE` 语法
- 当遇到重复的 `record_key` 时，更新 `payload` 和 `timestamp` 字段
- 适用于需要保持最新状态的场景

**SQL 示例：**
```sql
INSERT INTO mqtt_messages (record_key, payload, timestamp) 
VALUES (?, ?, ?) 
ON DUPLICATE KEY UPDATE 
    payload = VALUES(payload), 
    timestamp = VALUES(timestamp)
```

### 自定义 SQL 模板

通过 `sql_template` 可以自定义插入语句（仅在非批量模式下生效）：

```json
{
  "sql_template": "INSERT INTO mqtt_messages (record_key, payload, timestamp) VALUES (?, ?, ?)",
  "enable_batch_insert": false
}
```

**注意事项：**
- SQL 模板必须包含 3 个占位符 `?`
- 占位符按顺序绑定：`record_key`, `payload`, `timestamp`
- 批量插入模式下该选项无效

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

## 使用 robust-ctl 创建 MySQL 连接器

### 基本语法

使用 `robust-ctl` 命令行工具可以方便地创建和管理 MySQL 连接器：

```bash
robust-ctl mqtt connector create \
  --connector-name <连接器名称> \
  --connector-type <连接器类型> \
  --config <配置> \
  --topic-id <主题ID>
```

### 创建 MySQL 连接器

#### 1. 基本创建命令

```bash
# 创建 MySQL 连接器
robust-ctl mqtt connector create \
  --connector-name "mysql_connector_01" \
  --connector-type "MySQL" \
  --config '{"host": "localhost", "port": 3306, "database": "mqtt_data", "username": "root", "password": "password123", "table": "mqtt_messages"}' \
  --topic-id "sensor/data"
```

#### 2. 参数说明

| 参数 | 说明 | 示例值 |
|------|------|--------|
| `--connector-name` | 连接器名称，必须唯一 | `mysql_connector_01` |
| `--connector-type` | 连接器类型，固定为 `MySQL` | `MySQL` |
| `--config` | JSON 格式的配置信息 | `{"host": "localhost", "port": 3306, ...}` |
| `--topic-id` | 要监听的 MQTT 主题 | `sensor/data` |

#### 3. 高性能配置示例

```bash
# 创建高性能 MySQL 连接器
robust-ctl mqtt connector create \
  --connector-name "high_perf_mysql" \
  --connector-type "MySQL" \
  --config '{"host": "mysql.example.com", "port": 3306, "database": "iot_data", "username": "iot_user", "password": "secure_password", "table": "sensor_data", "pool_size": 30, "enable_batch_insert": true, "enable_upsert": true}' \
  --topic-id "iot/sensors/+/data"
```

### 管理连接器

#### 1. 列出所有连接器
```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 列出指定名称的连接器
robust-ctl mqtt connector list --connector-name "mysql_connector_01"
```

#### 2. 删除连接器
```bash
# 删除指定连接器
robust-ctl mqtt connector delete --connector-name "mysql_connector_01"
```

### 完整操作示例

#### 场景：创建 IoT 数据存储系统

```bash
# 1. 创建传感器数据 MySQL 连接器
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_mysql" \
  --connector-type "MySQL" \
  --config '{"host": "mysql.iot.local", "port": 3306, "database": "iot_platform", "username": "iot_writer", "password": "iot_pass_2023", "table": "sensor_readings", "pool_size": 25, "enable_batch_insert": true}' \
  --topic-id "iot/sensors/+/readings"

# 2. 创建设备状态 MySQL 连接器
robust-ctl mqtt connector create \
  --connector-name "device_status_mysql" \
  --connector-type "MySQL" \
  --config '{"host": "mysql.iot.local", "port": 3306, "database": "iot_platform", "username": "iot_writer", "password": "iot_pass_2023", "table": "device_status", "enable_upsert": true}' \
  --topic-id "iot/devices/+/status"

# 3. 创建告警消息 MySQL 连接器
robust-ctl mqtt connector create \
  --connector-name "alarm_mysql" \
  --connector-type "MySQL" \
  --config '{"host": "mysql.iot.local", "port": 3306, "database": "iot_platform", "username": "iot_writer", "password": "iot_pass_2023", "table": "alarm_logs", "pool_size": 15}' \
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
-- 查询最近 1 小时的数据
SELECT record_key, payload, FROM_UNIXTIME(timestamp/1000) as time 
FROM mqtt_messages 
WHERE timestamp > UNIX_TIMESTAMP(DATE_SUB(NOW(), INTERVAL 1 HOUR)) * 1000
ORDER BY timestamp DESC;

-- 查询特定记录键的消息
SELECT payload, FROM_UNIXTIME(timestamp/1000) as time 
FROM mqtt_messages 
WHERE record_key = 'sensor_001'
ORDER BY timestamp DESC
LIMIT 100;

-- 查询特定时间范围的数据
SELECT * FROM mqtt_messages 
WHERE timestamp BETWEEN 1640995200000 AND 1640995300000
ORDER BY timestamp;
```

### 聚合查询

```sql
-- 统计每小时的消息数量
SELECT 
    FROM_UNIXTIME(timestamp/1000, '%Y-%m-%d %H:00:00') as hour,
    COUNT(*) as message_count
FROM mqtt_messages 
WHERE timestamp > UNIX_TIMESTAMP(DATE_SUB(NOW(), INTERVAL 24 HOUR)) * 1000
GROUP BY hour
ORDER BY hour;

-- 统计各记录键的消息数量
SELECT 
    record_key,
    COUNT(*) as message_count,
    FROM_UNIXTIME(MIN(timestamp)/1000) as first_message,
    FROM_UNIXTIME(MAX(timestamp)/1000) as last_message
FROM mqtt_messages 
GROUP BY record_key
ORDER BY message_count DESC;
```

## 性能优化

### 数据库优化

1. **表分区**
```sql
-- 按月份分区
CREATE TABLE mqtt_messages (
    record_key VARCHAR(255) NOT NULL,
    payload TEXT,
    timestamp BIGINT NOT NULL,
    PRIMARY KEY (record_key, timestamp)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
PARTITION BY RANGE (timestamp) (
    PARTITION p202401 VALUES LESS THAN (1706745600000),
    PARTITION p202402 VALUES LESS THAN (1709251200000),
    PARTITION pmax VALUES LESS THAN MAXVALUE
);
```

2. **定期清理**
```sql
-- 删除 30 天前的数据
DELETE FROM mqtt_messages 
WHERE timestamp < UNIX_TIMESTAMP(DATE_SUB(NOW(), INTERVAL 30 DAY)) * 1000;
```

3. **MySQL 配置优化**
```ini
# MySQL 配置优化
max_connections = 200
innodb_buffer_pool_size = 1G
innodb_log_file_size = 256M
innodb_flush_log_at_trx_commit = 2
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
  "enable_upsert": true
}
```

## 监控和故障排除

### 日志监控

连接器会输出详细的运行日志：

```
INFO  Connector mysql_connector_01 successfully connected to MySQL database: mqtt_data
INFO  Connector mysql_connector_01 successfully wrote 100 records to MySQL table mqtt_messages
ERROR Connector mysql_connector_01 failed to write data to MySQL table mqtt_messages, error: connection timeout
```

### 常见问题

1. **连接失败**
   - 检查网络连通性
   - 验证数据库凭据
   - 确认防火墙设置
   - 检查 MySQL 服务是否运行

2. **写入性能低**
   - 启用批量插入
   - 增加连接池大小
   - 优化数据库索引
   - 检查磁盘 I/O 性能

3. **数据重复**
   - 启用 UPSERT 功能
   - 设置合适的主键约束

4. **连接池耗尽**
   - 增加 `pool_size` 参数
   - 优化消息处理速度
   - 检查是否有慢查询

## MySQL vs PostgreSQL 对比

| 特性 | MySQL 连接器 | PostgreSQL 连接器 |
|------|-------------|-------------------|
| 数据模型 | 记录键 + 载荷 + 时间戳 | 客户端ID + 主题 + 载荷 + 数据 + 时间戳 |
| UPSERT 语法 | `ON DUPLICATE KEY UPDATE` | `ON CONFLICT ... DO UPDATE` |
| 自定义 SQL | 支持 `sql_template` | 不支持 |
| 批量插入 | 支持 | 支持 |
| 连接池 | sqlx 连接池 | tokio-postgres 客户端 |
| 适用场景 | 通用数据存储 | 结构化数据存储 |

## 总结

MySQL 连接器是 RobustMQ 数据集成系统的重要组件，提供了高性能的关系型数据库桥接能力。通过合理的配置和使用，可以满足 IoT 数据存储、历史数据分析、数据持久化和业务数据集成等多种业务需求。

该连接器充分利用了 MySQL 的高性能和成熟稳定的特性，结合 Rust 语言的内存安全和零成本抽象优势，实现了高效、可靠的数据存储，是构建现代化 IoT 数据平台和分析系统的重要工具。

**核心特性：**
- ✅ 高性能批量插入
- ✅ UPSERT 操作支持
- ✅ 连接池管理
- ✅ 自定义 SQL 模板
- ✅ 灵活的数据模型
- ✅ 完善的错误处理

