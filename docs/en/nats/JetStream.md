# JetStream

> 🚧 **In development** — JetStream support is actively being implemented. This document covers the core concepts and RobustMQ's support plan.

## What is JetStream

JetStream is the persistence layer for NATS. It builds on top of Core NATS pub/sub by adding message storage, consumer management, and delivery guarantees. Core NATS is at-most-once — if a message is lost, it's gone. JetStream is at-least-once or exactly-once.

The key difference:

| | Core NATS | JetStream |
|-|-----------|-----------|
| Persistence | No — pure in-memory | Yes — messages written to a Stream |
| Delivery semantics | at-most-once | at-least-once / exactly-once |
| Consumer state | None | Yes (Consumer tracks progress) |
| Offline messages | Lost | Retained, replayed on reconnect |
| Replay | Not supported | Supported (from any offset) |
| Best for | Real-time push, low latency | Event sourcing, audit logs, reliable delivery |

---

## Core Concepts

### Stream

A Stream is JetStream's storage unit. Each Stream binds to a set of subjects — any message published to those subjects is persisted into the Stream.

```bash
# Create a Stream binding all messages under orders.>
nats stream add ORDERS \
  --subjects "orders.>" \
  --storage file \
  --retention limits \
  --max-age 24h
```

Key Stream configuration:

| Option | Description |
|--------|-------------|
| `subjects` | Bound subjects, wildcards supported |
| `storage` | `file` (persistent) or `memory` |
| `retention` | `limits` (by size/age), `interest` (while consumers exist), `workqueue` (delete after consume) |
| `max-age` | Maximum message retention duration |
| `max-msgs` | Maximum message count in Stream |
| `max-bytes` | Maximum bytes in Stream |

### Consumer

A Consumer is a view into a Stream that tracks a subscriber's progress. Each Consumer independently remembers where it left off.

```bash
# Create a Push Consumer (server pushes to a subject)
nats consumer add ORDERS payments-service \
  --filter "orders.created" \
  --deliver all \
  --ack explicit

# Create a Pull Consumer (client fetches on demand)
nats consumer add ORDERS audit-log \
  --filter "orders.>" \
  --deliver all \
  --pull
```

Consumer types:

| Type | Description |
|------|-------------|
| **Push Consumer** | Server pushes messages to a subject — similar to a subscription |
| **Pull Consumer** | Client fetches messages on demand — better for batch processing and flow control |

Deliver policy (starting position):

| Option | Description |
|--------|-------------|
| `all` | Start from the oldest message in the Stream |
| `new` | Only receive messages published after the Consumer is created |
| `last` | Start from the most recent message |
| `by_start_time` | Start from a specific timestamp |
| `by_start_sequence` | Start from a specific sequence number |

### ACK Mechanism

JetStream requires explicit ACKs from consumers. Unacknowledged messages are redelivered after a timeout.

```
Ack         # Positive acknowledge — message processed
Nak         # Negative acknowledge — redeliver immediately
InProgress  # Still processing — reset the ACK timeout
Term        # Terminate — do not redeliver
```

---

## Relationship to Core NATS and mq9

RobustMQ supports three NATS usage modes, each with a distinct purpose:

| | Core NATS | JetStream | mq9 |
|-|-----------|-----------|-----|
| **Persistence** | No | Yes | Yes |
| **Consumer state** | None | Yes (offset) | None (store-first push) |
| **Priority** | None | None | Three levels (critical/urgent/normal) |
| **TTL** | None | Per Stream config | Per mailbox config |
| **Best for** | Real-time push, low latency | Event streams, audit, reliable delivery | AI Agent async communication |
| **Target users** | General pub/sub | Data pipelines, microservices | AI Agents |

All three can run on the same RobustMQ instance — choose based on your needs:

- Extreme low latency, occasional message loss acceptable → **Core NATS**
- No message loss, replay support, consumer progress tracking → **JetStream**
- AI Agent async communication, offline delivery, priority ordering → **mq9**

---

## Current Status

JetStream is actively being developed. Completed:

- Stream and Consumer concept design and storage layer mapping
- Pull Consumer basic functionality

In progress:

- Push Consumer
- ACK and redelivery mechanism
- Exactly-once semantics
- Stream management API (NATS CLI compatible)

Follow [GitHub Milestones](https://github.com/robustmq/robustmq/milestones) for progress updates.
