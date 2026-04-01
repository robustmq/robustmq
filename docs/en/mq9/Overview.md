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

mq9 sits between Core pub/sub and JetStream. Core NATS is too lightweight — no persistence, offline messages are lost, it can't be a mailbox. JetStream is too heavy — streams, consumers, offsets, replay, a full Kafka-equivalent set of semantics that you don't need just to send a message between Agents. mq9 adds optional persistence, priority, and TTL lifecycle management on top of pub/sub, without introducing streams, consumer groups, or offsets.

---

## Three Core Primitives

### Mailbox

Every Agent has its own mailbox. The sender doesn't need to know if the recipient is online — just send to their mailbox, and the message waits. The recipient gets it in real time via subscription when online, or pulls it via QUERY as a fallback.

Two mailbox types:
- `standard`: messages accumulate, stored by priority. For task instructions, request-reply, offline buffering.
- `latest`: only the most recent message is kept — new messages overwrite old ones. For status reporting, capability declarations, heartbeats.

Mailboxes do not need explicit deletion. TTL expiry triggers automatic destruction, and messages are cleaned up with the mailbox.

### Broadcast

An Agent broadcasts an event without knowing who is listening. Interested Agents subscribe themselves. New Agents joining the system automatically receive future events — topology changes are zero-cost. Broadcast requires no CREATE; publish and subscribe directly.

Broadcast supports wildcard subscriptions: by domain, by event type, or subscribe to everything. Queue groups enable competing consumers — broadcast a task, multiple Workers subscribe, exactly one claims it.

### Priority

Mailboxes support three priority levels: `urgent`, `normal`, `notify`. Higher-priority messages are processed first. FIFO is guaranteed within each level. Critical instructions are never buried under routine work.

| Priority | Description | Persisted by default | Suggested TTL |
|----------|-------------|---------------------|---------------|
| `urgent` | Critical messages, processed first | true | 86400s |
| `normal` | Routine messages, FIFO | true | 3600s |
| `notify` | Informational, background processing | false | — |

---

## Why Existing Solutions Fall Short

| Solution | Core Problem |
|----------|-------------|
| HTTP | Synchronous — the recipient must be online. Agents go offline constantly; this assumption breaks constantly. |
| Redis pub/sub | No persistence. Offline recipients don't get the message. Guaranteeing delivery requires hand-rolling a reliable queue on top of Redis. |
| Kafka | Designed for high-volume data pipelines. No ephemeral mailbox concept, no priority queue, topic creation requires admin operations. Agents are throwaway; Kafka assumes long-lived, carefully maintained resources. |
| RabbitMQ | Flexible AMQP model, but performance ceiling and single-node architecture are hard limitations. Community focus is not on Agent scenarios. |
| NATS Core | No persistence. Offline messages are lost. |
| JetStream | Stream, consumer, offset concepts are too heavy for mailbox semantics. |
| Homegrown queues | Every team rebuilds the same wheel, every implementation is different, cross-team interop is impossible. |

All of these share the same underlying flaw: **they assume the recipient is online, or require manual handling of the offline case.** Agents going offline frequently is a problem all existing solutions require you to work around — not a problem they solve.

---

## Eight Core Scenarios

Four commands, combined, cover the essential communication patterns in Agent systems.

| Scenario | Protocol combination | Key advantage |
|----------|---------------------|---------------|
| Sub-Agent reports task result to orchestrator | MAILBOX.CREATE + INBOX.normal | Orchestrator doesn't block; messages survive offline periods |
| Orchestrator tracks all sub-Agent states | `latest` mailbox + wildcard subscription | New Agents auto-discovered; TTL expiry auto-signals death; zero maintenance |
| Task broadcast with competing Workers | BROADCAST + queue group | Competing consumers need zero config, no rebalancing, dynamic Worker scaling |
| Agent broadcasts an anomaly alert | BROADCAST + optional persist | Publisher needs no subscriber list; critical alerts can be persisted |
| Cloud sends commands to offline edge Agent | INBOX urgent/normal + QUERY fallback | Native priority; messages survive network outages; processed by priority on reconnect |
| Human-in-the-loop approval workflow | INBOX bidirectional + correlation_id | Humans and Agents use the same protocol; no separate approval service needed |
| Agent A asks Agent B a question, B may be offline | INBOX + reply_to + correlation_id | B offline doesn't lose the request; A doesn't block; async request-reply |
| Agent registers capabilities, others discover it | `latest` mailbox + BROADCAST.capability.query | Decentralized; no registry service; TTL auto-cleanup |

---

## Design Principles

**No new SDK required**: mq9 is built on the NATS protocol. Any NATS client — Go, Python, Rust, Java, JavaScript — is already an mq9 client. No new dependencies. The `$mq9.AI.*` namespace design makes the protocol self-documenting — reading a subject tells you its full semantics.

**mail_id is not tied to Agent identity**: mq9 recognizes mail_ids, not agent_ids. A mail_id is a communication channel. One Agent can hold multiple mail_ids for different tasks; they expire and clean up automatically. This is channel-level design, not identity-level.

**Store first, then push**: Messages are written to storage first, then pushed to online subscribers. Push is the fast path; storage is the fallback. MAILBOX.QUERY provides a final safety net for any missed pushes.

**No new concepts invented**: ACK reuses NATS native reply-to. Subscriptions reuse NATS native sub semantics. Competing consumers reuse NATS native queue groups.

**Storage chosen per message**: Running on RobustMQ's unified storage layer with three tiers — Memory (coordination signals, disposable), RocksDB (ephemeral persistence, the mailbox default, TTL cleanup), File Segment (long-term, audit logs). Not every message needs three replicas; not every message can afford to be lost.

**Single node is enough, scale when needed**: A single instance covers most Agent communication workloads. One command to start, no cluster required. When high availability is needed, switch to cluster mode — the API is unchanged, Agents notice nothing.
