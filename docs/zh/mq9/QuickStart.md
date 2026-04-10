# 快速开始

本指南使用 NATS CLI 带你完整体验 mq9 的核心操作，连接公共演示服务器。无需账号、无需配置、无需 SDK——只需一个终端。

---

## 准备工作

安装 [NATS CLI](https://docs.nats.io/using-nats/nats-tools/nats_cli)，这是与 mq9 交互唯一需要的工具。

---

## 连接公共服务器

RobustMQ 演示服务器地址：

```
nats://demo.robustmq.com:4222
```

这是共享环境，任何知道 subject 名称的人都能订阅，请勿发送敏感数据。以下示例均连接此服务器——在每条命令中加上 `-s nats://demo.robustmq.com:4222`，或一次性设置环境变量：

```bash
export NATS_URL=nats://demo.robustmq.com:4222
```

---

## 创建邮箱

邮箱是 mq9 的基本通信地址。使用 `nats req`（请求/回复）创建邮箱，服务端通过 NATS reply-to 返回分配的 `mail_id`：

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":60}'
```

响应：

```json
{"mail_id":"mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag"}
```

`mail_id` 是唯一的访问凭证。任何知道它的人都能向这个邮箱发消息或订阅。私有通信场景下请妥善保管。

此处 TTL 设为 60 秒仅供演示方便。生产环境中请根据任务的预期生命周期选择合适的 TTL——TTL 到期后邮箱及其所有消息自动销毁，无需手动清理。

---

## 按优先级发送消息

mq9 支持三个优先级：`critical`、`urgent`、`normal`（默认，无后缀）。发送消息的 subject 格式为：

```
$mq9.AI.MAILBOX.MSG.{mail_id}.{priority}   # urgent 或 critical
$mq9.AI.MAILBOX.MSG.{mail_id}              # 默认（normal），无后缀
```

将 `mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag` 替换为上一步拿到的 `mail_id`：

```bash
# 最高优先级——立即处理；适用于中止信号、紧急指令、安全事件
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.critical' '{"type":"abort","task_id":"t-001"}'

# 紧急——适用于任务中断、时效性指令
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.urgent'   '{"type":"interrupt","task_id":"t-002"}'

# 默认优先级（normal）——常规通信；适用于任务分发、结果返回
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag' '{"type":"task","payload":"process dataset A"}'
```

发送是即发即忘（`nats pub`）。服务端立即将每条消息写入存储，发送方无需等待订阅者在线。

> **禁止通配符发布。** Subject `$mq9.AI.MAILBOX.MSG.*.*` 会被服务端拒绝，发送时必须指定精确的 `mail_id`。

---

## 订阅接收消息

订阅邮箱消息：

```bash
# 订阅所有优先级（critical、urgent、normal）
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.*'
```

上面发送的消息会在订阅后立即推送——无论订阅者发送时是否在线。这是 mq9 的**先存储后推送语义**：每次订阅都会先按优先级顺序（critical → urgent → normal，同级 FIFO）推送所有未过期的存储消息，然后切换到实时推送新消息。

这意味着订阅先于还是晚于消息发送，结果是一样的。Agent 网络中断后重连，会自动收到所有错过的消息。

只订阅某一优先级：

```bash
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.urgent'
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.critical'
```

---

## 列出消息元数据

不消费消息，只查看邮箱中当前存储的消息——向 LIST subject 发送请求：

```bash
nats req '$mq9.AI.MAILBOX.LIST.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag' '{}'
```

响应：

```json
{
  "messages": [
    {"msg_id": "msg-001", "priority": "critical", "ts": 1712600001},
    {"msg_id": "msg-002", "priority": "urgent",   "ts": 1712600002},
    {"msg_id": "msg-003", "priority": "normal",   "ts": 1712600003}
  ]
}
```

响应返回每条消息的 `msg_id`、`priority` 和 `ts`（Unix 时间戳）。不包含消息体——LIST 用于检视，不用于检索。使用 `msg_id` 可对特定消息执行删除操作。

---

## 删除消息

在邮箱 TTL 到期前删除某条特定消息：

```bash
nats req '$mq9.AI.MAILBOX.DELETE.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.msg-002' '{}'
```

竞争消费场景下，Worker 完成任务后可以显式删除任务消息来确认完成。Subject 格式为：

```
$mq9.AI.MAILBOX.DELETE.{mail_id}.{msg_id}
```

---

## 创建公开邮箱

公开邮箱的 `mail_id` 由用户自定义——你选择的名字就是地址。适用于多方需要共同发现地址、无需带外协调的共享任务队列或能力公告。

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 3600,
  "public": true,
  "name": "task.queue",
  "desc": "共享 Worker 任务队列"
}'
```

响应：

```json
{"mail_id":"task.queue"}
```

`mail_id` 就是你提供的 `name`。邮箱自动注册到 `$mq9.AI.PUBLIC.LIST`，任何订阅该系统地址的 Agent 都可以发现它。TTL 到期后自动从列表中移除。

CREATE 是幂等的：如果名为 `task.queue` 的邮箱已存在，此调用返回成功但不重置 TTL。Worker 启动时可以安全地调用它而不必担心覆盖已有邮箱。

---

## 队列组（竞争消费）

当多个 Worker 以相同队列组名订阅同一邮箱时，mq9 将每条消息只投递给其中一个 Worker。打开两个终端，各运行相同命令：

**终端 1：**

```bash
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers
```

**终端 2：**

```bash
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers
```

发送几条消息：

```bash
nats pub '$mq9.AI.MAILBOX.MSG.task.queue' '{"task":"job-1"}'
nats pub '$mq9.AI.MAILBOX.MSG.task.queue' '{"task":"job-2"}'
nats pub '$mq9.AI.MAILBOX.MSG.task.queue' '{"task":"job-3"}'
```

每条消息只会出现在一个终端——Broker 将它们分配到队列组中。Worker 可以随时加入或退出，队列自动调整，无需任何配置变更。

由于 mq9 使用先存储后推送，如果 Worker 在删除任务消息之前崩溃，消息仍保留在存储中，当任意组成员重连时会重新投递。将公开邮箱与队列组结合，即可实现零配置、容灾容错的任务队列。

---

## 下一步

- **协议** — 完整 subject 参考、请求参数、消息结构和存储分层：[协议设计](./Protocol.md)
- **核心功能** — 优先级语义、先存储后推送、TTL 生命周期和竞争消费的深度解析：[核心功能](./Features.md)
- **概览** — 设计理念、与 NATS Core 和 JetStream 的定位差异，以及八个典型 Agent 场景：[概览](./Overview.md)
