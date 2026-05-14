# Core Features

## Overview

From a functional perspective, mq9 is an advanced mailbox for Agents — claim an address, send messages to others, actively fetch your own messages, and acknowledge what's been handled. Everything else is handled by mq9 in the background.

mq9 adds the following core capabilities on top of a basic mailbox:

- **Pull consumption + ACK**: Clients actively FETCH to pull messages; ACK advances the consumption offset; supports resume-from-offset
- **Priority**: Three levels (critical / urgent / normal); urgent messages are dequeued first
- **Message attributes**: key deduplication, tags filtering, delay delivery, per-message TTL
- **Agent registry and discovery**: Built-in Agent registry with full-text and semantic vector search
- **TTL**: Mailboxes have a lifecycle — auto-destroyed on expiry with no manual cleanup needed

---

## Feature Reference

### 1. Pull Consumption and ACK

mq9 uses **pull mode**: clients actively call `FETCH` to retrieve messages, then call `ACK` after processing to advance the consumption offset for that consumer group.

**Two consumption modes:**

| Mode | Usage | Use case |
|------|-------|----------|
| Stateful | Pass `group_name` | Broker records offset; resumes from last ACK on reconnect — suitable for persistent workers |
| Stateless | Omit `group_name` | Each call independently applies the `deliver` policy; no offset recorded — suitable for one-off reads and debugging |

**Stateful offset behavior:**

| Condition | Behavior |
|-----------|----------|
| Offset exists | Resume from last ACK position; `deliver` policy is ignored |
| Offset exists + `force_deliver: true` | Ignore offset; restart from `deliver` policy |
| No offset (first time) | Apply `deliver` policy to determine start position |

**deliver start policies:**

| Value | Description |
|-------|-------------|
| `latest` (default) | Only pull messages from this point forward |
| `earliest` | Start from the oldest message in the mailbox |
| `from_time` | Start from after the specified Unix timestamp |
| `from_id` | Start from the specified msg_id (inclusive) |

**Consumption flow:**

```text
Client FETCH → broker returns message list → client processes → ACK → broker advances offset
                                                                       ↓
                                                             next FETCH resumes here
```

---

### 2. Priority System

Each message specifies its priority via the `mq9-priority` header, with three levels:

| Priority | Header value | Typical use |
|----------|-------------|-------------|
| `critical` | `mq9-priority: critical` | Abort signals, emergency commands, security events |
| `urgent` | `mq9-priority: urgent` | Approval requests, time-sensitive notifications |
| `normal` (default) | omit header | Task dispatch, result delivery, routine communication |

**Ordering guarantees:**

- Within the same priority level: FIFO — messages are dequeued in send order
- Across priority levels: critical before urgent before normal
- Ordering is enforced by the storage layer; consumers do not need to sort

---

### 3. Message Attributes

The following attributes can be attached when sending a message, transmitted via NATS headers:

| Attribute | Header | Description |
|-----------|--------|-------------|
| Dedup key | `mq9-key: {key}` | Only the latest message with the same key is kept; older ones are overwritten. Suitable for state-update messages (e.g. task progress) |
| Tags | `mq9-tags: {tag1},{tag2}` | Comma-separated, e.g. `billing,vip`. Can be filtered via the `tags` field in QUERY |
| Delayed delivery | `mq9-delay: {seconds}` | Message becomes visible after the specified number of seconds. Delayed messages return `msg_id: -1` |
| Per-message TTL | `mq9-ttl: {seconds}` | Message expires at `send_time + ttl` automatically, independent of mailbox TTL |

**Dedup key example:** continuously reporting task progress, keeping only the latest state:

```text
SEND key=status {"status":"running"}   → msg_id=1
SEND key=status {"status":"60%"}       → msg_id=2, overwrites previous
SEND key=status {"status":"done"}      → msg_id=3, only this one is kept
QUERY key=status                       → returns msg_id=3
```

---

### 4. TTL and Lifecycle

The mailbox declares a TTL at creation. On expiry, the mailbox and all its messages are automatically destroyed with no manual cleanup:

```json
{"name": "task.queue", "ttl": 3600}
```

**Behavior rules:**

- TTL starts counting from mailbox creation and cannot be renewed
- There is no manual delete-mailbox command; TTL is the only cleanup mechanism
- Creating a mailbox with a duplicate name returns an error (`mailbox xxx already exists`); CREATE is not idempotent
- `ttl: 0` or omitting ttl means the mailbox never expires

**Per-message TTL is independent of mailbox TTL:** messages can set their own expiry via `mq9-ttl` header, expiring earlier than the mailbox itself.

---

### 5. Message Query and Delete

**QUERY** — query messages currently stored in the mailbox without affecting the consumption offset:

| Query type | Parameter | Description |
|------------|-----------|-------------|
| Full scan | no parameters | Return all messages |
| By key | `key: "status"` | Return the latest message with this key |
| By tags | `tags: ["billing", "vip"]` | Return messages that carry all specified tags |
| By time | `since: <unix_ts>` | Return messages after this timestamp |
| Paginated | `limit: 20` | Return at most N messages |

**DELETE** — delete a specific message by msg_id.

---

### 6. Agent Registry and Discovery

mq9 has a built-in Agent registry with three discovery modes:

| Mode | Parameter | Description |
|------|-----------|-------------|
| Semantic search | `semantic: "process payments and generate invoices"` | Vector similarity matching; understands natural language intent |
| Full-text search | `text: "payment invoice"` | Keyword matching |
| List all | no parameters | Return all registered Agents under this tenant |

Search priority: `semantic` > `text` > neither.

Supports pagination (`limit` + `page`, page starts at 1).

**Typical flow:**

```text
Agent starts → REGISTER (with capability description)
                    ↓
Other Agent → DISCOVER (semantic: "find a translation agent") → returns matching list
                    ↓
              sends message to matched agent's mail_address
Agent shuts down → UNREGISTER
```

Registration content (`payload`) can be plain text or an A2A AgentCard JSON string. The content is indexed for both full-text and vector search.

---

### 7. mail_address Format

**Character set**: lowercase letters (a-z), digits (0-9), dot (`.`)

**Length**: 1 to 128 characters

**Rules**: `.` may only appear in the middle; the first and last character must be a lowercase letter or digit; consecutive `.` are not allowed

| Valid examples | Invalid examples |
|---------------|-----------------|
| `task.001` | `task-001` (contains hyphen) |
| `agent.inbox` | `Task.001` (contains uppercase) |
| `session.20260502` | `.task.001` (dot at start) |
| `acme.org.task.queue` | `task..001` (consecutive dots) |

**Security model:** The unguessability of `mail_address` is the only access control boundary. Knowing the mail_address lets you send and receive; without it, there's no way to interact — no token, no ACL.

---

## Comparison with NATS JetStream

| | NATS JetStream | mq9 |
|--|---------------|-----|
| Consumption mode | push or pull | pull (FETCH + ACK) |
| Consumer state | server-maintained offsets, consumer groups, ACK | server-maintained consumer group offsets, ACK advances |
| Message filtering | subject filtering | key, tags, since, limit |
| Priority | no built-in priority | three levels (critical/urgent/normal) |
| Agent discovery | none | built-in, with semantic vector search |
| Delayed messages | supported | supported (mq9-delay header) |
| Per-message TTL | supported | supported (mq9-ttl header) |
| Access method | any NATS client | any NATS client (subjects follow `$mq9.AI.*` convention) |
