# mq9 协议设计

## 协议基础

mq9 基于 NATS 协议构建。所有 mq9 操作都是标准的 NATS pub/sub/req-reply 调用，Subject 命名遵循 `$mq9.` 前缀约定。

任何支持 NATS 的客户端库都可以直接使用 mq9，无需额外 SDK。

## Subject 命名规范

```
$mq9.AI.<操作>.<参数...>
```

| Subject | 类型 | 说明 |
|---------|------|------|
| `$mq9.AI.MAILBOX.CREATE` | req/reply | 创建 Agent 身份和邮箱 |
| `$mq9.AI.INBOX.<mail_id>.<priority>` | pub/sub | 向指定 Agent 邮箱投递消息 |
| `$mq9.AI.BROADCAST.<domain>.<event>` | pub/sub | 广播事件 |
| `$mq9.AI.STATUS.<mail_id>` | pub/sub | Agent 状态上报 |

## 邮箱协议

### 创建邮箱

Agent 首次使用时，通过 req/reply 申请身份：

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{}'
```

响应：

```json
{
  "mail_id": "agt-uuid-001",
  "token": "tok-xxxxxxxx"
}
```

`mail_id` 是全局唯一标识，后续所有通信使用此 ID。

### 投递消息

向指定 Agent 的邮箱投递消息，即使对方不在线，消息也会持久化等待：

```bash
nats pub '$mq9.AI.INBOX.<mail_id>.<priority>' '<payload>'
```

`<priority>` 取值：

| 优先级 | Subject | 说明 |
|--------|---------|------|
| `urgent` | `$mq9.AI.INBOX.agt-001.urgent` | 最高优先级，Agent 上线后最先处理 |
| `normal` | `$mq9.AI.INBOX.agt-001.normal` | 常规任务，FIFO |
| `notify` | `$mq9.AI.INBOX.agt-001.notify` | 通知类，不保证持久化 |

示例：

```bash
# 发送紧急停止指令
nats pub '$mq9.AI.INBOX.agt-001.urgent' \
  '{"type":"stop","reason":"anomaly detected"}'

# 发送常规任务
nats pub '$mq9.AI.INBOX.agt-001.normal' \
  '{"from":"agt-002","task_id":"t-001","payload":"process dataset A"}'

# 发送心跳通知
nats pub '$mq9.AI.INBOX.agt-001.notify' \
  '{"type":"heartbeat","ts":1710000000}'
```

### 接收消息

Agent 上线后订阅自己的邮箱，通配符 `*` 匹配所有优先级：

```bash
# 接收所有优先级的消息
nats sub '$mq9.AI.INBOX.agt-001.*'

# 只接收紧急消息
nats sub '$mq9.AI.INBOX.agt-001.urgent'
```

## 广播协议

### 发布广播

```bash
nats pub '$mq9.AI.BROADCAST.<domain>.<event>' '<payload>'
```

`domain` 是业务域，`event` 是事件类型，两者都支持通配符订阅。

示例：

```bash
# 广播任务可用事件
nats pub '$mq9.AI.BROADCAST.task.available' \
  '{"task_id":"t-001","type":"analysis","priority":"high"}'

# 广播系统异常
nats pub '$mq9.AI.BROADCAST.system.anomaly' \
  '{"source":"monitor-agent","level":"critical"}'
```

### 订阅广播

```bash
# 订阅 task 域的所有事件
nats sub '$mq9.AI.BROADCAST.task.*'

# 订阅所有域的 anomaly 事件
nats sub '$mq9.AI.BROADCAST.*.anomaly'

# 订阅所有广播
nats sub '$mq9.AI.BROADCAST.>'
```

### 竞争消费（Queue Group）

多个 Worker Agent 竞争同一批任务时，使用 NATS queue group 保证每条消息只被一个 Worker 处理：

```bash
nats sub '$mq9.AI.BROADCAST.task.available' --queue workers
```

## 状态协议

Agent 可以通过状态 Subject 上报自身状态，让其他 Agent 感知其生命周期：

```bash
# 上报在线状态
nats pub '$mq9.AI.STATUS.agt-001' '{"status":"online","ts":1710000000}'

# 上报下线
nats pub '$mq9.AI.STATUS.agt-001' '{"status":"offline"}'
```

主 Agent 通过通配符订阅感知所有子 Agent 状态：

```bash
nats sub '$mq9.AI.STATUS.*'
```

## 消息格式建议

mq9 不强制消息格式，payload 是任意字节。推荐使用 JSON，字段参考：

```json
{
  "from": "agt-uuid-001",
  "task_id": "t-001",
  "type": "task|result|status|heartbeat",
  "payload": "...",
  "ts": 1710000000
}
```

## 与 NATS 原生协议的关系

mq9 是在 RobustMQ 的 NATS 协议层之上，通过 Subject 命名约定和服务端逻辑增强而来：

- 普通 NATS pub/sub：实时送达，不在线则丢失
- mq9 `$mq9.AI.INBOX.*`：持久化到存储层，离线也不丢失

两者可以混用。mq9 只是对特定 Subject 前缀的消息启用持久化和优先级调度，其余 NATS 行为不变。
