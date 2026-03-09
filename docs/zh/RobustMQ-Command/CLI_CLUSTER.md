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

### 3.5 tenant

管理集群租户（多租户支持）。

#### 3.5.1 tenant list

列出所有租户。

语法：

```bash
robust-ctl cluster tenant list
```

示例：

```bash
robust-ctl cluster tenant list
robust-ctl cluster --output json tenant list
robust-ctl cluster --server 192.168.10.15:8080 tenant list
```

输出示例（table 模式）：

```
+-------------+-----------------+-------------+
| tenant_name | desc            | create_time |
+-------------+-----------------+-------------+
| business-a  | 业务 A 租户      | 1738800000  |
| staging     | 预发布环境租户    | 1738900000  |
+-------------+-----------------+-------------+
```

---

#### 3.5.2 tenant create

创建租户。

语法：

```bash
robust-ctl cluster tenant create -n <TENANT_NAME> [-d <DESC>]
```

参数：

| 参数 | 简写 | 必填 | 说明 |
|------|------|------|------|
| `--tenant-name` | `-n` | 是 | 租户名称（1-128 字符） |
| `--desc` | `-d` | 否 | 租户描述（最长 500 字符） |

示例：

```bash
# 创建租户（带描述）
robust-ctl cluster tenant create -n business-a -d "业务 A 租户"

# 创建租户（无描述）
robust-ctl cluster tenant create -n staging
```

---

#### 3.5.3 tenant delete

删除租户。

语法：

```bash
robust-ctl cluster tenant delete -n <TENANT_NAME>
```

参数：

| 参数 | 简写 | 必填 | 说明 |
|------|------|------|------|
| `--tenant-name` | `-n` | 是 | 要删除的租户名称 |

示例：

```bash
robust-ctl cluster tenant delete -n business-a
```

---

## 4. 说明

- `config set` 当前为透传模型，具体字段由服务端按 `config-type` 解析。
- 推荐对自动化脚本统一使用 `--output json`。
- 租户用于逻辑隔离，适用于同一集群服务多个业务或多环境（开发/测试/生产）的场景。
