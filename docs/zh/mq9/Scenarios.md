# 应用场景

mq9 围绕八个具体的 Agent 通信模式设计，每个模式对应特定的 mq9 功能组合。

---

## 1. 子 Agent 结果返回

编排者启动一个子 Agent 执行耗时任务，无法阻塞等待结果——它还有其他工作要做。子 Agent 独立完成后将结果存入编排者控制的邮箱。由于 mq9 采用先存储后拉取，即使编排者在子 Agent 完成时正忙或临时断线，结果也会在那里等待。

编排者在启动时创建私有邮箱，将 `mail_address` 通过任务载荷传递给子 Agent。无需轮询、无需注册回调、无需共享状态——一个邮箱搞定。

```bash
# 编排者：创建私有回复邮箱（TTL 覆盖预期最长任务时间）
nats request '$mq9.AI.MAILBOX.CREATE' '{"ttl": 3600}'
# 响应: {"mail_address": "d7a5072lko83"}

# 编排者：将任务发给子 Agent（通过带外方式或其邮箱），带上 reply_to
nats request '$mq9.AI.MSG.SEND.task.dispatch' \
  '{"task": "summarize /data/corpus", "reply_to": "d7a5072lko83"}'

# 子 Agent：完成后将结果写入编排者邮箱
nats request '$mq9.AI.MSG.SEND.d7a5072lko83' \
  '{"status": "ok", "summary": "..."}'

# 编排者：主动拉取结果
nats request '$mq9.AI.MSG.FETCH.d7a5072lko83' \
  '{"group_name": "orchestrator", "deliver": "earliest"}'
# ACK 推进位点
nats request '$mq9.AI.MSG.ACK.d7a5072lko83' \
  '{"group_name": "orchestrator", "mail_address": "d7a5072lko83", "msg_id": 1}'
```

**核心功能：** 私有邮箱、先存储后拉取、FETCH+ACK 异步结果取回。

---

## 2. 多 Worker 任务队列

生产者将任务发送到共享队列，多个 Worker 竞争消费——每条任务只被处理一次。Worker 可以随时加入或退出，无需重新配置。如果 Worker 在 ACK 前崩溃，消息仍在存储中，下一个 Worker 可以用新的 group_name 或 `force_deliver: true` 重新获取。

```bash
# 一次性创建共享公开邮箱
nats request '$mq9.AI.MAILBOX.CREATE' '{
  "name": "task.queue",
  "ttl": 86400,
  "desc": "共享 Worker 任务队列"
}'

# 生产者：发布任务（通过 mq9-priority header 指定优先级）
nats request '$mq9.AI.MSG.SEND.task.queue' \
  --header 'mq9-priority:critical' \
  '{"task": "reindex", "id": "t-101"}'
nats request '$mq9.AI.MSG.SEND.task.queue' \
  --header 'mq9-priority:urgent' \
  '{"task": "interrupt", "id": "t-102"}'
nats request '$mq9.AI.MSG.SEND.task.queue' \
  '{"task": "summarize", "id": "t-103"}'

# Worker A：拉取并处理（带 group_name 记录位点）
nats request '$mq9.AI.MSG.FETCH.task.queue' \
  '{"group_name": "workers", "deliver": "earliest", "config": {"num_msgs": 1}}'
# 处理完毕后 ACK
nats request '$mq9.AI.MSG.ACK.task.queue' \
  '{"group_name": "workers", "mail_address": "task.queue", "msg_id": 1}'
```

**核心功能：** 命名邮箱、有状态消费（group_name 记录位点）、三级优先级排序。

---

## 3. 通过 TTL 追踪 Worker 健康状态

编排者需要知道哪些 Worker 当前存活，无需主动轮询。Worker 通过重建短 TTL 邮箱来发送心跳，并通过 AGENT.REGISTER 注册自身。如果 Worker 死亡，其注册记录 TTL 到期，自动从 DISCOVER 结果中消失。

```bash
# Worker 启动：注册自身到 Agent 注册表
nats request '$mq9.AI.AGENT.REGISTER' '{
  "name": "worker-42",
  "payload": "图像处理 Worker，支持 JPEG/PNG 格式，GPU 加速"
}'

# Worker：定期上报状态（心跳）
nats request '$mq9.AI.AGENT.REPORT' '{
  "name": "worker-42",
  "report_info": "running, processed: 1024 tasks"
}'

# 编排者：通过 DISCOVER 列出所有在线 Worker
nats request '$mq9.AI.AGENT.DISCOVER' '{}'

# Worker 关闭：注销注册
nats request '$mq9.AI.AGENT.UNREGISTER' '{"name": "worker-42"}'
```

**核心功能：** Agent 注册与注销、DISCOVER 发现存活 Agent、REPORT 状态上报。

---

## 4. 告警广播

任何 Agent 都可以检测到异常并向告警邮箱发布消息。处理器主动 FETCH 拉取——即使处理器临时不在线，告警消息已落存储，重连后拉取即可。high-priority 确保追赶积压时告警优先被处理。

```bash
# 告警发送方：向共享告警邮箱发布最高优先级消息
nats request '$mq9.AI.MSG.SEND.alerts' \
  --header 'mq9-priority:critical' \
  '{
    "type": "anomaly",
    "agent": "monitor-7",
    "detail": "CPU > 95% 持续 5 分钟",
    "ts": 1712600100
  }'

# 处理器 A：拉取告警（有状态，断点续拉）
nats request '$mq9.AI.MSG.FETCH.alerts' \
  '{"group_name": "alert-handlers", "deliver": "earliest"}'

# 处理器 A：确认处理
nats request '$mq9.AI.MSG.ACK.alerts' \
  '{"group_name": "alert-handlers", "mail_address": "alerts", "msg_id": 5}'
```

**核心功能：** 消息持久化（处理器离线仍能后续拉取）、critical 优先级、FETCH+ACK 消费。

---

## 5. 云端到边缘指令下发

云端编排者需要向可能因间歇性网络而离线数小时的边缘 Agent 下发指令。边缘 Agent 重连后主动 FETCH 拉取，按优先级顺序获取所有待处理指令——高优先级中止或重配置指令先于常规任务。

```bash
# 云端：向边缘 Agent 邮箱发布指令
# 最高优先级重配置
nats request '$mq9.AI.MSG.SEND.edge.agent' \
  --header 'mq9-priority:critical' \
  '{"cmd": "reconfigure", "params": {"sampling_rate": 100}}'

# 默认优先级（normal）例行任务
nats request '$mq9.AI.MSG.SEND.edge.agent' \
  '{"cmd": "run_diagnostic", "target": "sensor-bank-2"}'

# 边缘 Agent：重连后拉取所有待处理指令（按优先级顺序返回）
nats request '$mq9.AI.MSG.FETCH.edge.agent' \
  '{"group_name": "edge-agent", "deliver": "earliest", "config": {"num_msgs": 10}}'

# 边缘 Agent：处理完毕后 ACK
nats request '$mq9.AI.MSG.ACK.edge.agent' \
  '{"group_name": "edge-agent", "mail_address": "edge.agent", "msg_id": 2}'
```

**核心功能：** 消息持久化、重连后按优先级顺序拉取、私有邮箱。

---

## 6. 人机混合审批工作流

Agent 生成了一个需要人工审查后才能继续的决策——例如，修改生产数据库或代表用户发送通信之前。人类使用与其他 Agent 完全相同的 mq9 协议进行交互。

```python
import nats
import asyncio, json

async def run():
    nc = await nats.connect("nats://demo.robustmq.com:4222")

    # Agent：创建私有回复邮箱用于接收审批响应
    reply = await nc.request("$mq9.AI.MAILBOX.CREATE", b'{"ttl": 7200}')
    reply_id = json.loads(reply.data)["mail_address"]

    # Agent：发布决策供人工审查
    await nc.request(
        "$mq9.AI.MSG.SEND.approvals",
        json.dumps({
            "action": "delete_dataset",
            "target": "ds-prod-2024",
            "reply_to": reply_id
        }).encode()
    )

    # 人工（通过任意 NATS 客户端或 UI）：拉取 approvals 邮箱中的审批请求
    # nats request '$mq9.AI.MSG.FETCH.approvals' '{"deliver": "earliest"}'
    # 审查后写入决策到 reply_id 邮箱：
    # nats request '$mq9.AI.MSG.SEND.<reply_id>' '{"approved": true, "reviewer": "alice"}'

    # Agent：准备好时拉取审批结果
    reply_resp = await nc.request(
        f"$mq9.AI.MSG.FETCH.{reply_id}",
        json.dumps({"deliver": "earliest", "config": {"max_wait_ms": 7200000}}).encode()
    )
    messages = json.loads(reply_resp.data).get("messages", [])
    decision = json.loads(messages[0]["payload"]) if messages else {}
    print("审批决策:", decision)

asyncio.run(run())
```

**核心功能：** 人与 Agent 使用相同协议、异步 FETCH 消费、先存储后拉取。

---

## 7. 异步请求-回复

Agent A 需要 Agent B 的处理结果，但 B 可能不是立即可用，A 又不能阻塞。A 创建一个私有回复邮箱，在请求中通过 `reply_to` 字段嵌入 `mail_address`，然后继续其他工作。B 按自己的节奏处理请求，将结果写入 A 的回复邮箱。A 在准备消费响应时主动 FETCH。

```bash
# Agent A：创建私有回复邮箱
nats request '$mq9.AI.MAILBOX.CREATE' '{"ttl": 600}'
# 响应: {"mail_address": "reply.a1b2c3"}

# Agent A：向 Agent B 的邮箱发送请求，包含 reply_to 字段
nats request '$mq9.AI.MSG.SEND.agent.b' '{
  "request": "translate",
  "text": "Hello world",
  "lang": "fr",
  "reply_to": "reply.a1b2c3"
}'

# Agent A：继续其他工作...

# Agent B：拉取自己邮箱中的请求
nats request '$mq9.AI.MSG.FETCH.agent.b' \
  '{"group_name": "b-worker", "deliver": "earliest"}'

# Agent B：处理完后将结果写入 A 的回复邮箱
nats request '$mq9.AI.MSG.SEND.reply.a1b2c3' '{"result": "Bonjour le monde"}'
# ACK 自己的消费位点
nats request '$mq9.AI.MSG.ACK.agent.b' \
  '{"group_name": "b-worker", "mail_address": "agent.b", "msg_id": 1}'

# Agent A：准备好时拉取回复——结果已存储在那里
nats request '$mq9.AI.MSG.FETCH.reply.a1b2c3' \
  '{"deliver": "earliest"}'
```

**核心功能：** 私有邮箱作为回复地址、FETCH+ACK pull 消费、非阻塞异步模式。

---

## 8. Agent 能力发现

Agent 通过 REGISTER 注册自身能力描述，其他 Agent 通过 DISCOVER 按关键词或语义查找合适的 Agent——无需中心化配置文件或手动维护地址表。当能力 Agent 关闭后调用 UNREGISTER，发现结果中自动消失。

```bash
# 能力 Agent：启动时注册自身
nats request '$mq9.AI.AGENT.REGISTER' '{
  "name": "agent.code-review",
  "payload": "接受代码审查请求，支持 Rust/Go/Python，返回发现的问题列表（JSON 格式）"
}'

# 消费方 Agent：按关键词全文检索
nats request '$mq9.AI.AGENT.DISCOVER' '{
  "text": "code review",
  "limit": 10
}'

# 消费方 Agent：按语义向量检索（理解自然语言意图）
nats request '$mq9.AI.AGENT.DISCOVER' '{
  "semantic": "帮我检查 Rust 代码的问题",
  "limit": 5
}'
# 返回匹配的 Agent 列表，包含 name、mail_address、payload 等字段

# 消费方 Agent：直接向发现的 Agent 邮箱发送任务
nats request '$mq9.AI.MSG.SEND.agent.code-review' '{
  "file": "src/main.rs",
  "context": "性能审查"
}'

# 能力 Agent：关闭时注销
nats request '$mq9.AI.AGENT.UNREGISTER' '{"name": "agent.code-review"}'
```

**核心功能：** Agent 注册与发现、全文检索 + 语义向量检索、去中心化能力注册。
