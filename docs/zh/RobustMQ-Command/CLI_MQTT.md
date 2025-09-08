# MQTT 代理管理命令

## MQTT 代理管理 (`mqtt`)

MQTT 代理相关操作，包括会话管理、用户管理、ACL、黑名单等。

### 基本语法
```bash
robust-ctl mqtt [选项] <操作>
```

### 选项
- `--server, -s <服务器>`: 服务器地址 (默认: 127.0.0.1:8080)

---

### 1.1 会话管理 (`session`)

管理 MQTT 会话。

```bash
# 列出所有会话
robust-ctl mqtt session list
```

---

### 1.2 订阅管理 (`subscribes`)

管理 MQTT 订阅。

```bash
# 列出所有订阅
robust-ctl mqtt subscribes list
```

---

### 1.3 用户管理 (`user`)

管理 MQTT 用户账户。

```bash
# 列出所有用户
robust-ctl mqtt user list

# 创建用户
robust-ctl mqtt user create \
  --username <用户名> \
  --password <密码> \
  [--is-superuser]

# 删除用户
robust-ctl mqtt user delete --username <用户名>
```

**参数说明：**
- `--username, -u`: 用户名 (必需)
- `--password, -p`: 密码 (必需)
- `--is-superuser, -i`: 是否为超级用户 (可选，默认 false)

---

### 1.4 访问控制列表 (`acl`)

管理 MQTT 访问控制规则。

```bash
# 列出所有 ACL 规则
robust-ctl mqtt acl list

# 创建 ACL 规则
robust-ctl mqtt acl create \
  --cluster-name <集群名称> \
  --resource-type <资源类型> \
  --resource-name <资源名称> \
  --topic <主题> \
  --ip <IP地址> \
  --action <操作> \
  --permission <权限>

# 删除 ACL 规则
robust-ctl mqtt acl delete \
  --cluster-name <集群名称> \
  --resource-type <资源类型> \
  --resource-name <资源名称> \
  --topic <主题> \
  --ip <IP地址> \
  --action <操作> \
  --permission <权限>
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

### 1.5 黑名单管理 (`blacklist`)

管理 MQTT 黑名单。

```bash
# 列出所有黑名单条目
robust-ctl mqtt blacklist list

# 创建黑名单条目
robust-ctl mqtt blacklist create \
  --cluster-name <集群名称> \
  --blacklist-type <黑名单类型> \
  --resource-name <资源名称> \
  --end-time <结束时间> \
  --desc <描述>

# 删除黑名单条目
robust-ctl mqtt blacklist delete \
  --cluster-name <集群名称> \
  --blacklist-type <黑名单类型> \
  --resource-name <资源名称>
```

**参数说明：**
- `--cluster-name, -c`: 集群名称 (必需)
- `--blacklist-type`: 黑名单类型 (ClientId, IpAddress, Username)
- `--resource-name, -r`: 资源名称 (必需)
- `--end-time`: 结束时间 (Unix 时间戳) (必需)
- `--desc`: 描述 (必需)

---

### 1.6 客户端连接管理 (`client`)

管理 MQTT 客户端连接。

```bash
# 列出所有客户端连接
robust-ctl mqtt client list
```

---

### 1.7 主题管理 (`topic`)

管理 MQTT 主题。

```bash
# 列出所有主题
robust-ctl mqtt topic list
```

---

### 1.8 主题重写规则 (`topic-rewrite`)

管理主题重写规则。

```bash
# 列出所有主题重写规则
robust-ctl mqtt topic-rewrite list

# 创建主题重写规则
robust-ctl mqtt topic-rewrite create \
  --action <操作> \
  --source-topic <源主题> \
  --dest-topic <目标主题> \
  --regex <正则表达式>

# 删除主题重写规则
robust-ctl mqtt topic-rewrite delete \
  --action <操作> \
  --source-topic <源主题>
```

**参数说明：**
- `--action, -a`: 操作类型 (必需)
- `--source-topic, -s`: 源主题 (必需)
- `--dest-topic, -d`: 目标主题 (创建时必需)
- `--regex, -r`: 正则表达式 (创建时必需)

---

### 1.9 连接器管理 (`connector`)

管理 MQTT 连接器。

```bash
# 列出所有连接器
robust-ctl mqtt connector list --connector-name <名称>

# 创建连接器
robust-ctl mqtt connector create \
  --connector-name <连接器名称> \
  --connector-type <连接器类型> \
  --config <配置> \
  --topic-id <主题ID>

# 删除连接器
robust-ctl mqtt connector delete --connector-name <连接器名称>
```

**参数说明：**
- `--connector-name, -c`: 连接器名称 (必需)
- `--connector-type, -c`: 连接器类型 (创建时必需)
- `--config, -c`: 配置信息 (创建时必需)
- `--topic-id, -t`: 主题 ID (创建时必需)

---

### 1.10 模式管理 (`schema`)

管理 MQTT 消息模式。

```bash
# 列出所有模式
robust-ctl mqtt schema list --schema-name <名称>

# 创建模式
robust-ctl mqtt schema create \
  --schema-name <模式名称> \
  --schema-type <模式类型> \
  --schema <模式定义> \
  --desc <描述>

# 删除模式
robust-ctl mqtt schema delete --schema-name <模式名称>

# 列出模式绑定
robust-ctl mqtt schema list-bind \
  --schema-name <模式名称> \
  --resource-name <资源名称>

# 绑定模式
robust-ctl mqtt schema bind \
  --schema-name <模式名称> \
  --resource-name <资源名称>

# 解绑模式
robust-ctl mqtt schema unbind \
  --schema-name <模式名称> \
  --resource-name <资源名称>
```

**参数说明：**
- `--schema-name, -s`: 模式名称 (必需)
- `--schema-type, -t`: 模式类型 (创建时必需)
- `--schema, -s`: 模式定义 (创建时必需)
- `--desc, -d`: 描述 (创建时必需)
- `--resource-name, -r`: 资源名称 (绑定操作时必需)

---

### 1.11 自动订阅规则 (`auto-subscribe`)

管理自动订阅规则。

```bash
# 列出所有自动订阅规则
robust-ctl mqtt auto-subscribe list

# 创建自动订阅规则
robust-ctl mqtt auto-subscribe create \
  --topic <主题> \
  [--qos <QOS等级>] \
  [--no-local] \
  [--retain-as-published] \
  [--retained-handling <处理方式>]

# 删除自动订阅规则
robust-ctl mqtt auto-subscribe delete --topic <主题>
```

**参数说明：**
- `--topic, -t`: 主题 (必需)
- `--qos, -q`: QoS 级别 (可选，默认 0)
- `--no-local, -n`: 不接收本地消息 (可选，默认 false)
- `--retain-as-published, -r`: 保持发布状态 (可选，默认 false)
- `--retained-handling, -R`: 保留消息处理方式 (可选，默认 0)

---

### 1.12 发布消息 (`publish`)

发布 MQTT 消息。

```bash
# 发布消息 (交互模式)
robust-ctl mqtt publish \
  --username <用户名> \
  --password <密码> \
  --topic <主题> \
  [--qos <QOS等级>] \
  [--retained]
```

**参数说明：**
- `--username, -u`: 用户名 (必需)
- `--password, -p`: 密码 (必需)
- `--topic, -t`: 主题 (必需)
- `--qos, -q`: QoS 级别 (可选，默认 0)
- `--retained`: 保留消息 (可选，默认 false)

---

### 1.13 订阅消息 (`subscribe`)

订阅 MQTT 消息。

```bash
# 订阅消息
robust-ctl mqtt subscribe \
  --username <用户名> \
  --password <密码> \
  --topic <主题> \
  [--qos <QOS等级>]
```

**参数说明：**
- `--username, -u`: 用户名 (必需)
- `--password, -p`: 密码 (必需)
- `--topic, -t`: 主题 (必需)
- `--qos, -q`: QoS 级别 (可选，默认 0)

---

### 1.14 可观测性功能

#### 慢订阅监控 (`slow-subscribe`)
```bash
# 列出慢订阅
robust-ctl mqtt slow-subscribe list
```

#### 连接抖动检测 (`flapping-detect`)
```bash
# 列出连接抖动检测结果
robust-ctl mqtt flapping-detect
```

#### 系统告警 (`system-alarm`)
```bash
# 列出系统告警
robust-ctl mqtt system-alarm list
```
