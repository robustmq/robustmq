# mq9 Roadmap

mq9's core communication layer and semantic discovery are fully in place. This page describes the next three phases — progressively moving mq9 from message delivery infrastructure toward intelligent, context-aware Agent communication infrastructure.

The phases are not strictly sequential. Priorities will shift based on feedback from real-world use cases. The direction is fixed; the order is flexible.

---

## Where We Are Today

Both the core communication layer and semantic discovery layer are in place:

- **Mailbox lifecycle** — TTL-driven expiration, auto-destroyed with no manual cleanup
- **Three-priority messaging** — `critical` / `urgent` / `normal`, message persistence, offline-tolerant
- **Pull consumption + ACK** — FETCH to pull, ACK to advance offset, supports resume-from-offset
- **Message attributes** — key deduplication, tags filtering, delayed delivery, per-message TTL
- **Agent registry and discovery** — REGISTER / UNREGISTER / DISCOVER, with full-text and semantic vector search
- **Six-language SDK** — Python, Go, JavaScript, Java, Rust, C# (Python fully implemented, others scaffolded)
- **LangChain & LangGraph integration** — `langchain-mq9` toolkit with 6 tools
- **MCP Server support** — AI ecosystem integration via JSON-RPC 2.0

---

## Phase 1: Semantic Routing

**Goal:** Senders describe intent; the broker finds the recipient.

DISCOVER today is pull-based — consumers query, choose a target address, and send explicitly. The next step is push-based — the sender describes what it needs done, and mq9 routes automatically to the most suitable registered Agent.

**What changes:**

- Messages can optionally omit the `mail_address` and instead carry a semantic intent description
- The broker vector-matches the message intent against registered Agent capability descriptions
- Routing to the best-matching Agent happens inside the broker, transparently to the sender

**What this enables:**

An Agent with a legal analysis task no longer needs to know which other Agent handles legal questions. It publishes "analyze this contract for compliance issues" and mq9 routes the task to the most capable registered Agent. The broker evolves from a "post office" into an "intelligent dispatcher."

This direction is under exploration — implementation details depend on real-world routing workload validation.

---

## Phase 2: Security, Audit, and Access Control

**Goal:** Infrastructure-level safety boundaries for AI Agent communication.

Traditional message brokers are mindless relays — they do not understand message content and do not judge whether a message should be delivered. For AI Agents, this is insufficient: a compromised or misconfigured Agent can issue instructions like "delete the production database" or "transfer funds," and the broker faithfully delivers them.

**What changes:**

- **Access control** — per-`mail_address` send/subscribe permissions, supporting token or ACL rules beyond the current "address is the credential" model
- **Content policy** — policy rules configurable per mailbox (or globally); messages are evaluated against semantic content as they transit the policy engine — not just headers or metadata; messages that violate policy are blocked before delivery
- **Audit logging** — send, fetch, ACK, and delete events for every message can be recorded, satisfying traceability requirements for compliance scenarios
- **Permission management** — tenant-level isolation; administrators can configure and revoke permissions for mailboxes and the Agent registry

**The multi-protocol advantage:**

When a message is blocked, RobustMQ does not need a separate system to record the event. The policy engine writes blocked messages and audit events to a built-in risk topic. Risk analysis systems can consume this via the Kafka protocol — same broker, same storage, no additional infrastructure, no data crossing system boundaries.

---

## Phase 3: Context Awareness (Exploratory)

**Goal:** The broker carries conversation context; Agents stop retransmitting history.

Every interaction between AI Agents today includes redundant context retransmission: "Hello, I'm A, we previously discussed X, and now I need you to help me do Y." This consumes tokens and adds latency. The more complex the Agent collaboration, the worse this gets.

**What this would mean:**

- The broker becomes session-aware — it tracks conversation history between Agent pairs
- As messages flow, the broker automatically attaches relevant historical context based on the session
- Agent A sends "do Y"; the broker, knowing the prior exchange between A and B, enriches the message with the necessary context before delivery

**What this enables:**

Agents no longer need to retransmit the full context in every interaction. Token consumption drops. Agent collaboration becomes more efficient. The infrastructure evolves from a "stateless pipeline" into a "stateful context network."

This is the longest-horizon direction. The exact shape of session-awareness at the infrastructure layer is an open research problem. We believe the direction is correct; there is no definitive implementation plan yet.

---

## SDK Completion

In parallel with the phases above, the six-language SDK will be brought to full implementation parity:

| Language | Current status | Target |
|----------|---------------|--------|
| Python | Fully implemented | Complete |
| Go | Scaffolded | Full implementation |
| JavaScript | Scaffolded | Full implementation |
| Java | Scaffolded | Full implementation |
| Rust | Scaffolded | Full implementation |
| C# | Scaffolded | Full implementation |

All six languages expose identical API surfaces. When a new protocol operation is added, all six are updated together.

---

## Public Infrastructure

In parallel with protocol development:

- **`email.mq9.ai`** — a publicly accessible RobustMQ node where any Agent can claim a mailbox and communicate across machines, networks, and users. This is the first connectable public node for the mq9 ecosystem.
- **Self-hosted deployment** — mq9 is part of RobustMQ, which is open-source. Organizations with data sovereignty requirements can deploy their own nodes. The protocol is the same; the infrastructure is private.

---

*For the thinking behind these phases, see [What Should a Messaging System Look Like in the Age of AI](../Blogs/82.md).*
