# MQTT Commands

## Command Shape

```bash
robust-ctl mqtt [--server <addr>] [--output table|json] [--page N] [--limit N] <subcommand>
```

## Common Subcommands

### Overview and basic lists

```bash
robust-ctl mqtt overview
robust-ctl mqtt client list
robust-ctl mqtt session list
robust-ctl mqtt subscribes list
robust-ctl mqtt topic list
```

### User and ACL

```bash
robust-ctl mqtt user list
robust-ctl mqtt user create --username u1 --password p1
robust-ctl mqtt user delete --username u1

robust-ctl mqtt acl list
robust-ctl mqtt acl create --resource-type ClientId --resource-name c1 --topic "a/#" --ip "127.0.0.1" --action All --permission Allow
robust-ctl mqtt acl delete --resource-type ClientId --resource-name c1 --topic "a/#" --ip "127.0.0.1" --action All --permission Allow
```

### Blacklist

```bash
robust-ctl mqtt blacklist list
robust-ctl mqtt blacklist create --blacklist-type ClientId --resource-name c1 --end-time 1735689600 --desc "test"
robust-ctl mqtt blacklist delete --blacklist-type ClientId --resource-name c1
```

### Topic Rewrite / Connector / Schema

```bash
robust-ctl mqtt topic-rewrite list
robust-ctl mqtt connector list
robust-ctl mqtt schema list
robust-ctl mqtt schema list-bind
```

### Observability

```bash
robust-ctl mqtt flapping-detect
robust-ctl mqtt slow-subscribe list
robust-ctl mqtt system-alarm list
```

### Auto Subscribe

```bash
robust-ctl mqtt auto-subscribe list
robust-ctl mqtt auto-subscribe create --topic "a/b" --qos 1
robust-ctl mqtt auto-subscribe delete --topic "a/b"
```

### MQTT publish/subscribe

```bash
robust-ctl mqtt publish --username u1 --password p1 --topic "a/b" --qos 1
robust-ctl mqtt subscribe --username u1 --password p1 --topic "a/b" --qos 1
```

## Output

- default: `table`
- machine-readable: `--output json`
