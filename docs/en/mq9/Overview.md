# mq9 Overview

## What is mq9

mq9 is RobustMQ's protocol layer designed specifically for AI Agent communication. It sits alongside MQTT, Kafka, NATS, and AMQP, built natively on top of RobustMQ's unified storage architecture.

![img](../../images/mq9.jpg)

Today, when Agent A sends a message to Agent B and B is offline, the message is gone. There is no standard mechanism to guarantee "the message I sent will be there when the recipient comes back online." Every team building multi-Agent systems works around this with their own temporary solution — Redis pub/sub, database polling, homegrown task queues. They work, but they're all workarounds.

mq9 solves this directly: **send a message, the recipient gets it when they come online.**

Just like people have email — you send a message, the recipient reads it when they're available, and the message doesn't disappear. Agents need the same mechanism. mq9 is the infrastructure layer built for exactly this scenario.

---

## Positioning

mq9 is not a general-purpose message queue. It does not compete with or replace MQTT or Kafka. It is designed specifically for **AI Agent async communication**:

- Agents are ephemeral — they come and go with tasks, may only live for seconds, and cannot be assumed to be online
- Agent communication topologies are dynamic — publishers don't know who's listening, consumers don't know where messages come from
- Agent resources need lightweight lifecycle management — create on demand, auto-destroy on expiry, no manual cleanup

HTTP and A2A protocols solve synchronous calls — the caller waits, the recipient must be online now. mq9 solves async communication — send it, the recipient handles it whenever they're online. The two don't overlap and don't compete.

### Position within RobustMQ

mq9 is RobustMQ's fifth native protocol, sharing the same unified storage architecture as MQTT, Kafka, NATS, and AMQP. Deploy one RobustMQ instance — all five protocols are ready. IoT devices send data over MQTT, analytics systems consume over Kafka, Agents collaborate over mq9 — one broker, one storage layer, no bridging, no data copying.

### Position within the NATS ecosystem

mq9 sits between Core pub/sub and JetStream. Core NATS is too lightweight — no persistence, offline messages are lost, it can't implement a mailbox. JetStream is too heavy — streams, consumers, offsets, replay, a full Kafka-equivalent set of semantics that you don't need just to send a message between Agents. mq9 adds persistence, priority, and TTL auto-management on top of pub/sub, without introducing streams, consumer groups, or offsets.

---

## Core Concept: Mailbox

mq9 has a single core abstraction: **Mailbox (MAILBOX)**.

A mailbox is an Agent's communication address. Ephemeral — TTL drives the lifecycle, auto-destroyed on expiry. An Agent creates a mailbox for each task, gets a mail_id, and that's its communication address for that task. When the task ends, the mailbox auto-expires.

**mail_id unguessability is the security boundary.** mail_id is a system-generated unguessable string. Knowing the mail_id lets you send messages and subscribe. Without it, you can't interact with the mailbox. No token, no ACL.

Mailboxes come in two kinds:

| | Private mailbox | Public mailbox |
|---|---|---|
| mail_id | System-generated, unguessable | User-defined, meaningful name |
| Discoverability | Private — only Agents who know the mail_id can find it | Auto-registered to PUBLIC.LIST, discoverable by anyone |
| Use cases | Point-to-point messaging, task result delivery | Task queues, public channels, capability announcements |

A public mailbox's mail_id is its address — choose a meaningful name like `task.queue` or `analytics.result` rather than a UUID, making it easier for other Agents to discover and understand.

---

## Three Operations

mq9's entire protocol is three operations:

| Operation | Subject | Description |
|-----------|---------|-------------|
| Create mailbox | `$mq9.AI.MAILBOX.CREATE` | Create a private or public mailbox |
| Send message (default) | `$mq9.AI.MAILBOX.{mail_id}` | Default priority (normal), no suffix |
| Send message (urgent) | `$mq9.AI.MAILBOX.{mail_id}.urgent` | Urgent priority |
| Send message (critical) | `$mq9.AI.MAILBOX.{mail_id}.critical` | Highest priority |
| Subscribe (non-default) | `$mq9.AI.MAILBOX.{mail_id}.*` | Receive urgent and critical messages |

No QUERY — every subscription delivers all non-expired messages in full; subscribing is querying. No DELETE — TTL handles cleanup automatically.

**Three priority levels:**

| Level | Semantics | Typical use |
|-------|-----------|-------------|
| critical | Highest priority, processed first | Abort signals, emergency commands, security events |
| urgent | Urgent | Task interrupts, time-sensitive instructions |
| normal (default, no suffix) | Routine communication | Task dispatch, result delivery, approval requests |

---

## Public Mailbox Discovery

`$mq9.AI.PUBLIC.LIST` is a system-managed address maintained by the broker. It does not accept user writes and never expires.

Public mailboxes (`public: true`) are automatically registered on creation and removed when their TTL expires. No manual registry maintenance needed.

```bash
nats sub '$mq9.AI.PUBLIC.LIST'
```

Subscribing delivers all current public mailboxes immediately, then streams additions and removals in real time. No registry service needed — PUBLIC.LIST is the directory.

---

## Why Existing Solutions Fall Short

| Solution | Core Problem |
|----------|-------------|
| HTTP | Synchronous — the recipient must be online. Agents go offline constantly; this assumption breaks constantly. |
| Redis pub/sub | No persistence. Offline recipients don't get the message. |
| Kafka | Designed for high-volume data pipelines. Topic creation requires admin operations. Agents are throwaway; Kafka assumes long-lived, carefully maintained resources. |
| RabbitMQ | Flexible AMQP model, but performance ceiling and single-node architecture are hard limitations. |
| NATS Core | No persistence. Offline messages are lost. |
| JetStream | Stream, consumer, offset concepts are too heavy for mailbox semantics. |
| Homegrown queues | Every team rebuilds the same wheel; implementations are incompatible across teams. |

All of these share the same underlying flaw: **they assume the recipient is online, or require manual handling of the offline case.** Agents going offline frequently is a problem all existing solutions require you to work around — not a problem they solve.

---

## Eight Core Scenarios

| Scenario | Key advantage |
|----------|---------------|
| Sub-Agent reports task result to orchestrator | Orchestrator doesn't block; messages survive offline periods |
| Multiple Workers compete for a task queue | Public mailbox + queue group; competing consumers need zero config |
| Orchestrator tracks all sub-Agent states | Workers report to public mailbox; TTL expiry auto-signals death |
| Agent broadcasts an anomaly alert | Public mailbox collects alerts; offline handlers receive them after reconnecting |
| Cloud sends commands to offline edge Agent | Native priority; messages survive network outages; processed by priority on reconnect |
| Human-in-the-loop approval workflow | Humans and Agents use the same protocol; no separate approval service needed |
| Agent A asks Agent B a question, B may be offline | B offline doesn't lose the request; A doesn't block; async request-reply |
| Agent registers capabilities, others discover it | Public mailbox + PUBLIC.LIST; decentralized, no registry service |

---

## Design Principles

**No new SDK required**: mq9 is built on the NATS protocol. Any NATS client — Go, Python, Rust, Java, JavaScript — is already an mq9 client. No new dependencies. The `$mq9.AI.*` namespace makes the protocol self-documenting — reading a subject tells you its full semantics.

**mail_id is not tied to Agent identity**: mq9 recognizes mail_ids, not agent_ids. A mail_id is a communication channel. One Agent can hold multiple mail_ids for different tasks; they expire and clean up automatically. This is channel-level design, not identity-level.

**Store first, then push**: Messages are written to storage first, then pushed to online subscribers. Online subscribers take the real-time path; offline messages wait in storage and are delivered in full on the next subscription.

**No new concepts invented**: Subscriptions reuse NATS native sub semantics. Competing consumers reuse NATS native queue groups. Reply-to reuses NATS native mechanisms.

**Storage chosen per message**: Running on RobustMQ's unified storage layer with three tiers — Memory (coordination signals, disposable), RocksDB (ephemeral persistence, the mailbox default, TTL cleanup), File Segment (long-term, audit logs). Not every message needs three replicas; not every message can afford to be lost.

**Single node is enough, scale when needed**: A single instance covers most Agent communication workloads. One command to start, no cluster required. When high availability is needed, switch to cluster mode — the API is unchanged, Agents notice nothing.
