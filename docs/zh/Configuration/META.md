# Meta 服务配置说明

> 本文档介绍 Meta 服务（元数据服务/Placement Center）相关的所有配置项。通用配置信息请参考 [COMMON.md](COMMON.md)。

## Meta 服务概述

Meta 服务（也称为 Meta Service）是 RobustMQ 的元数据管理服务，负责：
- 集群节点管理
- 元数据存储和同步
- 服务发现
- 配置管理

---

## Meta 运行时配置

### 运行时配置
```toml
[place.runtime]
heartbeat_timeout_ms = 30000      # 心跳超时时间(毫秒)
heartbeat_check_time_ms = 1000    # 心跳检查间隔(毫秒)
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `heartbeat_timeout_ms` | `u64` | `30000` | 节点心跳超时时间（毫秒） |
| `heartbeat_check_time_ms` | `u64` | `1000` | 心跳检查间隔时间（毫秒） |

### 心跳机制说明
- **心跳超时**: 当节点在指定时间内未发送心跳，将被标记为不可用
- **心跳检查**: 定期检查所有节点的心跳状态
- **故障转移**: 当主节点故障时，自动选举新的主节点

---

## Meta 存储配置

### RocksDB 存储配置
```toml
[rocksdb]
data_path = "./data/meta-service/data"  # 数据存储路径
max_open_files = 10000                      # 最大打开文件数
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `data_path` | `string` | `"./data"` | Meta 服务数据存储目录 |
| `max_open_files` | `i32` | `10000` | RocksDB 最大同时打开的文件数 |

---

## Meta 集群配置

### 集群节点配置
```toml
# 元数据中心节点地址配置
[meta_service]
1 = "127.0.0.1:1228"
2 = "127.0.0.1:1229"
3 = "127.0.0.1:1230"
```

### 配置说明

| 配置项 | 类型 | 说明 |
|--------|------|------|
| `meta_service` | `table` | 元数据中心节点地址映射表 |
| 键 | `string` | 节点ID（通常与 broker_id 对应） |
| 值 | `string` | 节点地址，格式为 "IP:端口" |

### 集群部署模式
1. **单节点模式**: 只配置一个节点，适用于测试和开发
2. **多节点模式**: 配置多个节点，提供高可用性
3. **推荐配置**: 生产环境建议使用奇数个节点（3、5、7个）

---

## Meta 服务角色配置

### 角色配置
```toml
roles = ["meta", "broker"]    # 节点角色列表
```

### 角色说明
- **meta**: 元数据服务角色
- **broker**: MQTT Broker 角色
- **journal**: Journal 引擎角色

### 部署模式
1. **一体化部署**: `roles = ["meta", "broker", "journal"]`
2. **分离式部署**:
   - Meta 节点: `roles = ["meta"]`
   - Broker 节点: `roles = ["broker"]`
   - Journal 节点: `roles = ["journal"]`

---

## Meta 服务完整配置示例

### 单节点配置
```toml
# 基础配置
cluster_name = "dev-cluster"
broker_id = 1
roles = ["meta", "broker"]
grpc_port = 1228

# 元数据中心配置
[meta_service]
1 = "127.0.0.1:1228"

# Meta 运行时配置
[place.runtime]
heartbeat_timeout_ms = 10000
heartbeat_check_time_ms = 2000

# 存储配置
[rocksdb]
data_path = "./data/meta"
max_open_files = 5000
```

### 高可用集群配置
```toml
# 基础配置
cluster_name = "production-cluster"
broker_id = 1
roles = ["meta"]
grpc_port = 1228

# 元数据中心集群配置
[meta_service]
1 = "192.168.1.10:1228"
2 = "192.168.1.11:1228"
3 = "192.168.1.12:1228"

# Meta 运行时配置
[place.runtime]
heartbeat_timeout_ms = 30000
heartbeat_check_time_ms = 5000

# 存储配置
[rocksdb]
data_path = "/data/robustmq/meta"
max_open_files = 20000
```

---

## 环境变量覆盖示例

### Meta 相关环境变量
```bash
# 集群基础配置
export ROBUSTMQ_CLUSTER_NAME="production-cluster"
export ROBUSTMQ_BROKER_ID=1
export ROBUSTMQ_GRPC_PORT=1228

# Meta 运行时配置
export ROBUSTMQ_PLACE_RUNTIME_HEARTBEAT_TIMEOUT_MS=30000
export ROBUSTMQ_PLACE_RUNTIME_HEARTBEAT_CHECK_TIME_MS=5000

# 存储配置
export ROBUSTMQ_ROCKSDB_DATA_PATH="/data/robustmq/meta"
export ROBUSTMQ_ROCKSDB_MAX_OPEN_FILES=20000
```

---

## 性能调优建议

### 高并发场景
```toml
[place.runtime]
heartbeat_timeout_ms = 15000      # 缩短心跳超时
heartbeat_check_time_ms = 3000    # 增加心跳检查频率

[rocksdb]
max_open_files = 50000           # 增加文件句柄数

[network]
handler_thread_num = 16          # 增加处理线程数
queue_size = 10000              # 增加队列大小
```

### 低延迟场景
```toml
[place.runtime]
heartbeat_check_time_ms = 500    # 更频繁的心跳检查

[network]
lock_max_try_mut_times = 10      # 减少锁重试次数
lock_try_mut_sleep_time_ms = 5   # 减少锁睡眠时间
```

### 高可靠性场景
```toml
[place.runtime]
heartbeat_timeout_ms = 60000     # 增加心跳容忍度
heartbeat_check_time_ms = 10000  # 适中的检查频率

[rocksdb]
data_path = "/data/robustmq/meta"  # 使用持久化存储路径
max_open_files = 30000             # 足够的文件句柄
```

---

## 集群管理

### 节点发现
Meta 服务使用 `meta_service` 配置进行节点发现：
1. 每个节点启动时会连接到配置的所有 Meta 节点
2. 通过心跳机制维持节点状态
3. 自动处理节点故障和恢复

### 数据一致性
- 使用 Raft 协议保证数据一致性
- 支持多副本数据存储
- 自动处理脑裂和网络分区

### 故障恢复
- 自动检测节点故障
- 支持节点动态加入和离开
- 数据自动同步和恢复

---

## 故障排除

### 常见问题
1. **节点无法加入集群**
   - 检查 `meta_service` 配置
   - 验证网络连通性
   - 确认端口未被占用

2. **心跳超时**
   - 调整 `heartbeat_timeout_ms`
   - 检查网络延迟
   - 优化系统负载

3. **数据存储问题**
   - 检查 `data_path` 目录权限
   - 验证磁盘空间
   - 调整 `max_open_files`

4. **选举失败**
   - 确保集群节点数为奇数
   - 检查节点时间同步
   - 验证网络分区情况

### 调试方法
```toml
# 启用详细日志
[log]
log_config = "./config/debug-tracing.toml"

# 缩短心跳间隔便于调试
[place.runtime]
heartbeat_check_time_ms = 1000
```

---

## 监控指标

Meta 服务提供以下监控指标：
- 节点健康状态
- 心跳延迟统计
- 数据同步状态
- 选举状态变化
- 存储使用情况

---

*文档版本: v1.0*
*最后更新: 2024-01-01*
*基于代码版本: RobustMQ v0.1.31*
