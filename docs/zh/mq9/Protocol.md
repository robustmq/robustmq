# 协议设计

---

## 1. 协议基础

mq9 是在 NATS subject 上定义的一套语义约定，运行在 RobustMQ 统一存储层之上。所有 mq9 操作都是标准的 NATS pub/sub 或 request/reply 调用，无需额外协议层或 SDK。

### 为什么选择 NATS 协议

mq9 选择 NATS 作为传输协议，而非自研通信协议，原因有三：

- **客户端生态完整**：NATS 有覆盖 40+ 语言的官方和社区客户端，AI 领域常用的 Python、Go、JavaScript、Rust 均有成熟实现。选择 NATS 意味着 mq9 从第一天起就对所有这些语言的开发者开箱即用，无需等待 SDK 覆盖。

- **语义匹配**：NATS 的 pub/sub 和 request/reply 原语恰好覆盖 mq9 所需的全部通信模式——fire-and-forget 发送、主动订阅、同步请求。subject 命名空间机制为 mq9 的邮箱地址系统提供了天然的表达方式。

- **Broker 完全自研**：使用 NATS 协议不意味着使用 NATS Server。mq9 的 Broker 是 RobustMQ 用 Rust 自研实现的，运行在 RobustMQ 统一存储层之上。存储、优先级调度、TTL 管理、先存储后推送语义——全部是 RobustMQ 自身的能力。NATS 只是客户端与 Broker 之间的通信协议，就像 HTTP 是 Web 的传输协议一样。

| 操作类型 | NATS 原语 | 说明 |
|---------|---------|------|
| 创建邮箱 | `nats req`（request/reply） | 客户端发请求，服务端通过 reply-to 返回结果 |
| 发送消息 | `nats pub`（fire-and-forget） | 即发即忘，不等待确认 |
| 订阅消息 | `nats sub`（subscribe） | 订阅即触发全量推送，后续实时推送 |
| 列出消息 | `nats req`（request/reply） | 查询邮箱中的消息元数据 |
| 删除消息 | `nats req`（request/reply） | 删除指定消息 |

消息编码：请求体和响应体均为 JSON，UTF-8 编码。标准 NATS 连接，默认端口 `4222`。

---

## 2. Subject 命名规范

所有 mq9 subject 以 `$mq9.AI` 为前缀。服务端对该前缀下的消息启用持久化和优先级调度，其他 NATS subject 行为不变。

```text
$mq9.AI
  └── MAILBOX
        ├── CREATE                                   # 创建邮箱
        ├── MSG.{mail_address}                            # 发送/订阅消息（normal，默认，无后缀）
        ├── MSG.{mail_address}.urgent                     # 发送/订阅紧急消息
        ├── MSG.{mail_address}.critical                   # 发送/订阅最高优先级消息
        ├── MSG.{mail_address}.*                          # 订阅所有优先级
        ├── LIST.{mail_address}                           # 列出邮箱消息元数据
        └── DELETE.{mail_address}.{msg_id}                # 删除指定消息
```

| Subject | NATS 原语 | 说明 |
|---------|---------|------|
| `$mq9.AI.MAILBOX.CREATE` | request | 创建私有或公开邮箱 |
| `$mq9.AI.MAILBOX.MSG.{mail_address}` | pub/sub | 发送/订阅 normal 优先级消息（默认，无后缀） |
| `$mq9.AI.MAILBOX.MSG.{mail_address}.urgent` | pub/sub | 发送/订阅 urgent 优先级消息 |
| `$mq9.AI.MAILBOX.MSG.{mail_address}.critical` | pub/sub | 发送/订阅 critical 优先级消息 |
| `$mq9.AI.MAILBOX.MSG.{mail_address}.*` | sub | 订阅**所有**优先级（critical + urgent + normal） |
| `$mq9.AI.MAILBOX.LIST.{mail_address}` | request | 列出邮箱中所有消息的元数据 |
| `$mq9.AI.MAILBOX.DELETE.{mail_address}.{msg_id}` | request | 删除邮箱中的指定消息 |
| `$mq9.AI.PUBLIC.LIST` | sub | 发现所有公开邮箱，系统内置 |

### mail_address 格式

私有邮箱的 `mail_address` 由服务端生成，格式为 `{uuid}@mq9`，例如：

```text
d7a5072lko83@mq9
```

公开邮箱的 `mail_address` 由用户在创建时通过 `name` 字段指定，支持点号分隔，例如 `task.queue`、`vision.results`。

`mail_address` 不可猜测即安全边界：知道 `mail_address` 即可发消息和订阅，不知道则无从操作，无 token，无 ACL。

---

## 3. 基础概念

**邮箱（Mailbox）** — mq9 的基本通信地址。通过 `MAILBOX.CREATE` 创建，返回 `mail_address`。不绑定 Agent 身份，一个 Agent 可以为不同任务创建不同邮箱。

**TTL** — 邮箱创建时声明的生存时间，到期后邮箱及其所有消息自动销毁，无需手动清理。`MAILBOX.CREATE` 幂等，对同一名称重复调用静默返回成功，TTL 以第一次创建为准。

**优先级（Priority）** — 每条消息属于三个优先级之一，编码在 subject 后缀中：

| 优先级 | Subject 形式 | 典型场景 |
|--------|------------|---------|
| `critical` | `MSG.{mail_address}.critical` | 中止信号、紧急指令、安全事件 |
| `urgent` | `MSG.{mail_address}.urgent` | 任务中断、时效性指令 |
| `normal`（默认） | `MSG.{mail_address}`（无后缀） | 任务分发、结果返回、常规通信 |

同优先级内 FIFO，跨优先级 critical 先于 urgent 先于 normal，顺序由存储层保证，消费方无需自行排序。

**先存储后推送** — 消息到达后先写存储，订阅者在线则同时实时推送，不在线则等待。每次订阅触发全量推送所有未过期消息，然后切换实时推送。Agent 重连不漏消息。

**无服务端消费者状态** — 服务端不追踪已读/未读，不维护每消费者位点，没有 ACK，没有 offset。去重依赖客户端通过 `msg_id` 自行实现。

**msg_id** — 服务端为每条消息分配的唯一标识（uint64），用于客户端去重和消息删除。

---

## 4. 命令字

### 4.1 创建邮箱（MAILBOX.CREATE）

Subject：`$mq9.AI.MAILBOX.CREATE` · NATS 原语：request/reply

#### CREATE 请求参数

| 字段 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `ttl` | uint64 | 否 | 服务端默认 | 邮箱生存时间（秒），到期后邮箱及消息自动销毁 |
| `public` | bool | 否 | `false` | 是否为公开邮箱，`true` 时自动注册到 `$mq9.AI.PUBLIC.LIST` |
| `name` | string | 公开邮箱必填 | — | 公开邮箱的自定义 `mail_address`，支持点号分隔（如 `task.queue`） |
| `desc` | string | 否 | `""` | 邮箱描述，公开邮箱建议填写，会随 PUBLIC.LIST 推送 |

#### CREATE 响应字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `mail_address` | string | 邮箱唯一标识。私有邮箱为 `{uuid}@mq9` 格式，公开邮箱为请求中的 `name` 值 |
| `is_new` | bool | `true` 表示本次调用新建了邮箱；`false` 表示邮箱已存在（幂等返回） |

#### CREATE 示例

创建私有邮箱：

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":3600}'
```

```json
{"mail_address":"d7a5072lko83@mq9","is_new":true}
```

创建公开邮箱：

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 86400,
  "public": true,
  "name": "task.queue",
  "desc": "共享 Worker 任务队列"
}'
```

```json
{"mail_address":"task.queue","is_new":true}
```

幂等调用（邮箱已存在）：

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":86400,"public":true,"name":"task.queue"}'
```

```json
{"mail_address":"task.queue","is_new":false}
```

> CREATE 是幂等的。邮箱已存在时返回成功但 `is_new` 为 `false`，原始 TTL 保留不变。Worker 启动时可以安全地调用，无需判断邮箱是否存在。

---

### 4.2 发送消息（MAILBOX.MSG pub）

Subject：`$mq9.AI.MAILBOX.MSG.{mail_address}[.{priority}]` · NATS 原语：publish

知道 `mail_address` 即可发送，无需授权。消息写入存储后立即返回，发送方不等待订阅者在线。

#### MSG-pub Subject 编码规则

| 优先级 | Subject |
|--------|---------|
| normal（默认） | `$mq9.AI.MAILBOX.MSG.{mail_address}`（无后缀） |
| urgent | `$mq9.AI.MAILBOX.MSG.{mail_address}.urgent` |
| critical | `$mq9.AI.MAILBOX.MSG.{mail_address}.critical` |

> 禁止通配符发布。`$mq9.AI.MAILBOX.MSG.*.*` 等通配符 subject 会被服务端拒绝，发布时必须指定精确的 `mail_address`。

#### MSG-pub 响应字段

发布操作为 fire-and-forget，无响应体。若使用 request 模式发送则返回：

| 字段 | 类型 | 说明 |
|------|------|------|
| `msg_id` | uint64 | 服务端为该消息分配的唯一标识，可用于后续 DELETE 操作 |

#### MSG-pub 消息体（建议结构）

mq9 不强制消息体格式，payload 为任意字节序列。以下是推荐的 JSON 约定：

| 字段 | 类型 | 说明 |
|------|------|------|
| `from` | string | 发件方 `mail_address` |
| `type` | string | 消息类型，业务自定义（如 `task_dispatch`、`task_result`、`abort`） |
| `correlation_id` | string | 关联原始请求的标识，用于 request-reply 模式 |
| `reply_to` | string | 回复地址，收件方将响应发送到此 `mail_address` |
| `payload` | object | 消息内容，mq9 不解析 |
| `ts` | int64 | 发送时间戳（Unix 秒） |

#### MSG-pub 示例

```bash
# critical — 最高优先级
nats pub '$mq9.AI.MAILBOX.MSG.d7a5072lko83@mq9.critical' \
  '{"type":"abort","task_id":"t-001","ts":1712600001}'

# urgent — 紧急
nats pub '$mq9.AI.MAILBOX.MSG.d7a5072lko83@mq9.urgent' \
  '{"type":"interrupt","task_id":"t-002","ts":1712600002}'

# normal — 默认，无后缀
nats pub '$mq9.AI.MAILBOX.MSG.d7a5072lko83@mq9' \
  '{"type":"task","payload":{"job":"process dataset A"},"reply_to":"sender-001@mq9","ts":1712600003}'
```

---

### 4.3 订阅消息（MAILBOX.MSG sub）

Subject：`$mq9.AI.MAILBOX.MSG.{mail_address}[.{priority}|.*]` · NATS 原语：subscribe

#### MSG-sub 订阅模式

| Subject | 含义 |
|---------|------|
| `$mq9.AI.MAILBOX.MSG.{mail_address}.*` | 订阅**所有**优先级（critical + urgent + normal） |
| `$mq9.AI.MAILBOX.MSG.{mail_address}.critical` | 仅订阅 critical 优先级 |
| `$mq9.AI.MAILBOX.MSG.{mail_address}.urgent` | 仅订阅 urgent 优先级 |
| `$mq9.AI.MAILBOX.MSG.{mail_address}` | 仅订阅 normal 优先级（无后缀） |

> `.*` 匹配所有优先级。服务端在收到 `.*` 订阅时，将 critical、urgent、normal 三个优先级的消息全部推送给订阅者。这是订阅完整邮箱的标准方式。

#### MSG-sub 推送语义

每次订阅触发以下序列：

1. 推送所有未过期的存储消息，顺序为 critical → urgent → normal，同优先级内 FIFO
2. 推送完成后切换为实时推送：新消息到达后立即推送给在线订阅者

服务端不追踪消费状态。重新订阅会从头重放所有未过期消息。

#### MSG-sub 示例

```bash
# 订阅所有优先级（推荐）
nats sub '$mq9.AI.MAILBOX.MSG.d7a5072lko83@mq9.*'

# 仅订阅 critical
nats sub '$mq9.AI.MAILBOX.MSG.d7a5072lko83@mq9.critical'

# 仅订阅 urgent
nats sub '$mq9.AI.MAILBOX.MSG.d7a5072lko83@mq9.urgent'

# 仅订阅 normal
nats sub '$mq9.AI.MAILBOX.MSG.d7a5072lko83@mq9'

# 竞争消费（Queue Group）——每条消息只投递给其中一个 Worker
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers
```

---

### 4.4 列出消息（MAILBOX.LIST）

Subject：`$mq9.AI.MAILBOX.LIST.{mail_address}` · NATS 原语：request/reply

查看邮箱中当前存储的消息元数据，不消费消息（消息仍留在存储中）。

#### LIST 请求参数

请求体为空 JSON 对象：`{}`

#### LIST 响应字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `mail_address` | string | 邮箱唯一标识 |
| `messages` | array | 消息元数据列表 |

messages 数组元素：

| 字段 | 类型 | 说明 |
|------|------|------|
| `msg_id` | uint64 | 服务端分配的消息唯一标识，用于 DELETE |
| `payload` | string | 消息体内容（原始字符串） |
| `priority` | string | 优先级：`critical`、`urgent`、`normal` |
| `header` | bytes \| null | NATS 消息头（可选） |
| `create_time` | uint64 | 消息写入时间戳（Unix 秒） |

#### LIST 示例

```bash
nats req '$mq9.AI.MAILBOX.LIST.d7a5072lko83@mq9' '{}'
```

```json
{
  "mail_address": "d7a5072lko83@mq9",
  "messages": [
    {
      "msg_id": 1001,
      "payload": "{\"type\":\"abort\",\"task_id\":\"t-001\"}",
      "priority": "critical",
      "header": null,
      "create_time": 1712600001
    },
    {
      "msg_id": 1002,
      "payload": "{\"type\":\"interrupt\",\"task_id\":\"t-002\"}",
      "priority": "urgent",
      "header": null,
      "create_time": 1712600002
    },
    {
      "msg_id": 1003,
      "payload": "{\"type\":\"task\",\"payload\":{\"job\":\"process dataset A\"}}",
      "priority": "normal",
      "header": null,
      "create_time": 1712600003
    }
  ]
}
```

---

### 4.5 删除消息（MAILBOX.DELETE）

Subject：`$mq9.AI.MAILBOX.DELETE.{mail_address}.{msg_id}` · NATS 原语：request/reply

从邮箱存储中删除指定消息。竞争消费场景下，Worker 成功处理消息后显式删除，避免重新投递。

#### DELETE 请求参数

请求体为空 JSON 对象：`{}`

`mail_address` 和 `msg_id` 编码在 subject 中，解析规则：倒数第一个 `.` 之后的 token 为 `msg_id`，之前的所有内容为 `mail_address`。例如 `task.queue@mq9.1003` 解析为 `mail_address=task.queue@mq9`，`msg_id=1003`。

#### DELETE 响应字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `deleted` | bool | `true` 表示删除成功；`false` 表示消息不存在或已过期 |

#### DELETE 示例

```bash
nats req '$mq9.AI.MAILBOX.DELETE.d7a5072lko83@mq9.1002' '{}'
```

```json
{"deleted":true}
```

公开邮箱（mail_address 含 `@mq9`）：

```bash
nats req '$mq9.AI.MAILBOX.DELETE.task.queue@mq9.1003' '{}'
```

```json
{"deleted":true}
```

---

### 4.6 发现公开邮箱（PUBLIC.LIST）

Subject：`$mq9.AI.PUBLIC.LIST` · NATS 原语：subscribe

系统内置地址，由 Broker 维护，不接受用户写入。`public: true` 的邮箱创建后自动注册，TTL 到期自动移除。

订阅即全量推送——当前所有公开邮箱立即推送，后续新增和到期实时推送。

#### PUBLIC.LIST 推送格式

邮箱创建时：

| 字段 | 类型 | 说明 |
|------|------|------|
| `event` | string | 固定为 `"created"` |
| `mail_address` | string | 公开邮箱标识 |
| `desc` | string | 创建时提供的描述 |
| `ttl` | uint64 | 生存时间（秒） |

邮箱到期时：

| 字段 | 类型 | 说明 |
|------|------|------|
| `event` | string | 固定为 `"expired"` |
| `mail_address` | string | 已到期的邮箱标识 |

#### PUBLIC.LIST 示例

```bash
nats sub '$mq9.AI.PUBLIC.LIST'
```

```json
{"event":"created","mail_address":"task.queue@mq9","desc":"共享 Worker 任务队列","ttl":86400}
{"event":"created","mail_address":"vision.results@mq9","desc":"视觉处理结果","ttl":3600}
{"event":"expired","mail_address":"vision.results@mq9"}
```

---

## 6. 与 NATS 原生协议的关系

mq9 在 RobustMQ 的 NATS 协议层之上，通过 subject 命名约定和服务端增强实现：

| | NATS Core | mq9 |
|--|---------|-----|
| 持久化 | 无，订阅者离线消息丢失 | TTL 限定持久化，按优先级分层存储 |
| 消费者状态 | 无 | 无（设计如此） |
| 消息顺序 | 不保证跨 subject | 同 mail_address 内按优先级排序，同优先级 FIFO |
| 接入方式 | 任何 NATS 客户端 | 任何 NATS 客户端，subject 遵循 `$mq9.AI.*` 约定 |

mq9 只对 `$mq9.AI.*` 前缀的 subject 启用增强逻辑，其余 NATS 行为不变。已有 NATS 客户端无需任何修改，直接使用 mq9 subject 即可。
