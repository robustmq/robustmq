# Placement-Center 配置说明
##  集群名称
```
# 定义集群名称，用于标识当前集群, 默认名字placement-center
cluster_name = "placement-test"
```

## 节点相关配置
```
[node]
# 定义节点ID，用于唯一标识当前节点，默认1
node_id = 1

# 定义节点的网络地址，默认127.0.0.1
addr = "127.0.0.1"

# 定义节点列表，包含节点ID和对应的gRPC地址
nodes = { 1 = "127.0.0.1:1228" }
```
## 网络相关配置
```
[network]
# 定义节点的gRPC服务端口
grpc_port = 1228

# 定义节点的HTTP服务端口，用于获取集群状态,节点等信息
http_port = 1227
```
## 进程相关参数
```
[system]
# 定义运行时线程数量 runtime_work_threads * 2
runtime_work_threads = 100
```
## Broker 节点心跳检测相关参数
```
[heartbeat]
# 定义心跳超时时间，单位为毫秒，用于检测节点故障，默认30000
heartbeat_timeout_ms = 30000

# 定义心跳检查时间间隔，单位为毫秒，用于定期检查节点状态，默认1000
heartbeat_check_time_ms = 1000
```

## RocksDB 相关配置
```
[rocksdb]
# 定义数据存储路径，rocksdb
data_path = "./robust-data/placement-center/data"

# 配置RocksDB数据库的最大打开文件数,支持大量并发读取操作，默认10000
max_open_files = 10000

```
## 日志配置，指定日志路径和配置文件
```
[log]
# 日志配置文件路径，默认./config/log4rs.yaml
log_config = "./config/log4rs.yaml"

# 日志文件存储目录，默认目录./logs/placement-center
log_path = "./robust-data/placement-center/logs"
```
