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
  │     ├── CREATE                    # 申请邮箱，返回 mail_id
  │     └── QUERY.{mail_id}          # 查询未读消息（兜底拉取）
  │
  ├── INBOX.
  │     └── {mail_id}.
  │           ├── urgent              # 紧急消息
  │           ├── normal              # 普通消息
  │           └── notify              # 通知
  │
  └── BROADCAST.
        └── {domain}.
              └── {event}            # 事件广播
```

### Subject 参考表

| Subject | 操作类型 | 说明 |
|---------|---------|------|
| `$mq9.AI.MAILBOX.CREATE` | req/reply | 申请邮箱，返回 mail_id 和 token |
| `$mq9.AI.MAILBOX.QUERY.{mail_id}` | req/reply | 查询邮箱未读消息（兜底拉取） |
| `$mq9.AI.INBOX.{mail_id}.urgent` | pub/sub | 向邮箱投递紧急消息 |
| `$mq9.AI.INBOX.{mail_id}.normal` | pub/sub | 向邮箱投递普通消息 |
| `$mq9.AI.INBOX.{mail_id}.notify` | pub/sub | 向邮箱投递通知 |
| `$mq9.AI.INBOX.{mail_id}.*` | sub | 订阅邮箱所有优先级的消息 |
| `$mq9.AI.BROADCAST.{domain}.{event}` | pub/sub | 发布或订阅广播事件 |
| `$mq9.AI.BROADCAST.{domain}.*` | sub | 订阅某域的所有事件 |
| `$mq9.AI.BROADCAST.*.{event}` | sub | 订阅所有域的某类事件 |
| `$mq9.AI.BROADCAST.#` | sub | 订阅所有广播 |

---

## 基础概念

**mail_id**：通过 `MAILBOX.CREATE` 申请的全局唯一通信地址。不绑定 Agent 身份——一个 Agent 可以为不同任务申请不同的 mail_id，用完不管，TTL 自动清理。mail_id 是通道级的，不是身份级的。

**邮箱类型**：申请邮箱时通过 `type` 参数声明：
- `standard`（默认）：普通邮箱，消息累积，按优先级存储。
- `latest`：状态邮箱，只保留最新一条，新消息覆盖旧消息。适合状态上报、能力声明等场景。

**TTL**：邮箱和消息均有 TTL。邮箱不需要显式删除，纯 TTL 管理生命周期，到期自动销毁，消息随邮箱一起清理。

**persist**：消息的持久化策略。INBOX 消息默认持久化，BROADCAST 消息默认不持久化，可显式覆盖。

**token**：轻量身份验证。申请邮箱时返回 token，QUERY 时携带，保证邮箱归属。

**消息流程**：消息到达 → 写存储层 → 检查在线订阅者 → 在线则推送 → 离线则消息在存储等待 → 对方上线订阅或通过 QUERY 主动拉取。

---

## MAILBOX.CREATE — 申请邮箱

```bash
# 请求
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "type": "standard",
  "ttl": 3600
}'

# 响应
{
  "mail_id": "m-uuid-001",
  "token": "tok-xxx",
  "inbox": "$mq9.AI.INBOX.m-uuid-001"
}
```

### 请求参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `type` | string | 否 | `standard`（默认）或 `latest` |
| `ttl` | int | 否 | 邮箱生存时间（秒），不填则使用默认值 |

### 邮箱类型

| type | 行为 | 适用场景 |
|------|------|---------|
| `standard` | 消息累积，按优先级存储 | 任务指令、请求响应、离线消息 |
| `latest` | 只保留最新一条，覆盖写 | 状态上报、能力声明、心跳 |

---

## MAILBOX.QUERY — 查询邮箱

QUERY 是推送的兜底机制。正常情况下，在线订阅者通过 SUB 实时收到推送。QUERY 用于：Agent 上线后检查离线期间的积压消息、推送可能遗漏时主动补偿、确认邮箱是否还有未处理的消息。

```bash
# 请求
nats req '$mq9.AI.MAILBOX.QUERY.m-uuid-001' '{
  "token": "tok-xxx"
}'

# 响应
{
  "mail_id": "m-uuid-001",
  "unread": 5,
  "messages": [ ... ]
}
```

---

## INBOX — 点对点邮箱

### 发送消息

```bash
# 发送普通消息
nats pub '$mq9.AI.INBOX.m-uuid-001.normal' '{
  "from": "m-uuid-002",
  "type": "task_result",
  "correlation_id": "req-uuid-001",
  "reply_to": "$mq9.AI.INBOX.m-uuid-002.normal",
  "payload": { ... },
  "ts": 1234567890
}'

# 发送紧急消息
nats pub '$mq9.AI.INBOX.m-uuid-001.urgent' '{
  "from": "m-uuid-003",
  "type": "emergency_stop",
  "payload": { ... },
  "ts": 1234567890
}'
```

### 消息字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `from` | string | 是 | 发件方 mail_id |
| `type` | string | 是 | 消息类型，由业务自定义 |
| `correlation_id` | string | 否 | 请求响应配对，异步 request-reply 场景使用 |
| `reply_to` | string | 否 | 回复地址，收件方可直接 PUB 到该 subject |
| `deadline` | int | 否 | 期望处理截止时间（Unix 时间戳，毫秒） |
| `payload` | any | 否 | 业务数据 |
| `ts` | int | 是 | 发送时间戳（毫秒） |

### 优先级

| 优先级 | 说明 | 默认持久化 | 建议 TTL |
|--------|------|-----------|---------|
| `urgent` | 紧急消息，优先处理 | true | 86400s |
| `normal` | 普通消息，顺序处理 | true | 3600s |
| `notify` | 通知，后台处理 | false | — |

### 接收消息

Agent 上线后订阅自己的邮箱。服务端会推送已积压的离线消息。

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
    "{\"type\":\"standard\",\"ttl\":3600}".getBytes(),
    Duration.ofSeconds(3));
// reply 包含 mail_id 和 token

// 订阅邮箱（接收方）
Dispatcher d = nc.createDispatcher((msg) -> {
    System.out.println("收到消息: " + new String(msg.getData()));
});
d.subscribe("$mq9.AI.INBOX.m-uuid-001.*");

// 发送消息（发送方）
nc.publish("$mq9.AI.INBOX.m-uuid-001.normal",
    "{\"from\":\"m-uuid-002\",\"type\":\"task_result\",\"payload\":\"完成\",\"ts\":1234567890}".getBytes());

// 查询未读消息（兜底）
Message qReply = nc.request("$mq9.AI.MAILBOX.QUERY.m-uuid-001",
    "{\"token\":\"tok-xxx\"}".getBytes(), Duration.ofSeconds(3));
```

---

## BROADCAST — 公共广播

广播不需要 CREATE，不需要任何前置操作。直接发，直接订阅。

### 发布广播

```bash
nats pub '$mq9.AI.BROADCAST.{domain}.{event}' '{
  "from": "m-uuid-004",
  "type": "event_type",
  "severity": "high",
  "reply_to": "$mq9.AI.INBOX.m-uuid-004.normal",
  "payload": { ... },
  "ts": 1234567890
}'
```

广播默认不持久化。重要广播需要持久化时，显式设置 `persist=true`。

`{domain}.{event}` 由用户自定义，mq9 只规定前缀 `$mq9.AI.BROADCAST.`。

### domain 建议命名

| domain | 说明 |
|--------|------|
| `system` | 系统级事件，如异常、重启、扩缩容 |
| `task` | 任务相关，如 available、completed、failed |
| `capability` | 能力发现，如 query、response |
| `data` | 数据变更事件 |
| 自定义 | 业务域自定义 |

### 订阅广播

```bash
# 订阅单个域的所有事件
nats sub '$mq9.AI.BROADCAST.system.*'

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
    "{\"task_id\":\"t-001\",\"type\":\"data_analysis\"}".getBytes());

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
| broadcast（默认） | — | Memory | 不持久化，可显式覆盖 |

---

## 消息格式建议

mq9 不强制消息格式，payload 是任意字节。推荐使用 JSON，字段参考：

```json
{
  "from": "m-uuid-001",
  "type": "task_result",
  "correlation_id": "req-uuid-001",
  "reply_to": "$mq9.AI.INBOX.m-uuid-001.normal",
  "deadline": 1234567890,
  "payload": { },
  "ts": 1234567890
}
```

---

## 与 NATS 原生协议的关系

mq9 是在 RobustMQ 的 NATS 协议层之上，通过 subject 命名约定和服务端逻辑增强而来：

- 普通 NATS pub/sub：实时送达，订阅者不在线则消息丢失
- mq9 `$mq9.AI.INBOX.*`：消息先写存储层，离线不丢失，上线后推送或 QUERY 拉取

两者可以混用。mq9 只对匹配 `$mq9.AI.*` 前缀的消息启用持久化和优先级调度，其余 NATS 行为不变。已有 NATS 客户端无需任何修改，直接使用 mq9 subject 即可。
