# RobustMQ NATS Overview

## What is NATS

NATS is a lightweight, high-performance open-source messaging system created by Derek Collison. It is a CNCF incubating project licensed under Apache 2.0. The design philosophy of NATS is radical simplicity: a pure text protocol over TCP, a handful of commands, a single binary with no external dependencies.

NATS positions itself as "connective technology" for communication across cloud, edge, and IoT environments. Its key characteristics are low latency (microsecond-range), high throughput (tens of millions of messages per second on a single node), and a minimal footprint (10–20 MB idle memory).

NATS has a two-layer architecture:

- **Core NATS**: Pure pub/sub with no durable persistence, at-most-once delivery, extreme performance
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

RobustMQ's NATS implementation uses the same storage infrastructure as MQTT,
Kafka, and AMQP, but protocol handlers still resolve topics by tenant, name, and
`TopicSource`. A NATS subject is not automatically the same logical topic as an
MQTT topic, Kafka topic, or AMQP queue. Cross-protocol access is possible only
when the handlers resolve a compatible mapping and the protocol semantics allow
it; there is no blanket bridge between every protocol.

The storage layer supports three engines, selected based on the scenario:

| Storage Engine | Characteristics | Use Cases |
|---------------|----------------|-----------|
| Memory | Pure in-memory, microsecond latency | NATS Core subject topics |
| RocksDB | Persistent, TTL auto-cleanup | mq9 subject topics and AMQP/MQTT topics where configured |
| File Segment | High throughput, sequential writes | Kafka topics and other topics configured for segment storage |

NATS Core subjects and mq9 mailboxes are logical Topics. In the current topic
creation path, each Topic receives a generated `storage_name_list`; the helper
that creates `$sys.inner.*` topics does not become the subject Topic's backing
pool because `with_partition(...)` regenerates the subject shard names.

### Supported Scope

| Feature | Status |
|---------|--------|
| Pub/Sub | ✅ Available |
| Request/Reply | ✅ Available |
| Queue Groups (competing consumers) | ✅ Available |
| Subject wildcards (`*` and `>`) | ✅ Available |
| Message Headers (HPUB/HMSG) | ✅ Available |
| Connection authentication | ✅ Username/password when enabled |
| TLS | ✅ Available |
| JetStream | 🚧 In development |

### mq9: AI Agent Communication Protocol Built on NATS

RobustMQ defines the `$mq9.AI.*` namespace on top of the NATS protocol, purpose-built for AI Agent asynchronous communication. mq9 is not a new protocol — it is a semantic convention defined on NATS subjects. All NATS clients work directly with it, no changes needed.

mq9's design follows the same pattern as JetStream: no new protocol commands, all operations via standard NATS pub/sub/req-reply. The server activates persistence and priority scheduling for messages matching the `$mq9.AI.*` prefix.

See [mq9 Overview](https://mq9.robustmq.com/docs/overview) for details.

---

## Compatibility With Native NATS

RobustMQ implements the currently supported NATS Core command subset: `INFO`,
`CONNECT`, `PUB`, `HPUB`, `SUB`, `UNSUB`, `MSG`, `HMSG`, `PING`, `PONG`, `+OK`,
and `-ERR`. Standard clients can connect for these operations, but compatibility
is not equivalent to the complete NATS Core feature set. In particular, token,
JWT, and NKey fields are parsed for wire compatibility but authentication is
currently performed through username/password checks when authentication is
enabled; JetStream is not implemented.

```bash
# Connect to RobustMQ using the official NATS CLI
nats pub "hello.world" "test message" --server nats://localhost:4222
nats sub "hello.world" --server nats://localhost:4222
```

---

## Position Within RobustMQ

RobustMQ is a multi-protocol unified messaging engine. NATS is one of its native protocols, sitting alongside MQTT, Kafka, and AMQP.

A shared storage layer makes compatible cross-protocol views possible, but the
mapping is explicit rather than automatic. For example, NATS Core subscription
discovery currently selects topics whose source is `NATS`; it does not treat all
MQTT, Kafka, or AMQP topics as NATS subjects. A deployment that needs a message
to cross protocols must use a topic/tenant mapping supported by both handlers.

This has practical value in edge and AI scenarios: IoT devices send data over MQTT, edge Agents coordinate tasks over NATS/mq9, analytics systems consume over Kafka — one broker, one storage layer, zero extra operational overhead.
