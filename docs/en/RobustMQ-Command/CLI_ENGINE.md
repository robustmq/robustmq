# Engine Commands

## Command Shape

```bash
robust-ctl engine [--server <addr>] [--output table|json] [--page N] [--limit N] <subcommand>
```

## shard

```bash
robust-ctl engine shard list
robust-ctl engine shard list --shard-name demo
robust-ctl engine shard create --shard-name demo --config '{"replica":1}'
robust-ctl engine shard delete --shard-name demo
```

## segment

```bash
robust-ctl engine segment list --shard-name demo
```

## offset

```bash
robust-ctl engine offset by-timestamp --shard-name demo --timestamp 1735689600 --strategy earliest
robust-ctl engine offset by-group --group-name g1
robust-ctl engine offset commit --group-name g1 --offsets-json '{"demo":1024}'
```

`--offsets-json` must be a JSON object: `{"<shard_name>": <offset>}`.
