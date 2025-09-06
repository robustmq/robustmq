# RobustMQ CLI 命令行使用手册

## 概述

`robust-ctl` 是 RobustMQ 的命令行管理工具，基于 Rust clap 库构建，提供对 MQTT 代理、集群配置和日志引擎的管理功能。

## 安装和构建

```bash
# 构建项目
cargo build --release

# 运行工具
./target/release/robust-ctl --help
```

## 基本语法

```bash
robust-ctl [OPTIONS] <COMMAND>
```

## 全局选项

- `--help, -h`: 显示帮助信息
- `--version, -V`: 显示版本信息

## 主要命令

### 1. MQTT 代理管理 (`mqtt`)

MQTT 代理相关操作，包括会话管理、用户管理、ACL、黑名单等。

#### 基本语法
```bash
robust-ctl mqtt [OPTIONS] <ACTION>
```

#### 选项
- `--server, -s <SERVER>`: 服务器地址 (默认: 127.0.0.1:8080)

---

#### 1.1 会话管理 (`session`)

管理 MQTT 会话。

```bash
# 列出所有会话
robust-ctl mqtt session list
```

---

#### 1.2 订阅管理 (`subscribes`)

管理 MQTT 订阅。

```bash
# 列出所有订阅
robust-ctl mqtt subscribes list
```

---

#### 1.3 用户管理 (`user`)

管理 MQTT 用户账户。

```bash
# 列出所有用户
robust-ctl mqtt user list

# 创建用户
robust-ctl mqtt user create \
  --username <USERNAME> \
  --password <PASSWORD> \
  [--is-superuser]

# 删除用户
robust-ctl mqtt user delete --username <USERNAME>
```

**参数说明：**
- `--username, -u`: 用户名 (必需)
- `--password, -p`: 密码 (必需)
- `--is-superuser, -i`: 是否为超级用户 (可选，默认 false)

---

#### 1.4 访问控制列表 (`acl`)

管理 MQTT 访问控制规则。

```bash
# 列出所有 ACL 规则
robust-ctl mqtt acl list

# 创建 ACL 规则
robust-ctl mqtt acl create \
  --cluster-name <CLUSTER_NAME> \
  --resource-type <RESOURCE_TYPE> \
  --resource-name <RESOURCE_NAME> \
  --topic <TOPIC> \
  --ip <IP> \
  --action <ACTION> \
  --permission <PERMISSION>

# 删除 ACL 规则
robust-ctl mqtt acl delete \
  --cluster-name <CLUSTER_NAME> \
  --resource-type <RESOURCE_TYPE> \
  --resource-name <RESOURCE_NAME> \
  --topic <TOPIC> \
  --ip <IP> \
  --action <ACTION> \
  --permission <PERMISSION>
```

**参数说明：**
- `--cluster-name, -c`: 集群名称 (必需)
- `--resource-type`: 资源类型 (ClientId, Username, IpAddress 等)
- `--resource-name`: 资源名称 (必需)
- `--topic`: 主题 (必需)
- `--ip`: IP 地址 (必需)
- `--action`: 操作类型 (All, Publish, Subscribe, PubSub)
- `--permission`: 权限 (Allow, Deny)

---

#### 1.5 黑名单管理 (`blacklist`)

管理 MQTT 黑名单。

```bash
# 列出所有黑名单
robust-ctl mqtt blacklist list

# 创建黑名单条目
robust-ctl mqtt blacklist create \
  --cluster-name <CLUSTER_NAME> \
  --blacklist-type <BLACKLIST_TYPE> \
  --resource-name <RESOURCE_NAME> \
  --end-time <END_TIME> \
  --desc <DESCRIPTION>

# 删除黑名单条目
robust-ctl mqtt blacklist delete \
  --cluster-name <CLUSTER_NAME> \
  --blacklist-type <BLACKLIST_TYPE> \
  --resource-name <RESOURCE_NAME>
```

**参数说明：**
- `--cluster-name, -c`: 集群名称 (必需)
- `--blacklist-type`: 黑名单类型 (ClientId, IpAddress, Username)
- `--resource-name, -r`: 资源名称 (必需)
- `--end-time`: 结束时间 (Unix 时间戳) (必需)
- `--desc`: 描述 (必需)

---

#### 1.6 客户端连接管理 (`client`)

管理 MQTT 客户端连接。

```bash
# 列出所有客户端连接
robust-ctl mqtt client list
```

---

#### 1.7 主题管理 (`topic`)

管理 MQTT 主题。

```bash
# 列出所有主题
robust-ctl mqtt topic list
```

---

#### 1.8 主题重写规则 (`topic-rewrite`)

管理主题重写规则。

```bash
# 列出所有主题重写规则
robust-ctl mqtt topic-rewrite list

# 创建主题重写规则
robust-ctl mqtt topic-rewrite create \
  --action <ACTION> \
  --source-topic <SOURCE_TOPIC> \
  --dest-topic <DEST_TOPIC> \
  --regex <REGEX>

# 删除主题重写规则
robust-ctl mqtt topic-rewrite delete \
  --action <ACTION> \
  --source-topic <SOURCE_TOPIC>
```

**参数说明：**
- `--action, -a`: 操作类型 (必需)
- `--source-topic, -s`: 源主题 (必需)
- `--dest-topic, -d`: 目标主题 (创建时必需)
- `--regex, -r`: 正则表达式 (创建时必需)

---

#### 1.9 连接器管理 (`connector`)

管理 MQTT 连接器。

```bash
# 列出所有连接器
robust-ctl mqtt connector list --connector-name <NAME>

# 创建连接器
robust-ctl mqtt connector create \
  --connector-name <CONNECTOR_NAME> \
  --connector-type <CONNECTOR_TYPE> \
  --config <CONFIG> \
  --topic-id <TOPIC_ID>

# 删除连接器
robust-ctl mqtt connector delete --connector-name <CONNECTOR_NAME>
```

**参数说明：**
- `--connector-name, -c`: 连接器名称 (必需)
- `--connector-type, -c`: 连接器类型 (创建时必需)
- `--config, -c`: 配置信息 (创建时必需)
- `--topic-id, -t`: 主题 ID (创建时必需)

---

#### 1.10 模式管理 (`schema`)

管理 MQTT 消息模式。

```bash
# 列出所有模式
robust-ctl mqtt schema list --schema-name <NAME>

# 创建模式
robust-ctl mqtt schema create \
  --schema-name <SCHEMA_NAME> \
  --schema-type <SCHEMA_TYPE> \
  --schema <SCHEMA> \
  --desc <DESCRIPTION>

# 删除模式
robust-ctl mqtt schema delete --schema-name <SCHEMA_NAME>

# 列出模式绑定
robust-ctl mqtt schema list-bind \
  --schema-name <SCHEMA_NAME> \
  --resource-name <RESOURCE_NAME>

# 绑定模式
robust-ctl mqtt schema bind \
  --schema-name <SCHEMA_NAME> \
  --resource-name <RESOURCE_NAME>

# 解绑模式
robust-ctl mqtt schema unbind \
  --schema-name <SCHEMA_NAME> \
  --resource-name <RESOURCE_NAME>
```

**参数说明：**
- `--schema-name, -s`: 模式名称 (必需)
- `--schema-type, -t`: 模式类型 (创建时必需)
- `--schema, -s`: 模式定义 (创建时必需)
- `--desc, -d`: 描述 (创建时必需)
- `--resource-name, -r`: 资源名称 (绑定操作时必需)

---

#### 1.11 自动订阅规则 (`auto-subscribe`)

管理自动订阅规则。

```bash
# 列出所有自动订阅规则
robust-ctl mqtt auto-subscribe list

# 创建自动订阅规则
robust-ctl mqtt auto-subscribe create \
  --topic <TOPIC> \
  [--qos <QOS>] \
  [--no-local] \
  [--retain-as-published] \
  [--retained-handling <HANDLING>]

# 删除自动订阅规则
robust-ctl mqtt auto-subscribe delete --topic <TOPIC>
```

**参数说明：**
- `--topic, -t`: 主题 (必需)
- `--qos, -q`: QoS 级别 (可选，默认 0)
- `--no-local, -n`: 不接收本地消息 (可选，默认 false)
- `--retain-as-published, -r`: 保持发布状态 (可选，默认 false)
- `--retained-handling, -R`: 保留消息处理方式 (可选，默认 0)

---

#### 1.12 发布消息 (`publish`)

发布 MQTT 消息。

```bash
# 发布消息 (交互模式)
robust-ctl mqtt publish \
  --username <USERNAME> \
  --password <PASSWORD> \
  --topic <TOPIC> \
  [--qos <QOS>] \
  [--retained]
```

**参数说明：**
- `--username, -u`: 用户名 (必需)
- `--password, -p`: 密码 (必需)
- `--topic, -t`: 主题 (必需)
- `--qos, -q`: QoS 级别 (可选，默认 0)
- `--retained`: 保留消息 (可选，默认 false)

---

#### 1.13 订阅消息 (`subscribe`)

订阅 MQTT 消息。

```bash
# 订阅消息
robust-ctl mqtt subscribe \
  --username <USERNAME> \
  --password <PASSWORD> \
  --topic <TOPIC> \
  [--qos <QOS>]
```

**参数说明：**
- `--username, -u`: 用户名 (必需)
- `--password, -p`: 密码 (必需)
- `--topic, -t`: 主题 (必需)
- `--qos, -q`: QoS 级别 (可选，默认 0)

---

#### 1.14 可观测性功能

##### 慢订阅监控 (`slow-subscribe`)
```bash
# 列出慢订阅
robust-ctl mqtt slow-subscribe list
```

##### 连接抖动检测 (`flapping-detect`)
```bash
# 列出连接抖动检测结果
robust-ctl mqtt flapping-detect
```

##### 系统告警 (`system-alarm`)
```bash
# 列出系统告警
robust-ctl mqtt system-alarm list
```

---

### 2. 集群管理 (`cluster`)

集群配置管理。

#### 基本语法
```bash
robust-ctl cluster [OPTIONS] <ACTION>
```

#### 选项
- `--server, -s <SERVER>`: 服务器地址 (默认: 127.0.0.1:8080)

#### 配置管理 (`config`)

```bash
# 获取集群配置
robust-ctl cluster config get
```

---

### 3. 日志引擎管理 (`journal`)

日志引擎相关操作 (目前处于开发中)。

#### 基本语法
```bash
robust-ctl journal [OPTIONS]
```

#### 选项
- `--server, -s <SERVER>`: 服务器地址 (默认: 127.0.0.1:8080)
- `--action, -a <ACTION>`: 操作类型 (默认: status)

---

## 使用示例

### 基础操作示例

```bash
# 1. 查看帮助
robust-ctl --help
robust-ctl mqtt --help
robust-ctl mqtt user --help

# 2. 连接到指定服务器
robust-ctl mqtt --server 192.168.1.100:8080 session list

# 3. 用户管理
robust-ctl mqtt user create --username testuser --password testpass
robust-ctl mqtt user list
robust-ctl mqtt user delete --username testuser

# 4. ACL 管理
robust-ctl mqtt acl create \
  --cluster-name mycluster \
  --resource-type ClientId \
  --resource-name client001 \
  --topic "sensor/+" \
  --ip "192.168.1.0/24" \
  --action Publish \
  --permission Allow

# 5. 发布和订阅
robust-ctl mqtt publish \
  --username testuser \
  --password testpass \
  --topic "test/topic" \
  --qos 1

robust-ctl mqtt subscribe \
  --username testuser \
  --password testpass \
  --topic "test/topic" \
  --qos 1

# 6. 集群配置
robust-ctl cluster config get
```

### 高级管理示例

```bash
# 1. 黑名单管理
robust-ctl mqtt blacklist create \
  --cluster-name mycluster \
  --blacklist-type ClientId \
  --resource-name malicious_client \
  --end-time 1735689600 \
  --desc "Blocked due to suspicious activity"

# 2. 主题重写规则
robust-ctl mqtt topic-rewrite create \
  --action redirect \
  --source-topic "old/topic/+" \
  --dest-topic "new/topic/+" \
  --regex "old/(.*)"

# 3. 自动订阅规则
robust-ctl mqtt auto-subscribe create \
  --topic "system/alerts/+" \
  --qos 2 \
  --no-local \
  --retain-as-published

# 4. 模式管理
robust-ctl mqtt schema create \
  --schema-name temperature_schema \
  --schema-type json \
  --schema '{"type":"object","properties":{"temp":{"type":"number"}}}' \
  --desc "Temperature sensor data schema"
```

---

## 错误处理

工具提供详细的错误信息和帮助：

- 使用 `--help` 或 `-h` 获取命令帮助
- 检查服务器连接状态
- 验证参数格式和权限
- 查看详细的错误消息和建议

---

## 配置文件

工具默认连接到 `127.0.0.1:8080`，可以通过 `--server` 参数指定不同的服务器地址。

---

## 注意事项

1. **权限要求**: 某些操作需要管理员权限
2. **网络连接**: 确保能够访问 RobustMQ 服务器
3. **参数验证**: 工具会验证参数格式和有效性
4. **交互模式**: 发布和订阅命令支持交互模式
5. **分页支持**: 列表命令支持分页显示 (默认每页 10000 条)

---

## 版本信息

- 工具版本: 0.0.1
- 作者: RobustMQ Team
- 基于: Rust clap 库

---

## 更多信息

更多详细信息请参考 [RobustMQ 官方文档](https://robustmq.com)。
