# Cassandra 连接器

## 概述

Cassandra 连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息写入 Apache Cassandra 分布式 NoSQL 数据库。Cassandra 以高可用性、可扩展性和高写入吞吐量著称，非常适合 IoT 时序数据存储、设备注册管理和大规模事件日志等场景。

该连接器使用 [scylla](https://github.com/scylladb/scylla-rust-driver) Rust 驱动，同时兼容 Apache Cassandra 和 ScyllaDB。

## 功能特性

- 基于 scylla Rust 驱动，异步原生 Tokio
- 同时兼容 Apache Cassandra 和 ScyllaDB
- 支持用户名/密码认证
- 使用 Prepared Statement，高效批量写入
- 支持多节点集群连接
- 可配置连接超时

## 表结构

连接器使用 CQL 将数据写入用户预先创建的表。推荐的表结构：

```sql
CREATE KEYSPACE IF NOT EXISTS mqtt
  WITH REPLICATION = {'class': 'SimpleStrategy', 'replication_factor': 1};

CREATE TABLE mqtt.mqtt_messages (
    msgid text,
    topic text,
    qos int,
    payload text,
    arrived bigint,
    PRIMARY KEY (msgid, topic)
);
```

### 字段说明

| 字段名 | 类型 | 说明 |
|--------|------|------|
| `msgid` | text | 消息键值（通常为消息来源标识） |
| `topic` | text | 预留字段（空字符串） |
| `qos` | int | 预留字段（默认 0） |
| `payload` | text | MQTT 消息内容 |
| `arrived` | bigint | 消息到达时间戳 |

## 配置说明

### 连接器配置

```rust
pub struct CassandraConnectorConfig {
    pub nodes: Vec<String>,       // Cassandra 节点地址列表
    pub port: u16,                // 端口号
    pub keyspace: String,         // Keyspace 名称
    pub table: String,            // 表名
    pub username: String,         // 用户名
    pub password: String,         // 密码
    pub replication_factor: u32,  // 复制因子
    pub timeout_secs: u64,        // 连接超时（秒）
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 示例 |
|--------|------|------|--------|------|------|
| `nodes` | Array | 是 | - | Cassandra 节点地址列表 | `["127.0.0.1"]` |
| `port` | Number | 否 | `9042` | CQL 原生协议端口 | `9042` |
| `keyspace` | String | 是 | - | Keyspace 名称 | `mqtt` |
| `table` | String | 是 | - | 表名 | `mqtt_messages` |
| `username` | String | 否 | 空 | 认证用户名 | `cassandra` |
| `password` | String | 否 | 空 | 认证密码 | `cassandra` |
| `replication_factor` | Number | 否 | `1` | 复制因子 | `3` |
| `timeout_secs` | Number | 否 | `15` | 连接超时时间（秒），范围：1-300 | `15` |

### 配置示例

#### 基础配置

```json
{
  "nodes": ["127.0.0.1"],
  "keyspace": "mqtt",
  "table": "mqtt_messages"
}
```

#### 带认证的集群配置

```json
{
  "nodes": ["cass-node1", "cass-node2", "cass-node3"],
  "port": 9042,
  "keyspace": "mqtt",
  "table": "mqtt_messages",
  "username": "admin",
  "password": "secret",
  "replication_factor": 3,
  "timeout_secs": 30
}
```

#### 完整连接器配置

```json
{
  "cluster_name": "default",
  "connector_name": "cassandra_connector_01",
  "connector_type": "cassandra",
  "config": "{\"nodes\": [\"127.0.0.1\"], \"keyspace\": \"mqtt\", \"table\": \"mqtt_messages\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## 使用 robust-ctl 创建 Cassandra 连接器

### 基本语法

```bash
robust-ctl mqtt connector create \
  --connector-name <连接器名称> \
  --connector-type <连接器类型> \
  --config <配置> \
  --topic-id <主题ID>
```

### 创建示例

#### 1. 单节点

```bash
robust-ctl mqtt connector create \
  --connector-name "cassandra_connector_01" \
  --connector-type "cassandra" \
  --config '{"nodes": ["127.0.0.1"], "keyspace": "mqtt", "table": "mqtt_messages"}' \
  --topic-id "sensor/data"
```

#### 2. 多节点集群

```bash
robust-ctl mqtt connector create \
  --connector-name "cassandra_cluster" \
  --connector-type "cassandra" \
  --config '{"nodes": ["cass1", "cass2", "cass3"], "port": 9042, "keyspace": "iot", "table": "device_events", "username": "admin", "password": "secret", "replication_factor": 3}' \
  --topic-id "device/#"
```

### 管理连接器

```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 查看指定连接器
robust-ctl mqtt connector list --connector-name "cassandra_connector_01"

# 删除连接器
robust-ctl mqtt connector delete --connector-name "cassandra_connector_01"
```

### 完整操作示例

#### 场景：IoT 设备事件存储

```bash
# 1. 在 Cassandra 中创建 Keyspace 和 Table
# cqlsh -e "CREATE KEYSPACE mqtt WITH REPLICATION = {'class': 'SimpleStrategy', 'replication_factor': 1}"
# cqlsh -e "CREATE TABLE mqtt.device_events (msgid text, topic text, qos int, payload text, arrived bigint, PRIMARY KEY (msgid, topic))"

# 2. 创建连接器
robust-ctl mqtt connector create \
  --connector-name "device_to_cassandra" \
  --connector-type "cassandra" \
  --config '{"nodes": ["127.0.0.1"], "keyspace": "mqtt", "table": "device_events"}' \
  --topic-id "device/+"

# 3. 查看连接器
robust-ctl mqtt connector list
```

## 性能优化建议

### 1. 集群部署
- 生产环境至少部署 3 个节点
- 设置合理的 `replication_factor`（通常为 3）

### 2. 表引擎与分区
- 合理选择 Partition Key，避免热点分区
- 对于时序数据，可以使用复合 Partition Key（如 `device_id` + 日期）

### 3. 批量写入
- 连接器内置批量写入，每批最多 100 条记录
- Cassandra 对并发写入非常友好

### 4. 安全建议
- 生产环境启用认证
- 创建专用写入用户，限制权限
- 考虑启用 TLS 加密传输

## 监控和故障排查

### 1. 查看连接器状态

```bash
robust-ctl mqtt connector list --connector-name "cassandra_connector_01"
```

### 2. 常见问题

**问题 1：连接失败**
- 检查 Cassandra 服务是否运行
- 验证节点地址和端口（默认 9042）
- 确认网络连通性和防火墙设置

**问题 2：Keyspace/Table 不存在**
- 确认 Keyspace 和 Table 已预先创建
- 检查名称拼写

**问题 3：认证失败**
- 检查用户名/密码是否正确
- 确认 Cassandra 已启用认证（`authenticator: PasswordAuthenticator`）

**问题 4：写入延迟**
- 检查 Cassandra 集群负载
- 增加节点数量分散负载
- 检查网络延迟

## 总结

Cassandra 连接器为 RobustMQ 提供了与分布式 NoSQL 数据库的集成能力。通过 scylla 驱动实现了：

- **双兼容**：同时支持 Apache Cassandra 和 ScyllaDB
- **高可用**：支持多节点集群连接，自动故障转移
- **高吞吐**：Prepared Statement + 批量写入，适合大规模 IoT 数据
- **活跃生态**：scylla 驱动社区活跃，月下载量 20w+
