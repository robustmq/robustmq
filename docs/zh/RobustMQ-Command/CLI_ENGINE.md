# Engine 命令

## 1. 命令结构

```bash
robust-ctl engine [--server <addr>] [--output table|json] [--page N] [--limit N] <subcommand>
```

## 2. 子命令总览

- `shard list/create/delete`
- `segment list`
- `offset by-timestamp/by-group/commit`

## 3. 详细命令

### 3.1 shard

#### shard list

语法：

```bash
robust-ctl engine shard list [--shard-name <name>]
```

示例：

```bash
robust-ctl engine shard list
robust-ctl engine --page 1 --limit 20 shard list
robust-ctl engine shard list --shard-name demo
robust-ctl engine --output json shard list
```

#### shard create

语法：

```bash
robust-ctl engine shard create --shard-name <name> --config <json>
```

示例：

```bash
robust-ctl engine shard create \
  --shard-name demo \
  --config '{"replica_num":1,"storage_type":"EngineMemory","max_segment_size":1073741824,"retention_sec":86400}'
```

#### shard delete

```bash
robust-ctl engine shard delete --shard-name demo
```

### 3.2 segment

#### segment list

语法：

```bash
robust-ctl engine segment list --shard-name <name>
```

示例：

```bash
robust-ctl engine segment list --shard-name demo
robust-ctl engine --output json segment list --shard-name demo
```

### 3.3 offset

#### by-timestamp

语法：

```bash
robust-ctl engine offset by-timestamp \
  --shard-name <name> \
  --timestamp <unix_ts> \
  --strategy earliest|latest
```

示例：

```bash
robust-ctl engine offset by-timestamp \
  --shard-name demo \
  --timestamp 1735689600 \
  --strategy earliest
```

#### by-group

```bash
robust-ctl engine offset by-group --group-name g1
```

#### commit

语法：

```bash
robust-ctl engine offset commit --group-name <name> --offsets-json <json_map>
```

示例：

```bash
robust-ctl engine offset commit \
  --group-name g1 \
  --offsets-json '{"demo":1024,"demo-2":2048}'
```

`--offsets-json` 必须是 JSON 对象：`{"<shard_name>": <offset>}`。

## 4. 输出说明

- 默认：`table`
- 自动化：`--output json`
