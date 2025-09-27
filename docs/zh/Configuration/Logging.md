# 日志配置

RobustMQ 使用基于 `tracing` 的现代化日志系统，支持多种输出方式和灵活的过滤配置。

## 配置文件

日志配置文件：`config/server-tracing.toml`

## 基本配置

### 控制台输出

```toml
[stdout]
kind = "console"
level = "info"
ansi = true
formatter = "pretty"
```

### 文件输出

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

## 配置参数

### 通用参数

| 参数 | 类型 | 说明 | 可选值 |
|------|------|------|--------|
| `kind` | String | 输出器类型 | `console`, `rolling_file`, `TokioConsole` |
| `level` | String | 日志级别 | `off`, `error`, `warn`, `info`, `debug`, `trace` |
| `ansi` | Boolean | 启用颜色输出 | `true`, `false` |
| `formatter` | String | 输出格式 | `compact`, `pretty`, `json` |

### 文件输出参数

| 参数 | 类型 | 说明 | 可选值 |
|------|------|------|--------|
| `rotation` | String | 轮转策略 | `minutely`, `hourly`, `daily`, `never` |
| `directory` | String | 日志目录 | 文件路径 |
| `prefix` | String | 文件前缀 | 任意字符串 |
| `suffix` | String | 文件后缀 | 任意字符串 |
| `max_log_files` | Number | 最大文件数 | 正整数 |

## 高级配置

### 模块过滤

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

### 多目标过滤

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

### Tokio 调试

```toml
[tokio_console]
kind = "TokioConsole"
bind = "127.0.0.1:5675"
grpc_web = true
```

## 预设配置

### 开发环境

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

### 生产环境

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

## 模块化日志

RobustMQ 支持按模块分离日志：

```toml
# HTTP 请求日志
[http_request]
kind = "rolling_file"
targets = [{ path = "admin_server::server", level = "info" }]
directory = "./data/broker/logs"
prefix = "http_request"

# Raft 共识日志
[raft]
kind = "rolling_file"
targets = [{ path = "openraft", level = "info" }]
directory = "./data/broker/logs"
prefix = "raft"

# Journal 服务日志
[journal]
kind = "rolling_file"
targets = [{ path = "journal_server", level = "info" }]
directory = "./data/broker/logs"
prefix = "journal"

# Meta 服务日志
[meta]
kind = "rolling_file"
targets = [{ path = "meta_service", level = "info" }]
directory = "./data/broker/logs"
prefix = "meta"
```

通过合理的日志配置，可以有效提升 RobustMQ 的可观测性和运维效率。
