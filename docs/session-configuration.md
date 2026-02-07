# RobustMQ 会话类型配置

## 概述

RobustMQ 支持两种会话类型：

- **临时会话（Transient Session）**：默认模式，会话数据仅存储在内存中，提供更高的性能
- **持久会话（Durable Session）**：会话数据持久化到 RocksDB + Raft，节点重启后可恢复

## 配置说明

### 配置项

在 `config/robustmq-server.toml` 中配置：

```toml
[mqtt_runtime]
default_user = "admin"
default_password = "robustmq"
max_connection_num = 1000000

# 是否启用持久会话
# - false (默认): 使用临时会话，更高性能，适合实时场景
# - true: 使用持久会话，数据持久化，适合关键业务
durable_sessions_enable = false
```

### 默认值

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `durable_sessions_enable` | `false` | 默认使用临时会话（更高性能） |

## 会话类型对比

### 临时会话 (durable_sessions_enable = false)

**特点：**
- ✅ 会话数据仅存储在内存中
- ✅ 更高的吞吐量（>100K msg/s）
- ✅ 更低的延迟（<10ms P99）
- ⚠️ 节点重启后会话丢失
- ⚠️ 消息队列有大小限制

**适合场景：**
- 实时监控仪表盘
- 设备状态心跳上报
- 实时数据展示
- 临时测试连接
- 对延迟敏感的场景

**性能指标：**
```
吞吐量：  ~100K msg/s（单节点）
P99 延迟：  ~10ms
内存占用：  中等
磁盘 I/O：  无
```

### 持久会话 (durable_sessions_enable = true)

**特点：**
- ✅ 会话数据持久化到磁盘（RocksDB + Raft）
- ✅ 节点重启后可恢复
- ✅ 消息不会因队列满而丢失
- ✅ 支持 Raft 多副本保证可靠性
- ⚠️ 较高的延迟（批量写入）
- ⚠️ 吞吐量相对较低

**适合场景：**
- 金融交易通知
- 工业控制指令
- 关键业务消息
- 需要离线消息重放
- 对可靠性要求极高的场景

**性能指标：**
```
吞吐量：  ~50K msg/s（单节点）
P99 延迟：  ~30ms
内存占用：  低
磁盘 I/O：  高
```

## 使用示例

### 场景 1：实时监控系统（推荐临时会话）

```toml
# config/robustmq-server.toml
[mqtt_runtime]
durable_sessions_enable = false  # 使用临时会话
max_connection_num = 1000000
```

**客户端代码（无需特殊设置）：**
```rust
use paho_mqtt as mqtt;

let client = mqtt::Client::new("tcp://localhost:1883").unwrap();
let conn_opts = mqtt::ConnectOptionsBuilder::new()
    .clean_session(true)  // MQTT 3.1.1
    .finalize();

client.connect(conn_opts).unwrap();
```

### 场景 2：金融交易系统（推荐持久会话）

```toml
# config/robustmq-server.toml
[mqtt_runtime]
durable_sessions_enable = true  # 使用持久会话
max_connection_num = 100000
```

**客户端代码：**
```rust
use paho_mqtt as mqtt;

let client = mqtt::Client::new("tcp://localhost:1883").unwrap();

// MQTT 5.0
let conn_opts = mqtt::ConnectOptionsBuilder::new_v5()
    .clean_start(false)  // 保留会话
    .finalize();

// 或 MQTT 3.1.1
let conn_opts = mqtt::ConnectOptionsBuilder::new()
    .clean_session(false)  // 保留会话
    .finalize();

client.connect(conn_opts).unwrap();
```

### 场景 3：混合场景（根据业务需要切换）

可以通过修改配置文件并重启 broker 来切换会话类型：

```bash
# 当前使用临时会话
vim config/robustmq-server.toml
# 修改 durable_sessions_enable = true

# 重启 broker
systemctl restart robustmq-broker
```

## 性能调优建议

### 临时会话性能优化

```toml
[mqtt_runtime]
durable_sessions_enable = false
max_connection_num = 2000000  # 增加最大连接数

# 临时会话主要消耗内存，确保足够的 RAM
# 每个会话约占用 1-5 KB 内存（取决于订阅数量和消息队列大小）
```

### 持久会话性能优化

```toml
[mqtt_runtime]
durable_sessions_enable = true
max_connection_num = 500000

# 持久会话主要消耗磁盘 I/O
# 建议使用 SSD 存储以获得更好的性能
```

```toml
[rocksdb]
# 增加写缓冲区大小
write_buffer_size = 134217728  # 128 MB

# 增加最大写缓冲区数量
max_write_buffer_number = 6

# 启用压缩
compression = "lz4"
```

## 迁移指南

### 从持久会话迁移到临时会话

1. 备份数据（如需要）
2. 修改配置：`durable_sessions_enable = false`
3. 重启 broker
4. 客户端重新连接

**注意：** 所有离线会话和消息将丢失

### 从临时会话迁移到持久会话

1. 修改配置：`durable_sessions_enable = true`
2. 重启 broker
3. 客户端重新连接

**注意：** 迁移后性能可能下降，但可靠性显著提升

## 监控指标

### 关键指标

```bash
# 查看当前会话类型
curl http://localhost:9090/metrics | grep session_type

# 临时会话指标
robustmq_transient_sessions_total
robustmq_transient_messages_queued
robustmq_transient_messages_dropped

# 持久会话指标
robustmq_durable_sessions_total
robustmq_durable_messages_persisted
robustmq_durable_write_latency_seconds
```

## 常见问题

### Q1: 为什么默认使用临时会话？

**A:** 临时会话提供更好的性能和更低的延迟，适合大多数实时场景。如果需要更高的可靠性，可以启用持久会话。

### Q2: 可以在运行时切换会话类型吗？

**A:** 目前不支持热切换，需要修改配置文件并重启 broker。

### Q3: 临时会话会丢消息吗？

**A:** 客户端在线时不会丢消息。客户端离线或节点重启时，未送达的消息会丢失。

### Q4: 持久会话的性能可以优化吗？

**A:** 可以通过以下方式优化：
- 使用 SSD 存储
- 调整 RocksDB 参数
- 增加内存缓存
- 使用批量写入

### Q5: 如何选择合适的会话类型？

**A:** 参考决策表：

| 场景特征 | 推荐会话类型 |
|---------|-------------|
| 实时性要求高（<10ms） | 临时会话 |
| 消息不能丢失 | 持久会话 |
| 需要离线消息重放 | 持久会话 |
| 高频数据上报（>1K/s） | 临时会话 |
| 关键业务指令 | 持久会话 |
| 设备心跳 | 临时会话 |

## 未来计划

在后续版本中，我们计划支持：
- ✅ 根据客户端参数自动选择会话类型（Clean Session flag）
- ✅ 针对特定主题的会话策略
- ✅ 会话类型热切换
- ✅ 更细粒度的会话配置

## 参考文档

- [MQTT 协议规范](https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html)
- [RocksDB 性能调优](https://github.com/facebook/rocksdb/wiki/RocksDB-Tuning-Guide)
- [Raft 共识算法](https://raft.github.io/)
