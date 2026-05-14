# Use Cases

mq9 is designed around eight concrete Agent communication patterns. Each pattern maps to a specific mq9 feature combination.

---

## 1. Sub-Agent Result Delivery

An orchestrator spawns a sub-agent for a long-running task and cannot block waiting for the result — it has other work to do. The sub-agent completes independently and deposits its result into a mailbox the orchestrator controls. Because mq9 stores messages first, the result is waiting even if the orchestrator is busy or temporarily disconnected when the sub-agent finishes.

The orchestrator creates a private mailbox and shares the `mail_address` with the sub-agent at spawn time. No polling, no callback registration, no shared state — just a mailbox.

```bash
# Orchestrator: create private reply mailbox (TTL covers max expected task duration)
nats request '$mq9.AI.MAILBOX.CREATE' '{"ttl": 3600}'
# Response: {"mail_address": "d7a5072lko83"}

# Orchestrator: send task to sub-agent's mailbox with reply_to
nats request '$mq9.AI.MSG.SEND.task.dispatch' \
  '{"task": "summarize /data/corpus", "reply_to": "d7a5072lko83"}'

# Sub-agent: deposit result when done
nats request '$mq9.AI.MSG.SEND.d7a5072lko83' \
  '{"status": "ok", "summary": "..."}'

# Orchestrator: FETCH result whenever ready
nats request '$mq9.AI.MSG.FETCH.d7a5072lko83' \
  '{"group_name": "orchestrator", "deliver": "earliest"}'
# ACK to advance offset
nats request '$mq9.AI.MSG.ACK.d7a5072lko83' \
  '{"group_name": "orchestrator", "mail_address": "d7a5072lko83", "msg_id": 1}'
```

**Key mq9 features:** private mailbox, store-first, FETCH+ACK async result pickup.

---

## 2. Multi-Worker Task Queue

A producer sends tasks into a shared queue. Multiple workers compete to consume — each task is processed exactly once. Workers can join or leave at any time without reconfiguration. If a worker crashes before ACKing, the message remains in storage and the next worker can pick it up.

```bash
# Create shared mailbox once
nats request '$mq9.AI.MAILBOX.CREATE' '{
  "name": "task.queue",
  "ttl": 86400,
  "desc": "Shared worker task queue"
}'

# Producer: publish tasks with priority via mq9-priority header
nats request '$mq9.AI.MSG.SEND.task.queue' \
  --header 'mq9-priority:critical' \
  '{"task": "reindex", "id": "t-101"}'
nats request '$mq9.AI.MSG.SEND.task.queue' \
  --header 'mq9-priority:urgent' \
  '{"task": "interrupt", "id": "t-102"}'
nats request '$mq9.AI.MSG.SEND.task.queue' \
  '{"task": "summarize", "id": "t-103"}'

# Worker A: fetch one message (stateful — broker tracks offset for this group)
nats request '$mq9.AI.MSG.FETCH.task.queue' \
  '{"group_name": "workers", "deliver": "earliest", "config": {"num_msgs": 1}}'
# ACK after successful processing
nats request '$mq9.AI.MSG.ACK.task.queue' \
  '{"group_name": "workers", "mail_address": "task.queue", "msg_id": 1}'
```

**Key mq9 features:** named mailbox, stateful consumption (group_name tracks offset), three-tier priority ordering.

---

## 3. Worker Health Tracking via TTL

An orchestrator needs to know which workers are alive without polling them. Workers register themselves via AGENT.REGISTER and send periodic heartbeats via AGENT.REPORT. If a worker dies, the orchestrator uses DISCOVER to see which agents are no longer responding.

```bash
# Worker at startup: register with capability description
nats request '$mq9.AI.AGENT.REGISTER' '{
  "name": "worker-42",
  "payload": "Image processing worker, supports JPEG/PNG, GPU-accelerated"
}'

# Worker: periodic heartbeat
nats request '$mq9.AI.AGENT.REPORT' '{
  "name": "worker-42",
  "report_info": "running, processed: 1024 tasks"
}'

# Orchestrator: list all registered workers
nats request '$mq9.AI.AGENT.DISCOVER' '{}'

# Worker at shutdown: unregister
nats request '$mq9.AI.AGENT.UNREGISTER' '{"name": "worker-42"}'
```

**Key mq9 features:** Agent register/unregister, DISCOVER for live agent enumeration, REPORT for heartbeat.

---

## 4. Alert Broadcasting

Any agent can detect an anomaly and publish an alert to a shared mailbox. Handlers actively FETCH — even if temporarily offline, alerts are persisted and available on reconnect. `critical` priority ensures alerts are returned before any accumulated lower-priority backlog.

```bash
# Alert sender: publish to shared alert mailbox at highest priority
nats request '$mq9.AI.MSG.SEND.alerts' \
  --header 'mq9-priority:critical' \
  '{
    "type": "anomaly",
    "agent": "monitor-7",
    "detail": "CPU > 95% for 5m",
    "ts": 1712600100
  }'

# Handler A: fetch alerts (stateful, resumes from last ACK on reconnect)
nats request '$mq9.AI.MSG.FETCH.alerts' \
  '{"group_name": "alert-handlers", "deliver": "earliest"}'

# Handler A: ACK after processing
nats request '$mq9.AI.MSG.ACK.alerts' \
  '{"group_name": "alert-handlers", "mail_address": "alerts", "msg_id": 5}'
```

**Key mq9 features:** message persistence (handlers receive alerts even if temporarily offline), critical priority, FETCH+ACK consumption.

---

## 5. Cloud-to-Edge Command Delivery

A cloud orchestrator delivers commands to edge agents that may be offline for hours due to intermittent connectivity. When the edge agent reconnects, it actively FETCHes all pending commands in priority order — `critical` abort or reconfigure commands before routine `normal` tasks. No retry logic or bridging required on the cloud side.

```bash
# Cloud: publish commands to edge agent's private mailbox
# Critical-priority reconfiguration
nats request '$mq9.AI.MSG.SEND.edge.agent' \
  --header 'mq9-priority:critical' \
  '{"cmd": "reconfigure", "params": {"sampling_rate": 100}}'

# Default-priority (normal) routine task
nats request '$mq9.AI.MSG.SEND.edge.agent' \
  '{"cmd": "run_diagnostic", "target": "sensor-bank-2"}'

# Edge agent: on reconnect, fetch all pending commands (returned in priority order)
nats request '$mq9.AI.MSG.FETCH.edge.agent' \
  '{"group_name": "edge-agent", "deliver": "earliest", "config": {"num_msgs": 10}}'

# Edge agent: ACK after processing
nats request '$mq9.AI.MSG.ACK.edge.agent' \
  '{"group_name": "edge-agent", "mail_address": "edge.agent", "msg_id": 2}'
```

**Key mq9 features:** message persistence, priority-ordered pull on reconnect, private mailbox.

---

## 6. Human-in-the-Loop Approval Workflow

An agent generates a decision requiring human review before proceeding — for example, modifying a production database or sending a communication on behalf of a user. Humans interact using the exact same mq9 protocol as any other agent. No separate approval service or webhook infrastructure needed.

```python
import nats
import asyncio, json

async def run():
    nc = await nats.connect("nats://demo.robustmq.com:4222")

    # Agent: create private reply mailbox for the approval response
    reply = await nc.request("$mq9.AI.MAILBOX.CREATE", b'{"ttl": 7200}')
    reply_id = json.loads(reply.data)["mail_address"]

    # Agent: publish decision for human review
    await nc.request(
        "$mq9.AI.MSG.SEND.approvals",
        json.dumps({
            "action": "delete_dataset",
            "target": "ds-prod-2024",
            "reply_to": reply_id
        }).encode()
    )

    # Human (via any NATS client or UI): fetch pending approvals and publish decision
    # nats request '$mq9.AI.MSG.FETCH.approvals' '{"deliver": "earliest"}'
    # nats request '$mq9.AI.MSG.SEND.<reply_id>' '{"approved": true, "reviewer": "alice"}'

    # Agent: FETCH approval result when ready
    reply_resp = await nc.request(
        f"$mq9.AI.MSG.FETCH.{reply_id}",
        json.dumps({"deliver": "earliest", "config": {"max_wait_ms": 7200000}}).encode()
    )
    messages = json.loads(reply_resp.data).get("messages", [])
    decision = json.loads(messages[0]["payload"]) if messages else {}
    print("Approval decision:", decision)

asyncio.run(run())
```

**Key mq9 features:** same protocol for human and agent interaction, async FETCH consumption, store-first delivery.

---

## 7. Async Request-Reply

Agent A needs a result from Agent B, but B may not be available immediately and A cannot afford to block. A creates a private reply mailbox, embeds the `mail_address` in the request as a `reply_to` field, and continues other work. B processes the request at its own pace and sends the result to A's reply mailbox. A FETCHes the reply whenever ready.

```bash
# Agent A: create private reply mailbox
nats request '$mq9.AI.MAILBOX.CREATE' '{"ttl": 600}'
# Response: {"mail_address": "reply.a1b2c3"}

# Agent A: send request to Agent B's mailbox with reply_to field
nats request '$mq9.AI.MSG.SEND.agent.b' '{
  "request": "translate",
  "text": "Hello world",
  "lang": "fr",
  "reply_to": "reply.a1b2c3"
}'

# Agent A: continues other work...

# Agent B: fetch its own pending requests
nats request '$mq9.AI.MSG.FETCH.agent.b' \
  '{"group_name": "b-worker", "deliver": "earliest"}'

# Agent B: send result to A's reply mailbox
nats request '$mq9.AI.MSG.SEND.reply.a1b2c3' '{"result": "Bonjour le monde"}'
# ACK Agent B's own offset
nats request '$mq9.AI.MSG.ACK.agent.b' \
  '{"group_name": "b-worker", "mail_address": "agent.b", "msg_id": 1}'

# Agent A: FETCH reply when ready — result already stored
nats request '$mq9.AI.MSG.FETCH.reply.a1b2c3' \
  '{"deliver": "earliest"}'
```

**Key mq9 features:** private mailbox as reply address, FETCH+ACK pull consumption, non-blocking async pattern.

---

## 8. Agent Capability Discovery

Agents register their capabilities via REGISTER. Other agents use DISCOVER to find appropriate agents by keyword or semantic similarity — no central config file, no manual address book. When a capability agent shuts down and calls UNREGISTER, it disappears from DISCOVER results.

```bash
# Capability agent: register at startup
nats request '$mq9.AI.AGENT.REGISTER' '{
  "name": "agent.code-review",
  "payload": "Accepts code review requests for Rust/Go/Python; returns findings as JSON"
}'

# Consumer agent: full-text search
nats request '$mq9.AI.AGENT.DISCOVER' '{
  "text": "code review",
  "limit": 10
}'

# Consumer agent: semantic vector search (understands natural language intent)
nats request '$mq9.AI.AGENT.DISCOVER' '{
  "semantic": "find an agent that can check my Rust code for bugs",
  "limit": 5
}'
# Returns matching agent list with name, mail_address, payload, etc.

# Consumer agent: send a task to the discovered agent's mailbox
nats request '$mq9.AI.MSG.SEND.agent.code-review' '{
  "file": "src/main.rs",
  "context": "performance review"
}'

# Capability agent: unregister at shutdown
nats request '$mq9.AI.AGENT.UNREGISTER' '{"name": "agent.code-review"}'
```

**Key mq9 features:** Agent register/discover, full-text + semantic vector search, decentralized capability registration.
