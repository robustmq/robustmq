# 体验 mq9

## 前提：启动 Broker

参考 [快速安装](Quick-Install.md) 完成安装，然后启动服务：

```bash
robust-server start
```

mq9 随 RobustMQ 启动，无需额外配置，默认监听 NATS 端口 `4222`。

---

## 准备 NATS CLI

mq9 基于 NATS 协议，只需安装 NATS CLI 即可体验所有操作：

```bash
# macOS
brew install nats-io/nats-tools/nats

# Linux / Windows
# 参考：https://docs.nats.io/using-nats/nats-tools/nats_cli
```

安装完成后设置连接地址：

```bash
export NATS_URL=nats://localhost:4222
```

---

## 注册 Agent

注册 Agent 及其能力描述。其他 Agent 可以通过关键词或语义意图发现它：

```bash
nats request '$mq9.AI.AGENT.REGISTER' '{
  "name": "agent.translator",
  "mailbox": "agent.translator",
  "payload": "多语言翻译 Agent，支持中英日韩互译"
}'
```

---

## 发现 Agent

通过语义意图或关键词发现 Agent：

```bash
# 语义检索
nats request '$mq9.AI.AGENT.DISCOVER' '{"semantic":"帮我把中文翻译成英文","limit":5}'

# 全文检索
nats request '$mq9.AI.AGENT.DISCOVER' '{"text":"translator","limit":10}'
```

---

## 创建邮箱

每个 Agent 有一个持久化邮箱，消息在 Agent 上线前持久存储：

```bash
nats request '$mq9.AI.MAILBOX.CREATE' '{"name":"agent.translator","ttl":3600}'
```

响应：

```json
{"error":"","mail_address":"agent.translator"}
```

---

## 发送消息

通过 `mq9-priority` header 指定三个优先级：

```bash
# 最高优先级——中止信号、紧急指令
nats request '$mq9.AI.MSG.SEND.agent.translator' \
  --header 'mq9-priority:critical' \
  '{"type":"abort","task_id":"t-001"}'

# 紧急
nats request '$mq9.AI.MSG.SEND.agent.translator' \
  --header 'mq9-priority:urgent' \
  '{"type":"interrupt","task_id":"t-002"}'

# 默认优先级（normal，不填 header）
nats request '$mq9.AI.MSG.SEND.agent.translator' \
  '{"type":"task","payload":"process dataset A"}'
```

消息即使接收方离线也会持久化等待。

---

## 拉取消息（FETCH + ACK）

mq9 使用 pull 模式消费。客户端主动调用 FETCH 拉取消息，按优先级排序（critical → urgent → normal）：

```bash
nats request '$mq9.AI.MSG.FETCH.agent.translator' '{
  "group_name": "my-worker",
  "deliver": "earliest",
  "config": {"num_msgs": 10}
}'
```

处理完消息后调用 ACK，broker 推进该消费组的消费位点：

```bash
nats request '$mq9.AI.MSG.ACK.agent.translator' '{
  "group_name": "my-worker",
  "mail_address": "agent.translator",
  "msg_id": 1
}'
```

下次 FETCH 从上次 ACK 处继续，不会重复消费。

---

## 下一步

- **完整文档** — [mq9.robustmq.com](https://mq9.robustmq.com/zh/)
- **协议设计** — [mq9.robustmq.com/zh/docs/protocol](https://mq9.robustmq.com/zh/docs/protocol)
- **SDK 接入** — [mq9.robustmq.com/zh/docs/sdk](https://mq9.robustmq.com/zh/docs/sdk)
- **LangChain 集成** — [mq9.robustmq.com/zh/docs/langchain](https://mq9.robustmq.com/zh/docs/langchain)
