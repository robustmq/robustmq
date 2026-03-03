# ClickHouse 连接器

## 概述

ClickHouse 连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息写入 ClickHouse 列式数据库。ClickHouse 擅长以极低延迟处理和分析海量数据，非常适合 IoT 数据的实时分析、时序数据存储和日志处理等场景。

该连接器使用 [ClickHouse 官方 Rust 客户端](https://github.com/ClickHouse/clickhouse-rs)，采用高效的 RowBinary 格式进行数据传输，支持 LZ4 压缩和 Schema 校验。

## 功能特性

- 基于 ClickHouse 官方 Rust 客户端，RowBinary 格式传输
- 支持 LZ4 压缩，高效传输
- 支持 Schema 校验，避免类型不匹配
- 支持用户名/密码认证
- 批量写入，高吞吐

## 表结构

连接器使用固定的行格式写入数据。用户需要在 ClickHouse 中预先创建匹配的表：

```sql
CREATE TABLE mqtt_messages (
    data String,
    key String,
    timestamp UInt64
) ENGINE = MergeTree()
ORDER BY timestamp;
```

### 字段说明

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `data` | String | MQTT 消息内容（payload） |
| `key` | String | 消息键值（通常为消息来源标识） |
| `timestamp` | UInt64 | 消息时间戳（Unix 秒） |

> 如果需要更复杂的数据转换，可以使用 ClickHouse 的 [Materialized View](https://clickhouse.com/docs/en/sql-reference/statements/create/view#materialized-view) 将数据从原始表转换到目标表。

## 配置说明

### 连接器配置

```rust
pub struct ClickHouseConnectorConfig {
    pub url: String,           // ClickHouse HTTP 地址
    pub database: String,      // 数据库名称
    pub table: String,         // 表名
    pub username: String,      // 用户名
    pub password: String,      // 密码
    pub pool_size: u32,        // 连接池大小
    pub timeout_secs: u64,     // 超时时间（秒）
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 示例 |
|--------|------|------|--------|------|------|
| `url` | String | 是 | - | ClickHouse HTTP 端点，必须以 `http://` 或 `https://` 开头 | `http://localhost:8123` |
| `database` | String | 是 | - | 数据库名称，长度不超过 256 字符 | `mqtt_data` |
| `table` | String | 是 | - | 表名，长度不超过 256 字符 | `mqtt_messages` |
| `username` | String | 否 | 空 | 连接用户名 | `default` |
| `password` | String | 否 | 空 | 连接密码 | `password123` |
| `pool_size` | Number | 否 | `8` | 连接池大小，范围：1-64 | `8` |
| `timeout_secs` | Number | 否 | `15` | 请求超时时间（秒），范围：1-300 | `15` |

### 配置示例

#### JSON 配置格式

**基础配置**
```json
{
  "url": "http://localhost:8123",
  "database": "mqtt_data",
  "table": "mqtt_messages"
}
```

**带认证配置**
```json
{
  "url": "http://clickhouse-server:8123",
  "database": "mqtt_data",
  "table": "mqtt_messages",
  "username": "emqx",
  "password": "public",
  "pool_size": 16,
  "timeout_secs": 30
}
```

#### 完整连接器配置

```json
{
  "cluster_name": "default",
  "connector_name": "clickhouse_connector_01",
  "connector_type": "clickhouse",
  "config": "{\"url\": \"http://localhost:8123\", \"database\": \"mqtt_data\", \"table\": \"mqtt_messages\", \"username\": \"default\", \"password\": \"\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## 使用 robust-ctl 创建 ClickHouse 连接器

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
  --connector-name "clickhouse_connector_01" \
  --connector-type "clickhouse" \
  --config '{"url": "http://localhost:8123", "database": "mqtt_data", "table": "mqtt_messages"}' \
  --topic-id "sensor/data"
```

#### 2. 带认证的 ClickHouse

```bash
robust-ctl mqtt connector create \
  --connector-name "clickhouse_auth" \
  --connector-type "clickhouse" \
  --config '{"url": "http://clickhouse-server:8123", "database": "iot_db", "table": "device_logs", "username": "admin", "password": "secret", "pool_size": 16}' \
  --topic-id "device/#"
```

### 管理连接器

```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 查看指定连接器
robust-ctl mqtt connector list --connector-name "clickhouse_connector_01"

# 删除连接器
robust-ctl mqtt connector delete --connector-name "clickhouse_connector_01"
```

### 完整操作示例

#### 场景：IoT 传感器数据分析

```bash
# 1. 在 ClickHouse 中创建表
# clickhouse-client --query "CREATE TABLE mqtt_data.sensor_messages (data String, key String, timestamp UInt64) ENGINE = MergeTree() ORDER BY timestamp"

# 2. 创建连接器
robust-ctl mqtt connector create \
  --connector-name "sensor_to_clickhouse" \
  --connector-type "clickhouse" \
  --config '{"url": "http://localhost:8123", "database": "mqtt_data", "table": "sensor_messages", "username": "default"}' \
  --topic-id "sensor/+"

# 3. 查看创建的连接器
robust-ctl mqtt connector list
```

## 性能优化建议

### 1. 连接池
- 根据并发量合理设置 `pool_size`
- 高吞吐场景建议 16-32

### 2. 表引擎选择
- 时序数据推荐 `MergeTree` 或 `ReplacingMergeTree`
- 需要 TTL 自动清理的场景，可在表定义中添加 `TTL timestamp + INTERVAL 30 DAY`

### 3. 批量写入
- 连接器内置批量写入，每批最多 100 条记录
- ClickHouse 对大批量写入更友好，避免逐条插入

### 4. 安全建议
- 生产环境使用独立的数据库用户
- 限制用户权限为仅 INSERT
- 如需 HTTPS，确保 ClickHouse 配置了 TLS

## 监控和故障排查

### 1. 查看连接器状态

```bash
robust-ctl mqtt connector list --connector-name "clickhouse_connector_01"
```

### 2. 常见问题

**问题 1：连接失败**
- 检查 ClickHouse 服务是否运行
- 验证 URL 地址和端口（默认 HTTP 端口为 8123）
- 确认用户名/密码是否正确
- 检查网络连通性和防火墙

**问题 2：写入失败（Schema 不匹配）**
- 确认 ClickHouse 表结构与连接器要求的字段（`data String, key String, timestamp UInt64`）一致
- 检查数据库和表名拼写

**问题 3：写入延迟**
- 检查 ClickHouse 服务器负载
- 适当增加 `pool_size`
- 检查网络延迟

## 总结

ClickHouse 连接器是 RobustMQ 数据集成系统的重要组件，利用 ClickHouse 的列式存储和高性能分析能力，为 MQTT 消息提供了高效的数据存储方案。通过简单的配置即可实现：

- **实时分析**：MQTT 消息实时写入 ClickHouse，支持即时查询分析
- **高吞吐**：批量写入 + RowBinary 格式，适合大规模 IoT 数据
- **官方支持**：使用 ClickHouse 官方 Rust 客户端，长期稳定
- **灵活扩展**：通过 Materialized View 实现数据转换和二次加工
