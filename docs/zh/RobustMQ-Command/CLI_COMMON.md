# RobustMQ CLI 通用说明

`robust-ctl` 是 RobustMQ 的管理 CLI，当前按三大域组织：

- `cluster`：集群状态、健康检查、集群配置
- `mqtt`：MQTT 管理面能力（用户、ACL、黑名单、连接器、Schema、订阅等）
- `engine`：存储引擎能力（shard/segment/offset）

## 1. 基本语法

```bash
robust-ctl <domain> [domain-options] <subcommand>
```

## 2. 参数与输出规范

### 2.1 连接参数

- `--server, -s`：管理 API 地址，默认 `127.0.0.1:8080`

示例：

```bash
robust-ctl mqtt --server 10.10.10.8:8080 user list
```

### 2.2 输出格式

- `--output table|json`
  - `table`（默认）：适合人工排查
  - `json`：适合脚本处理

示例：

```bash
robust-ctl cluster --output json healthy
robust-ctl mqtt --output json client list
```

### 2.3 分页参数（列表命令）

- `--page`：页码，默认 `1`
- `--limit`：每页大小，默认 `100`

示例：

```bash
robust-ctl mqtt --page 2 --limit 50 session list
robust-ctl engine --page 1 --limit 20 shard list
```

## 3. 快速开始

```bash
# 查看总帮助
robust-ctl --help

# 查看域帮助
robust-ctl cluster --help
robust-ctl mqtt --help
robust-ctl engine --help

# 常用命令
robust-ctl cluster status
robust-ctl mqtt overview
robust-ctl engine shard list
```

## 4. 命令文档导航

- [Cluster 命令](CLI_CLUSTER.md)
- [MQTT 命令](CLI_MQTT.md)
- [Engine 命令](CLI_ENGINE.md)
