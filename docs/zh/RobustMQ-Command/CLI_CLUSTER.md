# Cluster 命令

## 1. 命令结构

```bash
robust-ctl cluster [--server <addr>] [--output table|json] <subcommand>
```

## 2. 子命令总览

- `status`：查看集群状态
- `healthy`：查看健康状态
- `config get`：获取集群配置
- `config set`：设置动态配置

## 3. 详细命令

### 3.1 status

语法：

```bash
robust-ctl cluster status
```

示例：

```bash
robust-ctl cluster status
robust-ctl cluster --output json status
robust-ctl cluster --server 192.168.10.15:8080 status
```

### 3.2 healthy

语法：

```bash
robust-ctl cluster healthy
```

示例：

```bash
robust-ctl cluster healthy
robust-ctl cluster --output json healthy
```

### 3.3 config get

语法：

```bash
robust-ctl cluster config get
```

示例：

```bash
robust-ctl cluster config get
robust-ctl cluster --output json config get
```

### 3.4 config set

语法：

```bash
robust-ctl cluster config set --config-type <TYPE> --config <JSON_STRING>
```

参数：

- `--config-type`：配置类型（示例：`FlappingDetect`）
- `--config`：配置 JSON 字符串

示例：

```bash
robust-ctl cluster config set \
  --config-type FlappingDetect \
  --config '{"enable":true}'

robust-ctl cluster config set \
  --config-type SlowSubscribe \
  --config '{"enable":false}'
```

## 4. 说明

- `config set` 当前为透传模型，具体字段由服务端按 `config-type` 解析。
- 推荐对自动化脚本统一使用 `--output json`。
