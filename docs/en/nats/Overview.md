# RobustMQ NATS Overview

## What is NATS

NATS is a lightweight, high-performance open-source messaging system created by Derek Collison. It is a CNCF incubating project licensed under Apache 2.0. The design philosophy of NATS is radical simplicity: a pure text protocol over TCP, a handful of commands, a single binary with no external dependencies.

NATS positions itself as "connective technology" for communication across cloud, edge, and IoT environments. Its key characteristics are low latency (microsecond-range), high throughput (tens of millions of messages per second on a single node), and a minimal footprint (10–20 MB idle memory).

NATS has a two-layer architecture:

- **Core NATS**: Pure pub/sub with no persistence, at-most-once delivery, extreme performance
- **JetStream**: Persistence layer with Streams, Consumers, and at-least-once / exactly-once delivery semantics

---

## Why RobustMQ Supports NATS

Among all the protocols supported by RobustMQ, NATS is the best fit for AI Agent and edge computing scenarios:

**Text protocol, AI-native.** NATS is a pure text protocol. `PUB`, `SUB`, and `MSG` are just a few simple text commands. AI Agents can generate and parse this format directly, no binary codec needed. An Agent can even interact with RobustMQ directly over a TCP connection with zero dependencies.

**Subject namespace, natural capability routing.** NATS subjects use a `.`-separated hierarchical naming structure with `*` and `>` wildcards, making it natural to organize and route Agent capabilities:

```
ai.agent.translation.en-to-zh    # Translation Agent
ai.agent.code.review              # Code Review Agent
ai.tool.search.web                # Web Search Tool
```

**Queue Groups, zero-config load balancing.** Multiple Agents with the same capability join the same Queue Group, and NATS automatically distributes requests among them — no load balancer needed.

**Request-Reply, native Agent interaction pattern.** The core interaction pattern of an AI Agent is "send a request, wait for a response." NATS Request-Reply is exactly this semantic.

**Edge computing, naturally suited.** Single binary, no dependencies, extremely low memory usage — NATS is the most suitable messaging middleware for edge devices. Combined with the mq9 mailbox semantics, it enables offline message buffering at the edge and priority-ordered delivery on reconnect.

---

## RobustMQ's NATS Implementation

### Unified Storage

RobustMQ's NATS implementation shares the unified storage layer with MQTT, Kafka, and AMQP. Messages written via NATS can be consumed directly by other protocols — no bridging or data copying required.

The storage layer supports three engines, selected based on the scenario:

| Storage Engine | Characteristics | Use Cases |
|---------------|----------------|-----------|
| Memory | Pure in-memory, microsecond latency | Core NATS real-time pub/sub |
| RocksDB | Persistent, TTL auto-cleanup | mq9 mailboxes, offline messages |
| File Segment | High throughput, sequential writes | High-volume log scenarios |

### Supported Scope

RobustMQ currently supports the complete **NATS Core** feature set, including:

- Pub/Sub
- Request/Reply
- Queue Groups (competing consumers)
- Subject wildcards (`*` and `>`)
- Message Headers (HPUB/HMSG)
- Connection authentication
- TLS

**JetStream** support will be evaluated based on future priorities. JetStream semantics (Streams, Consumers, offset management) overlap with RobustMQ's unified storage abstraction, and the integration path requires careful design.

### mq9: AI Agent Communication Protocol Built on NATS

RobustMQ defines the `$mq9.AI.*` namespace on top of the NATS protocol, purpose-built for AI Agent asynchronous communication. mq9 is not a new protocol — it is a semantic convention defined on NATS subjects. All NATS clients work directly with it, no changes needed.

mq9's design follows the same pattern as JetStream: no new protocol commands, all operations via standard NATS pub/sub/req-reply. The server activates persistence and priority scheduling for messages matching the `$mq9.AI.*` prefix.

See [mq9 Overview](/en/mq9/Overview) for details.

---

## Compatibility With Native NATS

RobustMQ implements the complete NATS Core protocol. Any standard NATS client (Go, Python, Rust, Java, JavaScript) can connect directly — no code changes required.

```bash
# Connect to RobustMQ using the official NATS CLI
nats pub "hello.world" "test message" --server nats://localhost:4222
nats sub "hello.world" --server nats://localhost:4222
```

---

## Position Within RobustMQ

RobustMQ is a multi-protocol unified messaging engine. NATS is one of its native protocols, sitting alongside MQTT, Kafka, and AMQP.

A message written via MQTT can be consumed via NATS; a message written via NATS can be consumed via Kafka. One storage layer underneath, protocols are just read/write interfaces — no bridging, no copying.

This has practical value in edge and AI scenarios: IoT devices send data over MQTT, edge Agents coordinate tasks over NATS/mq9, analytics systems consume over Kafka — one broker, one storage layer, zero extra operational overhead.
