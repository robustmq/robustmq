# Redis 连接器

## 概述

Redis 连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息桥接到 Redis 数据库。该连接器支持 Single、Cluster、Sentinel 三种部署模式，通过命令模板灵活定义数据写入方式，适合实时缓存、消息队列、会话存储和计数器等场景。

## 配置说明

### 连接器配置

Redis 连接器使用 `RedisConnectorConfig` 结构进行配置：

```rust
pub struct RedisConnectorConfig {
    pub server: String,                       // Redis 服务器地址
    pub mode: RedisMode,                      // 部署模式
    pub database: u8,                         // 数据库编号
    pub username: Option<String>,             // 用户名
    pub password: Option<String>,             // 密码
    pub sentinel_master_name: Option<String>, // Sentinel 主节点名称
    pub command_template: String,             // 命令模板
    pub tls_enabled: bool,                    // 是否启用 TLS
    pub connect_timeout_ms: u64,              // 连接超时时间（毫秒）
    pub pool_size: u32,                       // 连接池大小
    pub max_retries: u32,                     // 最大重试次数
    pub retry_interval_ms: u64,              // 重试间隔（毫秒）
}

pub enum RedisMode {
    Single,    // 单机模式
    Cluster,   // 集群模式
    Sentinel,  // 哨兵模式
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 示例 |
|--------|------|------|--------|------|------|
| `server` | String | 是 | - | Redis 服务器地址，长度不超过 1024 字符 | `127.0.0.1:6379` |
| `mode` | String | 否 | `single` | 部署模式：`single`、`cluster`、`sentinel` | `single` |
| `database` | Number | 否 | `0` | 数据库编号，范围：0-15 | `0` |
| `username` | String | 否 | - | 用户名（Redis 6.0+ ACL） | `default` |
| `password` | String | 否 | - | 密码 | `password123` |
| `sentinel_master_name` | String | 否 | - | Sentinel 主节点名称，Sentinel 模式时必填 | `mymaster` |
| `command_template` | String | 是 | - | 命令模板，支持 `${key}`、`${payload}` 等占位符，长度不超过 4096 字符 | `SET ${key} ${payload}` |
| `tls_enabled` | Boolean | 否 | `false` | 是否启用 TLS 加密连接 | `true` |
| `connect_timeout_ms` | Number | 否 | `5000` | 连接超时时间（毫秒） | `10000` |
| `pool_size` | Number | 否 | `10` | 连接池大小，范围：1-100 | `20` |
| `max_retries` | Number | 否 | `3` | 最大重试次数 | `5` |
| `retry_interval_ms` | Number | 否 | `1000` | 重试间隔（毫秒） | `2000` |

### 命令模板

命令模板支持以下占位符：

| 占位符 | 说明 |
|--------|------|
| `${key}` | 消息键值 |
| `${payload}` | 消息内容 |
| `${timestamp}` | 消息时间戳 |

#### 命令模板示例

```
SET ${key} ${payload}
LPUSH mqtt:messages ${payload}
HSET mqtt:data ${key} ${payload}
PUBLISH mqtt:channel ${payload}
SETEX ${key} 3600 ${payload}
```

### 配置示例

#### JSON 配置格式

**单机模式**
```json
{
  "server": "127.0.0.1:6379",
  "command_template": "LPUSH mqtt:messages ${payload}"
}
```

**带密码认证**
```json
{
  "server": "127.0.0.1:6379",
  "password": "your_password",
  "database": 1,
  "command_template": "SET mqtt:${key} ${payload}",
  "pool_size": 20
}
```

**集群模式**
```json
{
  "server": "redis-1:6379,redis-2:6379,redis-3:6379",
  "mode": "cluster",
  "password": "cluster_password",
  "command_template": "SET mqtt:${key} ${payload}",
  "pool_size": 30,
  "connect_timeout_ms": 10000
}
```

**哨兵模式**
```json
{
  "server": "sentinel-1:26379,sentinel-2:26379,sentinel-3:26379",
  "mode": "sentinel",
  "sentinel_master_name": "mymaster",
  "password": "sentinel_password",
  "command_template": "LPUSH mqtt:events ${payload}",
  "pool_size": 15
}
```

## 使用 robust-ctl 创建 Redis 连接器

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
  --connector-name "redis_connector_01" \
  --connector-type "redis" \
  --config '{"server": "127.0.0.1:6379", "command_template": "LPUSH mqtt:messages ${payload}"}' \
  --topic-id "sensor/data"
```

#### 2. 带认证的 Redis

```bash
robust-ctl mqtt connector create \
  --connector-name "redis_auth" \
  --connector-type "redis" \
  --config '{"server": "127.0.0.1:6379", "password": "your_password", "database": 1, "command_template": "SET mqtt:${key} ${payload}", "pool_size": 20}' \
  --topic-id "device/status"
```

### 管理连接器

```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 查看指定连接器
robust-ctl mqtt connector list --connector-name "redis_connector_01"

# 删除连接器
robust-ctl mqtt connector delete --connector-name "redis_connector_01"
```

### 完整操作示例

#### 场景：IoT 实时数据缓存

```bash
# 1. 创建传感器数据缓存连接器
robust-ctl mqtt connector create \
  --connector-name "sensor_cache" \
  --connector-type "redis" \
  --config '{"server": "127.0.0.1:6379", "command_template": "SETEX sensor:${key} 3600 ${payload}", "pool_size": 20}' \
  --topic-id "iot/sensors/temperature"

# 2. 创建设备事件队列连接器
robust-ctl mqtt connector create \
  --connector-name "event_queue" \
  --connector-type "redis" \
  --config '{"server": "127.0.0.1:6379", "command_template": "LPUSH iot:events ${payload}", "database": 2}' \
  --topic-id "iot/events"

# 3. 查看创建的连接器
robust-ctl mqtt connector list
```

## 性能优化建议

### 1. 连接池
- 根据并发量合理设置 `pool_size`
- 高吞吐量场景建议增大连接池

### 2. 命令选择
- 使用批量命令（如 `LPUSH`）减少网络往返
- 对于需要过期的数据使用 `SETEX` 或 `EXPIRE`
- 避免使用阻塞命令

### 3. 部署模式
- 单机模式适合开发和测试
- 生产环境建议使用 Cluster 或 Sentinel 模式
- Sentinel 模式提供自动故障转移

### 4. 安全建议
- 生产环境建议启用 TLS
- 使用 ACL（Redis 6.0+）进行细粒度权限控制
- 定期轮换密码

## 监控和故障排查

### 1. 查看连接器状态

```bash
robust-ctl mqtt connector list --connector-name "redis_connector_01"
```

### 2. 常见问题

**问题 1：连接失败**
- 检查 Redis 服务是否正常运行
- 验证服务器地址和端口是否正确
- 检查网络连接和防火墙设置

**问题 2：认证失败**
- 验证密码是否正确
- 检查用户名和 ACL 配置（Redis 6.0+）

**问题 3：写入超时**
- 增加 `connect_timeout_ms` 配置
- 检查 Redis 服务负载
- 考虑增大连接池大小

**问题 4：Sentinel 模式连接问题**
- 确认 `sentinel_master_name` 配置正确
- 检查 Sentinel 节点是否正常运行
- 验证所有 Sentinel 节点的连通性

## 总结

Redis 连接器是 RobustMQ 数据集成系统的重要组件，提供了与 Redis 数据库的无缝集成能力。通过合理的配置和使用，可以实现：

- **实时缓存**：将 MQTT 消息实时写入 Redis，支持高速读写
- **灵活命令**：通过命令模板自定义数据写入方式
- **高可用**：支持 Single、Cluster、Sentinel 三种部署模式
- **高性能**：连接池和批量写入确保高吞吐量场景下的性能
