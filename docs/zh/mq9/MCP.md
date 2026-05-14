# MCP Server 集成

## 概述

mq9 内置了一个 MCP（Model Context Protocol）Server，让 AI 大模型（Claude、GPT-4o 等）可以通过工具调用直接操作 mq9 的邮箱和 Agent 注册表，无需手动处理 NATS 协议。

MCP Server 随 Admin Server 一起启动，无需额外部署。大模型通过标准 MCP 协议与之交互，即可完成：

- 创建邮箱、发送消息、拉取消息、确认消费
- 注册/注销 Agent、发现其他 Agent
- 查询和删除消息

---

## 接入方式

### 配置地址

MCP Server 默认挂载在 Admin Server 的 `/mcp` 路径下：

```
http://<admin-server-host>:<port>/mcp
```

### Claude Desktop 配置示例

```json
{
  "mcpServers": {
    "mq9": {
      "url": "http://localhost:9981/mcp"
    }
  }
}
```

### 其他 MCP 客户端

任何支持 MCP 2025-03-26 协议的客户端（Claude Code、Cursor、自定义 SDK 等）均可直接连接，认证方式为无需鉴权。

---

## 工具一览

| 工具名 | 功能 |
|--------|------|
| `mq9_create_mailbox` | 创建邮箱 |
| `mq9_send_message` | 发送消息到邮箱 |
| `mq9_fetch_messages` | 拉取消息（有状态/无状态） |
| `mq9_ack_message` | 确认消息已处理，推进消费位点 |
| `mq9_query_mailbox` | 查询邮箱消息（不影响消费位点） |
| `mq9_delete_message` | 删除指定消息 |
| `mq9_register_agent` | 注册 Agent 到发现注册表 |
| `mq9_discover_agents` | 发现已注册的 Agent |
| `mq9_unregister_agent` | 注销 Agent |

---

## 工具详细说明

### mq9_create_mailbox

创建一个邮箱。发送或接收消息前必须先创建邮箱。

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 否 | 邮箱名称，仅允许小写字母、数字和点（`.`）。省略时由 Broker 自动生成。 |
| `ttl` | integer | 否 | 邮箱生存时间（秒）。`0` 或省略表示永不过期。 |
| `desc` | string | 否 | 邮箱描述（仅用于可读性，不影响路由）。 |

**返回**

```json
{ "mail_address": "agent.inbox.abc123", "created": true }
```

**示例**

```
创建一个名为 task.queue 的邮箱，有效期 1 小时
→ mq9_create_mailbox({"name": "task.queue", "ttl": 3600})
```

---

### mq9_send_message

向指定邮箱发送一条消息。

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `mail_address` | string | 是 | 目标邮箱地址。 |
| `payload` | string | 是 | 消息正文（UTF-8 字符串，可以是纯文本或 JSON）。 |
| `priority` | string | 否 | 优先级：`normal`（默认）/ `urgent` / `critical`。同优先级按 FIFO；跨优先级：critical > urgent > normal。 |
| `key` | string | 否 | 去重/压实 key。相同 key 的消息只保留最新一条，旧消息被覆盖。适合状态类消息（如进度更新）。 |
| `tags` | string | 否 | 逗号分隔的标签，如 `billing,vip`。可在 `mq9_query_mailbox` 中按标签过滤。 |
| `delay` | integer | 否 | 延迟投递秒数。消息在延迟期内不可见，FETCH 不会返回它。延迟消息的 `msg_id` 返回 `-1`。 |
| `ttl` | integer | 否 | 消息级 TTL（秒）。消息在 `发送时间 + ttl` 后过期，与邮箱 TTL 独立。 |

**返回**

```json
{ "msg_id": 42, "mail_address": "task.queue" }
```

> 延迟消息的 `msg_id` 为 `-1`。

**示例**

```
发送紧急任务
→ mq9_send_message({
    "mail_address": "task.queue",
    "payload": "{\"type\": \"analyze\", \"doc_id\": \"abc123\"}",
    "priority": "urgent"
  })

带 key 的状态消息（只保留最新）
→ mq9_send_message({
    "mail_address": "task.001.callback",
    "payload": "{\"status\": \"running\"}",
    "key": "status"
  })

带标签（可按 billing 过滤）
→ mq9_send_message({
    "mail_address": "agent.order.inbox",
    "payload": "{\"order_id\": \"o-001\"}",
    "tags": "billing,vip"
  })

延迟 60 秒投递
→ mq9_send_message({
    "mail_address": "agent.inbox",
    "payload": "{\"text\": \"delayed task\"}",
    "delay": 60
  })

消息级 TTL 300 秒
→ mq9_send_message({
    "mail_address": "agent.inbox",
    "payload": "{\"text\": \"short-lived\"}",
    "ttl": 300
  })
```

---

### mq9_fetch_messages

从邮箱拉取消息。支持**有状态消费**（传 `group_name`，Broker 记录消费位点）和**无状态消费**（不传 `group_name`，每次独立拉取）。

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `mail_address` | string | 是 | 邮箱地址。 |
| `group_name` | string | 否 | 消费组名称。传入时启用有状态消费，同组的多次调用共享消费位点。省略时为无状态消费，每次调用独立，不记录位点。 |
| `reset_to` | string | 否 | 指定从哪里开始读取。省略时从上次 ACK 位置继续（有状态）或从最新消息开始（无状态）。 |
| `max_messages` | integer | 否 | 单次最多返回的消息数量，默认 100。 |
| `max_wait_ms` | integer | 否 | 邮箱为空时服务端等待的毫秒数，默认 500。设为 `0` 则立即返回，不等待。 |

**`reset_to` 取值**

| 值 | 说明 |
|----|------|
| 省略 | 有状态：从上次位点续读；无状态：从最新消息 |
| `earliest` | 强制从邮箱最早的消息开始 |
| `latest` | 强制跳过历史，只接收从现在起的新消息 |
| `time:1746000000` | 从指定 Unix 时间戳之后的消息开始 |
| `id:42` | 从指定 msg_id（含）开始 |

**返回**

```json
{
  "messages": [
    { "msg_id": 42, "payload": "...", "priority": "normal", "create_time": 1746000000 }
  ]
}
```

**示例**

```
有状态消费：从 task.queue 拉取消息，记录到 worker-1 组
→ mq9_fetch_messages({"mail_address": "task.queue", "group_name": "worker-1"})

无状态消费：从头读取 task.queue 的所有历史消息
→ mq9_fetch_messages({"mail_address": "task.queue", "reset_to": "earliest"})
```

---

### mq9_ack_message

确认消息已成功处理。Broker 将该消费组的位点推进到 `msg_id` 之后，下次 fetch 从此处续读。

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `mail_address` | string | 是 | 邮箱地址。 |
| `group_name` | string | 是 | 消费组名称，必须与 fetch 时一致。 |
| `msg_id` | integer | 是 | 最后一条已成功处理的消息 ID。 |

**返回**

```json
{ "msg_id": 42, "acked": true }
```

**示例**

```
确认 worker-1 已处理到 msg_id=42
→ mq9_ack_message({"mail_address": "task.queue", "group_name": "worker-1", "msg_id": 42})
```

---

### mq9_query_mailbox

查询邮箱中的消息，**不影响消费位点**。适用于检视邮箱内容、按条件筛选、或在决定消费前先预览。

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `mail_address` | string | 是 | 邮箱地址。 |
| `key` | string | 否 | 按消息 key 精确过滤，返回该 key 下最新的一条消息。 |
| `tags` | array | 否 | 按标签过滤，只返回同时带有所有指定标签的消息。 |
| `since` | integer | 否 | 只返回该 Unix 时间戳之后创建的消息。 |
| `limit` | integer | 否 | 最多返回的消息数量，默认 20。 |

**返回**

```json
{
  "messages": [
    { "msg_id": 10, "payload": "...", "priority": "urgent", "create_time": 1746000000 }
  ]
}
```

**示例**

```
查看 task.queue 中带有 billing 标签的最新 10 条消息
→ mq9_query_mailbox({"mail_address": "task.queue", "tags": ["billing"], "limit": 10})
```

---

### mq9_delete_message

删除邮箱中的指定消息。

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `mail_address` | string | 是 | 邮箱地址。 |
| `msg_id` | integer | 是 | 要删除的消息 ID（来自 fetch 或 query 的返回值）。 |

**返回**

```json
{ "msg_id": 42, "deleted": true }
```

---

### mq9_register_agent

将当前 Agent 注册到 mq9 Agent 注册表，供其他 Agent 发现。建议在 Agent 启动时调用一次。

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 是 | Agent 唯一标识名称。 |
| `payload` | string | 是 | Agent 能力描述。可以是纯文本描述，也可以是 A2A AgentCard 的 JSON 字符串。内容会被用于全文检索和向量语义检索。 |

**返回**

```json
{ "name": "payment-agent", "registered": true }
```

**示例**

```
注册一个支付 Agent
→ mq9_register_agent({
    "name": "payment-agent",
    "payload": "专门处理支付、发票和财务交易的 Agent"
  })
```

---

### mq9_discover_agents

在注册表中搜索已注册的 Agent。支持关键词全文检索、语义向量检索，以及分页。

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `text` | string | 否 | 关键词全文检索（例如 `"支付 发票"`）。 |
| `semantic` | string | 否 | 语义自然语言检索，使用向量相似度匹配（例如 `"处理付款并生成发票"`）。同时传入时，`semantic` 优先于 `text`。 |
| `limit` | integer | 否 | 每页返回的最大数量，默认 20。 |
| `page` | integer | 否 | 页码，从 1 开始，默认 1。 |

`text` 和 `semantic` 均省略时，返回该租户下所有已注册的 Agent 列表。

**返回**

```json
{
  "agents": [
    { "name": "payment-agent", "agent_info": "...", "description": "...", "agent_id": "..." }
  ]
}
```

**检索优先级**：`semantic`（向量检索）> `text`（全文检索）> 不传（列出全部）

**示例**

```
语义检索：找能处理支付的 Agent
→ mq9_discover_agents({"semantic": "处理付款并生成发票"})

关键词检索：找带有 invoice 关键词的 Agent
→ mq9_discover_agents({"text": "invoice"})

分页：第 2 页，每页 10 条
→ mq9_discover_agents({"text": "payment", "limit": 10, "page": 2})

列出全部
→ mq9_discover_agents({})
```

---

### mq9_unregister_agent

从注册表中移除 Agent。建议在 Agent 关闭时调用。

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 是 | 要注销的 Agent 名称。 |

**返回**

```json
{ "name": "payment-agent", "unregistered": true }
```

---

## 典型使用场景

### 场景一：Agent 异步协作

两个 Agent 通过 mq9 完成任务分发和结果回传：

```
编排者 Agent:
  1. mq9_create_mailbox({"ttl": 300})           → 创建临时回复邮箱
  2. mq9_send_message({                          → 发任务给 Worker
       "mail_address": "task.queue",
       "payload": '{"doc_id":"abc123","reply_to":"<回复邮箱>"}',
       "priority": "normal"
     })
  3. mq9_fetch_messages({                        → 等待 Worker 回复
       "mail_address": "<回复邮箱>",
       "group_name": "orchestrator"
     })
  4. mq9_ack_message({...})                      → 确认已处理

Worker Agent:
  1. mq9_fetch_messages({"mail_address": "task.queue", "group_name": "workers"})
  2. 处理任务...
  3. mq9_send_message({"mail_address": "<reply_to>", "payload": '{"status":"done"}'})
  4. mq9_ack_message({...})
```

### 场景二：Agent 发现与消息路由

```
1. mq9_discover_agents({"semantic": "翻译文本"})   → 找翻译 Agent
2. 从返回的 agent_info 中解析出其 mail_address
3. mq9_send_message({"mail_address": "<翻译Agent邮箱>", "payload": "..."})
```

### 场景三：Agent 自注册

```
Agent 启动时:
  mq9_register_agent({
    "name": "translation-agent-v2",
    "payload": "支持中英日韩翻译的 Agent，擅长技术文档和法律合同的专业翻译"
  })

Agent 关闭时:
  mq9_unregister_agent({"name": "translation-agent-v2"})
```

---

## 错误处理

所有工具在出错时会返回包含错误描述的异常，常见错误：

| 错误信息 | 原因 |
|---------|------|
| `mailbox xxx does not exist` | 邮箱不存在，需先创建 |
| `mailbox xxx already exists` | 邮箱名重复，CREATE 不幂等 |
| `message not found` | msg_id 不存在 |
| `payload must not be empty` | REGISTER 时 payload 为空 |
| `agent name must not be empty` | Agent 名称为空 |
