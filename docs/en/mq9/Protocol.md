# mq9 Protocol

## Protocol Foundation

mq9 is a semantic convention defined on top of NATS subjects, running on RobustMQ's unified storage layer. All mq9 operations are standard NATS pub/sub/req-reply calls, with subject names following the `$mq9.AI.*` namespace.

Any NATS client library works as an mq9 client directly — no additional SDK required.

---

## Subject Naming

### Subject Space

```text
$mq9.AI.
  ├── MAILBOX.
  │     └── CREATE                    # Create a mailbox or broadcast channel
  │
  ├── INBOX.
  │     └── {mail_id}.
  │           ├── urgent              # Urgent messages
  │           ├── normal              # Routine messages
  │           └── notify              # Notifications
  │
  └── BROADCAST.
        ├── system.
        │     ├── capability          # System bulletin board: capability
        │     ├── status              # System bulletin board: status
        │     └── channel             # System bulletin board: channel
        └── {domain}.
              └── {event}            # User-defined broadcasts
```

### Subject Reference

| Subject | Operation type | Description |
|---------|---------------|-------------|
| `$mq9.AI.MAILBOX.CREATE` | req/reply | Create a mailbox or broadcast channel. INBOX returns mail_id + token; BROADCAST returns success |
| `$mq9.AI.INBOX.{mail_id}.urgent` | pub/sub | Deliver an urgent message to a mailbox |
| `$mq9.AI.INBOX.{mail_id}.normal` | pub/sub | Deliver a routine message to a mailbox |
| `$mq9.AI.INBOX.{mail_id}.notify` | pub/sub | Deliver a notification to a mailbox |
| `$mq9.AI.INBOX.{mail_id}.*` | sub | Subscribe to all priority levels of a mailbox |
| `$mq9.AI.BROADCAST.system.capability` | pub/sub | System bulletin board: capability |
| `$mq9.AI.BROADCAST.system.status` | pub/sub | System bulletin board: status |
| `$mq9.AI.BROADCAST.system.channel` | pub/sub | System bulletin board: channel |
| `$mq9.AI.BROADCAST.{domain}.{event}` | pub/sub | Publish or subscribe to a user-defined broadcast channel |
| `$mq9.AI.BROADCAST.{domain}.*` | sub | Subscribe to all events in a domain |
| `$mq9.AI.BROADCAST.*.{event}` | sub | Subscribe to a specific event across all domains |
| `$mq9.AI.BROADCAST.#` | sub | Subscribe to all broadcasts |

---

## Core Concepts

**mail_id**: A globally unique communication address obtained via `MAILBOX.CREATE`. Not bound to Agent identity — one Agent can hold multiple mail_ids for different tasks, and they expire and clean up automatically. mail_id is channel-level, not identity-level.

**Mailbox type**: Declared with the `type` parameter when creating a mailbox:
- `standard` (default): messages accumulate, stored by priority.
- `latest`: only the most recent message is kept; new messages overwrite old ones. For status reporting, capability declarations.

**TTL**: Declared at INBOX or BROADCAST creation time; auto-destroyed on expiry, along with all messages. No explicit delete operation. TTL is fixed by the first CREATE call; subsequent CREATEs do not override it.

**token**: Returned when creating an INBOX. Anyone who knows the mail_id can send messages; the token is only used for mailbox management operations. BROADCAST has no token.

**msg_id**: A unique identifier per message; used by clients for deduplication.

**Subscription semantics**: INBOX and BROADCAST behave identically — each subscription delivers all non-expired messages in full. No read/unread distinction, no consume offset tracking. Zero consumer state on the server.

**Message flow**: Message arrives → written to storage → pushed to online subscribers in real time; if offline, waits → delivered in full on next subscription.

**CREATE idempotency**: Creating the same INBOX or BROADCAST again does not error; it silently succeeds. TTL is fixed by the first creation.

**No state queries**: mq9 does not expose "does this mailbox exist" or "does this channel exist" query interfaces. CREATE always returns success.

---

## MAILBOX.CREATE — Create a Mailbox or Broadcast Channel

A unified creation entry point. The subject prefix in the request body distinguishes whether an INBOX or BROADCAST is being created.

### Create a Private Mailbox (INBOX)

```bash
# Request
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "type": "standard",
  "ttl": 3600,
  "subject": "$mq9.AI.INBOX"
}'

# Response
{
  "mail_id": "m-uuid-001",
  "token": "tok-xxx",
  "inbox": "$mq9.AI.INBOX.m-uuid-001"
}
```

### Create a Broadcast Channel (BROADCAST)

```bash
# Request
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "type": "standard",
  "ttl": 3600,
  "subject": "$mq9.AI.BROADCAST.pipeline.complete"
}'

# Response
{
  "subject": "$mq9.AI.BROADCAST.pipeline.complete"
}
```

The server identifies the `$mq9.AI.BROADCAST.` prefix, does not generate a mail_id or token. After creation, anyone can publish to or subscribe from the channel.

CREATE is idempotent. If the channel already exists, it silently returns success; TTL is fixed by the first creation and is not overridden.

### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `type` | string | No | `standard` (default) or `latest` |
| `ttl` | int | No | Lifetime in seconds; uses system default if omitted |
| `subject` | string | Yes | `$mq9.AI.INBOX` to create a mailbox; `$mq9.AI.BROADCAST.{domain}.{event}` to create a broadcast channel |

---

## INBOX — Point-to-Point Mailbox

### Send a Message

```bash
# Send a routine message
nats pub '$mq9.AI.INBOX.m-uuid-001.normal' '{
  "msg_id": "msg-uuid-001",
  "from": "m-uuid-002",
  "type": "task_result",
  "correlation_id": "req-uuid-001",
  "reply_to": "$mq9.AI.INBOX.m-uuid-002.normal",
  "payload": { ... },
  "ts": 1234567890
}'

# Send an urgent message
nats pub '$mq9.AI.INBOX.m-uuid-001.urgent' '{
  "msg_id": "msg-uuid-002",
  "from": "m-uuid-003",
  "type": "emergency_stop",
  "payload": { ... },
  "ts": 1234567890
}'
```

### Message Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `msg_id` | string | Yes | Unique message identifier; used by clients for deduplication |
| `from` | string | Yes | Sender's mail_id |
| `type` | string | Yes | Message type; defined by the application |
| `correlation_id` | string | No | Request-reply pairing for async request-response patterns |
| `reply_to` | string | No | Reply address; the recipient can PUB directly to this subject |
| `deadline` | int | No | Expected processing deadline (Unix timestamp, milliseconds) |
| `payload` | any | No | Business data |
| `ts` | int | Yes | Send timestamp (milliseconds) |

### Priority Levels

| Priority | Description | Persisted | Suggested TTL |
|----------|-------------|-----------|---------------|
| `urgent` | Critical messages, processed first | Yes | 86400s |
| `normal` | Routine messages, FIFO | Yes | 3600s |
| `notify` | Informational, background processing | No | short |

Messages expire together with their mailbox. When the mailbox TTL expires, all messages inside are destroyed.

### Receive Messages

When an Agent comes online, it subscribes to its mailbox. The server delivers all non-expired messages in full.

```bash
# Subscribe to all priority levels
nats sub '$mq9.AI.INBOX.m-uuid-001.*'

# Subscribe to urgent messages only
nats sub '$mq9.AI.INBOX.m-uuid-001.urgent'
```

### Java Example

```java
// Dependency: io.nats:jnats:2.20.5
Connection nc = Nats.connect("nats://localhost:4222");

// Create a mailbox
Message reply = nc.request("$mq9.AI.MAILBOX.CREATE",
    "{\"type\":\"standard\",\"ttl\":3600,\"subject\":\"$mq9.AI.INBOX\"}".getBytes(),
    Duration.ofSeconds(3));
// reply contains mail_id and token

// Subscribe (recipient)
Dispatcher d = nc.createDispatcher((msg) -> {
    System.out.println("Received: " + new String(msg.getData()));
});
d.subscribe("$mq9.AI.INBOX.m-uuid-001.*");

// Send a message (sender)
nc.publish("$mq9.AI.INBOX.m-uuid-001.normal",
    "{\"msg_id\":\"msg-001\",\"from\":\"m-uuid-002\",\"type\":\"task_result\",\"payload\":\"done\",\"ts\":1234567890}".getBytes());
```

---

## BROADCAST — Public Broadcast

Broadcast channels must be CREATEd first; after that, anyone can publish to them and anyone can subscribe. Messages are persisted; offline subscribers receive all non-expired messages in full upon reconnection.

### Create and Publish

```bash
# Create the channel first
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "type": "standard",
  "ttl": 7200,
  "subject": "$mq9.AI.BROADCAST.pipeline.complete"
}'

# Publish a broadcast
nats pub '$mq9.AI.BROADCAST.pipeline.complete' '{
  "msg_id": "msg-uuid-003",
  "from": "m-uuid-004",
  "type": "pipeline_done",
  "reply_to": "$mq9.AI.INBOX.m-uuid-004.normal",
  "payload": { ... },
  "ts": 1234567890
}'
```

`{domain}.{event}` is defined by the application. mq9 only specifies the `$mq9.AI.BROADCAST.` prefix.

### Suggested Domain Names

| domain | Description |
|--------|-------------|
| `system` | System-level events (built-in, not modifiable) |
| `task` | Task events: available, completed, failed |
| `data` | Data change events |
| custom | Application-defined domains |

### Subscribe to Broadcasts

```bash
# Subscribe to the system bulletin board
nats sub '$mq9.AI.BROADCAST.system.*'

# Subscribe to all events in a single domain
nats sub '$mq9.AI.BROADCAST.pipeline.*'

# Subscribe to a specific event across all domains
nats sub '$mq9.AI.BROADCAST.*.anomaly'

# Subscribe to all broadcasts
nats sub '$mq9.AI.BROADCAST.#'
```

### Competing Consumers (Queue Group)

When multiple Workers compete for the same broadcast, a queue group ensures each message is handled by exactly one Worker:

```bash
nats sub '$mq9.AI.BROADCAST.task.available' --queue task.workers
```

### BROADCAST Java Example

```java
// Three Workers compete via queue group
for (int i = 1; i <= 3; i++) {
    final int id = i;
    Dispatcher worker = nc.createDispatcher((msg) -> {
        System.out.println("[Worker-" + id + "] claimed task: " + new String(msg.getData()));
    });
    // queue group ensures only one Worker receives each message
    worker.subscribe("$mq9.AI.BROADCAST.task.available", "task.workers");
}

// Orchestrator broadcasts a task
nc.publish("$mq9.AI.BROADCAST.task.available",
    "{\"msg_id\":\"t-001\",\"task_id\":\"t-001\",\"type\":\"data_analysis\"}".getBytes());

// Subscribe to anomaly alerts across all domains
Dispatcher alertHandler = nc.createDispatcher((msg) -> {
    System.out.println("Alert received: " + new String(msg.getData()));
});
alertHandler.subscribe("$mq9.AI.BROADCAST.*.anomaly");
```

---

## Storage Tiers

mq9 runs on RobustMQ's unified storage layer. Users choose storage per message; not every message needs the same level of durability.

| Storage tier | Characteristics | Use cases |
|-------------|----------------|-----------|
| Memory | Pure in-memory, no persistence | Coordination signals, heartbeats, disposable notifications |
| RocksDB | Ephemeral persistence, TTL auto-cleanup | Mailbox default; task instructions; offline buffering |
| File Segment | Long-term persistence, permanent | Audit logs, critical events |

Mailbox type and storage mapping:

| Mailbox type | Priority | Default storage | Notes |
|-------------|----------|----------------|-------|
| standard | urgent | RocksDB | Persisted, TTL=86400s |
| standard | normal | RocksDB | Persisted, TTL=3600s |
| standard | notify | Memory | Not persisted |
| latest | — | RocksDB | Only the most recent message kept |
| broadcast | — | RocksDB | Persisted, TTL declared at CREATE time |

---

## Message Format

mq9 does not enforce a message format — the payload is arbitrary bytes. JSON is recommended. Suggested fields:

```json
{
  "msg_id": "msg-uuid-001",
  "from": "m-uuid-001",
  "type": "task_result",
  "correlation_id": "req-uuid-001",
  "reply_to": "$mq9.AI.INBOX.m-uuid-001.normal",
  "deadline": 1234567890,
  "payload": {},
  "ts": 1234567890
}
```

---

## Relationship to Native NATS

mq9 is built on top of RobustMQ's NATS protocol layer, adding persistence and priority scheduling for specific subject prefixes:

- Native NATS pub/sub: delivered in real-time; if the subscriber is offline, the message is lost
- mq9 `$mq9.AI.INBOX.*` / `$mq9.AI.BROADCAST.*`: written to storage first; messages survive offline periods and are delivered in full on reconnect

Both can be used together. mq9 only activates its enhanced behavior for messages matching the `$mq9.AI.*` prefix. All other NATS behavior is unchanged. Existing NATS clients need no modification — use the mq9 subjects directly.
