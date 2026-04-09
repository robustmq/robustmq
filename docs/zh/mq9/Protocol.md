# mq9 协议

## 协议基础

mq9 是在 NATS subject 上定义的一套语义约定，运行在 RobustMQ 统一存储层之上。所有 mq9 操作都是标准的 NATS pub/sub 调用，subject 命名遵循 `$mq9.AI.*` 命名空间。

任何支持 NATS 的客户端库都可以直接使用 mq9，无需额外 SDK。

---

## Subject 命名规范

### Subject 空间

```text
$mq9.AI.
  ├── MAILBOX.
  │     ├── CREATE                         # 创建邮箱
  │     └── {mail_id}                      # 默认优先级消息（normal，无后缀）
  │           ├── .urgent                  # 紧急消息
  │           └── .critical                # 最高优先级消息
  │
  └── PUBLIC.LIST                          # 公开邮箱发现，系统内置
```

### Subject 参考表

| Subject | 操作类型 | 说明 |
|---------|---------|------|
| `$mq9.AI.MAILBOX.CREATE` | pub | 创建邮箱，响应通过 reply-to 返回 |
| `$mq9.AI.MAILBOX.{mail_id}` | pub/sub | 默认优先级消息（normal，无后缀） |
| `$mq9.AI.MAILBOX.{mail_id}.urgent` | pub/sub | 紧急消息 |
| `$mq9.AI.MAILBOX.{mail_id}.critical` | pub/sub | 最高优先级消息 |
| `$mq9.AI.MAILBOX.{mail_id}.*` | sub | 订阅 urgent 和 critical 消息（不含默认 normal） |
| `$mq9.AI.PUBLIC.LIST` | sub | 发现所有公开邮箱，系统内置 |

---

## 基础概念

**mail_id**：通过 `MAILBOX.CREATE` 创建邮箱时返回的通信地址。不绑定 Agent 身份，一个 Agent 可以为不同任务申请不同的 mail_id，用完不管，TTL 自动清理。

私有邮箱的 mail_id 由系统生成，不可猜测。公开邮箱的 mail_id 由用户自定义，有意义的名字——`task.queue`、`analytics.result`。

**mail_id 不可猜测即安全边界。** 知道 mail_id 就能发消息、能订阅。不知道 mail_id 就无从操作。没有 token，没有 ACL。

**TTL**：邮箱创建时声明，到期自动销毁，消息随之清理。没有 DELETE 命令。CREATE 幂等，重复创建同一邮箱不报错，TTL 以第一次创建为准。

**priority**：MAILBOX 支持三个优先级：critical、urgent、normal。同优先级 FIFO，跨优先级高优先处理。normal 是默认级别，使用无后缀的裸 subject 发送；urgent 和 critical 分别加 `.urgent` 和 `.critical` 后缀。存储层保证顺序，消费方无需自行排序。

**msg_id**：每条消息的唯一标识，客户端用于去重。

**订阅语义**：每次订阅全量推送所有未过期消息，后续新消息实时推送。不区分已读/未读，不追踪消费位点，没有 QUERY 命令。服务端零消费状态。

**消息流程**：消息到达 → 写存储 → 在线则实时推送，不在线则等待 → 订阅者订阅时全量推送所有未过期消息。

---

## MAILBOX.CREATE — 创建邮箱

### 创建私有邮箱

```bash
nats pub '$mq9.AI.MAILBOX.CREATE' '{"ttl":3600}'

# 响应
{"mail_id": "m-uuid-001"}
```

### 创建公开邮箱

```bash
nats pub '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 3600,
  "public": true,
  "name": "task.queue",
  "desc": "任务队列"
}'

# 响应
{"mail_id": "task.queue"}
```

`public: true` 的邮箱创建后自动注册到 `$mq9.AI.PUBLIC.LIST`，TTL 到期自动移除。

CREATE 是幂等的。邮箱已存在则静默返回成功，TTL 以第一次创建为准。

### 请求参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `ttl` | int | 否 | 生存时间（秒），不填则使用默认值 |
| `public` | bool | 否 | 是否公开，默认 false |
| `name` | string | 公开邮箱必填 | 公开邮箱的自定义 mail_id |
| `desc` | string | 否 | 邮箱描述，公开邮箱建议填写 |

---

## 发消息

知道 mail_id 即可发送，无需任何授权。

```bash
nats pub '$mq9.AI.MAILBOX.m-uuid-001.critical' 'payload'  # 最高优先级，立即处理
nats pub '$mq9.AI.MAILBOX.m-uuid-001.urgent'   'payload'  # 紧急
nats pub '$mq9.AI.MAILBOX.m-uuid-001'          'payload'  # 默认（normal），常规通信
```

### 优先级

| 级别 | 语义 | 典型场景 |
|------|------|---------|
| critical | 最高优先级，立即处理 | 中止信号、紧急指令、安全事件 |
| urgent | 紧急 | 任务中断、时效性指令 |
| normal（默认，无后缀）| 常规通信 | 任务分发、结果返回、审批请求 |

存储层保证优先级顺序。边缘设备离线后上线，先收到 critical 消息，再收到 urgent，最后是 normal。消费方无需自行排序。

### 消息结构（建议）

```json
{
  "msg_id": "msg-uuid-001",
  "from": "m-uuid-002",
  "type": "task_result",
  "correlation_id": "req-uuid-001",
  "reply_to": "m-uuid-002",
  "payload": {},
  "ts": 1234567890
}
```

| 字段 | 说明 |
|------|------|
| msg_id | 消息唯一标识，客户端去重用 |
| from | 发件方 mail_id |
| type | 消息类型，业务自定义 |
| correlation_id | 关联原始请求，用于 request-reply |
| reply_to | 回复地址，收件方回复时发到这个 mail_id |
| payload | 消息内容，mq9 不解析 |
| ts | 发送时间戳 |

消息结构由业务自定义，mq9 不强制。以上是建议约定。

---

## 订阅邮箱

```bash
nats sub '$mq9.AI.MAILBOX.m-uuid-001'       # 仅默认优先级（normal，无后缀）
nats sub '$mq9.AI.MAILBOX.m-uuid-001.*'     # urgent 和 critical（不含默认 normal）
nats sub '$mq9.AI.MAILBOX.m-uuid-001.urgent'   # 仅 urgent
nats sub '$mq9.AI.MAILBOX.m-uuid-001.critical' # 仅 critical
# 订阅所有优先级（包括默认 normal）：同时订阅以上两个 subject
```

订阅即全量推送——所有未过期消息立即推送，后续新消息实时推送。Agent 重连后不会漏消息。

**竞争消费**——公开邮箱可以用 queue group，多个订阅者竞争消费，每条消息只被一个订阅者处理：

```bash
nats sub '$mq9.AI.MAILBOX.task.queue.*' --queue workers
```

**禁止的操作**：

```bash
nats sub '$mq9.AI.MAILBOX.*'  # 禁止，broker 拒绝
nats sub '$mq9.AI.MAILBOX.#'  # 禁止，broker 拒绝
```

订阅必须精确到 mail_id，不允许通配符订阅所有邮箱。

---

## PUBLIC.LIST — 公开邮箱发现

系统内置地址，broker 维护，不接受用户写入，TTL 永不过期。

`public: true` 的邮箱创建后自动注册，TTL 到期自动移除。

```bash
nats sub '$mq9.AI.PUBLIC.LIST'
```

订阅即全量推送——当前所有公开邮箱立即推送，后续新增和移除实时推送。

推送内容：

```json
# 新增
{"event": "created", "mail_id": "task.queue", "desc": "任务队列", "ttl": 3600}

# 移除
{"event": "expired", "mail_id": "task.queue"}
```

---

## 存储分级

mq9 运行在 RobustMQ 统一存储层之上，提供三种存储能力：

| 存储层 | 特性 | 适用场景 |
|--------|------|---------|
| Memory | 纯内存，最轻，不持久化 | 协调信号、心跳，丢了重发 |
| RocksDB | 临时持久化，TTL 到期自动清理 | 邮箱默认选择，离线消息 |
| File Segment | 长期持久化，永久保留 | 审计日志，关键事件 |

| 优先级 | 默认存储层 | 说明 |
|--------|----------|------|
| critical | RocksDB | 持久化 |
| urgent | RocksDB | 持久化 |
| normal（默认）| Memory | 不持久化，丢了重发 |

---

## 协议总览

| Subject | 方向 | 持久化 | 说明 |
|---------|------|--------|------|
| `$mq9.AI.MAILBOX.CREATE` | PUB | — | 创建邮箱 |
| `$mq9.AI.MAILBOX.{id}` | PUB/SUB | 否 | 默认优先级消息（normal，无后缀） |
| `$mq9.AI.MAILBOX.{id}.urgent` | PUB/SUB | 是 | 紧急消息 |
| `$mq9.AI.MAILBOX.{id}.critical` | PUB/SUB | 是 | 最高优先级消息 |
| `$mq9.AI.PUBLIC.LIST` | SUB | 是 | 发现公开邮箱，系统内置 |

---

## 与 NATS 原生协议的关系

mq9 是在 RobustMQ 的 NATS 协议层之上，通过 subject 命名约定和服务端逻辑增强而来：

- 普通 NATS pub/sub：实时送达，订阅者不在线则消息丢失
- mq9 `$mq9.AI.MAILBOX.*`：消息先写存储层，离线不丢失，上线后全量推送所有未过期消息

两者可以混用。mq9 只对匹配 `$mq9.AI.*` 前缀的消息启用持久化和优先级调度，其余 NATS 行为不变。已有 NATS 客户端无需任何修改，直接使用 mq9 subject 即可。
