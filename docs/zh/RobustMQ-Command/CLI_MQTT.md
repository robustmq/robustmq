# MQTT 命令

## 1. 命令结构

```bash
robust-ctl mqtt [--server <addr>] [--output table|json] [--page N] [--limit N] <subcommand>
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

```bash
robust-ctl mqtt overview
robust-ctl mqtt session list
robust-ctl mqtt subscribes list
robust-ctl mqtt client list
robust-ctl mqtt topic list
```

分页示例：

```bash
robust-ctl mqtt --page 2 --limit 50 session list
robust-ctl mqtt --output json --page 1 --limit 20 client list
```

### 3.2 用户管理

#### user list

```bash
robust-ctl mqtt user list
```

#### user create

```bash
robust-ctl mqtt user create \
  --username u1 \
  --password p1 \
  --is-superuser
```

#### user delete

```bash
robust-ctl mqtt user delete --username u1
```

### 3.3 ACL 管理

#### acl list

```bash
robust-ctl mqtt acl list
```

#### acl create

```bash
robust-ctl mqtt acl create \
  --cluster-name c1 \
  --resource-type ClientId \
  --resource-name client-01 \
  --topic "a/#" \
  --ip "127.0.0.1" \
  --action All \
  --permission Allow
```

#### acl delete

```bash
robust-ctl mqtt acl delete \
  --cluster-name c1 \
  --resource-type ClientId \
  --resource-name client-01 \
  --topic "a/#" \
  --ip "127.0.0.1" \
  --action All \
  --permission Allow
```

### 3.4 黑名单管理

#### blacklist list

```bash
robust-ctl mqtt blacklist list
```

#### blacklist create

```bash
robust-ctl mqtt blacklist create \
  --cluster-name c1 \
  --blacklist-type ClientId \
  --resource-name bad-client \
  --end-time 1735689600 \
  --desc "temporary block"
```

#### blacklist delete

```bash
robust-ctl mqtt blacklist delete \
  --cluster-name c1 \
  --blacklist-type ClientId \
  --resource-name bad-client
```

### 3.5 Topic Rewrite

#### list

```bash
robust-ctl mqtt topic-rewrite list
```

#### create

```bash
robust-ctl mqtt topic-rewrite create \
  --action Publish \
  --source-topic "src/+" \
  --dest-topic "dst/+" \
  --regex "src/(.*)"
```

#### delete

```bash
robust-ctl mqtt topic-rewrite delete \
  --action Publish \
  --source-topic "src/+"
```

### 3.6 Connector

#### list

> 当前 CLI 实现里 `connector list` 需要传 `--connector-name` 参数。

```bash
robust-ctl mqtt connector list --connector-name demo
```

#### create

```bash
robust-ctl mqtt connector create \
  --connector-name c1 \
  --connector-type kafka \
  --config '{"brokers":"127.0.0.1:9092","topic":"demo"}' \
  --topic-name demo/topic
```

#### delete

```bash
robust-ctl mqtt connector delete --connector-name c1
```

### 3.7 Schema

#### list

> 当前 CLI 实现里 `schema list` 需要传 `--schema-name` 参数（用于命令参数对齐）。

```bash
robust-ctl mqtt schema list --schema-name demo
```

#### create

```bash
robust-ctl mqtt schema create \
  --schema-name s1 \
  --schema-type json \
  --schema '{"type":"object"}' \
  --desc "demo schema"
```

#### delete

```bash
robust-ctl mqtt schema delete --schema-name s1
```

#### list-bind / bind / unbind

```bash
robust-ctl mqtt schema list-bind --schema-name s1 --resource-name demo/topic
robust-ctl mqtt schema bind --schema-name s1 --resource-name demo/topic
robust-ctl mqtt schema unbind --schema-name s1 --resource-name demo/topic
```

### 3.8 Auto Subscribe

```bash
robust-ctl mqtt auto-subscribe list

robust-ctl mqtt auto-subscribe create \
  --topic "a/b" \
  --qos 1 \
  --no-local \
  --retain-as-published \
  --retained-handling 0

robust-ctl mqtt auto-subscribe delete --topic "a/b"
```

### 3.9 可观测命令

```bash
robust-ctl mqtt flapping-detect
robust-ctl mqtt slow-subscribe list
robust-ctl mqtt system-alarm list
```

### 3.10 MQTT 发布与订阅

#### publish（交互式）

```bash
robust-ctl mqtt publish \
  --username u1 \
  --password p1 \
  --topic "demo/topic" \
  --qos 1 \
  --retained
```

#### subscribe

```bash
robust-ctl mqtt subscribe \
  --username u1 \
  --password p1 \
  --topic "demo/topic" \
  --qos 1
```

## 4. 输出说明

- 默认：`table`
- 脚本化：`--output json`

示例：

```bash
robust-ctl mqtt --output json user list
```
