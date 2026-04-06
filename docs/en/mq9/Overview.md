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

## Three Commands

mq9's entire protocol has just three commands:

| Command | Description |
|---------|-------------|
| `MAILBOX.CREATE` | Create a mailbox or broadcast channel. INBOX returns mail_id + token; BROADCAST returns success |
| `INBOX` | Point-to-point message delivery, persisted, three priority levels (urgent / normal / notify) |
| `BROADCAST` | Public broadcast, persisted, anyone can publish or subscribe, supports wildcards and queue groups |

No QUERY — every subscription delivers all non-expired messages in full; subscribing is querying. No DELETE — TTL handles cleanup automatically.

---

## Two Core Abstractions

### Mailbox (INBOX)

Private. Has mail_id and token; anyone can send to it, but only the owner can receive. Declare TTL at creation; auto-destroyed on expiry.

Mailboxes do not need explicit deletion. TTL expiry triggers automatic destruction, and messages are cleaned up with the mailbox.

### Broadcast Channel (BROADCAST)

Public. No mail_id, no token; anyone can create one, anyone can publish to it, anyone can subscribe. The creator does not own it exclusively.

Broadcast channels are also persisted and have TTL. Offline subscribers receive all non-expired messages upon reconnection.

### System Bulletin Board

mq9 has three built-in permanent broadcast channels, automatically created at system startup, TTL never expires, cannot be deleted:

| Bulletin board | Purpose |
|----------------|---------|
| `$mq9.AI.BROADCAST.system.capability` | Capability announcement — what I can do |
| `$mq9.AI.BROADCAST.system.status` | Status announcement — I'm alive, current load |
| `$mq9.AI.BROADCAST.system.channel` | Channel announcement — I created a new broadcast channel |

Once an Agent comes online and subscribes to `$mq9.AI.BROADCAST.system.*`, it can discover the entire network. No registry service needed.

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

| Scenario | Protocol combination | Key advantage |
|----------|---------------------|---------------|
| Sub-Agent reports task result to orchestrator | MAILBOX.CREATE + INBOX.normal | Orchestrator doesn't block; messages survive offline periods |
| Orchestrator tracks all sub-Agent states | BROADCAST + wildcard subscription | New Agents auto-discovered; TTL expiry auto-signals death; zero maintenance |
| Task broadcast with competing Workers | BROADCAST + queue group | Competing consumers need zero config, no rebalancing, dynamic Worker scaling |
| Agent broadcasts an anomaly alert | BROADCAST + persistence | Publisher needs no subscriber list; offline handlers receive alerts after reconnecting |
| Cloud sends commands to offline edge Agent | INBOX urgent/normal | Native priority; messages survive network outages; processed by priority on reconnect |
| Human-in-the-loop approval workflow | INBOX bidirectional + correlation_id | Humans and Agents use the same protocol; no separate approval service needed |
| Agent A asks Agent B a question, B may be offline | INBOX + reply_to + correlation_id | B offline doesn't lose the request; A doesn't block; async request-reply |
| Agent registers capabilities, others discover it | BROADCAST.system.capability | Decentralized; no registry service; TTL auto-cleanup |

---

## Design Principles

**No new SDK required**: mq9 is built on the NATS protocol. Any NATS client — Go, Python, Rust, Java, JavaScript — is already an mq9 client. No new dependencies. The `$mq9.AI.*` namespace design makes the protocol self-documenting — reading a subject tells you its full semantics.

**mail_id is not tied to Agent identity**: mq9 recognizes mail_ids, not agent_ids. A mail_id is a communication channel. One Agent can hold multiple mail_ids for different tasks; they expire and clean up automatically. This is channel-level design, not identity-level.

**Store first, then push**: Messages are written to storage first, then pushed to online subscribers. Online subscribers take the real-time path; offline messages wait in storage and are delivered in full on the next subscription.

**No new concepts invented**: ACK reuses NATS native reply-to. Subscriptions reuse NATS native sub semantics. Competing consumers reuse NATS native queue groups.

**Storage chosen per message**: Running on RobustMQ's unified storage layer with three tiers — Memory (coordination signals, disposable), RocksDB (ephemeral persistence, the mailbox default, TTL cleanup), File Segment (long-term, audit logs). Not every message needs three replicas; not every message can afford to be lost.

**Single node is enough, scale when needed**: A single instance covers most Agent communication workloads. One command to start, no cluster required. When high availability is needed, switch to cluster mode — the API is unchanged, Agents notice nothing.
