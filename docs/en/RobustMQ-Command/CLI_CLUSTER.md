# Cluster Commands

## Command Shape

```bash
robust-ctl cluster [--server <addr>] [--output table|json] <subcommand>
```

## Subcommands

### 1) status

Check cluster status.

```bash
robust-ctl cluster status
robust-ctl cluster --output json status
```

### 2) healthy

Check cluster healthy result.

```bash
robust-ctl cluster healthy
robust-ctl cluster --output json healthy
```

### 3) config

#### get

```bash
robust-ctl cluster config get
```

#### set

```bash
robust-ctl cluster config set \
  --config-type FlappingDetect \
  --config '{"enable":true}'
```

`config` is passed through to server-side config API as-is.
