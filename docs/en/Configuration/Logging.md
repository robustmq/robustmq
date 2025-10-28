# Logging Configuration

RobustMQ uses a modern logging system based on `tracing` that supports multiple output methods and flexible filtering configurations.

## Configuration File

Logging configuration file: `config/server-tracing.toml`

## Basic Configuration

### Console Output

```toml
[stdout]
kind = "console"
level = "info"
ansi = true
formatter = "pretty"
```

### File Output

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

## Configuration Parameters

### Common Parameters

| Parameter | Type | Description | Options |
|-----------|------|-------------|---------|
| `kind` | String | Appender type | `console`, `rolling_file`, `TokioConsole` |
| `level` | String | Log level | `off`, `error`, `warn`, `info`, `debug`, `trace` |
| `ansi` | Boolean | Enable color output | `true`, `false` |
| `formatter` | String | Output format | `compact`, `pretty`, `json` |

### File Output Parameters

| Parameter | Type | Description | Options |
|-----------|------|-------------|---------|
| `rotation` | String | Rotation strategy | `minutely`, `hourly`, `daily`, `never` |
| `directory` | String | Log directory | File path |
| `prefix` | String | File prefix | Any string |
| `suffix` | String | File suffix | Any string |
| `max_log_files` | Number | Maximum file count | Positive integer |

## Advanced Configuration

### Module Filtering

```toml
[http_request]
kind = "rolling_file"
targets = [{ path = "admin_server::server", level = "info" }]
rotation = "daily"
directory = "./data/broker/logs"
prefix = "http_request"
suffix = "log"
max_log_files = 50
```

### Multi-target Filtering

```toml
[openraft_except_engine]
kind = "rolling_file"
targets = [
    { path = "openraft", level = "info" },
    { path = "openraft::engine", level = "off" },
]
rotation = "daily"
directory = "./data/meta-service/logs"
prefix = "openraft-except-engine"
suffix = "log"
max_log_files = 10
```

### Tokio Debugging

```toml
[tokio_console]
kind = "TokioConsole"
bind = "127.0.0.1:5675"
grpc_web = true
```

## Preset Configurations

### Development Environment

```toml
[stdout]
kind = "console"
level = "debug"
formatter = "pretty"
ansi = true

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

## Modular Logging

RobustMQ supports module-based log separation:

```toml
# HTTP request logs
[http_request]
kind = "rolling_file"
targets = [{ path = "admin_server::server", level = "info" }]
directory = "./data/broker/logs"
prefix = "http_request"

# Raft consensus logs
[raft]
kind = "rolling_file"
targets = [{ path = "openraft", level = "info" }]
directory = "./data/broker/logs"
prefix = "raft"

# Journal service logs
[journal]
kind = "rolling_file"
targets = [{ path = "journal_server", level = "info" }]
directory = "./data/broker/logs"
prefix = "journal"

# Meta service logs
[meta]
kind = "rolling_file"
targets = [{ path = "meta_service", level = "info" }]
directory = "./data/broker/logs"
prefix = "meta"
```

Through proper logging configuration, you can effectively improve RobustMQ's observability and operational efficiency.
