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

这是共享环境，任何知道 subject 名称的人都能操作，请勿发送敏感数据。以下示例均连接此服务器——在每条命令中加上 `-s nats://demo.robustmq.com:4222`，或一次性设置环境变量：

```bash
export NATS_URL=nats://demo.robustmq.com:4222
```

---

## 创建邮箱

邮箱是 mq9 的基本通信地址。使用 `nats request`（请求/回复）创建邮箱，服务端通过 NATS reply-to 返回分配的 `mail_address`：

```bash
nats request '$mq9.AI.MAILBOX.CREATE' '{"name":"quickstart.demo","ttl":300}'
```

响应：

```json
{"error":"","mail_address":"quickstart.demo"}
```

`mail_address` 是唯一的访问凭证。任何知道它的人都能向这个邮箱发消息或拉取消息。私有通信场景下请妥善保管。

TTL 设为 300 秒仅供演示方便。生产环境中请根据任务的预期生命周期选择合适的 TTL——TTL 到期后邮箱及其所有消息自动销毁，无需手动清理。

---

## 发送消息

向邮箱发送消息，通过 `mq9-priority` header 指定优先级：

```bash
# 最高优先级——立即处理；适用于中止信号、紧急指令
nats request '$mq9.AI.MSG.SEND.quickstart.demo' \
  --header 'mq9-priority:critical' \
  '{"type":"abort","task_id":"t-001"}'

# 紧急——适用于任务中断、时效性指令
nats request '$mq9.AI.MSG.SEND.quickstart.demo' \
  --header 'mq9-priority:urgent' \
  '{"type":"interrupt","task_id":"t-002"}'

# 默认优先级（normal）——常规通信；适用于任务分发、结果返回
nats request '$mq9.AI.MSG.SEND.quickstart.demo' \
  '{"type":"task","payload":"process dataset A"}'
```

每个发送命令都会返回响应（包含 `msg_id`）：

```json
{"error":"","msg_id":1}
```

---

## 拉取消息（FETCH）

mq9 使用 **pull 模式**：客户端主动调用 FETCH 拉取消息，而非被动等待推送。

```bash
nats request '$mq9.AI.MSG.FETCH.quickstart.demo' '{
  "group_name": "my-worker",
  "deliver": "earliest",
  "config": {"num_msgs": 10}
}'
```

响应包含按优先级排序的消息列表（critical → urgent → normal，同级 FIFO）：

```json
{
  "error": "",
  "messages": [
    {"msg_id": 1, "payload": "{\"type\":\"abort\",...}", "priority": "critical", "create_time": 1712600001},
    {"msg_id": 2, "payload": "{\"type\":\"interrupt\",...}", "priority": "urgent", "create_time": 1712600002},
    {"msg_id": 3, "payload": "{\"type\":\"task\",...}", "priority": "normal", "create_time": 1712600003}
  ]
}
```

传入 `group_name` 时，broker 会记录消费位点。下次 FETCH 会从上次 ACK 处续拉，不会重复消费。

---

## 确认消息（ACK）

处理完消息后调用 ACK，broker 推进该消费组的消费位点：

```bash
nats request '$mq9.AI.MSG.ACK.quickstart.demo' '{
  "group_name": "my-worker",
  "mail_address": "quickstart.demo",
  "msg_id": 3
}'
```

响应：

```json
{"error":""}
```

ACK 后再次 FETCH，会从 `msg_id: 3` 之后继续拉取新消息，不会重复收到已 ACK 的消息。

---

## 查询消息（QUERY）

QUERY 查看邮箱中当前存储的消息，不影响消费位点：

```bash
# 全量查询
nats request '$mq9.AI.MSG.QUERY.quickstart.demo' '{}'

# 按标签过滤（需发送时附加 mq9-tags header）
nats request '$mq9.AI.MSG.QUERY.quickstart.demo' '{"tags":["urgent"]}'

# 按时间范围
nats request '$mq9.AI.MSG.QUERY.quickstart.demo' '{"since":1712600000,"limit":20}'
```

---

## 删除消息

在邮箱 TTL 到期前删除某条特定消息：

```bash
nats request '$mq9.AI.MSG.DELETE.quickstart.demo.2' '{}'
```

subject 格式：`$mq9.AI.MSG.DELETE.{mail_address}.{msg_id}`

---

## Agent 注册与发现

mq9 内置 Agent 注册表，支持 Agent 能力的发布和检索。

**注册 Agent：**

```bash
nats request '$mq9.AI.AGENT.REGISTER' '{
  "name": "demo.translator",
  "payload": "多语言翻译 Agent，支持中英日韩互译，实时返回翻译结果"
}'
```

**按语义检索：**

```bash
nats request '$mq9.AI.AGENT.DISCOVER' '{
  "semantic": "帮我把中文翻译成英文",
  "limit": 5
}'
```

**按关键词检索：**

```bash
nats request '$mq9.AI.AGENT.DISCOVER' '{
  "text": "translator",
  "limit": 10
}'
```

**注销 Agent：**

```bash
nats request '$mq9.AI.AGENT.UNREGISTER' '{"name":"demo.translator"}'
```

---

## 下一步

- **协议** — 完整 subject 参考、请求参数和消息结构：[协议设计](./Protocol.md)
- **核心功能** — FETCH+ACK 消费、优先级、消息属性、TTL 生命周期的深度解析：[核心功能](./Features.md)
- **概览** — 设计理念与典型 Agent 场景：[概览](./Overview.md)
