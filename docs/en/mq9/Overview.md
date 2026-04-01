# mq9 Overview

## What is mq9

mq9 is RobustMQ's fifth native protocol layer, designed specifically for AI Agent communication.

![img](../../images/mq9.jpg)

Today, when Agent A sends a message to Agent B and B is offline, the message is gone. Every team works around this with Redis pub/sub, database polling, or homegrown queues. It works, but it's a workaround.

mq9 solves it directly: **send a message, the recipient gets it when they come online.**

Just like people have email — you send a message, the recipient reads it when they're available, and the message doesn't disappear. Agents need the same mechanism.

## Positioning

mq9 is not a general-purpose message queue, nor a replacement for MQTT or Kafka. It is designed specifically for **AI Agent async communication**:

- Agents need to communicate, but cannot guarantee both sides are online simultaneously
- Multi-Agent systems need to observe the state and events of other Agents
- Different tasks have different priorities — critical instructions must not be buried under routine work

mq9 is RobustMQ's fifth native protocol, sitting alongside MQTT, Kafka, NATS, and AMQP on the same unified storage layer. Deploy one RobustMQ instance — all five protocols are ready.

## Three Core Primitives

### Mailbox

Every Agent has its own inbox. The sender doesn't need to know if the recipient is online — just send to their mailbox, and the message waits. The Agent receives it when it comes back online.

Use cases: sub-Agent notifying the orchestrator of task completion, cloud sending commands to edge devices, approval requests in human-in-the-loop workflows.

### Broadcast

An Agent broadcasts an event without knowing who is listening. Interested Agents subscribe themselves. New Agents joining the system automatically receive future events — topology changes are zero-cost.

Use cases: orchestrator publishing tasks for Workers to claim, system anomaly notifications, Agent lifecycle state synchronization.

### Priority

Mailboxes support three priority levels: `urgent`, `normal`, `notify`. Critical instructions are never buried under routine tasks. FIFO is guaranteed within each level.

- `urgent`: highest priority, processed first when the Agent comes online
- `normal`: routine tasks
- `notify`: informational messages, no persistence guarantee

## Design Principles

**No new SDK required**: mq9 is built on the NATS protocol. Any NATS client — Go, Python, Rust, Java, JavaScript — is already an mq9 client. No new dependencies needed.

**Unified storage**: mq9 messages share the same storage engine as MQTT, Kafka, and other protocol messages. Data is written once and consumed by any protocol.

**Single binary**: No separate mq9 service to deploy. mq9 support is included when RobustMQ starts.

## Use Cases

| Scenario | Description |
|----------|-------------|
| Sub-Agent notifies Orchestrator | Sub-Agent sends results to the orchestrator's mailbox. The orchestrator picks up results when ready — no blocking. |
| Monitor all Agent states | One wildcard subscription automatically tracks every sub-Agent coming online, running, or dying. No registration needed. |
| Task broadcast with competing consumers | Orchestrator broadcasts a task, capable Workers compete to claim it. Queue group ensures each task is handled by exactly one Worker. |
| Edge Agent offline buffering | Cloud sends instructions to an edge Agent that's offline. Messages wait, then are processed by priority when the edge reconnects. |
| Human-in-the-loop workflows | Agent hits a decision point requiring human judgment, sends to a human's inbox, and resumes when the approval comes back. |
| Cross-system event triggers | An event in one system broadcasts to trigger Agent responses in another system. |
