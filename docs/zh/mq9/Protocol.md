# mq9 协议

## 协议基础

mq9 是在 NATS subject 上定义的一套语义约定，运行在 RobustMQ 统一存储层之上。所有 mq9 操作都是标准的 NATS pub/sub/req-reply 调用，subject 命名遵循 `$mq9.AI.*` 命名空间。

任何支持 NATS 的客户端库都可以直接使用 mq9，无需额外 SDK。

---

## Subject 命名规范

### Subject 空间

```text
$mq9.AI.
  ├── MAILBOX.
  │     └── CREATE                    # 创建邮箱或广播频道
  │
  ├── INBOX.
  │     └── {mail_id}.
  │           ├── urgent              # 紧急消息
  │           ├── normal              # 普通消息
  │           └── notify              # 通知
  │
  └── BROADCAST.
        ├── system.
        │     ├── capability          # 系统公告板：能力
        │     ├── status              # 系统公告板：状态
        │     └── channel             # 系统公告板：频道
        └── {domain}.
              └── {event}            # 用户自定义广播
```

### Subject 参考表

| Subject | 操作类型 | 说明 |
|---------|---------|------|
| `$mq9.AI.MAILBOX.CREATE` | req/reply | 创建邮箱或广播频道。INBOX 返回 mail_id + token，BROADCAST 返回成功 |
| `$mq9.AI.INBOX.{mail_id}.urgent` | pub/sub | 向邮箱投递紧急消息 |
| `$mq9.AI.INBOX.{mail_id}.normal` | pub/sub | 向邮箱投递普通消息 |
| `$mq9.AI.INBOX.{mail_id}.notify` | pub/sub | 向邮箱投递通知 |
| `$mq9.AI.INBOX.{mail_id}.*` | sub | 订阅邮箱所有优先级的消息 |
| `$mq9.AI.BROADCAST.system.capability` | pub/sub | 系统公告板：能力 |
| `$mq9.AI.BROADCAST.system.status` | pub/sub | 系统公告板：状态 |
| `$mq9.AI.BROADCAST.system.channel` | pub/sub | 系统公告板：频道 |
| `$mq9.AI.BROADCAST.{domain}.{event}` | pub/sub | 发布或订阅自定义广播频道 |
| `$mq9.AI.BROADCAST.{domain}.*` | sub | 订阅某域的所有事件 |
| `$mq9.AI.BROADCAST.*.{event}` | sub | 订阅所有域的某类事件 |
| `$mq9.AI.BROADCAST.#` | sub | 订阅所有广播 |

---

## 基础概念

**mail_id**：通过 `MAILBOX.CREATE` 申请的全局唯一通信地址。不绑定 Agent 身份——一个 Agent 可以为不同任务申请不同的 mail_id，用完不管，TTL 自动清理。mail_id 是通道级的，不是身份级的。

**邮箱类型**：申请邮箱时通过 `type` 参数声明：
- `standard`（默认）：普通邮箱，消息累积，按优先级存储。
- `latest`：状态邮箱，只保留最新一条，新消息覆盖旧消息。适合状态上报、能力声明等场景。

**TTL**：INBOX 和 BROADCAST 创建时声明，到期自动销毁，消息随之清理。无显式删除操作。TTL 以第一次 CREATE 为准，重复 CREATE 不覆盖。

**token**：创建 INBOX 时返回。任何人知道 mail_id 即可发消息，token 仅用于邮箱管理操作。BROADCAST 无 token。

**msg_id**：每条消息的唯一标识，客户端用于去重。

**订阅语义**：INBOX 和 BROADCAST 行为一致——每次订阅全量推送所有未过期消息。不区分已读/未读，不追踪消费位点。服务端零消费状态。

**消息流程**：消息到达 → 写存储 → 在线则实时推送，不在线则等待 → 订阅者下次订阅时全量推送所有未过期消息。

**CREATE 幂等性**：重复创建同一个 INBOX 或 BROADCAST 不报错，静默返回成功。TTL 以第一次创建为准。

**无状态查询**：mq9 不提供"邮箱是否存在""频道是否存在"的查询接口。CREATE 永远返回成功。

---

## MAILBOX.CREATE — 创建邮箱或广播频道

统一的创建入口。通过请求中的 subject 前缀区分创建 INBOX 还是 BROADCAST。

### 创建私人邮箱（INBOX）

```bash
# 请求
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "type": "standard",
  "ttl": 3600,
  "subject": "$mq9.AI.INBOX"
}'

# 响应
{
  "mail_id": "m-uuid-001",
  "token": "tok-xxx",
  "inbox": "$mq9.AI.INBOX.m-uuid-001"
}
```

### 创建广播频道（BROADCAST）

```bash
# 请求
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "type": "standard",
  "ttl": 3600,
  "subject": "$mq9.AI.BROADCAST.pipeline.complete"
}'

# 响应
{
  "subject": "$mq9.AI.BROADCAST.pipeline.complete"
}
```

服务端识别 `$mq9.AI.BROADCAST.` 前缀，不生成 mail_id，不生成 token。频道创建后任何人可发可订。

CREATE 是幂等的。频道已存在则静默返回成功，TTL 以第一次创建为准，后续不覆盖。

### 请求参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `type` | string | 否 | `standard`（默认）或 `latest` |
| `ttl` | int | 否 | 生存时间（秒），不填则使用默认值 |
| `subject` | string | 是 | `$mq9.AI.INBOX` 创建邮箱；`$mq9.AI.BROADCAST.{domain}.{event}` 创建广播频道 |

---

## INBOX — 点对点邮箱

### 发送消息

```bash
# 发送普通消息
nats pub '$mq9.AI.INBOX.m-uuid-001.normal' '{
  "msg_id": "msg-uuid-001",
  "from": "m-uuid-002",
  "type": "task_result",
  "correlation_id": "req-uuid-001",
  "reply_to": "$mq9.AI.INBOX.m-uuid-002.normal",
  "payload": { ... },
  "ts": 1234567890
}'

# 发送紧急消息
nats pub '$mq9.AI.INBOX.m-uuid-001.urgent' '{
  "msg_id": "msg-uuid-002",
  "from": "m-uuid-003",
  "type": "emergency_stop",
  "payload": { ... },
  "ts": 1234567890
}'
```

### 消息字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `msg_id` | string | 是 | 消息唯一标识，客户端用于去重 |
| `from` | string | 是 | 发件方 mail_id |
| `type` | string | 是 | 消息类型，由业务自定义 |
| `correlation_id` | string | 否 | 请求响应配对，异步 request-reply 场景使用 |
| `reply_to` | string | 否 | 回复地址，收件方可直接 PUB 到该 subject |
| `deadline` | int | 否 | 期望处理截止时间（Unix 时间戳，毫秒） |
| `payload` | any | 否 | 业务数据 |
| `ts` | int | 是 | 发送时间戳（毫秒） |

### 优先级

| 优先级 | 说明 | 持久化 | 建议 TTL |
|--------|------|--------|---------|
| `urgent` | 紧急消息，优先处理 | 是 | 86400s |
| `normal` | 普通消息，顺序处理 | 是 | 3600s |
| `notify` | 通知，后台处理 | 否 | 短 |

消息随邮箱一起过期。邮箱 TTL 到期，其中所有消息一起销毁。

### 接收消息

Agent 上线后订阅自己的邮箱，服务端全量推送所有未过期消息。

```bash
# 订阅所有优先级的消息
nats sub '$mq9.AI.INBOX.m-uuid-001.*'

# 只订阅紧急消息
nats sub '$mq9.AI.INBOX.m-uuid-001.urgent'
```

### Java 示例

```java
// 依赖: io.nats:jnats:2.20.5
Connection nc = Nats.connect("nats://localhost:4222");

// 申请邮箱
Message reply = nc.request("$mq9.AI.MAILBOX.CREATE",
    "{\"type\":\"standard\",\"ttl\":3600,\"subject\":\"$mq9.AI.INBOX\"}".getBytes(),
    Duration.ofSeconds(3));
// reply 包含 mail_id 和 token

// 订阅邮箱（接收方）
Dispatcher d = nc.createDispatcher((msg) -> {
    System.out.println("收到消息: " + new String(msg.getData()));
});
d.subscribe("$mq9.AI.INBOX.m-uuid-001.*");

// 发送消息（发送方）
nc.publish("$mq9.AI.INBOX.m-uuid-001.normal",
    "{\"msg_id\":\"msg-001\",\"from\":\"m-uuid-002\",\"type\":\"task_result\",\"payload\":\"完成\",\"ts\":1234567890}".getBytes());
```

---

## BROADCAST — 公共广播

广播频道需要先 CREATE 创建，之后任何人可发，任何人可订阅。消息持久化，离线订阅者上线后全量收到未过期消息。

### 创建并发布广播

```bash
# 先创建频道
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "type": "standard",
  "ttl": 7200,
  "subject": "$mq9.AI.BROADCAST.pipeline.complete"
}'

# 发布广播
nats pub '$mq9.AI.BROADCAST.pipeline.complete' '{
  "msg_id": "msg-uuid-003",
  "from": "m-uuid-004",
  "type": "pipeline_done",
  "reply_to": "$mq9.AI.INBOX.m-uuid-004.normal",
  "payload": { ... },
  "ts": 1234567890
}'
```

`{domain}.{event}` 由用户自定义，mq9 只规定前缀 `$mq9.AI.BROADCAST.`。

### domain 建议命名

| domain | 说明 |
|--------|------|
| `system` | 系统级事件（系统内置，不可修改） |
| `task` | 任务相关，如 available、completed、failed |
| `data` | 数据变更事件 |
| 自定义 | 业务域自定义 |

### 订阅广播

```bash
# 订阅系统公告板
nats sub '$mq9.AI.BROADCAST.system.*'

# 订阅单个域的所有事件
nats sub '$mq9.AI.BROADCAST.pipeline.*'

# 订阅所有域的特定事件
nats sub '$mq9.AI.BROADCAST.*.anomaly'

# 订阅所有广播
nats sub '$mq9.AI.BROADCAST.#'
```

### 竞争消费（Queue Group）

多个 Worker 订阅同一个广播 subject，使用 queue group 保证每条消息只被一个 Worker 处理：

```bash
nats sub '$mq9.AI.BROADCAST.task.available' --queue task.workers
```

### BROADCAST Java 示例

```java
// 广播任务，Worker 竞争消费
for (int i = 1; i <= 3; i++) {
    final int id = i;
    Dispatcher worker = nc.createDispatcher((msg) -> {
        System.out.println("[Worker-" + id + "] 抢到任务: " + new String(msg.getData()));
    });
    // queue group 保证只有一个 Worker 收到
    worker.subscribe("$mq9.AI.BROADCAST.task.available", "task.workers");
}

// 主 Agent 广播任务
nc.publish("$mq9.AI.BROADCAST.task.available",
    "{\"msg_id\":\"t-001\",\"task_id\":\"t-001\",\"type\":\"data_analysis\"}".getBytes());

// 订阅跨域告警
Dispatcher alertHandler = nc.createDispatcher((msg) -> {
    System.out.println("收到告警: " + new String(msg.getData()));
});
alertHandler.subscribe("$mq9.AI.BROADCAST.*.anomaly");
```

---

## 存储分级

mq9 运行在 RobustMQ 统一存储层之上，提供三种存储能力。用户按需选择，不是所有消息都需要相同级别的保障。

| 存储层 | 特性 | 适用场景 |
|--------|------|---------|
| Memory | 纯内存，最轻，不持久化 | Agent 协调信号、心跳、临时通知，丢了重发 |
| RocksDB | 临时持久化，TTL 到期自动清理 | 邮箱默认选择，任务指令，离线消息 |
| File Segment | 长期持久化，永久保留 | 审计日志，关键事件 |

各邮箱类型与存储的对应关系：

| 邮箱类型 | 优先级 | 默认存储层 | 说明 |
|---------|--------|----------|------|
| standard | urgent | RocksDB | 持久化，TTL=86400s |
| standard | normal | RocksDB | 持久化，TTL=3600s |
| standard | notify | Memory | 不持久化 |
| latest | — | RocksDB | 只保留最新一条 |
| broadcast | — | RocksDB | 持久化，TTL 以 CREATE 时声明为准 |

---

## 消息格式建议

mq9 不强制消息格式，payload 是任意字节。推荐使用 JSON，字段参考：

```json
{
  "msg_id": "msg-uuid-001",
  "from": "m-uuid-001",
  "type": "task_result",
  "correlation_id": "req-uuid-001",
  "reply_to": "$mq9.AI.INBOX.m-uuid-001.normal",
  "deadline": 1234567890,
  "payload": {},
  "ts": 1234567890
}
```

---

## 与 NATS 原生协议的关系

mq9 是在 RobustMQ 的 NATS 协议层之上，通过 subject 命名约定和服务端逻辑增强而来：

- 普通 NATS pub/sub：实时送达，订阅者不在线则消息丢失
- mq9 `$mq9.AI.INBOX.*` / `$mq9.AI.BROADCAST.*`：消息先写存储层，离线不丢失，上线后全量推送所有未过期消息

两者可以混用。mq9 只对匹配 `$mq9.AI.*` 前缀的消息启用持久化和优先级调度，其余 NATS 行为不变。已有 NATS 客户端无需任何修改，直接使用 mq9 subject 即可。
