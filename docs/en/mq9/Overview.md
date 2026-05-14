# mq9 Overview

## What is mq9

mq9 is RobustMQ's protocol layer designed specifically for AI Agent communication. It sits alongside MQTT, Kafka, NATS, and AMQP, built natively on top of RobustMQ's unified storage architecture.

![img](../../images/mq9.jpg)

### What problem it solves

In multi-Agent systems, Agents are not servers — they are task-driven, they start, execute, and die, coming online and offline at any time. When Agent A sends a message to Agent B and B is offline, the message is gone. Every team building multi-Agent systems works around this with their own temporary solution:

- **Redis pub/sub**: No persistence — messages are lost if the recipient is offline
- **Kafka**: Topics require advance creation and maintenance; not designed for throwaway Agents
- **Homegrown queues**: Every team rebuilds the same thing; Agent implementations are incompatible across teams

These approaches work, but they're all workarounds — **offline delivery is treated as a boundary condition handled manually, not a guarantee provided by the infrastructure.**

mq9 solves it directly: **send a message, the recipient gets it when they come online.** Just like email — you send it, the recipient reads it whenever they're available, and the message doesn't disappear.

A system today might have dozens of Agents; tomorrow it might have millions. mq9 is designed for that scale from the start: mailboxes created on demand, TTL auto-destruction, horizontally scalable Broker. From the first Agent to millions — the same API, the same operational model.

### Current capabilities

Already available: mailbox lifecycle (TTL auto-destruction), three-tier priority messages (critical / urgent / normal), pull consumption (FETCH + ACK with offset tracking), message attributes (key dedup, tags filtering, delay delivery, per-message TTL), Agent registry with full-text and semantic vector search, Python SDK, LangChain/LangGraph toolkit, and MCP Server.

### Future direction

The mailbox solves "messages get delivered." But as Agent networks mature, more is needed: intent routing (messages automatically find the best recipient), policy interception (the transport layer understands semantics and enforces access control), and context awareness (conversation history travels with messages, reducing repeated token transmission).

These directions are mq9's evolution roadmap — see [Roadmap](./Roadmap.md). The thinking behind them: [What should a messaging system look like in the AI era](../Blogs/82.md).

---

## Positioning

mq9 is not a general-purpose message queue. It does not compete with or replace MQTT or Kafka. It is designed specifically for **AI Agent async communication**. HTTP and A2A protocols solve synchronous calls — the caller waits, the recipient must be online. mq9 solves async communication — send it, the recipient handles it whenever they're online. The two don't overlap and don't compete.

### Position within RobustMQ

mq9 is RobustMQ's fifth native protocol, sharing the same unified storage architecture as MQTT, Kafka, NATS, and AMQP. Deploy one RobustMQ instance — all capabilities are ready. IoT devices send data over MQTT, analytics systems consume over Kafka, Agents collaborate over mq9 — one broker, one storage layer, no bridging, no data copying.

### Position within the NATS ecosystem

mq9 is built on top of the NATS protocol, but NATS is only the transport layer — the communication protocol between client and Broker, just like HTTP is the transport protocol for the Web. mq9's Broker is implemented by RobustMQ entirely in Rust; storage, priority scheduling, TTL management, and pull consumption semantics are all RobustMQ's own capabilities, with no relation to NATS Server.

The choice of NATS is pragmatic: NATS has official and community clients covering 40+ languages; Python, Go, JavaScript, and Rust — the most common languages in AI — all have mature implementations. mq9 is ready out of the box for developers in all these languages from day one, with no need to wait for SDK coverage. NATS request/reply primitives cover all the communication patterns mq9 needs.

In semantic terms, mq9 sits between NATS Core and JetStream but is optimized for Agent workloads: pull consumption + ACK offset tracking, three-tier priority scheduling, message attributes (key/tags/delay/ttl), and a built-in Agent registry. These are mq9-specific capabilities with no JetStream equivalent.

---

## Core Concept: Mailbox

mq9 has a single core abstraction: **Mailbox (MAILBOX)**.

Why a mailbox? Because mq9 treats Agents like people. The most natural async communication between people is email — you write it, send it, the recipient reads it whenever they're available; you don't have to wait, and the message doesn't disappear. Agent communication is fundamentally the same scenario: send it, the recipient gets it when they come online. Mailbox is the most intuitive mapping.

Following that analogy:

- **Address**: Every mailbox has a `mail_address` — its communication address. The address is specified at creation (e.g. `task.queue`). Unguessability is the security boundary — knowing the `mail_address` lets you send and receive; without it, there's no way to interact.

- **Letters**: Messages sent to a mailbox can carry attributes — priority (critical / urgent / normal) via `mq9-priority` header; dedup key via `mq9-key`; filter tags via `mq9-tags`; delayed delivery via `mq9-delay`; per-message TTL via `mq9-ttl`.

- **Pickup**: Clients actively FETCH messages and ACK to advance the consumption offset. The next FETCH resumes from the last ACK — no duplicate processing. Passing a `group_name` enables stateful consumption (broker tracks offset); omitting it enables stateless consumption (each FETCH is independent).

- **Mailbox lifetime**: Mailboxes declare a TTL at creation; they auto-destroy on expiry, taking all pending messages with them. No manual cleanup needed — forget about it when the task ends, the system handles it.

---

## Operations at a Glance

| Operation | Subject | Description |
|-----------|---------|-------------|
| Create mailbox | `$mq9.AI.MAILBOX.CREATE` | Create a mailbox; name is user-defined, ttl declares lifecycle |
| Send message | `$mq9.AI.MSG.SEND.{mail_address}` | Priority specified via `mq9-priority` header |
| Fetch messages | `$mq9.AI.MSG.FETCH.{mail_address}` | Pull mode; supports stateful and stateless consumption |
| ACK message | `$mq9.AI.MSG.ACK.{mail_address}` | Advance consumer group offset; enables resume-from-offset |
| Query messages | `$mq9.AI.MSG.QUERY.{mail_address}` | Query by key/tags/since; does not affect offset |
| Delete message | `$mq9.AI.MSG.DELETE.{mail_address}.{msg_id}` | Delete a specific message |
| Register Agent | `$mq9.AI.AGENT.REGISTER` | Register Agent with capability description |
| Unregister Agent | `$mq9.AI.AGENT.UNREGISTER` | Unregister Agent |
| Report status | `$mq9.AI.AGENT.REPORT` | Agent heartbeat / status reporting |
| Discover Agents | `$mq9.AI.AGENT.DISCOVER` | Full-text or semantic vector search |

**Three priority levels:**

| Level | Header value | Typical use |
|-------|-------------|-------------|
| `critical` (highest) | `mq9-priority: critical` | Abort signals, emergency commands, security events |
| `urgent` | `mq9-priority: urgent` | Approval requests, time-sensitive notifications |
| `normal` (default) | omit header | Task dispatch, result delivery, routine communication |

---

## Design Principles

**Pull consumption + ACK**: Clients actively FETCH messages and ACK to advance the consumption offset. Messages are not lost when consumers are temporarily offline — on reconnect, FETCH resumes from the last ACK.

**mail_address is not tied to Agent identity**: mq9 recognizes `mail_address`, not `agent_id`. One Agent can create different mailboxes for different tasks, leave them alone when done, and TTL handles cleanup automatically. Channel-level design, not identity-level.

**No new concepts invented**: Request/reply reuses NATS native semantics. Offset tracking is analogous to Kafka consumer groups. Message attributes are transmitted via NATS headers. No proprietary transport format.

**Broker is fully self-developed**: NATS is only the transport protocol. Storage, priority scheduling, TTL management, consumption offsets, and Agent registry are all implemented by RobustMQ in Rust, running on RobustMQ's unified storage layer.

**Single node is enough, scale when needed**: A single instance covers most workloads, started with one command. When high availability is needed, switch to cluster mode — the API is unchanged, Agents notice nothing.
