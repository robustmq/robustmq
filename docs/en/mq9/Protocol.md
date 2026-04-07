# mq9 Protocol

## Protocol Foundation

mq9 is a semantic convention defined on top of NATS subjects, running on RobustMQ's unified storage layer. All mq9 operations are standard NATS pub/sub calls, with subject names following the `$mq9.AI.*` namespace.

Any NATS client library works as an mq9 client directly — no additional SDK required.

---

## Subject Naming

### Subject Space

```text
$mq9.AI.
  ├── MAILBOX.
  │     ├── CREATE                         # Create a mailbox
  │     └── {mail_id}.
  │           ├── high                     # High-priority messages
  │           ├── normal                   # Routine messages
  │           └── low                      # Low-priority messages
  │
  └── PUBLIC.LIST                          # Public mailbox discovery, system-managed
```

### Subject Reference

| Subject | Operation | Description |
|---------|-----------|-------------|
| `$mq9.AI.MAILBOX.CREATE` | pub | Create a mailbox; response returned via reply-to |
| `$mq9.AI.MAILBOX.{mail_id}.high` | pub/sub | High-priority messages |
| `$mq9.AI.MAILBOX.{mail_id}.normal` | pub/sub | Routine messages |
| `$mq9.AI.MAILBOX.{mail_id}.low` | pub/sub | Low-priority messages |
| `$mq9.AI.MAILBOX.{mail_id}.*` | sub | Subscribe to all priority levels of a mailbox |
| `$mq9.AI.PUBLIC.LIST` | sub | Discover all public mailboxes, system-managed |

---

## Core Concepts

**mail_id**: The communication address returned when creating a mailbox via `MAILBOX.CREATE`. Not bound to Agent identity — one Agent can apply for different mail_ids for different tasks, leave them alone when done, and TTL handles cleanup. Private mailbox mail_ids are system-generated and unguessable. Public mailbox mail_ids are user-defined with meaningful names — `task.queue`, `analytics.result`.

**mail_id unguessability is the security boundary.** Knowing the mail_id lets you send messages and subscribe. Without it, you can't interact with the mailbox. No token, no ACL.

**TTL**: Declared at mailbox creation time; auto-destroyed on expiry along with all messages. No DELETE command. CREATE is idempotent — creating the same mailbox again does not error; TTL is fixed by the first creation.

**priority**: MAILBOX supports three priority levels: high, normal, low. Same priority is FIFO; across priorities, higher priority is processed first. The storage layer guarantees ordering; consumers need not sort themselves.

**msg_id**: A unique identifier per message; used by clients for deduplication.

**Subscription semantics**: Every subscription delivers all non-expired messages in full, with new messages pushed in real time afterward. No read/unread distinction, no consume offset tracking, no QUERY command. Zero consumer state on the server.

**Message flow**: Message arrives → written to storage → pushed to online subscribers in real time; if offline, waits → delivered in full on next subscription.

---

## MAILBOX.CREATE — Create a Mailbox

### Create a Private Mailbox

```bash
nats pub '$mq9.AI.MAILBOX.CREATE' '{"ttl":3600}'

# Response
{"mail_id": "m-uuid-001"}
```

### Create a Public Mailbox

```bash
nats pub '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 3600,
  "public": true,
  "name": "task.queue",
  "desc": "Task queue"
}'

# Response
{"mail_id": "task.queue"}
```

Public mailboxes (`public: true`) are automatically registered to `$mq9.AI.PUBLIC.LIST` on creation and removed when their TTL expires.

CREATE is idempotent. If the mailbox already exists, it silently returns success; TTL is fixed by the first creation.

### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `ttl` | int | No | Lifetime in seconds; uses system default if omitted |
| `public` | bool | No | Whether to make the mailbox public; default false |
| `name` | string | Required for public mailboxes | Custom mail_id for a public mailbox |
| `desc` | string | No | Mailbox description; recommended for public mailboxes |

---

## Sending Messages

Knowing the mail_id is all that's needed to send — no authorization required.

```bash
nats pub '$mq9.AI.MAILBOX.m-uuid-001.high'   'payload'  # Urgent, processed first
nats pub '$mq9.AI.MAILBOX.m-uuid-001.normal' 'payload'  # Routine communication
nats pub '$mq9.AI.MAILBOX.m-uuid-001.low'    'payload'  # Background, not urgent
```

### Priority Levels

| Level | Semantics | Typical use |
|-------|-----------|-------------|
| high | Urgent, processed first | Task interrupts, emergency commands |
| normal | Routine communication | Task dispatch, result delivery, approval requests |
| low | Background, not urgent | Logs, status reports, notifications |

The storage layer guarantees priority ordering. An edge device that comes back online after being offline receives high messages first, then normal, then low. Consumers need not sort themselves.

### Message Structure (Recommended)

```json
{
  "msg_id": "msg-uuid-001",
  "from": "m-uuid-002",
  "type": "task_result",
  "correlation_id": "req-uuid-001",
  "reply_to": "m-uuid-002",
  "payload": {},
  "ts": 1234567890
}
```

| Field | Description |
|-------|-------------|
| msg_id | Unique message identifier; used by clients for deduplication |
| from | Sender's mail_id |
| type | Message type; defined by the application |
| correlation_id | Links to the original request; used for request-reply |
| reply_to | Reply address; the recipient sends responses to this mail_id |
| payload | Message content; mq9 does not parse this |
| ts | Send timestamp |

Message structure is defined by the application; mq9 does not enforce it. The above is a recommended convention.

---

## Subscribing to a Mailbox

```bash
nats sub '$mq9.AI.MAILBOX.m-uuid-001.*'     # All priority levels
nats sub '$mq9.AI.MAILBOX.m-uuid-001.high'  # High priority only
```

Subscribing delivers all non-expired messages immediately, then pushes new messages in real time. Agents that reconnect do not miss messages.

**Competing consumers** — Public mailboxes support queue groups, allowing multiple subscribers to compete, with each message handled by exactly one subscriber:

```bash
nats sub '$mq9.AI.MAILBOX.task.queue.*' --queue workers
```

**Prohibited operations:**

```bash
nats sub '$mq9.AI.MAILBOX.*'  # Rejected by broker
nats sub '$mq9.AI.MAILBOX.#'  # Rejected by broker
```

Subscriptions must specify an exact mail_id. Wildcard subscriptions across all mailboxes are not permitted.

---

## PUBLIC.LIST — Public Mailbox Discovery

System-managed address maintained by the broker. Does not accept user writes. Never expires.

Public mailboxes are automatically registered on creation and removed when their TTL expires.

```bash
nats sub '$mq9.AI.PUBLIC.LIST'
```

Subscribing delivers all current public mailboxes immediately, then streams additions and removals in real time.

Push format:

```json
# Added
{"event": "created", "mail_id": "task.queue", "desc": "Task queue", "ttl": 3600}

# Removed
{"event": "expired", "mail_id": "task.queue"}
```

---

## Storage Tiers

mq9 runs on RobustMQ's unified storage layer, providing three storage capabilities:

| Storage tier | Characteristics | Use cases |
|-------------|----------------|-----------|
| Memory | Pure in-memory, no persistence | Coordination signals, heartbeats — disposable |
| RocksDB | Ephemeral persistence, TTL auto-cleanup | Mailbox default; offline message buffering |
| File Segment | Long-term persistence, permanent | Audit logs, critical events |

| Priority | Default storage | Notes |
|----------|----------------|-------|
| high | RocksDB | Persisted |
| normal | RocksDB | Persisted |
| low | Memory | Not persisted — retransmit if lost |

---

## Protocol Summary

| Subject | Direction | Persisted | Description |
|---------|-----------|-----------|-------------|
| `$mq9.AI.MAILBOX.CREATE` | PUB | — | Create a mailbox |
| `$mq9.AI.MAILBOX.{id}.high` | PUB/SUB | Yes | High-priority messages |
| `$mq9.AI.MAILBOX.{id}.normal` | PUB/SUB | Yes | Routine messages |
| `$mq9.AI.MAILBOX.{id}.low` | PUB/SUB | No | Low-priority messages |
| `$mq9.AI.PUBLIC.LIST` | SUB | Yes | Discover public mailboxes, system-managed |

---

## Relationship to Native NATS

mq9 is built on top of RobustMQ's NATS protocol layer, enhancing specific subject prefixes with server-side logic:

- Native NATS pub/sub: delivered in real-time; if the subscriber is offline, the message is lost
- mq9 `$mq9.AI.MAILBOX.*`: messages are written to storage first; they survive offline periods and are delivered in full on reconnect

Both can be used together. mq9 only activates its enhanced behavior for messages matching the `$mq9.AI.*` prefix. All other NATS behavior is unchanged. Existing NATS clients need no modification — use the mq9 subjects directly.
