# MQTT Broker Command

> [!NOTE]
> 当前命令行功能只完成了单集群内容的查询，后续会支持多集群查询功能，如果有感兴趣的小伙伴也可以帮忙构建。

当前Mqtt Broker 命令行工具已经提供了多种功能，你可以从下面的表格当中大致了解一下当前命令行工具已经
支持的功能。

> [!NOTE]
> 如果你有想要支持的功能可以在我们的 GitHub 仓库当中提一个 Issue, 同时也欢迎大家帮忙构建新的功能。

| 命令                    | 功能描述              |
|-----------------------|-------------------|
| `status`              | 查看集群状态以及一些相关的内容信息 |
| `config`              | 查看当前运行集群的内部配置     |
| `session`             | 会话管理              |
| `subscribe`           | 订阅管理              |
| `user`                | 用户管理              |
| `acl`                 | 访问控制列表管理          |
| `blacklist`           | 黑名单管理             |
| `flapping-detect`     | 消息抖动管理            |
| `connection`          | 连接管理              |
| `slow-subscribe`      | 慢订阅管理             |
| `system-alarm`        | 系统告警管理            |
| `topic`               | 主题管理-待修复          |
| `topic-rewrite-rule`  | 主题重写规则            |
| `connector`           | connector管理       |
| `schema`              | schema管理          |
| `auto-subscribe-rule` | 自动订阅规则            |
| `publish`             | publish消息         |
| `subscribe`           | 订阅主题              |


## 1. 集群状态

MQTT Broker 提供了集群状态查询功能，可以通过命令行工具查看集群的健康状态、节点信息等。

```console
% ./bin/robust-ctl mqtt status
cluster_name: broker-server
message_in_rate: 10
message_out_rate: 3
connection_num: 0
session_num: 0
topic_num: 64
nodes: [BrokerNodeRaw { cluster_name: "broker-server", cluster_type: "MQTTBrokerServer", extend_info: "{\"grpc_addr\":\"192.168.18.248:1228\",\"mqtt_addr\":\"192.168.18.248:1883\",\"mqtts_addr\":\"192.168.18.248:1884\",\"websocket_addr\":\"192.168.18.248:8083\",\"websockets_addr\":\"192.168.18.248:8084\",\"quic_addr\":\"192.168.18.248:9083\"}", node_id: 1, node_ip: "192.168.18.248", node_inner_addr: "192.168.18.248:1228", start_time: "2025-08-21 20:09:00", register_time: "2025-08-21 20:09:06" }]
placement_status: {"running_state":{"Ok":null},"id":1,"current_term":2,"vote":{"leader_id":{"term":2,"node_id":1},"committed":true},"last_log_index":69,"last_applied":{"leader_id":{"term":2,"node_id":1},"index":69},"snapshot":null,"purged":null,"state":"Leader","current_leader":1,"millis_since_quorum_ack":0,"last_quorum_acked":1755778493226613667,"membership_config":{"log_id":{"leader_id":{"term":0,"node_id":0},"index":0},"membership":{"configs":[[1]],"nodes":{"1":{"node_id":1,"rpc_addr":"127.0.0.1:1228"}}}},"heartbeat":{"1":1755778492849902083},"replication":{"1":{"leader_id":{"term":2,"node_id":1},"index":69}}}
tcp_connection_num: 0
tls_connection_num: 0
websocket_connection_num: 0
quic_connection_num: 0
subscribe_num: 0
exclusive_subscribe_num: 0
share_subscribe_leader_num: 0
share_subscribe_resub_num: 0
exclusive_subscribe_thread_num: 0
share_subscribe_leader_thread_num: 0
share_subscribe_follower_thread_num: 0
connection_num:[{"date":1755778388,"value":0},{"value":0,"date":1755778148},{"date":1755778448,"value":0},{"date":1755778268,"value":0},{"date":1755778208,"value":0},{"date":1755778328,"value":0}]
topic_num: [{"value":64,"date":1755778208},{"date":1755778328,"value":64},{"value":64,"date":1755778268},{"date":1755778148,"value":64},{"date":1755778388,"value":64},{"value":64,"date":1755778448}]
subscribe_num: [{"date":1755778388,"value":0},{"date":1755778208,"value":0},{"value":0,"date":1755778328},{"value":0,"date":1755778148},{"value":0,"date":1755778268},{"date":1755778448,"value":0}]
message_in_num: [{"date":1755778388,"value":1000},{"date":1755778268,"value":1000},{"date":1755778148,"value":1000},{"value":1000,"date":1755778328},{"value":1000,"date":1755778448},{"value":1000,"date":1755778208}]
message_out_num: [{"date":1755778208,"value":1000},{"value":1000,"date":1755778388},{"date":1755778148,"value":1000},{"value":1000,"date":1755778328},{"value":1000,"date":1755778448},{"date":1755778268,"value":1000}]
message_drop_num: [{"date":1755778208,"value":30},{"date":1755778328,"value":30},{"value":30,"date":1755778388},{"value":30,"date":1755778448},{"date":1755778268,"value":30},{"date":1755778148,"value":30}]
```

当前通过该命令可以查看到以上的信息，下面的表格主要解释每个字段的含义:

| 字段名                                 | 含义            |
|-------------------------------------|---------------|
| cluster_name                        | 集群名称          |
| message_in_rate                     | 消息接收速率        |
| message_out_rate                    | 消息发送速率        |
| connection_num                      | 当前连接数         |
| session_num                         | 当前会话数         |
| topic_num                           | 当前主题数         |
| nodes                               | 集群节点信息        |
| placement_status                    | 集群状态信息        |
| tcp_connection_num                  | TCP 连接数       |
| tls_connection_num                  | TLS 连接数       |
| websocket_connection_num            | WebSocket 连接数 |
| quic_connection_num                 | QUIC 连接数      |
| subscribe_num                       | 订阅数           |
| exclusive_subscribe_num             | 独占订阅数         |
| share_subscribe_leader_num          | 共享订阅领导者数      |
| share_subscribe_resub_num           | 共享订阅重新订阅数     |
| exclusive_subscribe_thread_num      | 独占订阅线程数       |
| share_subscribe_leader_thread_num   | 共享订阅领导者线程数    |
| share_subscribe_follower_thread_num | 共享订阅跟随者线程数    |
| connection_num                      | 连接数统计         |
| topic_num                           | 主题数统计         |
| subscribe_num                       | 订阅数统计         |
| message_in_num                      | 消息接收数统计       |
| message_out_num                     | 消息发送数统计       |
| message_drop_num                    | 消息丢弃数统计       |

## 2. 配置查询

MQTT Broker 提供了配置查询功能，由于在整体使用的过程中，我们可能会通过命令行进行配置的修改，
这些配置的修改可能会导致集群内部的配置信息和配置文件的信息产生不一致的情况，因此我们提供了配置查询功能，
可以通过命令行工具查看当前集群的配置信息。

```console
% ./bin/robust-ctl mqtt config get
{
  "cluster_name": "broker-server",
  "broker_id": 1,
  "roles": [
    "meta",
    "broker"
  ],
  "grpc_port": 1228,
  "placement_center": {
    "1": "127.0.0.1:1228"
  },
  "prometheus": {
    "enable": true,
    "port": 9090
  },
  "log": {
    "log_config": "./config/server-tracing.toml",
    "log_path": "./data/broker/logs"
  },
  "runtime": {
    "runtime_worker_threads": 4,
    "tls_cert": "./config/certs/cert.pem",
    "tls_key": "./config/certs/key.pem"
  },
  "network": {
    "accept_thread_num": 1,
    "handler_thread_num": 1,
    "response_thread_num": 1,
    "queue_size": 1000,
    "lock_max_try_mut_times": 30,
    "lock_try_mut_sleep_time_ms": 50
  },
  "p_prof": {
    "enable": false,
    "port": 6777,
    "frequency": 1000
  },
  "place_runtime": {
    "heartbeat_timeout_ms": 30000,
    "heartbeat_check_time_ms": 1000
  },
  "rocksdb": {
    "data_path": "./data/broker/data",
    "max_open_files": 10000
  },
  "journal_server": {
    "tcp_port": 1778
  },
  "journal_runtime": {
    "enable_auto_create_shard": true,
    "shard_replica_num": 2,
    "max_segment_size": 1073741824
  },
  "journal_storage": {
    "data_path": [
      "./data/journal/"
    ],
    "rocksdb_max_open_files": 10000
  },
  "mqtt_server": {
    "tcp_port": 1883,
    "tls_port": 1884,
    "websocket_port": 8083,
    "websockets_port": 8084,
    "quic_port": 9083
  },
  "mqtt_auth_storage": {
    "storage_type": "placement",
    "journal_addr": "",
    "mysql_addr": ""
  },
  "mqtt_message_storage": {
    "storage_type": "memory",
    "journal_addr": "",
    "mysql_addr": "",
    "rocksdb_data_path": "",
    "rocksdb_max_open_files": null
  },
  "mqtt_runtime": {
    "default_user": "admin",
    "default_password": "robustmq",
    "max_connection_num": 1000000
  },
  "mqtt_offline_message": {
    "enable": true,
    "expire_ms": 0,
    "max_messages_num": 0
  },
  "mqtt_slow_subscribe_config": {
    "enable": false,
    "max_store_num": 1000,
    "delay_type": "Whole"
  },
  "mqtt_flapping_detect": {
    "enable": false,
    "window_time": 1,
    "max_client_connections": 15,
    "ban_time": 5
  },
  "mqtt_protocol_config": {
    "max_session_expiry_interval": 1800,
    "default_session_expiry_interval": 30,
    "topic_alias_max": 65535,
    "max_qos": 2,
    "max_packet_size": 10485760,
    "max_server_keep_alive": 3600,
    "default_server_keep_alive": 60,
    "receive_max": 65535,
    "max_message_expiry_interval": 3600,
    "client_pkid_persistent": false
  },
  "mqtt_security": {
    "is_self_protection_status": false,
    "secret_free_login": false
  },
  "mqtt_schema": {
    "enable": true,
    "strategy": "ALL",
    "failed_operation": "Discard",
    "echo_log": true,
    "log_level": "info"
  },
  "mqtt_system_monitor": {
    "enable": false,
    "os_cpu_check_interval_ms": 60000,
    "os_cpu_high_watermark": 70.0,
    "os_cpu_low_watermark": 50.0,
    "os_memory_check_interval_ms": 60,
    "os_memory_high_watermark": 80.0
  }
}
```

========== 🏗️正在重新施工中👷 ==========

## 2. 用户管理

MQTT Broker 启用了用户验证功能，客户端在发布或订阅消息前，
必须提供有效的用户名和密码以通过验证。
未通过验证的客户端将无法与 Broker 通信。
这一功能可以增强系统的安全性，防止未经授权的访问。

### 2.1 创建用户

创建新的 MQTT Broker 用户。

```console
% ./bin/robust-ctl mqtt user create --username=testp --password=7355608 --is_superuser=false
Created successfully!
```

### 2.2 删除用户

删除已有的 MQTT Broker 用户。

```console
% ./bin/robust-ctl mqtt user delete --username=testp
Deleted successfully!
```

### 2.3 用户列表

列出所有已创建的用户。

```console
% ./bin/robust-ctl mqtt user list
+----------+--------------+
| username | is_superuser |
+----------+--------------+
| admin    | true         |
+----------+--------------+
| testp    | false        |
+----------+--------------+
```

## 3. 发布、订阅消息

### 3.1 发布 MQTT 消息

```console
% ./bin/robust-ctl mqtt --server=127.0.0.1:1883 publish --username=admin --password=pwd123 --topic=test/topic1 --qos=0
able to connect: "127.0.0.1:1883"
you can post a message on the terminal:
1
> You typed: 1
2
> You typed: 2
3
> You typed: 3
4
> You typed: 4
5
> You typed: 5
^C>  Ctrl+C detected,  Please press ENTER to end the program.
```

### 3.2 订阅 MQTT 消息

```console
% ./bin/robust-ctl mqtt --server=127.0.0.1:1883 subscribe --username=admin --password=pwd123 --topic=test/topic1 --qos=0

able to connect: "127.0.0.1:1883"
subscribe success
payload: 1
payload: 2
payload: 3
payload: 4
payload: 5
^C Ctrl+C detected,  Please press ENTER to end the program.
End of input stream.
```

### 3.3 发布保留消息

```console
% ./bin/robust-ctl mqtt --server=127.0.0.1:1883 publish --username=admin --password=pwd123 --topic=\$share/group1/test/topic1 --qos=1 --retained
able to connect: "127.0.0.1:1883"
you can post a message on the terminal:
helloworld!
> You typed: helloworld!
published retained message
```

```console
% ./bin/robust-ctl mqtt --server=127.0.0.1:1883 subscribe --username=admin --password=pwd123 --topic=\$share/group1/test/topic1 --qos=0
able to connect: "127.0.0.1:1883"
subscribe success
Retain message: helloworld!
```

## 4. ACL（访问控制列表）管理

### 4.1 创建 ACL

创建新的 ACL 规则。

```console
% ./bin/robust-ctl mqtt --server=127.0.0.1:1883 acl create --cluster-name=admin --acl=xxx
able to connect: "127.0.0.1:1883"
Created successfully!
```

### 4.2 删除 ACL

删除已有的 ACL 规则。

```console
% ./bin/robust-ctl mqtt --server=127.0.0.1:1883 acl delete --cluster-name=admin --acl=xxx
able to connect: "127.0.0.1:1883"
Deleted successfully!
```

### 4.3 ACL 列表

列出所有已创建的 ACL 规则。

```console
% ./bin/robust-ctl mqtt --server=127.0.0.1:1883 acl list
+---------------+---------------+-------+----+--------+------------+
| resource_type | resource_name | topic | ip | action | permission |
+---------------+---------------+-------+----+--------+------------+
```

## 5. 黑名单管理

### 5.1 创建黑名单

创建新的黑名单规则。

```console
% ./bin/robust-ctl mqtt --server=127.0.0.1:1883 blacklist create --cluster-name=admin --blacklist=client_id
able to connect: "127.0.0.1:1883"
Created successfully!
```

### 5.2 删除黑名单

删除已有的黑名单规则。

```console
% ./bin/robust-ctl mqtt --server=127.0.0.1:1883 blacklist delete --cluster-name=admin --blacklist-type=client_id --resource-name=client1
able to connect: "127.0.0.1:1883"
Deleted successfully!
```

### 5.3 黑名单列表

列出所有已创建的黑名单规则。

```console
% ./bin/robust-ctl mqtt --server=127.0.0.1:1883 blacklist list
+----------------+---------------+----------+------+
| blacklist_type | resource_name | end_time | desc |
+----------------+---------------+----------+------+
```

## 6. 开启慢订阅功能

### 6.1 开启/关闭慢订阅

慢订阅统计功能主要是为了在消息到达 Broker 后，
Broker 来计算完成消息处理以及传输整个流程所消耗的时间(时延),
如果时延超过阈值，我们就会记录一条相关的信息在集群慢订阅日志当中，
运维人员可以通过命令查询整个集群下的慢订阅记录信息，
通过慢订阅信息来解决。

- 开启慢订阅

```console
% ./bin/robust-ctl mqtt slow-sub --enable=true
The slow subscription feature has been successfully enabled.
```

- 关闭慢订阅

```console
% ./bin/robust-ctl mqtt slow-sub --enable=false
The slow subscription feature has been successfully closed.
```

### 6.2 查询慢订阅记录

当我们启动了慢订阅统计功能之后, 集群就开启慢订阅统计功能，
这样我们可以通过对应的命令来去查询对应的慢订阅记录，
如果我们想要查看慢订阅记录，客户端可以输入如下命令

```console
% ./bin/robust-ctl mqtt slow-sub --query=true
+-----------+-------+----------+---------+-------------+
| client_id | topic | sub_name | time_ms | create_time |
+-----------+-------+----------+---------+-------------+
```

如果想要获取更多的慢订阅记录，
并且想要按照从小到大的顺序进行升序排序，
那么可以使用如下的命令

```console
% ./bin/robust-ctl mqtt slow-sub --list=200 --sort=asc
+-----------+-------+----------+---------+-------------+
| client_id | topic | sub_name | time_ms | create_time |
+-----------+-------+----------+---------+-------------+
```

对于慢订阅查询，我们同样支持筛选查询功能，我们支持使用 topic,
sub_name 以及 client_id 的方式来获取不同字段过滤后的结果，
其结果默认从大到小倒序排序，参考使用命令如下

```console
% ./bin/robust-ctl mqtt slow-sub --topic=topic_test1 --list=200
+-----------+-------+----------+---------+-------------+
| client_id | topic | sub_name | time_ms | create_time |
+-----------+-------+----------+---------+-------------+
```

## 7. 主题重写规则

很多物联网设备不支持重新配置或升级，修改设备业务主题会非常困难。

主题重写功能可以帮助使这种业务升级变得更容易：通过设置一套规则，它可以在订阅、发布时改变将原有主题重写为新的目标主题。

### 7.1 创建主题重写规则

```console
% ./bin/robust-ctl mqtt topic-rewrite create --action=xxx --source-topic=xxx --dest-topic=xxx --regex=xxx
Created successfully!
```

### 7.2 删除主题重写规则

```console
% ./bin/robust-ctl mqtt topic-rewrite delete --action=xxx --source-topic=xxx
Deleted successfully!
```

## 8. 连接抖动检测

在黑名单功能的基础上，支持自动封禁那些被检测到短时间内频繁登录的客户端，并且在一段时间内拒绝这些客户端的登录，以避免此类客户端过多占用服务器资源而影响其他客户端的正常使用。

- 开启连接抖动检测

```console
% ./bin/robust-ctl mqtt flaping-detect --is-enable=false --window-time=1 --max-client-connections=15 --ban-time=5
The flapping detect feature has been successfully enabled.
```

- 关闭连接抖动检测

```console
% ./bin/robust-ctl mqtt flaping-detect --is-enable=false
The flapping detect feature has been successfully closed.
```

## 9. 连接

连接列表命令用于查询 MQTT Broker 当前的连接状态，提供连接 ID、类型、协议、源地址等相关信息。

```console
% ./bin/robust-ctl mqtt list-connection
connection list:
+---------------+-----------------+----------+-------------+------+
| connection_id | connection_type | protocol | source_addr | info |
+---------------+-----------------+----------+-------------+------+
```

## 10. 主题

查看当前系统中所有订阅的主题。 list-topic 列出所有主题,该命令可用于监视主题的数量和分布。

```console
% ./bin/robust-ctl mqtt list-topic
topic list result:
+----------------------------------+---------------------------------------------------------+--------------+---------------------------+
| topic_id                         | topic_name                                              | cluster_name | is_contain_retain_message |
+----------------------------------+---------------------------------------------------------+--------------+---------------------------+
| b63fc4d3523644e1b1da0149bb376c74 | $SYS/brokers/10.7.141.123/version                       | mqtt-broker  | false                     |
+----------------------------------+---------------------------------------------------------+--------------+---------------------------+
......
```
