# MQTT 命令

## 1. 命令结构

```bash
robust-ctl mqtt [--server <admin-addr>] [--mqtt-server <mqtt-addr>] [--output table|json] [--page N] [--limit N] <subcommand>
```

## 2. 子命令总览

- 概览：`overview`
- 列表：`session list`、`subscribes list`、`client list`、`topic list`
- 用户：`user list/create/delete`
- ACL：`acl list/create/delete`
- 黑名单：`blacklist list/create/delete`
- Topic Rewrite：`topic-rewrite list/create/delete`
- Connector：`connector list/create/delete`
- Schema：`schema list/create/delete/list-bind/bind/unbind`
- 自动订阅：`auto-subscribe list/create/delete`
- 可观测：`flapping-detect`、`slow-subscribe list`、`system-alarm list`
- MQTT 消息：`publish`、`subscribe`

## 3. 详细命令

### 3.1 概览与基础列表

下面按“概览 -> 会话 -> 订阅 -> 连接 -> Topic”的顺序检查 Broker。每个命令
后面都给出一次通过集成测试得到的输出示例。

#### `robust-ctl mqtt overview`

```bash
robust-ctl mqtt overview
```

```text
📊 Cluster Overview
Cluster Name                   <run-id>
Message In Rate                0
Message Out Rate               0
Connection Num                 1
Session Num                    2
Topic Num                      9
Subscribe Num                  1
Connector Num                  1
TCP Connections                3
TLS Connections                0
WebSocket Connections          0
QUIC Connections               0

🧩 Node List
Node ID    Node IP              Inner Addr                Roles                          Start Time
1          127.0.0.1            127.0.0.1:<grpc-port>     ["meta", "broker", "engine"]   <timestamp>
```

#### `robust-ctl mqtt session list`

```bash
robust-ctl mqtt session list
```

```text
+-----------+----------------+----------------------+--------------------------+-------------+---------------+-----------+----------------+---------------+
| client_id | session_expiry | is_contain_last_will | last_will_delay_interval | create_time | connection_id | broker_id | reconnect_time | distinct_time |
+===========+================+======================+==========================+=============+===============+===========+================+===============+
| <client-id> | 3              | false                | 0                        | <unix-time> | 4             | 1         | <unix-time>    | 0             |
+-----------+----------------+----------------------+--------------------------+-------------+---------------+-----------+----------------+---------------+
| <client-id-2> | 3            | false                | 0                        | <unix-time> | 0             | 0         | 0              | <unix-time>    |
+===========+================+======================+==========================+=============+===============+===========+================+===============+
+-----------+----------------+----------------------+--------------------------+-------------+---------------+-----------+----------------+---------------+
```

#### `robust-ctl mqtt subscribes list`

```bash
robust-ctl mqtt subscribes list
```

```text
subscribe list result:
+-----------+--------------+------+-----------+----------+-----+----------+-----------------+-----------------+-------------+-------+------------+
| client_id | is_share_sub | path | broker_id | protocol | qos | no_local | preserve_retain | retain_handling | create_time | pk_id | properties |
+===========+==============+======+===========+==========+=====+==========+=================+=================+=============+=======+============+
| <client-id> | false | <run-id>/messages | 1 | Mqtt5 | AtLeastOnce | 0 | 0 | OnEverySubscribe | <timestamp> | 1 | null |
+-----------+--------------+------+-----------+----------+-----+----------+-----------------+-----------------+-------------+-------+------------+
```

#### `robust-ctl mqtt client list`

```bash
robust-ctl mqtt client list
```

```text
connection list:
+-----------+---------------+----------+-------------+
| client_id   | connection_id | protocol | create_time |
+===========+===============+==========+=============+
| <client-id> | 4             | MQTT5    | <unix-time>  |
+-----------+---------------+----------+-------------+
```

#### `robust-ctl mqtt topic list`

```bash
robust-ctl mqtt topic list
```

```text
📋 Topic List (Total: 9)
+-----------------------+---------------+-----------+-------------+---------------------+
| Topic Name            | Storage Type  | Partition | Replication | Create Time         |
+=======================+===============+===========+=============+=====================+
| $delay-queue-index    | EngineRocksDB | 3         | 1           | <timestamp>         |
+-----------------------+---------------+-----------+-------------+---------------------+
| $SYS/events           | EngineRocksDB | 3         | 1           | <timestamp>         |
+-----------------------+---------------+-----------+-------------+---------------------+
| $last-will-message    | EngineRocksDB | 3         | 1           | <timestamp>         |
+-----------------------+---------------+-----------+-------------+---------------------+
| $retain-message       | EngineRocksDB | 3         | 1           | <timestamp>         |
+-----------------------+---------------+-----------+-------------+---------------------+
| $delay-queue-message  | EngineRocksDB | 3         | 1           | <timestamp>         |
+-----------------------+---------------+-----------+-------------+---------------------+
| $agent-report-info    | EngineRocksDB | 3         | 1           | <timestamp>         |
+-----------------------+---------------+-----------+-------------+---------------------+
| $sys/qos2-inner-topic | EngineRocksDB | 3         | 1           | <timestamp>         |
+-----------------------+---------------+-----------+-------------+---------------------+
| $delay-task-index     | EngineRocksDB | 3         | 1           | <timestamp>         |
+-----------------------+---------------+-----------+-------------+---------------------+
| <run-id>/messages     | EngineRocksDB | 3         | 1           | <timestamp>         |
+-----------------------+---------------+-----------+-------------+---------------------+
```

分页会在请求发送前生效：

#### `robust-ctl mqtt --page 2 --limit 50 session list`

```bash
robust-ctl mqtt --page 2 --limit 50 session list
```

```text
+-----------+----------------+----------------------+--------------------------+-------------+---------------+-----------+----------------+---------------+
| client_id | session_expiry | is_contain_last_will | last_will_delay_interval | create_time | connection_id | broker_id | reconnect_time | distinct_time |
+===========+================+======================+==========================+=============+===============+===========+================+===============+
+-----------+----------------+----------------------+--------------------------+-------------+---------------+-----------+----------------+---------------+
```

#### `robust-ctl mqtt --output json --page 1 --limit 20 client list`

```bash
robust-ctl mqtt --output json --page 1 --limit 20 client list
```

```json
{
  "data": [
    {
      "tenant": "default",
      "client_id": "<client-id>",
      "connection_id": 4,
      "mqtt_connection": { "is_login": true, "login_user": "admin" }
    }
  ],
  "total_count": 1
}
```

### 3.2 用户管理

#### `robust-ctl mqtt user list`

```bash
robust-ctl mqtt user list
```

```text
user list result:
+----------+--------------+
| username | is_superuser |
+==========+==============+
| <run-id>-user | false        |
+----------+--------------+
| admin    | true         |
+----------+--------------+
```

#### `robust-ctl mqtt user create`

```bash
robust-ctl mqtt user create --username <run-id>-user --password <run-id>-password
```

```text
Created successfully!
```

#### `robust-ctl mqtt user delete`

```bash
robust-ctl mqtt user delete --username <run-id>-user
```

```text
Deleted successfully!
```

### 3.3 ACL 管理

#### `robust-ctl mqtt acl list`

```bash
robust-ctl mqtt acl list
```

```text
acl list result:
| <run-id>-acl | | ClientId | <run-id>-client | <run-id>/messages | | All | Allow |
+------+------+---------------+---------------+-------+----+--------+------------+
| name | desc | resource_type | resource_name | topic | ip | action | permission |
+======+======+===============+===============+=======+====+========+============+
+------+------+---------------+---------------+-------+----+--------+------------+
```

#### `robust-ctl mqtt acl create`

```bash
robust-ctl mqtt acl create --tenant default --name <run-id>-acl --resource-type ClientId --resource-name <run-id>-client --topic "<run-id>/messages" --action All --permission Allow
```

```text
Created successfully!
```

#### `robust-ctl mqtt acl delete`

```bash
robust-ctl mqtt acl delete --tenant default --name <run-id>-acl
```

```text
Deleted successfully!
```

### 3.4 黑名单管理

#### `robust-ctl mqtt blacklist list`

```bash
robust-ctl mqtt blacklist list
```

```text
blacklist list result:
| <run-id>-blacklist | ClientId | <run-id>-client | <timestamp> | <run-id> blacklist |
+------+----------------+---------------+----------+-------------+
| name | blacklist_type | resource_name | end_time | description |
+======+================+===============+==========+=============+
+------+----------------+---------------+----------+-------------+
```

#### `robust-ctl mqtt blacklist create`

```bash
robust-ctl mqtt blacklist create --tenant default --name <run-id>-blacklist --blacklist-type ClientId --resource-name <run-id>-client --end-time <unix-time> --desc "<run-id> blacklist"
```

```text
Created successfully!
```

#### `robust-ctl mqtt blacklist delete`

```bash
robust-ctl mqtt blacklist delete --tenant default --name <run-id>-blacklist
```

```text
Deleted successfully!
```

### 3.5 Topic Rewrite

#### `robust-ctl mqtt topic-rewrite list`

```bash
robust-ctl mqtt topic-rewrite list
```

```text
topic rewrite rule list result:
| <run-id>-rewrite | <run-id> rewrite | default | All | <run-id>/source | <run-id>/destination | ^.*$ |
+------+------+--------+--------+--------------+------------+-------+
| name | desc | tenant | action | source_topic | dest_topic | regex |
+======+======+========+========+==============+============+=======+
+------+------+--------+--------+--------------+------------+-------+
```

#### `robust-ctl mqtt topic-rewrite create`

```bash
robust-ctl mqtt topic-rewrite create --tenant default --name <run-id>-rewrite --action All --source-topic "<run-id>/source" --dest-topic "<run-id>/destination" --regex '^.*$'
```

```text
Created successfully!
```

#### `robust-ctl mqtt topic-rewrite delete`

```bash
robust-ctl mqtt topic-rewrite delete --tenant default --name <run-id>-rewrite
```

```text
Deleted successfully!
```

### 3.6 Connector

当前 CLI 的 `connector list` 需要传入 `--connector-name`。集成测试使用本地
文件 Connector，因此不依赖 Kafka。

#### `robust-ctl mqtt connector list`

```bash
robust-ctl mqtt connector list --connector-name <run-id>-connector
```

```text
connector list result:
| <run-id>-connector | file | {"LocalFile":...} | <run-id>/messages | Running | 1 | <timestamp> | <timestamp> |
+----------------+----------------+------------------+----------+--------+-----------+-------------+-------------+
| connector name | connector type | connector config | topic id | status | broker id | create time | update time |
+================+================+==================+==========+========+===========+=============+=============+
+----------------+----------------+------------------+----------+--------+-----------+-------------+-------------+
```

#### `robust-ctl mqtt connector create`

```bash
robust-ctl mqtt connector create --tenant default --connector-name <run-id>-connector --connector-type file --config '{"local_file_path":"<temp-file>","rotation_strategy":"none"}' --topic-name <run-id>/messages
```

```text
Created successfully!
```

#### `robust-ctl mqtt connector delete`

```bash
robust-ctl mqtt connector delete --tenant default --connector-name <run-id>-connector
```

```text
Deleted successfully!
```

### 3.7 Schema

`schema list` 接受 `--schema-name`，`schema list-bind` 必须使用与 `schema bind`
相同的 `--tenant`。

控制面、Broker 缓存和发布校验的完整流程请参阅
[Schema Registry 架构](../Architect/Schema-Registry-Architecture)。

#### `robust-ctl mqtt schema list`

```bash
robust-ctl mqtt schema list --schema-name <run-id>-schema
```

```text
schema list result:
schema name: <run-id>-schema
schema type: json
schema desc: <run-id> schema
schema: {"type":"object","properties":{"value":{"type":"string"}}}
```

#### `robust-ctl mqtt schema create`

```bash
robust-ctl mqtt schema create --tenant default --schema-name <run-id>-schema --schema-type json --schema '{"type":"object","properties":{"value":{"type":"string"}}}' --desc "<run-id> schema"
```

```text
Created successfully!
```

#### `robust-ctl mqtt schema delete`

```bash
robust-ctl mqtt schema delete --tenant default --schema-name <run-id>-schema
```

```text
Deleted successfully!
```

#### `robust-ctl mqtt schema list-bind`

```bash
robust-ctl mqtt schema list-bind --tenant default --schema-name <run-id>-schema --resource-name <run-id>/schema-topic
```

```text
bind schema list result:
data type: resource
  - <run-id>-schema
data type: schema
  - <run-id>/schema-topic
```

#### `robust-ctl mqtt schema bind`

```bash
robust-ctl mqtt schema bind --tenant default --schema-name <run-id>-schema --resource-name <run-id>/schema-topic
```

```text
Created successfully!
```

#### `robust-ctl mqtt schema unbind`

```bash
robust-ctl mqtt schema unbind --tenant default --schema-name <run-id>-schema --resource-name <run-id>/schema-topic
```

```text
Deleted successfully!
```

### 3.8 Auto Subscribe

#### `robust-ctl mqtt auto-subscribe list`

```bash
robust-ctl mqtt auto-subscribe list
```

```text
auto subscribe rule list result:
| <run-id>-auto | | default | <run-id>/messages | AtLeastOnce | false | false | OnEverySubscribe |
+------+------+--------+-------+-----+----------+---------------------+-------------------+
| name | desc | tenant | topic | qos | no_local | retain_as_published | retained_handling |
+======+======+========+=======+=====+==========+=====================+===================+
+------+------+--------+-------+-----+----------+---------------------+-------------------+
```

#### `robust-ctl mqtt auto-subscribe create`

```bash
robust-ctl mqtt auto-subscribe create --tenant default --name <run-id>-auto --topic "<run-id>/messages" --qos 1
```

```text
Created successfully!
```

#### `robust-ctl mqtt auto-subscribe delete`

```bash
robust-ctl mqtt auto-subscribe delete --tenant default --name <run-id>-auto
```

```text
Deleted successfully!
```

### 3.9 可观测命令

#### `robust-ctl mqtt flapping-detect`

```bash
robust-ctl mqtt flapping-detect
```

```text
flapping detect list result:
+-----------+---------------------------------+--------------------+
| client_id | before_last_windows_connections | first_request_time |
+===========+=================================+====================+
+-----------+---------------------------------+--------------------+
```

#### `robust-ctl mqtt slow-subscribe list`

```bash
robust-ctl mqtt slow-subscribe list
```

```text
slow subscribe list result:
+-----------+------------+----------------+-----------+-------------+
| client_id | topic_name | subscribe_name | time_span | create_time |
+===========+============+================+===========+============+
+-----------+------------+----------------+-----------+-------------+
```

#### `robust-ctl mqtt system-alarm list`

```bash
robust-ctl mqtt system-alarm list
```

```text
system alarm list result:
+------+---------+-------------+
| name | message | create_time |
+======+=========+=============+
+------+---------+-------------+
```

### 3.10 MQTT 发布与订阅

#### `robust-ctl mqtt publish`

```bash
robust-ctl mqtt publish --mqtt-server <mqtt-addr> --username admin --password robustmq --topic "<run-id>/messages" --qos 1
```

```text
able to connect: "<mqtt-addr>"
you can post a message on the terminal:
> You typed: <payload>
> End of input stream.
```

使用 `--retained` 时还会输出：

```text
published retained message
```

#### `robust-ctl mqtt subscribe`

```bash
robust-ctl mqtt subscribe --mqtt-server <mqtt-addr> --username admin --password robustmq --topic "<run-id>/messages" --qos 1
```

```text
able to connect: "<mqtt-addr>"
subscribe success
payload: <retained-payload>
payload: <message-one-payload>
payload: <message-two-payload>
payload: <qos2-payload>
 Ctrl+C detected,  Please press ENTER to end the program.
End of input stream.
```

## 4. 输出说明

- 默认：`table`
- 脚本化：`--output json`
- `publish` / `subscribe` 使用 `--mqtt-server`（默认读取配置中的 MQTT TCP 端口，否则为 `127.0.0.1:1883`）；`--server` 仍然是 Admin API 地址。

示例：

```bash
robust-ctl mqtt --output json user list
```

## 5. 变更、交互与错误输出

上面的基础列表已经按“命令后紧跟输出”的方式给出验证结果。本节只记录
变更命令、交互式 MQTT 命令和预期失败的输出契约。尖括号中的内容需要替换
为当前运行生成的资源名称和端口；字段名、消息格式和退出码规则保持不变。

成功的变更命令返回退出码 `0`，并在 stdout 输出确认信息：

```text
$ robust-ctl mqtt user create --username <run-user> --password <run-password>
Created successfully!
```

表格模式输出人类可读的标题和行：

```text
user list result:
+----------+--------------+
| username | is_superuser |
+==========+==============+
| admin    | true         |
+----------+--------------+
```

JSON 模式只在 stdout 输出 JSON 文档，分页列表使用 `data` 和
`total_count`：

```json
{
  "data": [
    {
      "tenant": "default",
      "username": "admin",
      "is_superuser": true,
      "create_time": 1792462675
    }
  ],
  "total_count": 1
}
```

Schema 绑定结果按 `data_type` 分组，绑定和查询必须使用相同的租户：

```json
{
  "data": [
    { "data_type": "resource", "data": ["<run-id>-schema"] },
    { "data_type": "schema", "data": ["<run-id>/schema-topic"] }
  ],
  "total_count": 2
}
```

交互式 MQTT 输出按行返回；成功流程中每个 payload 只出现一次，然后通过
Ctrl-C 正常结束：

```text
able to connect: "127.0.0.1:<mqtt-port>"
subscribe success
payload: retained-<run-id>
payload: message-one-<run-id>
payload: message-two-<run-id>
payload: qos2-<run-id>
 Ctrl+C detected,  Please press ENTER to end the program.
End of input stream.
```

预期失败返回非零退出码，并把诊断信息保留在 stderr：

```text
# 无效 QoS：退出码 1
Exception:Server returned error: code=400, message=... QoS must be 0, 1 or 2

# Admin API 不可用：退出码 1
Exception:Request failed: error sending request for url (http://127.0.0.1:1/api/mqtt/overview)
```

Banner 属于诊断输出，不属于 JSON stdout。选择 `--output json` 后，只应对
stdout 使用 `jq` 解析。

完整的集成测试通过时，退出码为 `0`，并输出以下摘要：

```text
$ ./scripts/test/cli/mqtt/run.sh
[mqtt] retained delivery, QoS 1/2 publish, exact-once messages, and Ctrl-C passed
All MQTT CLI integration checks passed.
```

最近一次带数据验证共执行 72 次 CLI 调用，覆盖表格/JSON 只读检查、资源状态流转、
MQTT 发布订阅以及端点和错误检查。重复创建、无效 QoS、不可用的 Admin API
和无效 MQTT 凭据属于预期的非零场景，测试会明确断言这些退出码。
