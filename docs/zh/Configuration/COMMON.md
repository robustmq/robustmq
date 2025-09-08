# RobustMQ 配置文件通用指南

## 概述

RobustMQ 使用 TOML 格式的配置文件来管理系统配置。配置文件位于 `config/` 目录下，主要配置文件为 `server.toml`。

## 配置文档导航

- 🔧 **[MQTT 配置](MQTT.md)** - MQTT Broker 相关配置
- 🏗️ **[Meta 配置](META.md)** - 元数据服务配置
- 📝 **[Journal 配置](JOURNAL.md)** - 日志引擎配置

---

## 配置文件结构

### 主配置文件
- **`config/server.toml`** - 主要配置文件
- **`config/server.toml.template`** - 配置模板文件
- **`config/server-tracing.toml`** - 日志追踪配置

### 配置加载优先级
1. 命令行参数
2. 环境变量
3. 配置文件
4. 默认值

---

## 基础配置

### 集群基础信息
```toml
# 集群名称
cluster_name = "robust_mq_cluster_default"

# 节点 ID
broker_id = 1

# 节点角色
roles = ["meta", "broker", "journal"]

# gRPC 端口
grpc_port = 1228

# 元数据中心地址
[meta_addrs]
1 = "127.0.0.1:1228"
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `cluster_name` | `string` | `"robust_mq_cluster_default"` | 集群名称，集群内所有节点必须相同 |
| `broker_id` | `u64` | `1` | 节点唯一标识符 |
| `roles` | `array` | `["meta", "broker"]` | 节点角色：meta(元数据)、broker(MQTT)、journal(日志) |
| `grpc_port` | `u32` | `1228` | gRPC 服务端口 |
| `meta_addrs` | `table` | `{1 = "127.0.0.1:1228"}` | 元数据中心节点地址映射 |

---

## 运行时配置

### Runtime 配置
```toml
[runtime]
runtime_worker_threads = 4    # 运行时工作线程数
tls_cert = "./config/certs/cert.pem"  # TLS 证书路径
tls_key = "./config/certs/key.pem"    # TLS 私钥路径
```

### Network 配置
```toml
[network]
accept_thread_num = 1         # 接受连接线程数
handler_thread_num = 1        # 处理线程数  
response_thread_num = 1       # 响应线程数
queue_size = 1000            # 队列大小
lock_max_try_mut_times = 30   # 锁最大尝试次数
lock_try_mut_sleep_time_ms = 50  # 锁尝试睡眠时间(毫秒)
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `runtime_worker_threads` | `usize` | 自动检测 | Tokio 运行时工作线程数 |
| `tls_cert` | `string` | `"./config/certs/cert.pem"` | TLS 证书文件路径 |
| `tls_key` | `string` | `"./config/certs/key.pem"` | TLS 私钥文件路径 |
| `accept_thread_num` | `usize` | `1` | 接受新连接的线程数 |
| `handler_thread_num` | `usize` | `1` | 处理请求的线程数 |
| `response_thread_num` | `usize` | `1` | 发送响应的线程数 |
| `queue_size` | `usize` | `1000` | 内部队列大小 |

---

## 存储配置

### RocksDB 配置
```toml
[rocksdb]
data_path = "./data"          # 数据存储路径
max_open_files = 10000       # 最大打开文件数
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `data_path` | `string` | `"./data"` | RocksDB 数据存储目录 |
| `max_open_files` | `i32` | `10000` | RocksDB 最大打开文件数 |

---

## 监控配置

### Prometheus 配置
```toml
[prometheus]
enable = true                 # 是否启用 Prometheus 指标
port = 9090                  # Prometheus 指标端口
```

### PProf 配置
```toml
[p_prof]
enable = false               # 是否启用性能分析
port = 6060                 # PProf 服务端口
frequency = 100             # 采样频率
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `prometheus.enable` | `bool` | `true` | 是否启用 Prometheus 指标收集 |
| `prometheus.port` | `u32` | `9090` | Prometheus 指标暴露端口 |
| `p_prof.enable` | `bool` | `false` | 是否启用 PProf 性能分析 |
| `p_prof.port` | `u16` | `6060` | PProf 服务端口 |
| `p_prof.frequency` | `i32` | `100` | PProf 采样频率 |

---

## 日志配置

### Log 配置
```toml
[log]
log_config = "./config/server-tracing.toml"  # 日志配置文件路径
log_path = "./data/broker/logs"              # 日志输出目录
```

### 配置说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `log_config` | `string` | `"./config/broker-tracing.toml"` | 日志追踪配置文件路径 |
| `log_path` | `string` | `"./logs"` | 日志文件输出目录 |

---

## 环境变量覆盖

RobustMQ 支持使用环境变量覆盖配置文件中的设置。环境变量命名规则：

### 命名规则
```
{PREFIX}_{SECTION}_{KEY}
```

### 示例
```bash
# 覆盖集群名称
export ROBUSTMQ_CLUSTER_NAME="my-cluster"

# 覆盖 MQTT TCP 端口
export ROBUSTMQ_MQTT_SERVER_TCP_PORT=1884

# 覆盖日志路径
export ROBUSTMQ_LOG_LOG_PATH="/var/log/robustmq"

# 覆盖 Prometheus 端口
export ROBUSTMQ_PROMETHEUS_PORT=9091
```

### 环境变量优先级
环境变量的值会覆盖配置文件中的对应设置，优先级高于配置文件。

---

## 配置文件示例

### 基本配置示例
```toml
# 基础配置
cluster_name = "production-cluster"
broker_id = 1
roles = ["meta", "broker"]
grpc_port = 1228

# 元数据中心地址
[placement_center]
1 = "192.168.1.10:1228"
2 = "192.168.1.11:1228" 
3 = "192.168.1.12:1228"

# 运行时配置
[runtime]
runtime_worker_threads = 8
tls_cert = "./config/certs/prod-cert.pem"
tls_key = "./config/certs/prod-key.pem"

# 网络配置
[network]
accept_thread_num = 4
handler_thread_num = 8
response_thread_num = 4
queue_size = 2000

# 存储配置
[rocksdb]
data_path = "/data/robustmq"
max_open_files = 20000

# 监控配置
[prometheus]
enable = true
port = 9090

# 日志配置
[log]
log_config = "./config/production-tracing.toml"
log_path = "/var/log/robustmq"
```

---

## 配置验证

### 配置文件语法检查
```bash
# 检查 TOML 语法
cargo run --bin config-validator config/server.toml
```

### 常见配置错误
1. **TOML 语法错误** - 缺少引号、括号不匹配等
2. **端口冲突** - 多个服务使用相同端口
3. **路径不存在** - 证书文件、日志目录不存在
4. **权限问题** - 数据目录没有写权限

---

## 最佳实践

### 生产环境配置建议
1. **安全性**:
   - 使用强密码和证书
   - 限制网络访问权限
   - 启用 TLS 加密

2. **性能优化**:
   - 根据硬件配置调整线程数
   - 合理设置队列大小
   - 优化存储路径配置

3. **监控和日志**:
   - 启用 Prometheus 监控
   - 配置合适的日志级别
   - 设置日志轮转策略

4. **高可用性**:
   - 配置多个元数据中心节点
   - 设置合适的心跳超时时间
   - 配置数据备份策略

---

## 故障排除

### 常见问题
1. **配置加载失败** - 检查 TOML 语法和文件路径
2. **端口绑定失败** - 检查端口是否被占用
3. **证书加载失败** - 检查证书文件路径和格式
4. **存储初始化失败** - 检查数据目录权限和磁盘空间

### 调试方法
1. 启用详细日志输出
2. 检查配置加载日志
3. 使用配置验证工具
4. 查看系统资源使用情况

---

*文档版本: v1.0*  
*最后更新: 2024-01-01*  
*基于代码版本: RobustMQ v0.1.31*
