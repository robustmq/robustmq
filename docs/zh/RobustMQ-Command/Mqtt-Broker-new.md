# RobustMQ MQTT Broker 命令行工具指南

## 概述

RobustMQ MQTT Broker 命令行工具 (`robust-ctl mqtt`) 是一个功能强大的管理工具，为 MQTT 集群提供全方位的管理和监控功能。通过这个工具，您可以轻松管理用户、权限、连接、订阅，以及监控系统性能。

### 核心特性

- 🔧 **集群管理**: 实时查看集群状态和配置
- 👥 **用户权限**: 完整的用户管理和访问控制
- 📊 **性能监控**: 慢订阅检测、系统告警等
- 🔗 **连接管理**: 连接状态监控和抖动检测
- 📝 **消息管理**: 主题管理、重写规则
- 🔌 **数据集成**: 连接器和Schema管理
- 💬 **实时通信**: 内置发布/订阅功能

> [!NOTE]
> 当前命令行功能主要支持单集群内容的查询和管理，多集群功能正在开发中。如果您有功能需求或建议，欢迎在我们的 [GitHub 仓库](https://github.com/robustmq/robustmq) 中提交 Issue。

## 快速开始

### 基本语法

```bash
./bin/robust-ctl mqtt [OPTIONS] <COMMAND> [ARGS]
```

### 常用选项

- `-s, --server <SERVER>`: 指定服务器地址（默认: 127.0.0.1:1228）
- `-h, --help`: 显示帮助信息

### 快速检查集群状态

```bash
# 检查集群状态
./bin/robust-ctl mqtt status

# 查看集群配置
./bin/robust-ctl mqtt config get

# 查看用户列表
./bin/robust-ctl mqtt user list
```

## 命令总览

| 命令分类 | 命令 | 功能描述 |
|---------|-----|----------|
| **基础管理** | `status` | 查看集群状态和统计信息 |
| | `config` | 查看和管理集群配置 |
| **用户权限** | `user` | 用户管理（创建、删除、列表） |
| | `acl` | 访问控制列表管理 |
| | `blacklist` | 黑名单管理 |
| **会话连接** | `session` | 会话管理 |
| | `subscribes` | 订阅管理 |
| | `connection` | 连接管理 |
| **主题消息** | `list-topic` | 主题管理 |
| | `topic-rewrite-rule` | 主题重写规则 |
| | `auto-subscribe-rule` | 自动订阅规则 |
| **性能监控** | `slow-subscribe` | 慢订阅检测 |
| | `flapping-detect` | 连接抖动检测 |
| | `system-alarm` | 系统告警管理 |
| **数据集成** | `connector` | 连接器管理 |
| | `schema` | Schema管理 |
| **消息收发** | `publish` | 发布消息 |
| | `subscribe` | 订阅消息 |

---

## 基础管理

### 1. 集群状态查询

查看 MQTT 集群的实时状态和统计信息。

```bash
./bin/robust-ctl mqtt status
```

**输出示例:**

```console
cluster_name: broker-server
message_in_rate: 10
message_out_rate: 3
connection_num: 0
session_num: 0
topic_num: 64
nodes: [BrokerNodeRaw { cluster_name: "broker-server", ... }]
placement_status: {"running_state":{"Ok":null},"state":"Leader",...}
tcp_connection_num: 0
tls_connection_num: 0
websocket_connection_num: 0
quic_connection_num: 0
subscribe_num: 0
exclusive_subscribe_num: 0
share_subscribe_leader_num: 0
connection_num: [{"date":1755778388,"value":0}...]
topic_num: [{"value":64,"date":1755778208}...]
message_in_num: [{"date":1755778388,"value":1000}...]
message_out_num: [{"date":1755778208,"value":1000}...]
message_drop_num: [{"date":1755778208,"value":30}...]
```

**状态信息说明:**

| 字段 | 说明 |
|------|------|
| `cluster_name` | 集群名称 |
| `message_in_rate` / `message_out_rate` | 消息接收/发送速率（条/秒） |
| `connection_num` | 当前连接数 |
| `session_num` | 当前会话数 |
| `topic_num` | 主题数量 |
| `nodes` | 集群节点信息 |
| `placement_status` | 集群Raft状态信息 |
| `*_connection_num` | 各协议连接数（TCP/TLS/WebSocket/QUIC） |
| `*_subscribe_num` | 各类订阅统计 |

### 2. 集群配置管理

#### 2.1 查看集群配置

```bash
./bin/robust-ctl mqtt config get
```

**输出示例:**

```json
{
  "cluster_name": "broker-server",
  "broker_id": 1,
  "roles": ["meta", "broker"],
  "grpc_port": 1228,
  "mqtt_server": {
    "tcp_port": 1883,
    "tls_port": 1884,
    "websocket_port": 8083,
    "websockets_port": 8084,
    "quic_port": 9083
  },
  "mqtt_runtime": {
    "default_user": "admin",
    "default_password": "robustmq",
    "max_connection_num": 1000000
  }
  // ... 更多配置项
}
```

---

## 用户与权限管理

### 3. 用户管理

管理 MQTT 用户账户，包括创建、删除和查看用户。

#### 3.1 查看用户列表

```bash
./bin/robust-ctl mqtt user list
```

**输出示例:**

```
+----------+--------------+
| username | is_superuser |
+----------+--------------+
| admin    | true         |
| test     | false        |
+----------+--------------+
```

#### 3.2 创建用户

```bash
./bin/robust-ctl mqtt user create \
  --username newuser \
  --password password123 \
  --is-superuser false
```

**参数说明:**
- `--username, -u`: 用户名（必需）
- `--password, -p`: 密码（必需）
- `--is-superuser, -i`: 是否为超级用户（默认: false）

#### 3.3 删除用户

```bash
./bin/robust-ctl mqtt user delete --username newuser
```

### 4. 访问控制列表(ACL)管理

MQTT Broker 提供了 ACL 功能来控制用户对特定资源的访问权限。

### 4.1 查看 ACL 列表

```console
% ./bin/robust-ctl mqtt acl list
+---------------+---------------+-------+----+--------+------------+
| resource_type | resource_name | topic | ip | action | permission |
+---------------+---------------+-------+----+--------+------------+
| User          | testuser      | test  |    | Pub    | Allow      |
| User          | testuser      | test  |    | Sub    | Allow      |
+---------------+---------------+-------+----+--------+------------+
```

### 4.2 创建 ACL 规则

```console
% ./bin/robust-ctl mqtt acl create \
--cluster-name broker-server \
--acl '{"resource_type":"User","resource_name":"testuser","topic":"sensor/#","ip":"","action":"Pub","permission":"Allow"}'
Created successfully!
```

参数说明：
- `--cluster-name, -c`: 集群名称（必需）
- `--acl, -a`: ACL 规则的 JSON 字符串（必需）

### 4.3 删除 ACL 规则

```console
% ./bin/robust-ctl mqtt acl delete --cluster-name broker-server --acl '{"resource_type":"User","resource_name":"testuser","topic":"sensor/#","ip":"","action":"Pub","permission":"Allow"}'
Deleted successfully!
```

## 5. 黑名单管理

黑名单功能允许阻止特定的客户端或IP地址连接到MQTT Broker。

### 5.1 查看黑名单

```console
% ./bin/robust-ctl mqtt blacklist list
+----------------+---------------+----------+----------------+
| blacklist_type | resource_name | end_time | blacklist_type |
+----------------+---------------+----------+----------------+
| ClientId       | bad_client    | 0        | ClientId       |
| IpAddr         | 192.168.1.100 | 0        | IpAddr         |
+----------------+---------------+----------+----------------+
```

### 5.2 添加黑名单

```console
% ./bin/robust-ctl mqtt blacklist create \
--cluster-name broker-server \
--blacklist '{"blacklist_type":"ClientId","resource_name":"malicious_client","end_time":0}'
Created successfully!
```

参数说明：
- `--cluster-name, -c`: 集群名称（必需）
- `--blacklist, -b`: 黑名单规则的 JSON 字符串（必需）

### 5.3 删除黑名单

```console
% ./bin/robust-ctl mqtt blacklist delete \
--cluster-name broker-server \
--blacklist-type ClientId \
--resource-name malicious_client
Deleted successfully!
```

参数说明：
- `--cluster-name, -c`: 集群名称（必需）
- `--blacklist-type, -b`: 黑名单类型（必需）
- `--resource-name, -r`: 资源名称（必需）

## 6. 会话管理

查看当前系统中的所有 MQTT 会话信息。

```console
% ./bin/robust-ctl mqtt session list
+-----------+----------------+---------------------+--------------------------+-------------+---------------+-----------+-----------------+---------------+
| client_id | session_expiry | is_contain_last_will| last_will_delay_interval | create_time | connection_id | broker_id | reconnect_time  | distinct_time |
+-----------+----------------+---------------------+--------------------------+-------------+---------------+-----------+-----------------+---------------+
| client001 | 1800           | false               | 0                        | 1692345600  | 12345         | 1         | 1692345700      | 0             |
+-----------+----------------+---------------------+--------------------------+-------------+---------------+-----------+-----------------+---------------+
```

## 7. 订阅管理

### 7.1 查看订阅列表

```console
% ./bin/robust-ctl mqtt subscribes list
+-----------+-------------+----------+-----------+----------+-----+----------+------------------+------------------+-------------+-------+------------+
| client_id | is_share_sub| path     | broker_id | protocol | qos | no_local | preserve_retain  | retain_handling  | create_time | pk_id | properties |
+-----------+-------------+----------+-----------+----------+-----+----------+------------------+------------------+-------------+-------+------------+
| client001 | false       | sensor/# | 1         | MQTT     | 1   | false    | false            | 0                | 1692345600  | 1     |            |
+-----------+-------------+----------+-----------+----------+-----+----------+------------------+------------------+-------------+-------+------------+
```

### 7.2 查看订阅详情

```console
% ./bin/robust-ctl mqtt subscribes detail --client-id client001 --path "sensor/#"
subscribe info: {"client_id":"client001","topic":"sensor/#","qos":1}
=======================
sub: ExclusiveSub
thread: SubscribeThread1
=======================
```

参数说明：
- `--client-id, -c`: 客户端ID（必需）
- `--path, -p`: 订阅路径（必需）

## 8. 连接管理

查看当前系统中的所有连接信息。

```console
% ./bin/robust-ctl mqtt connection list
connection list:
+---------------+-----------------+----------+-------------+--------------------------------------------------+
| connection_id | connection_type | protocol | source_addr | info                                             |
+---------------+-----------------+----------+-------------+--------------------------------------------------+
| 12345         | TCP             | MQTT     | 192.168.1.10| {"client_id":"client001","username":"testuser"} |
+---------------+-----------------+----------+-------------+--------------------------------------------------+
```

## 9. 主题管理

查看当前系统中的所有主题信息。

```console
% ./bin/robust-ctl mqtt list-topic
topic list result:
+----------+------------+--------------+--------------------------+
| topic_id | topic_name | cluster_name | is_contain_retain_message|
+----------+------------+--------------+--------------------------+
| 1        | sensor/temp| broker-server| false                    |
| 2        | sensor/hum | broker-server| true                     |
+----------+------------+--------------+--------------------------+
```

## 10. 慢订阅管理

慢订阅功能用于监控和管理订阅性能问题。

### 10.1 启用/禁用慢订阅检测

```console
% ./bin/robust-ctl mqtt slow-subscribe enable --is-enable true
Enabled successfully! feature name: SlowSubscribe
```

### 10.2 配置慢订阅参数

```console
% ./bin/robust-ctl mqtt slow-subscribe set --max-store-num 2000 --delay-type Whole
Set slow subscribe config successfully! Current Config:
+----------------+-------+
| Config Options | Value |
+----------------+-------+
| enable         | true  |
| delay_type     | Whole |
| max_store_num  | 2000  |
+----------------+-------+
```

参数说明：
- `--max-store-num`: 最大存储数量，默认为1000
- `--delay-type`: 延迟类型（必需）

### 10.3 查看慢订阅列表

```console
% ./bin/robust-ctl mqtt slow-subscribe list --list 50 --sort DESC
+-----------+------------+----------------+-----------+-------------+
| client_id | topic_name | subscribe_name | time_span | create_time |
+-----------+------------+----------------+-----------+-------------+
| client001 | sensor/#   | sub_001        | 1500      | 1692345600  |
| client002 | device/#   | sub_002        | 1200      | 1692345610  |
+-----------+------------+----------------+-----------+-------------+
```

参数说明：
- `--list`: 显示的记录数量，默认为100
- `--sort`: 排序方式，支持 ASC 或 DESC，默认为 DESC
- `--topic`: 按主题过滤（可选）
- `--sub-name`: 按订阅名称过滤（可选）
- `--client-id`: 按客户端ID过滤（可选）

## 11. 系统告警管理

系统告警功能用于监控系统资源使用情况。

### 11.1 配置系统告警

```console
% ./bin/robust-ctl mqtt system-alarm set --enable true --cpu-high-watermark 80.0 --cpu-low-watermark 60.0 --memory-high-watermark 85.0 --os-cpu-check-interval-ms 30000
Set system alarm config successfully! Current Config:
+------------------------+-------+
| Config Options         | Value |
+------------------------+-------+
| enable                 | true  |
| memory-high-watermark  | 85.0  |
| cpu-high-watermark     | 80.0  |
| cpu-low-watermark      | 60.0  |
| cpu-check-interval-ms  | 30000 |
+------------------------+-------+
```

参数说明：
- `--enable`: 是否启用告警功能（可选）
- `--cpu-high-watermark`: CPU使用率高水位线（可选）
- `--cpu-low-watermark`: CPU使用率低水位线（可选）
- `--memory-high-watermark`: 内存使用率高水位线（可选）
- `--os-cpu-check-interval-ms`: CPU检查间隔（毫秒）（可选）

### 11.2 查看系统告警列表

```console
% ./bin/robust-ctl mqtt system-alarm list
system alarm list result:
+-----------------+-------------------------+-------------+-----------+
| name            | message                 | activate_at | activated |
+-----------------+-------------------------+-------------+-----------+
| HighCpuUsage    | CPU usage exceeded 80%  | 1692345600  | true      |
| HighMemoryUsage | Memory usage exceeded 85%| 1692345610  | false     |
+-----------------+-------------------------+-------------+-----------+
```

## 12. 连接抖动检测

连接抖动检测功能用于识别频繁连接/断开的客户端。

```console
% ./bin/robust-ctl mqtt flapping-detect --enable=true --window-time=5 --max-client-connections=20 --ban-time=10
The flapping detect feature has been successfully enabled.
```

参数说明：
- `--enable`: 启用或禁用抖动检测（必需）
- `--window-time`: 检测时间窗口（分钟），默认为1
- `--max-client-connections`: 最大客户端连接数，默认为15
- `--ban-time`: 封禁时间（分钟），默认为5

查看抖动检测结果：

```console
% ./bin/robust-ctl mqtt list-flapping-detect
+-----------+--------------------------------+--------------------+
| client_id | before_last_windows_connections| first_request_time |
+-----------+--------------------------------+--------------------+
| bad_client| 25                             | 1692345600         |
+-----------+--------------------------------+--------------------+
```

## 13. 主题重写规则

主题重写功能允许在消息传递过程中重写主题名称。

### 13.1 创建主题重写规则

```console
% ./bin/robust-ctl mqtt topic-rewrite-rule create --action republish --source-topic "old/topic" --dest-topic "new/topic" --regex "old/(.*)"
Created successfully!
```

参数说明：
- `--action, -a`: 重写动作（必需）
- `--source-topic, -s`: 源主题（必需）
- `--dest-topic, -d`: 目标主题（必需）
- `--regex, -r`: 正则表达式（必需）

### 13.2 删除主题重写规则

```console
% ./bin/robust-ctl mqtt topic-rewrite-rule delete --action republish --source-topic "old/topic"
Deleted successfully!
```

## 14. 连接器管理

连接器用于将MQTT数据连接到外部系统。

### 14.1 查看连接器列表

```console
% ./bin/robust-ctl mqtt connector list --connector-name webhook_connector
connector list result:
+--------------+------------------+----------------+------------------+----------+--------+-----------+-------------+-------------+
| cluster name | connector name   | connector type | connector config | topic id | status | broker id | create time | update time |
+--------------+------------------+----------------+------------------+----------+--------+-----------+-------------+-------------+
| broker-server| webhook_connector| Http           | {"url":"..."}    | topic1   | Active | 1         | 1692345600  | 1692345600  |
+--------------+------------------+----------------+------------------+----------+--------+-----------+-------------+-------------+
```

### 14.2 创建连接器

```console
% ./bin/robust-ctl mqtt connector create --connector-name new_webhook --connector-type Http --config '{"url":"http://example.com/webhook","method":"POST"}' --topic-id topic1
Created successfully!
```

### 14.3 更新连接器

```console
% ./bin/robust-ctl mqtt connector update --connector '{"name":"new_webhook","config":"updated_config"}'
Updated successfully!
```

### 14.4 删除连接器

```console
% ./bin/robust-ctl mqtt connector delete --connector-name new_webhook
Deleted successfully!
```

## 15. Schema 管理

Schema 用于定义和验证消息格式。

### 15.1 查看 Schema 列表

```console
% ./bin/robust-ctl mqtt schema list --schema-name sensor_schema
schema list result:
cluster name: broker-server
schema name: sensor_schema
schema type: json
schema desc: Sensor data schema
schema: {"type":"object","properties":{"temperature":{"type":"number"},"humidity":{"type":"number"}}}
```

### 15.2 创建 Schema

```console
% ./bin/robust-ctl mqtt schema create --schema-name new_schema --schema-type json --schema '{"type":"object"}' --desc "New schema description"
Created successfully!
```

### 15.3 更新 Schema

```console
% ./bin/robust-ctl mqtt schema update --schema-name new_schema --schema-type json --schema '{"type":"object","updated":true}' --desc "Updated schema"
Updated successfully!
```

### 15.4 删除 Schema

```console
% ./bin/robust-ctl mqtt schema delete --schema-name new_schema
Deleted successfully!
```

### 15.5 绑定 Schema

```console
% ./bin/robust-ctl mqtt schema bind --schema-name sensor_schema --resource-name topic1
Created successfully!
```

### 15.6 解绑 Schema

```console
% ./bin/robust-ctl mqtt schema unbind --schema-name sensor_schema --resource-name topic1
Deleted successfully!
```

### 15.7 查看绑定的 Schema

```console
% ./bin/robust-ctl mqtt schema list-bind --schema-name sensor_schema --resource-name topic1
bind schema list result:
cluster name: broker-server
schema name: sensor_schema
schema type: json
schema desc: Sensor data schema
schema: {"type":"object","properties":{"temperature":{"type":"number"}}}
```

## 16. 自动订阅规则

自动订阅规则允许客户端连接时自动订阅指定主题。

### 16.1 查看自动订阅规则

```console
% ./bin/robust-ctl mqtt auto-subscribe-rule list
+----------+-----+----------+---------------------+------------------+
| topic    | qos | no_local | retain_as_published | retained_handling|
+----------+-----+----------+---------------------+------------------+
| system/# | 0   | false    | false               | 0                |
| alerts/# | 1   | false    | true                | 1                |
+----------+-----+----------+---------------------+------------------+
```

### 16.2 设置自动订阅规则

```console
% ./bin/robust-ctl mqtt auto-subscribe-rule set --topic "notifications/#" --qos 1 --no-local false --retain-as-published true --retained-handling 2
Created successfully!
```

参数说明：
- `--topic, -t`: 订阅主题（必需）
- `--qos, -q`: QoS 等级，默认为0
- `--no-local, -n`: 是否禁用本地消息，默认为false
- `--retain-as-published, -r`: 是否保持发布时的保留标志，默认为false
- `--retained-handling, -R`: 保留消息处理方式，默认为0

### 16.3 删除自动订阅规则

```console
% ./bin/robust-ctl mqtt auto-subscribe-rule delete --topic "notifications/#"
Deleted successfully!
```

## 17. 消息发布和订阅

### 17.1 发布消息

```console
% ./bin/robust-ctl mqtt publish --username admin --password robustmq --topic test/topic --qos 1 --retained false
you can post a message on the terminal:
> Hello World
You typed: Hello World
> Ctrl+C detected, Please press ENTER to end the program.
```

参数说明：
- `--username, -u`: 用户名（必需）
- `--password, -p`: 密码（必需）
- `--topic, -t`: 发布主题（必需）
- `--qos, -q`: QoS 等级，默认为0
- `--retained`: 是否为保留消息，默认为false

### 17.2 订阅消息

```console
% ./bin/robust-ctl mqtt subscribe --username admin --password robustmq --topic test/topic --qos 1
subscribe success
payload: Hello World
Ctrl+C detected, Please press ENTER to end the program.
```

参数说明：
- `--username, -u`: 用户名（必需）
- `--password, -p`: 密码（必需）
- `--topic, -t`: 订阅主题（必需）
- `--qos, -q`: QoS 等级，默认为0

## 总结

RobustMQ 的 MQTT Broker 命令行工具提供了丰富的管理功能，
涵盖了集群监控、用户管理、权限控制、性能监控等各个方面。
通过这些命令，管理员可以有效地监控和管理 MQTT 集群的运行状态，确保系统的稳定性和安全性。

> [!TIP]
> 在使用命令行工具时，可以通过 `--help` 参数查看每个命令的详细帮助信息，例如：
> ```console
> % ./bin/robust-ctl mqtt user --help
> % ./bin/robust-ctl mqtt acl create --help
> ```
