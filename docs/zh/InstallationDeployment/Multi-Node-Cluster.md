# 多节点集群部署

本指南介绍如何部署 RobustMQ 多节点集群，适用于生产环境。

## 准备工作

### 系统要求

- **节点数量**: 建议至少 3 个节点（奇数个节点）
- **操作系统**: Linux (推荐 Ubuntu 20.04+ 或 CentOS 8+)
- **内存**: 每个节点至少 4GB
- **磁盘**: 每个节点至少 50GB 可用空间
- **网络**: 节点间网络延迟 < 10ms

### 下载二进制包

在每个节点上执行：

```bash
# 下载最新版本
wget https://github.com/robustmq/robustmq/releases/download/v1.0.0/robustmq-v1.0.0-linux-amd64.tar.gz

# 解压
tar -xzf robustmq-v1.0.0-linux-amd64.tar.gz
cd robustmq-v1.0.0-linux-amd64
```

## 集群配置

### 节点规划

假设部署 3 个节点的集群：

| 节点 | IP 地址 | 角色 | gRPC 端口 |
|------|---------|------|-----------|
| node1 | 192.168.1.10 | meta, broker | 1228 |
| node2 | 192.168.1.11 | meta, broker | 1228 |
| node3 | 192.168.1.12 | meta, broker | 1228 |

### 节点 1 配置 (config/node1.toml)

```toml
cluster_name = "robustmq-cluster"
broker_id = 1
roles = ["meta", "broker"]
grpc_port = 1228
meta_addrs = { 
    1 = "192.168.1.10:1228",
    2 = "192.168.1.11:1228", 
    3 = "192.168.1.12:1228"
}

[rocksdb]
data_path = "./data/broker/data"
max_open_files = 10000

[p_prof]
enable = false
port = 6777
frequency = 1000

[log]
log_config = "./config/logger.toml"
log_path = "./data/broker/logs"
```

### 节点 2 配置 (config/node2.toml)

```toml
cluster_name = "robustmq-cluster"
broker_id = 2
roles = ["meta", "broker"]
grpc_port = 1228
meta_addrs = { 
    1 = "192.168.1.10:1228",
    2 = "192.168.1.11:1228", 
    3 = "192.168.1.12:1228"
}

[rocksdb]
data_path = "./data/broker/data"
max_open_files = 10000

[p_prof]
enable = false
port = 6777
frequency = 1000

[log]
log_config = "./config/logger.toml"
log_path = "./data/broker/logs"
```

### 节点 3 配置 (config/node3.toml)

```toml
cluster_name = "robustmq-cluster"
broker_id = 3
roles = ["meta", "broker"]
grpc_port = 1228
meta_addrs = { 
    1 = "192.168.1.10:1228",
    2 = "192.168.1.11:1228", 
    3 = "192.168.1.12:1228"
}

[rocksdb]
data_path = "./data/broker/data"
max_open_files = 10000

[p_prof]
enable = false
port = 6777
frequency = 1000

[log]
log_config = "./config/logger.toml"
log_path = "./data/broker/logs"
```

## 启动集群

### 按顺序启动节点

**重要**: 必须按顺序启动节点，先启动所有 meta 节点。

```bash
# 在节点 1 上启动
./bin/broker-server start config/node1.toml

# 在节点 2 上启动
./bin/broker-server start config/node2.toml

# 在节点 3 上启动
./bin/broker-server start config/node3.toml
```

### 后台启动

```bash
# 在节点 1 上后台启动
nohup ./bin/broker-server start config/node1.toml > node1.log 2>&1 &

# 在节点 2 上后台启动
nohup ./bin/broker-server start config/node2.toml > node2.log 2>&1 &

# 在节点 3 上后台启动
nohup ./bin/broker-server start config/node3.toml > node3.log 2>&1 &
```

## 验证集群

### 检查节点状态

在每个节点上检查服务状态：

```bash
# 检查 gRPC 端口
netstat -tlnp | grep 1228

# 检查进程
ps aux | grep broker-server
```

### 测试 MQTT 连接

```bash
# 连接到任意节点发送消息
mqttx pub -h 192.168.1.10 -p 1883 -t "test/cluster" -m "Message to cluster"

# 在另一个节点订阅消息
mqttx sub -h 192.168.1.11 -p 1883 -t "test/cluster"
```

### 验证集群一致性

```bash
# 在节点 1 发送消息
mqttx pub -h 192.168.1.10 -p 1883 -t "test/consistency" -m "Test message"

# 在节点 2 和节点 3 都应该能收到消息
mqttx sub -h 192.168.1.11 -p 1883 -t "test/consistency"
mqttx sub -h 192.168.1.12 -p 1883 -t "test/consistency"
```

## 集群管理

### 查看集群状态

```bash
# 查看集群配置
./bin/robust-ctl cluster config get --server 192.168.1.10:8080
```

### 停止集群

```bash
# 停止所有节点
pkill -f "broker-server"

# 或者分别停止每个节点
kill $(ps aux | grep "node1.toml" | grep -v grep | awk '{print $2}')
kill $(ps aux | grep "node2.toml" | grep -v grep | awk '{print $2}')
kill $(ps aux | grep "node3.toml" | grep -v grep | awk '{print $2}')
```

## 端口说明

| 服务 | 端口 | 描述 |
|------|------|------|
| MQTT | 1883 | MQTT 协议端口 |
| Admin | 8080 | 管理接口端口 |
| gRPC | 1228 | 集群通信端口 |
| pprof | 6777 | 性能分析端口 |
