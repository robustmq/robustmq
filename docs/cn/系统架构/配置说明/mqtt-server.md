##### 默认的配置文件路径 config/mqtt-server.toml,可以通过参数 --conf 指定配置文件路径
```
mqtt-server --conf config/mqtt-server.toml
```

##### 配置文件示例
```
# 定义MQTT Broker集群的名称，用于标识和管理
cluster_name = "mqtt-broker"

# 设置当前Broker的唯一标识符，用于集群内区分不同的Broker节点
broker_id = 1

# gRPC服务的端口号，默认9981
grpc_port = 9981

# HTTP服务的端口号，默认9982，提供可以查询当前运行状态
http_port = 9982

# 元数据服务地址,可以配置多个
placement_center = ["127.0.0.1:1228"]
```

##### 网络配置
```
[network]
# MQTT协议, 默认1883和8883
tcp_port = 1883
tcps_port = 8883

# WebSocket协议, 默认8083和8084
websocket_port = 8083
websockets_port = 8084

# quid udp协议, 默认908s
quic_port = 9083

# 设置tls安全通信的证书和密钥, 默认无证书
tls_cert = "./config/example/certs/cert.pem"
tls_key = "./config/example/certs/key.pem"
```

##### TCP协议相关配置
```
[tcp_thread]
# 接受客户端连接的线程数量, 默认1
accept_thread_num = 1

# 处理客户端请求的线程数量, 默认1
handler_thread_num = 10

# 发送响应给客户端的线程数量, 默认1
response_thread_num = 1

# 最大连接数限制, 默认1000
max_connection_num = 1000

# 请求队列大小, 默认2000
request_queue_size = 2000

# 响应队列大小, 默认2000
response_queue_size = 2000

# 尝试获取锁的最大次数, 默认30
lock_max_try_mut_times = 30

# 尝试获取锁之间的时间间隔（毫秒）, 默认50
lock_try_mut_sleep_time_ms = 50
```

##### 系统配置
```
[system]
# 运行时工作线程数, 默认16
runtime_worker_threads = 128
# 默认用户名
default_user = "admin"
# 默认robustmq
default_password = "pwd123"
```

##### 存储配置
```
[storage]
# 存储类型, 默认为memory, 支持memory, mysql
storage_type = "memory"
mysql_addr = ""
```

##### 认证配置
```
[auth]
storage_type = "placement"
journal_addr = ""
mysql_addr = ""
```

##### 日志配置
```
[log]
# 日志配置文件路径, 默认./config/log4rs.yaml
log_config = "./config/log4rs.yaml"
# 日志文件保存路径, 当前./logs目录
log_path = "/tmp/robust/mqtt-broker/logs"
```