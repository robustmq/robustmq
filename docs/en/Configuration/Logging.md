# Logging Configuration

RobustMQ uses a logging system based on `tracing` and `tracing-subscriber`, supporting multiple output appenders and flexible filtering. Each appender is defined as a TOML section in the tracing configuration file.

## Configuration Files

Logging involves two configuration layers:

1. **`config/server.toml`** — specifies the path to the tracing config and the log output directory:

```toml
[log]
log_config = "./config/server-tracing.toml"
log_path = "./logs"
```

2. **`config/server-tracing.toml`** — defines all log appenders (console, file, tokio-console).

---

## Appender Types

Each TOML section in `server-tracing.toml` defines one appender. The section name is arbitrary (e.g. `[stdout]`, `[server]`, `[raft]`). The `kind` field determines the appender type.

| `kind` | Description |
|--------|-------------|
| `console` | Writes to stdout |
| `rolling_file` | Writes to rotating log files |
| `tokio_console` | Enables [tokio-console](https://github.com/tokio-rs/console) debugging |

---

## Filtering

Each appender must specify exactly **one** of the following filter modes:

### Level filter

Applies a global log level to all modules.

```toml
level = "info"
```

### Target filter

Filters logs from a single module path.

```toml
target = { path = "openraft", level = "info" }
```

### Targets filter

Filters logs from multiple module paths with independent levels. Useful for including one module while excluding a sub-module.

```toml
targets = [
    { path = "openraft", level = "info" },
    { path = "openraft::engine", level = "off" },
]
```

**Available log levels:** `off`, `error`, `warn`, `info`, `debug`, `trace`

---

## Console Appender

Writes log output to stdout.

```toml
[stdout]
kind = "console"
level = "info"
```

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `kind` | string | yes | — | Must be `"console"` |
| `level` / `target` / `targets` | — | yes | — | Filter mode (choose one) |
| `ansi` | bool | no | `true` | Enable ANSI color output |
| `formatter` | string | no | default format | Output format: `compact`, `pretty`, `json` |

---

## Rolling File Appender

Writes logs to files with time-based rotation and automatic cleanup.

```toml
[server]
kind = "rolling_file"
level = "info"
rotation = "daily"
directory = "./data/broker/logs"
prefix = "server"
suffix = "log"
max_log_files = 50
```

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `kind` | string | yes | — | Must be `"rolling_file"` |
| `level` / `target` / `targets` | — | yes | — | Filter mode (choose one) |
| `rotation` | string | yes | — | Rotation strategy: `minutely`, `hourly`, `daily`, `never` |
| `directory` | string | yes | — | Log file output directory |
| `prefix` | string | no | none | Log filename prefix |
| `suffix` | string | no | none | Log filename suffix |
| `max_log_files` | number | no | unlimited | Maximum number of rotated log files to keep |
| `ansi` | bool | no | `true` | Enable ANSI color in file output (usually set `false` for files) |
| `formatter` | string | no | default format | Output format: `compact`, `pretty`, `json` |

**Log file naming:** `{prefix}.{date}.{suffix}` — for example, with `prefix = "server"`, `suffix = "log"`, `rotation = "daily"`, the file is named like `server.2025-02-06.log`.

---

## Tokio Console Appender

Enables the [tokio-console](https://github.com/tokio-rs/console) subscriber for debugging async runtime behavior.

```toml
[tokio_console]
kind = "tokio_console"
bind = "127.0.0.1:5674"
grpc_web = true
```

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `kind` | string | yes | — | Must be `"tokio_console"` |
| `bind` | string | no | `127.0.0.1:6669` | gRPC server bind address |
| `grpc_web` | bool | no | `false` | Enable gRPC-Web support |

> Note: This appender does not support `level`, `ansi`, or `formatter` parameters.

---

## Default Configuration

The default `config/server-tracing.toml` ships with the following appenders:

```toml
# Console output
[stdout]
kind = "console"
level = "info"

# Main server logs
[server]
kind = "rolling_file"
level = "info"
rotation = "daily"
directory = "./data/broker/logs"
prefix = "server"
suffix = "log"
max_log_files = 50

# HTTP API request logs
[http_request]
kind = "rolling_file"
targets = [{ path = "admin_server::server", level = "info" }]
rotation = "daily"
directory = "./data/broker/logs"
prefix = "http_request"
suffix = "log"
max_log_files = 50

# Raft consensus logs
[raft]
kind = "rolling_file"
targets = [{ path = "openraft", level = "info" }]
rotation = "daily"
directory = "./data/broker/logs"
prefix = "raft"
suffix = "log"
max_log_files = 50

# Meta service logs
[meta]
kind = "rolling_file"
targets = [{ path = "meta_service", level = "info" }]
rotation = "daily"
directory = "./data/broker/logs"
prefix = "meta"
suffix = "log"
max_log_files = 50

# Openraft logs excluding engine module
[openraft_except_engine]
kind = "rolling_file"
targets = [
    { path = "openraft", level = "info" },
    { path = "openraft::engine", level = "off" },
]
rotation = "daily"
directory = "./data/broker/logs"
prefix = "openraft-except-engine"
suffix = "log"
max_log_files = 10
```

---

## Preset Examples

### Development Environment

```toml
[stdout]
kind = "console"
level = "debug"
formatter = "pretty"

[server]
kind = "rolling_file"
level = "debug"
rotation = "hourly"
directory = "./logs"
prefix = "dev"
suffix = "log"
max_log_files = 10
```

### Production Environment

```toml
[stdout]
kind = "console"
level = "info"
formatter = "compact"

[server]
kind = "rolling_file"
level = "info"
rotation = "daily"
directory = "/var/log/robustmq"
prefix = "server"
suffix = "log"
max_log_files = 30

[error]
kind = "rolling_file"
level = "error"
rotation = "daily"
directory = "/var/log/robustmq"
prefix = "error"
suffix = "log"
max_log_files = 90
```
