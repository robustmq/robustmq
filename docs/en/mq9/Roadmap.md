# mq9 Roadmap

mq9's foundational mailbox capabilities are complete. This page describes what comes next — four phases that progressively move mq9 from message delivery infrastructure toward intelligent, context-aware Agent communication infrastructure.

The phases are not strictly sequential. Priorities will shift based on feedback from real-world use cases. The direction is fixed; the order is flexible.

---

## Where We Are Today

The core communication layer is in place:

- **Mailbox lifecycle** — private and public mailboxes, TTL-driven expiration, idempotent CREATE
- **Three-priority messaging** — `critical` / `urgent` / `normal`, store-first delivery, offline-tolerant
- **Competitive consumption** — NATS queue groups, dynamic membership, crash-tolerant task queues
- **Public discovery** — `PUBLIC.LIST` for decentralized capability announcement
- **Six-language SDK** — Python, Go, JavaScript, Java, Rust, C# (Python fully implemented, others scaffolded)
- **LangChain & LangGraph integration** — `langchain-mq9` toolkit with 6 tools
- **MCP Server support** — AI ecosystem integration via JSON-RPC 2.0

---

## Phase 1: Semantic Service Discovery

**Goal:** Agents find each other by describing what they need, not by knowing a name in advance.

Today, public mailbox discovery requires knowing (or guessing) the mailbox name. An Agent registers as `agent.code-review`, and consumers must know that string. This works for structured naming conventions, but breaks down as the Agent ecosystem grows and names become unpredictable.

**What changes:**

- Vectorize the `desc` field of public mailboxes at creation time
- Add semantic search to `PUBLIC.LIST` — callers pass a natural-language query, the server returns the best-matching mailboxes ranked by similarity
- `PUBLIC.LIST` evolves from a firehose of all public mailboxes into an intelligent capability registry

**What this enables:**

An Agent that needs "something that can review Python code" no longer needs to know the exact mailbox name. It queries `PUBLIC.LIST` with a description, gets back the top matches, and sends its task to the best one. mq9 becomes the natural service registry for AI Agent ecosystems — no Consul, no Etcd, no configuration file.

---

## Phase 2: Semantic Routing

**Goal:** Senders describe intent; the broker finds the recipient.

Phase 1 is pull-based — consumers query and choose. Phase 2 is push-based — the sender describes what it needs done, and mq9 routes automatically to the most suitable recipient.

**What changes:**

- Messages can optionally omit the `mail_address` and instead carry a semantic intent description
- The broker vector-matches the message intent against registered Agent capability descriptions
- Routing to the best-matching Agent happens inside the broker, transparently to the sender

**What this enables:**

An Agent that has a legal analysis task no longer needs to know which other Agent can handle legal questions. It publishes "analyze this contract for compliance issues" and mq9 routes the task to the most capable registered Agent. This evolves the broker from a "post office" into an "intelligent dispatcher."

This direction is being explored — the implementation details depend on Phase 1's foundation and real-world routing workloads.

---

## Phase 3: Intent-Based Policy

**Goal:** Infrastructure-level safety boundaries for AI Agent communication.

Traditional message brokers are mindless relays — they do not understand message content and do not judge whether a message should be delivered. Application-layer security policies are the only defense. For AI Agents, this is insufficient: a compromised or misconfigured Agent can issue instructions like "delete the production database" or "transfer funds," and the broker faithfully delivers them.

**What changes:**

- Policy rules are configurable per mailbox (or globally)
- As messages transit through the policy engine, they are evaluated against the semantic content — not just headers or metadata
- Messages that violate policy are blocked before delivery; the violation is recorded

**The multi-protocol advantage:**

When a message is blocked, RobustMQ does not need a separate system to record the event. The policy engine writes the blocked message to a built-in risk topic. Risk analysis systems can consume this via the Kafka protocol — same broker, same storage, no additional infrastructure, no data crossing system boundaries.

This is a concrete example of RobustMQ's multi-protocol architecture applied to an AI security scenario: mq9 for Agent communication, Kafka for risk stream consumption, one deployment serving both.

---

## Phase 4: Context Awareness (Exploratory)

**Goal:** The broker carries conversation context; Agents stop retransmitting history.

Every interaction between AI Agents today includes redundant context retransmission: "Hello, I'm A, we previously discussed X, and now I need you to help me do Y." This consumes tokens and adds latency. The more complex the Agent collaboration, the worse this gets.

**What this would mean:**

- The broker becomes session-aware — it tracks conversation history between Agent pairs
- As messages flow, the broker automatically attaches relevant historical context based on the session
- Agent A sends "do Y"; the broker, knowing the prior exchange between A and B, enriches the message with the necessary context before delivery

**What this enables:**

Agents no longer need to retransmit the full context in every interaction. Token consumption drops. Agent collaboration becomes more efficient. The infrastructure evolves from a "stateless pipeline" into a "stateful context network."

This is the longest-horizon direction and furthest from implementation. The exact shape of session-awareness at the infrastructure layer is an open research problem. We believe the direction is correct; we do not have a definitive implementation plan.

---

## SDK Completion

Parallel to the four phases above, the six-language SDK will be brought to full implementation parity:

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
