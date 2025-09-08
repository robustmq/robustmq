# Journal 引擎配置说明

> 本文档介绍 Journal 引擎相关的所有配置项。通用配置信息请参考 [COMMON.md](COMMON.md)。

## Journal 引擎概述

Journal 引擎是 RobustMQ 的持久化存储层，负责：
- 消息持久化存储
- 高性能日志写入
- 数据分片和副本管理
- 存储空间管理

---

## Journal 服务器配置

### 服务器端口配置
```toml
[journal.server]
tcp_port = 1778              # Journal 服务 TCP 端口
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `tcp_port` | `u32` | `1778` | Journal 引擎服务监听端口 |

---

## Journal 运行时配置

### 运行时配置
```toml
[journal.runtime]
enable_auto_create_shard = true   # 是否自动创建分片
shard_replica_num = 2            # 分片副本数量
max_segment_size = 1073741824    # 最大段文件大小(字节)
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enable_auto_create_shard` | `bool` | `true` | 是否自动创建新的数据分片 |
| `shard_replica_num` | `u32` | `2` | 每个分片的副本数量 |
| `max_segment_size` | `u32` | `1073741824` | 单个段文件最大大小（字节，默认1GB） |

### 分片管理说明
- **自动分片**: 当启用时，系统会根据数据量自动创建新分片
- **副本数量**: 建议设置为奇数，以便进行故障恢复投票
- **段大小**: 影响I/O性能和存储效率，建议根据硬件配置调整

---

## Journal 存储配置

### 存储配置
```toml
[journal.storage]
data_path = [
    "./data/journal/data1",
    "./data/journal/data2",
    "./data/journal/data3"
]
rocksdb_max_open_files = 10000   # RocksDB 最大打开文件数
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `data_path` | `array` | `["./data/journal/"]` | 数据存储路径列表，支持多路径 |
| `rocksdb_max_open_files` | `i32` | `10000` | RocksDB 最大同时打开的文件数 |

### 多路径存储说明
- **负载均衡**: 数据会在多个路径间均衡分布
- **性能提升**: 可以利用多个磁盘的I/O能力
- **容错能力**: 单个磁盘故障不会影响整个系统

---

## Journal 完整配置示例

### 开发环境配置
```toml
# 基础配置
cluster_name = "dev-cluster"
broker_id = 1
roles = ["journal"]
grpc_port = 1228

# Journal 服务器配置
[journal.server]
tcp_port = 1778

# Journal 运行时配置
[journal.runtime]
enable_auto_create_shard = true
shard_replica_num = 1          # 开发环境单副本
max_segment_size = 104857600   # 100MB，适合开发测试

# Journal 存储配置
[journal.storage]
data_path = ["./data/journal"]
rocksdb_max_open_files = 5000
```

### 生产环境配置
```toml
# 基础配置
cluster_name = "production-cluster"
broker_id = 1
roles = ["journal"]
grpc_port = 1228

# Journal 服务器配置
[journal.server]
tcp_port = 1778

# Journal 运行时配置
[journal.runtime]
enable_auto_create_shard = true
shard_replica_num = 3          # 生产环境三副本
max_segment_size = 2147483648  # 2GB，适合高吞吐量

# Journal 存储配置
[journal.storage]
data_path = [
    "/data1/robustmq/journal",
    "/data2/robustmq/journal",
    "/data3/robustmq/journal",
    "/data4/robustmq/journal"
]
rocksdb_max_open_files = 50000
```

### 高性能配置
```toml
# 高性能 Journal 配置
[journal.runtime]
enable_auto_create_shard = true
shard_replica_num = 3
max_segment_size = 4294967296  # 4GB，减少文件切换频率

[journal.storage]
data_path = [
    "/nvme1/robustmq/journal",  # 使用 NVMe SSD
    "/nvme2/robustmq/journal",
    "/nvme3/robustmq/journal",
    "/nvme4/robustmq/journal"
]
rocksdb_max_open_files = 100000

# 网络优化
[network]
handler_thread_num = 32        # 增加处理线程
response_thread_num = 16       # 增加响应线程
queue_size = 20000            # 增大队列容量
```

---

## 环境变量覆盖示例

### Journal 相关环境变量
```bash
# Journal 服务器配置
export ROBUSTMQ_JOURNAL_SERVER_TCP_PORT=1778

# Journal 运行时配置
export ROBUSTMQ_JOURNAL_RUNTIME_ENABLE_AUTO_CREATE_SHARD=true
export ROBUSTMQ_JOURNAL_RUNTIME_SHARD_REPLICA_NUM=3
export ROBUSTMQ_JOURNAL_RUNTIME_MAX_SEGMENT_SIZE=2147483648

# Journal 存储配置
export ROBUSTMQ_JOURNAL_STORAGE_ROCKSDB_MAX_OPEN_FILES=50000
```

---

## 性能调优建议

### 写入性能优化
```toml
[journal.runtime]
max_segment_size = 4294967296    # 增大段文件大小，减少文件切换

[journal.storage]
rocksdb_max_open_files = 100000  # 增加文件句柄数
data_path = [
    "/ssd1/journal",             # 使用高速SSD
    "/ssd2/journal"
]

[network]
handler_thread_num = 16          # 增加处理线程数
queue_size = 50000              # 增大队列容量
```

### 存储空间优化
```toml
[journal.runtime]
max_segment_size = 536870912     # 512MB，适中的段大小
shard_replica_num = 2           # 适当的副本数

[journal.storage]
data_path = ["/data/journal"]    # 单路径，节省空间
```

### 可靠性优化
```toml
[journal.runtime]
enable_auto_create_shard = true
shard_replica_num = 3           # 三副本保证可靠性
max_segment_size = 1073741824   # 1GB 段大小

[journal.storage]
data_path = [
    "/raid1/journal",           # 使用 RAID 存储
    "/raid2/journal"
]
rocksdb_max_open_files = 30000
```

---

## 数据管理

### 分片策略
1. **自动分片**: 系统根据数据量自动创建新分片
2. **手动分片**: 可以通过 API 手动创建分片
3. **分片大小**: 由 `max_segment_size` 控制

### 副本管理
1. **副本数量**: 由 `shard_replica_num` 控制
2. **副本分布**: 系统自动在不同节点间分布副本
3. **数据同步**: 使用异步复制保证数据一致性

### 数据清理
- 自动清理过期数据
- 支持手动触发压缩
- 定期进行垃圾回收

---

## 监控和运维

### 关键监控指标
- 写入吞吐量（消息/秒）
- 存储使用量（GB）
- 分片数量和分布
- 副本同步延迟
- 磁盘I/O使用率

### 运维操作
1. **数据备份**: 定期备份数据目录
2. **扩容操作**: 添加新的存储路径
3. **性能监控**: 监控磁盘和网络使用情况
4. **故障恢复**: 从副本恢复数据

---

## 故障排除

### 常见问题
1. **写入性能差**
   - 检查磁盘I/O性能
   - 调整段文件大小
   - 增加处理线程数

2. **存储空间不足**
   - 清理过期数据
   - 添加新的存储路径
   - 调整数据保留策略

3. **副本同步延迟**
   - 检查网络带宽
   - 调整副本数量
   - 优化网络配置

4. **服务启动失败**
   - 检查端口占用情况
   - 验证存储路径权限
   - 确认配置文件语法

### 调试配置
```toml
# 启用详细日志
[log]
log_config = "./config/debug-tracing.toml"

# 减小段文件大小便于调试
[journal.runtime]
max_segment_size = 10485760  # 10MB

# 启用性能分析
[p_prof]
enable = true
port = 6060
```

---

*文档版本: v1.0*  
*最后更新: 2024-01-01*  
*基于代码版本: RobustMQ v0.1.31*
