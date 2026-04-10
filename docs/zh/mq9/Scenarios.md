# 应用场景

mq9 围绕八个具体的 Agent 通信模式设计，每个模式对应特定的 mq9 功能组合。

---

### 1. 子 Agent 结果返回

编排者启动一个子 Agent 执行耗时任务，无法阻塞等待结果——它还有其他工作要做。子 Agent 独立完成后将结果存入编排者控制的邮箱。由于 mq9 采用先存储后推送，即使编排者在子 Agent 完成时正忙或临时断线，结果也会在那里等待。

编排者在启动时创建私有邮箱，将 `mail_id` 通过任务载荷传递给子 Agent。无需轮询、无需注册回调、无需共享状态——一个邮箱搞定。

```bash
# 编排者：创建私有回复邮箱（TTL 覆盖预期最长任务时间）
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl": 3600}'
# 响应: {"mail_id": "mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag"}

# 通过带外方式（如任务载荷）将 mail_id 传给子 Agent
nats pub '$mq9.AI.MAILBOX.MSG.m-task-dispatch.normal' \
  '{"task": "summarize /data/corpus", "reply_to": "mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag"}'

# 子 Agent：完成后存入结果
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.normal' \
  '{"status": "ok", "summary": "..."}'

# 编排者：随时订阅——结果已存储在那里
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.*'
```

**核心功能：** 私有邮箱、先存储后推送、异步结果取回。

---

### 2. 多 Worker 任务队列

生产者将任务发送到共享队列，多个 Worker 竞争消费——每条任务只被处理一次。Worker 可以随时加入或退出，无需重新配置。这是经典的工作队列，但内置容灾能力：如果 Worker 在确认任务前崩溃，消息仍留在存储中，下一个可用 Worker 会重新收到。

```bash
# 一次性创建共享公开邮箱（幂等——在每个 Worker 启动时调用也安全）
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 86400,
  "public": true,
  "name": "task.queue",
  "desc": "共享 Worker 任务队列"
}'

# Worker：以相同队列组名订阅——每条消息只投递给一个 Worker
# 终端 1
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers
# 终端 2
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers

# 生产者：按优先级发布任务
nats pub '$mq9.AI.MAILBOX.MSG.task.queue.critical' '{"task": "reindex", "id": "t-101"}'
nats pub '$mq9.AI.MAILBOX.MSG.task.queue.urgent'   '{"task": "interrupt", "id": "t-102"}'
nats pub '$mq9.AI.MAILBOX.MSG.task.queue'          '{"task": "summarize", "id": "t-103"}'
```

**核心功能：** 公开邮箱、队列组（竞争消费）、优先级排序。

---

### 3. 通过 TTL 追踪 Worker 健康状态

编排者需要知道哪些 Worker 当前存活，无需主动轮询。Worker 通过刷新邮箱来发送心跳。如果 Worker 死亡，其邮箱 TTL 到期，自动从 `PUBLIC.LIST` 消失。编排者订阅 `PUBLIC.LIST` 实时追踪注册和注销——无需健康检查接口，无需外部看门狗服务。

```bash
# 每个 Worker：启动时创建短 TTL 公开邮箱（名字编码了身份信息）
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 30,
  "public": true,
  "name": "worker.health.worker-42",
  "desc": "Worker 42 心跳"
}'

# Worker：定期重新创建（每约 20 秒，在 TTL 到期前）
# CREATE 是幂等的——如果邮箱仍存在则返回成功，不重置 TTL。
# 让邮箱自然过期后重新创建，模拟活跃心跳更新周期。
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 30,
  "public": true,
  "name": "worker.health.worker-42"
}'

# 编排者：订阅公开列表，监听注册和注销事件
nats sub '$mq9.AI.PUBLIC.LIST'
```

**核心功能：** TTL 自动清理、公开邮箱、`PUBLIC.LIST` 用于发现和过期追踪。

---

### 4. 告警广播

任何 Agent 都可以检测到异常并向所有已注册的处理器广播告警。处理器可能在告警触发时不在线——它们重连后仍会收到告警，因为 mq9 先写存储。高优先级确保处理器追赶积压时，告警消息优先于低优先级消息投递。

```bash
# 告警发送方：向共享告警邮箱发布最高优先级消息
nats pub '$mq9.AI.MAILBOX.MSG.alerts.critical' '{
  "type": "anomaly",
  "agent": "monitor-7",
  "detail": "CPU > 95% 持续 5 分钟",
  "ts": 1712600100
}'

# 处理器 A：订阅——立即收到，或重连时收到
nats sub '$mq9.AI.MAILBOX.MSG.alerts.*'

# 处理器 B：稍后订阅，仍能从存储中收到告警
nats sub '$mq9.AI.MAILBOX.MSG.alerts.*'
```

**核心功能：** 先存储后推送（处理器离线时仍能收到告警）、critical 优先级、公开邮箱。

---

### 5. 云端到边缘指令下发

云端编排者需要向可能因间歇性网络而离线数小时的边缘 Agent 下发指令。边缘 Agent 重连后必须按正确优先级顺序收到所有待处理指令——高优先级中止或重配置指令先于常规任务。云端无需任何消息中间件桥接或重试逻辑。

```bash
# 云端：向边缘 Agent 的私有邮箱发布指令（mail_id 在部署时共享）
# 最高优先级重配置
nats pub '$mq9.AI.MAILBOX.MSG.m-edge-agent-9a3f.critical' '{
  "cmd": "reconfigure",
  "params": {"sampling_rate": 100}
}'

# 默认优先级（normal）例行任务
nats pub '$mq9.AI.MAILBOX.MSG.m-edge-agent-9a3f' '{
  "cmd": "run_diagnostic",
  "target": "sensor-bank-2"
}'

# 边缘 Agent：重连后订阅——按优先级顺序收到所有存储的指令
nats sub '$mq9.AI.MAILBOX.MSG.m-edge-agent-9a3f.*'
```

**核心功能：** 离线投递（先存储后推送）、重连时的优先级排序、私有邮箱。

---

### 6. 人机混合审批工作流

Agent 生成了一个需要人工审查后才能继续的决策——例如，修改生产数据库或代表用户发送通信之前。人类使用与其他 Agent 完全相同的 mq9 协议进行交互。无需单独的审批服务、webhook 基础设施或带外通信渠道。Agent 完全基于邮箱消息挂起和恢复。

```python
import nats
import asyncio, json

async def run():
    nc = await nats.connect("nats://demo.robustmq.com:4222")

    # Agent：创建私有回复邮箱用于接收审批响应
    reply = await nc.request("$mq9.AI.MAILBOX.CREATE", b'{"ttl": 7200}')
    reply_id = json.loads(reply.data)["mail_id"]

    # Agent：发布决策供人工审查
    await nc.publish(
        f"$mq9.AI.MAILBOX.MSG.approvals.normal",
        json.dumps({
            "action": "delete_dataset",
            "target": "ds-prod-2024",
            "reply_to": reply_id
        }).encode()
    )

    # 人工（通过任意 NATS 客户端或 UI）：订阅 approvals，审查后发布决策
    # nats sub '$mq9.AI.MAILBOX.MSG.approvals.*'
    # nats pub '$mq9.AI.MAILBOX.MSG.<reply_id>.normal' '{"approved": true, "reviewer": "alice"}'

    # Agent：准备继续时订阅回复邮箱
    sub = await nc.subscribe(f"$mq9.AI.MAILBOX.MSG.{reply_id}.*")
    msg = await sub.next_msg(timeout=7200)
    decision = json.loads(msg.data)
    print("审批决策:", decision)

asyncio.run(run())
```

**核心功能：** 人与 Agent 使用相同协议、异步、先存储后推送。

---

### 7. 异步请求-回复

Agent A 需要 Agent B 的处理结果，但 B 可能不是立即可用，A 又不能阻塞。A 创建一个私有回复邮箱，在请求中通过 `reply_to` 字段嵌入 `mail_id`，然后继续其他工作。B 按自己的节奏处理请求，将结果发送到 A 的回复邮箱。A 在准备消费响应时订阅回复邮箱。

```bash
# Agent A：创建私有回复邮箱
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl": 600}'
# 响应: {"mail_id": "m-reply-a1b2c3"}

# Agent A：向 Agent B 的邮箱发送请求，包含 reply_to 字段
nats pub '$mq9.AI.MAILBOX.MSG.m-agent-b-inbox.normal' '{
  "request": "translate",
  "text": "Hello world",
  "lang": "fr",
  "reply_to": "m-reply-a1b2c3"
}'

# Agent A：继续其他工作...

# Agent B：处理请求后将结果发送到回复邮箱
nats pub '$mq9.AI.MAILBOX.MSG.m-reply-a1b2c3.normal' '{
  "result": "Bonjour le monde"
}'

# Agent A：准备好时订阅回复邮箱——结果已存储在那里
nats sub '$mq9.AI.MAILBOX.MSG.m-reply-a1b2c3.*'
```

**核心功能：** 私有邮箱作为回复地址、先存储后推送、非阻塞异步模式。

---

### 8. Agent 能力发现

Agent 通过创建具有描述性、结构化名称的公开邮箱来公告自己的能力。其他 Agent 订阅 `PUBLIC.LIST` 来实时发现当前可用的能力——无需中心化注册表、服务网格或配置文件。当能力 Agent 关闭时，其邮箱 TTL 到期，自动从列表中消失。

```bash
# 能力 Agent：通过创建命名公开邮箱注册自己
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 3600,
  "public": true,
  "name": "agent.code-review",
  "desc": "接受代码审查请求；以 JSON 格式返回发现结果"
}'

# 另一个 Agent：订阅 PUBLIC.LIST 发现可用能力
nats sub '$mq9.AI.PUBLIC.LIST'
# 收到条目如: {"name": "agent.code-review", "desc": "...", "mail_id": "agent.code-review"}

# 消费方 Agent：直接向发现的能力发送任务
nats pub '$mq9.AI.MAILBOX.MSG.agent.code-review.normal' '{
  "file": "src/main.rs",
  "context": "性能审查"
}'
```

**核心功能：** 带描述性名称的公开邮箱、`PUBLIC.LIST` 实时发现、去中心化能力注册。
