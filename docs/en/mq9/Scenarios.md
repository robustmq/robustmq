# Use Cases

mq9 is designed around eight concrete Agent communication patterns. Each pattern maps to a specific mq9 feature combination.

---

### 1. Sub-Agent Result Delivery

An orchestrator spawns a sub-agent for a long-running task and cannot block waiting for the result — it has other work to do. The sub-agent completes independently and deposits its result into a mailbox the orchestrator controls. Because mq9 uses store-first delivery, the result is waiting even if the orchestrator is busy or temporarily disconnected when the sub-agent finishes.

The orchestrator creates a private mailbox and shares the `mail_id` with the sub-agent at spawn time. No polling, no callback registration, no shared state — just a mailbox.

```bash
# Orchestrator: create private reply mailbox (TTL covers max expected task duration)
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl": 3600}'
# Response: {"mail_id": "mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag"}

# Share mail_id with sub-agent out-of-band (e.g. in the task payload)
nats pub '$mq9.AI.MAILBOX.MSG.m-task-dispatch.normal' \
  '{"task": "summarize /data/corpus", "reply_to": "mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag"}'

# Sub-agent: deliver result when done
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.normal' \
  '{"status": "ok", "summary": "..."}'

# Orchestrator: subscribe whenever ready — result is already stored
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.*'
```

**Key mq9 features:** private mailbox, store-first delivery, async result pickup.

---

### 2. Multi-Worker Task Queue

A producer sends tasks into a shared queue. Multiple workers compete to consume — each task must be processed exactly once. Workers can join or leave at any time without reconfiguration. This is a classical work queue, but with built-in crash tolerance: if a worker dies before acknowledging a task, the message remains in storage and is re-delivered to the next available worker.

```bash
# Create shared public mailbox once (idempotent — safe to call at every worker start)
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 86400,
  "public": true,
  "name": "task.queue",
  "desc": "Shared worker task queue"
}'

# Workers: subscribe with the same queue group — each message delivered to exactly one worker
# Terminal 1
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers
# Terminal 2
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers

# Producer: publish tasks at appropriate priorities
nats pub '$mq9.AI.MAILBOX.MSG.task.queue.critical' '{"task": "reindex", "id": "t-101"}'
nats pub '$mq9.AI.MAILBOX.MSG.task.queue.urgent'   '{"task": "interrupt", "id": "t-102"}'
nats pub '$mq9.AI.MAILBOX.MSG.task.queue'          '{"task": "summarize", "id": "t-103"}'
```

**Key mq9 features:** public mailbox, queue group (competitive consumption), priority ordering.

---

### 3. Worker Health Tracking via TTL

An orchestrator needs to know which workers are currently alive without polling them. Workers send periodic heartbeats by refreshing their mailbox. If a worker dies, its mailbox TTL expires and it disappears from `PUBLIC.LIST` automatically. The orchestrator subscribes to `PUBLIC.LIST` to track registrations and expirations in real time — no health-check endpoint, no external watchdog service.

```bash
# Each worker: create a short-TTL public mailbox at startup (name encodes identity)
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 30,
  "public": true,
  "name": "worker.health.worker-42",
  "desc": "Worker 42 heartbeat"
}'

# Worker: refresh by re-creating periodically (every ~20s to stay ahead of TTL)
# CREATE is idempotent — if mailbox still exists, returns success without resetting TTL.
# Let the mailbox expire and recreate it to simulate a live heartbeat update cycle.
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 30,
  "public": true,
  "name": "worker.health.worker-42"
}'

# Orchestrator: watch the public list for registrations and expirations
nats sub '$mq9.AI.PUBLIC.LIST'
```

**Key mq9 features:** TTL auto-cleanup, public mailbox, `PUBLIC.LIST` for discovery and expiration tracking.

---

### 4. Alert Broadcasting

Any agent can detect an anomaly and broadcast an alert to all registered handlers. Handlers may be offline when the alert fires — they still receive it when they reconnect, because mq9 writes to storage first. `critical` priority ensures alert messages are delivered before any backlog of lower-priority messages when a handler catches up.

```bash
# Alert sender: publish to a shared alert mailbox with highest priority
nats pub '$mq9.AI.MAILBOX.MSG.alerts.critical' '{
  "type": "anomaly",
  "agent": "monitor-7",
  "detail": "CPU > 95% for 5m",
  "ts": 1712600100
}'

# Handler A: subscribe — receives alert immediately or when it reconnects
nats sub '$mq9.AI.MAILBOX.MSG.alerts.*'

# Handler B: subscribes later and still gets the alert from storage
nats sub '$mq9.AI.MAILBOX.MSG.alerts.*'
```

**Key mq9 features:** store-first delivery (handlers receive alerts even if offline), critical priority, public mailbox.

---

### 5. Cloud-to-Edge Command Delivery

A cloud orchestrator needs to deliver commands to edge agents that may be offline for hours due to intermittent connectivity. When the edge agent reconnects, it must receive all pending commands in the correct priority order — `critical` abort or reconfigure commands before routine `normal` tasks. No message broker bridging or retry logic is required on the cloud side.

```bash
# Cloud: publish commands to edge agent's private mailbox (mail_id shared at provisioning)
# Critical-priority reconfiguration
nats pub '$mq9.AI.MAILBOX.MSG.m-edge-agent-9a3f.critical' '{
  "cmd": "reconfigure",
  "params": {"sampling_rate": 100}
}'

# Default-priority (normal) routine task
nats pub '$mq9.AI.MAILBOX.MSG.m-edge-agent-9a3f' '{
  "cmd": "run_diagnostic",
  "target": "sensor-bank-2"
}'

# Edge agent: subscribe on reconnect — receives all stored commands in priority order
nats sub '$mq9.AI.MAILBOX.MSG.m-edge-agent-9a3f.*'
```

**Key mq9 features:** offline delivery (store-first), priority ordering on reconnect, private mailbox.

---

### 6. Human-in-the-Loop Approval Workflow

An agent generates a decision that requires human review before it proceeds — for example, before modifying a production database or sending a communication on behalf of a user. The human interacts using the exact same mq9 protocol as any other agent. No separate approval service, webhook infrastructure, or out-of-band communication channel is needed. The agent suspends and resumes based entirely on mailbox messages.

```python
import nats
import asyncio, json

async def run():
    nc = await nats.connect("nats://demo.robustmq.com:4222")

    # Agent: create private reply mailbox for the approval response
    reply = await nc.request("$mq9.AI.MAILBOX.CREATE", b'{"ttl": 7200}')
    reply_id = json.loads(reply.data)["mail_id"]

    # Agent: publish decision for human review
    await nc.publish(
        f"$mq9.AI.MAILBOX.MSG.approvals.normal",
        json.dumps({
            "action": "delete_dataset",
            "target": "ds-prod-2024",
            "reply_to": reply_id
        }).encode()
    )

    # Human (via any NATS client or UI): subscribes to approvals, reviews, publishes decision
    # nats sub '$mq9.AI.MAILBOX.MSG.approvals.*'
    # nats pub '$mq9.AI.MAILBOX.MSG.<reply_id>.normal' '{"approved": true, "reviewer": "alice"}'

    # Agent: subscribe to reply mailbox when ready to continue
    sub = await nc.subscribe(f"$mq9.AI.MAILBOX.MSG.{reply_id}.*")
    msg = await sub.next_msg(timeout=7200)
    decision = json.loads(msg.data)
    print("Approval decision:", decision)

asyncio.run(run())
```

**Key mq9 features:** same protocol for human and agent interaction, async, store-first delivery.

---

### 7. Async Request-Reply

Agent A needs a result from Agent B, but B may not be available immediately and A cannot afford to block. A creates a private reply mailbox, embeds the `mail_id` in the request as a `reply_to` field, and continues other work. B processes the request at its own pace and sends the result to A's reply mailbox. A subscribes to the reply mailbox whenever it is ready to consume the response.

```bash
# Agent A: create private reply mailbox
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl": 600}'
# Response: {"mail_id": "m-reply-a1b2c3"}

# Agent A: send request to Agent B's mailbox with reply_to field
nats pub '$mq9.AI.MAILBOX.MSG.m-agent-b-inbox.normal' '{
  "request": "translate",
  "text": "Hello world",
  "lang": "fr",
  "reply_to": "m-reply-a1b2c3"
}'

# Agent A: continues other work here ...

# Agent B: processes request and sends result to reply mailbox
nats pub '$mq9.AI.MAILBOX.MSG.m-reply-a1b2c3.normal' '{
  "result": "Bonjour le monde"
}'

# Agent A: subscribes to reply mailbox when ready — result already stored
nats sub '$mq9.AI.MAILBOX.MSG.m-reply-a1b2c3.*'
```

**Key mq9 features:** private mailbox as reply address, store-first delivery, non-blocking async pattern.

---

### 8. Agent Capability Discovery

Agents announce their capabilities by creating public mailboxes with descriptive, structured names. Other agents subscribe to `PUBLIC.LIST` to discover what capabilities are available at any moment — without a central registry, service mesh, or configuration file. When a capability agent shuts down, its mailbox TTL expires and it disappears from the list automatically.

```bash
# Capability agent: register itself by creating a named public mailbox
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 3600,
  "public": true,
  "name": "agent.code-review",
  "desc": "Accepts code review requests; returns findings as JSON"
}'

# Another agent: subscribe to PUBLIC.LIST to discover available capabilities
nats sub '$mq9.AI.PUBLIC.LIST'
# Receives entries like: {"name": "agent.code-review", "desc": "...", "mail_id": "agent.code-review"}

# Consumer agent: send a task directly to the discovered capability
nats pub '$mq9.AI.MAILBOX.MSG.agent.code-review.normal' '{
  "file": "src/main.rs",
  "context": "performance review"
}'
```

**Key mq9 features:** public mailbox with descriptive name, `PUBLIC.LIST` for real-time discovery, decentralized capability registration.
