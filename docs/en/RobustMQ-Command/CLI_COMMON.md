# RobustMQ CLI Common Guide

`robust-ctl` is organized into 3 domains:

- `cluster`: cluster status and configuration
- `mqtt`: MQTT management APIs
- `engine`: storage engine APIs (shard/segment/offset)

## Basic Syntax

```bash
robust-ctl <domain> [options] <subcommand>
```

## Common Options

All domains support:

- `--server,-s`: admin API address, default `127.0.0.1:8080`
- `--output`: `table` (default) or `json`

List-like commands support:

- `--page`: page number (default 1)
- `--limit`: page size (default 100)

## Quick Examples

```bash
# cluster
robust-ctl cluster status
robust-ctl cluster healthy --output json

# mqtt
robust-ctl mqtt --limit 20 --page 1 user list
robust-ctl mqtt --output json overview

# engine
robust-ctl engine shard list
robust-ctl engine segment list --shard-name demo
```

## Navigation

- [Cluster Commands](CLI_CLUSTER.md)
- [MQTT Commands](CLI_MQTT.md)
- [Engine Commands](CLI_ENGINE.md)
