# MQTT Commands

## Command Shape

```bash
robust-ctl mqtt [--server <admin-addr>] [--mqtt-server <mqtt-addr>] [--output table|json] [--page N] [--limit N] <subcommand>
```

## Common Subcommands

### 3.1 Overview and basic lists

Run the commands in this order when checking a broker: overview first, then
sessions, subscriptions, connections, and topics. The examples below show the
default table output captured from an isolated broker.

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
| <client-id> | false        | <run-id>/messages | 1 | Mqtt5 | AtLeastOnce | 0 | 0 | OnEverySubscribe | <timestamp> | 1 | null |
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

Pagination is applied before the request is sent. For example:

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

### 3.2 User management

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

### 3.3 ACL management

#### `robust-ctl mqtt acl list`

```bash
robust-ctl mqtt acl list
```

```text
acl list result:
+------+------+---------------+---------------+-------+----+--------+------------+
| name | desc | resource_type | resource_name | topic | ip | action | permission |
+======+======+===============+===============+=======+====+========+============+
| <run-id>-acl | | ClientId | <run-id>-client | <run-id>/messages | | All | Allow |
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

### 3.4 Blacklist management

#### `robust-ctl mqtt blacklist list`

```bash
robust-ctl mqtt blacklist list
```

```text
blacklist list result:
+------+----------------+---------------+----------+-------------+
| name | blacklist_type | resource_name | end_time | description |
+======+================+===============+==========+=============+
| <run-id>-blacklist | ClientId | <run-id>-client | <timestamp> | <run-id> blacklist |
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
+------+------+--------+--------+--------------+------------+-------+
| name | desc | tenant | action | source_topic | dest_topic | regex |
+======+======+========+========+==============+============+=======+
| <run-id>-rewrite | <run-id> rewrite | default | All | <run-id>/source | <run-id>/destination | ^.*$ |
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

`connector list` requires `--connector-name` in the current CLI. The
integration test uses the local-file connector so it does not need Kafka.

#### `robust-ctl mqtt connector list`

```bash
robust-ctl mqtt connector list --connector-name <run-id>-connector
```

```text
connector list result:
+----------------+----------------+------------------+----------+--------+-----------+-------------+-------------+
| connector name | connector type | connector config | topic id | status | broker id | create time | update time |
+================+================+==================+==========+========+===========+=============+============+
| <run-id>-connector | file | {"LocalFile":...} | <run-id>/messages | Running | 1 | <timestamp> | <timestamp> |
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

`schema list` accepts `--schema-name`, and `schema list-bind` requires the same
`--tenant` used by `schema bind`.

For the control-plane, broker-cache, and publish-validation flow, see the
[Schema Registry Architecture](../Architect/Schema-Registry-Architecture).

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
+------+------+--------+-------+-----+----------+---------------------+-------------------+
| name | desc | tenant | topic | qos | no_local | retain_as_published | retained_handling |
+======+======+========+=======+=====+==========+=====================+===================+
| <run-id>-auto | | default | <run-id>/messages | AtLeastOnce | false | false | OnEverySubscribe |
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

### 3.9 Observability

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

### 3.10 MQTT publish and subscribe

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

With `--retained`, the command confirms retained delivery:

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

## Output

- default: `table`
- machine-readable: `--output json`
- publish/subscribe use `--mqtt-server` (default: configured MQTT TCP port, then `127.0.0.1:1883`); `--server` remains the Admin API endpoint.

## Mutation, Interactive, and Error Output

The basic list commands above show each command immediately followed by its
verified output. This section records the output contracts for mutations,
interactive MQTT commands, and expected failures. Replace angle-bracket values
with the current run's generated names and ports.

Successful mutations return exit status `0` and a confirmation on stdout:

```text
$ robust-ctl mqtt user create --username <run-user> --password <run-password>
Created successfully!
```

Table-mode lists are human-readable:

```text
user list result:
+----------+--------------+
| username | is_superuser |
+==========+==============+
| admin    | true         |
+----------+--------------+
```

JSON-mode lists contain `data` and `total_count` and are emitted as clean JSON
on stdout:

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

Schema binding results are grouped by `data_type` and require the same tenant
for bind and list operations:

```json
{
  "data": [
    { "data_type": "resource", "data": ["<run-id>-schema"] },
    { "data_type": "schema", "data": ["<run-id>/schema-topic"] }
  ],
  "total_count": 2
}
```

Interactive MQTT output is line-oriented and a successful flow emits each
payload once before clean Ctrl-C termination:

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

Expected failures return non-zero status and retain diagnostics on stderr:

```text
# invalid QoS: exit status 1
Exception:Server returned error: code=400, message=... QoS must be 0, 1 or 2

# unavailable Admin API: exit status 1
Exception:Request failed: error sending request for url (http://127.0.0.1:1/api/mqtt/overview)
```

The banner is diagnostic output and does not belong to JSON stdout. Use `jq` on
stdout only when `--output json` is selected.

A complete integration run ends with exit status `0`:

```text
$ ./scripts/test/cli/mqtt/run.sh
[mqtt] retained delivery, QoS 1/2 publish, exact-once messages, and Ctrl-C passed
All MQTT CLI integration checks passed.
```

The verified populated run executed 72 CLI invocations covering table/JSON reads,
stateful resources, MQTT publish/subscribe, and endpoint/error checks.
Duplicate creation, invalid QoS, unavailable Admin API, and invalid MQTT
credentials are expected non-zero cases and are asserted explicitly.
