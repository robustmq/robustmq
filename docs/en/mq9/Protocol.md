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
  │     ├── CREATE                    # Request a mailbox, returns mail_id
  │     └── QUERY.{mail_id}          # Pull unread messages (fallback)
  │
  ├── INBOX.
  │     └── {mail_id}.
  │           ├── urgent              # Urgent messages
  │           ├── normal              # Routine messages
  │           └── notify              # Notifications
  │
  └── BROADCAST.
        └── {domain}.
              └── {event}            # Event broadcast
```

### Subject Reference

| Subject | Operation type | Description |
|---------|---------------|-------------|
| `$mq9.AI.MAILBOX.CREATE` | req/reply | Request a mailbox, returns mail_id and token |
| `$mq9.AI.MAILBOX.QUERY.{mail_id}` | req/reply | Pull unread messages (fallback) |
| `$mq9.AI.INBOX.{mail_id}.urgent` | pub/sub | Deliver an urgent message to a mailbox |
| `$mq9.AI.INBOX.{mail_id}.normal` | pub/sub | Deliver a routine message to a mailbox |
| `$mq9.AI.INBOX.{mail_id}.notify` | pub/sub | Deliver a notification to a mailbox |
| `$mq9.AI.INBOX.{mail_id}.*` | sub | Subscribe to all priority levels of a mailbox |
| `$mq9.AI.BROADCAST.{domain}.{event}` | pub/sub | Publish or subscribe to a broadcast event |
| `$mq9.AI.BROADCAST.{domain}.*` | sub | Subscribe to all events in a domain |
| `$mq9.AI.BROADCAST.*.{event}` | sub | Subscribe to a specific event across all domains |
| `$mq9.AI.BROADCAST.#` | sub | Subscribe to all broadcasts |

---

## Core Concepts

**mail_id**: A globally unique communication address obtained via `MAILBOX.CREATE`. Not bound to Agent identity — one Agent can hold multiple mail_ids for different tasks, and they expire and clean up automatically. mail_id is channel-level, not identity-level.

**Mailbox type**: Declared with the `type` parameter when creating a mailbox:
- `standard` (default): messages accumulate, stored by priority.
- `latest`: only the most recent message is kept; new messages overwrite old ones. For status reporting, capability declarations, heartbeats.

**TTL**: Both mailboxes and messages have TTL. Mailboxes do not need explicit deletion — TTL manages the lifecycle entirely. On expiry, the mailbox and all its messages are destroyed automatically.

**persist**: Persistence policy for messages. INBOX messages default to persisted; BROADCAST messages default to non-persisted, but can be overridden explicitly.

**token**: Lightweight authentication. Returned when the mailbox is created. Required when calling QUERY. Ensures mailbox ownership without complex access control.

**Message flow**: Message arrives → written to storage → check for online subscribers → push if online → if offline, message waits in storage → subscriber comes online and receives via SUB push or explicit QUERY pull.

---

## MAILBOX.CREATE — Request a Mailbox

```bash
# Request
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "type": "standard",
  "ttl": 3600
}'

# Response
{
  "mail_id": "m-uuid-001",
  "token": "tok-xxx",
  "inbox": "$mq9.AI.INBOX.m-uuid-001"
}
```

### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `type` | string | No | `standard` (default) or `latest` |
| `ttl` | int | No | Mailbox lifetime in seconds; uses system default if omitted |

### Mailbox Types

| type | Behavior | Use cases |
|------|----------|-----------|
| `standard` | Messages accumulate, stored by priority | Task instructions, request-reply, offline buffering |
| `latest` | Only the most recent message is kept | Status reporting, capability declarations, heartbeats |

---

## MAILBOX.QUERY — Pull Unread Messages

QUERY is the fallback for push delivery. Under normal conditions, online subscribers receive messages via push (SUB). Use QUERY when: an Agent comes online and needs to check messages that arrived while offline; push delivery may have been missed; confirming no unprocessed messages remain.

```bash
# Request
nats req '$mq9.AI.MAILBOX.QUERY.m-uuid-001' '{
  "token": "tok-xxx"
}'

# Response
{
  "mail_id": "m-uuid-001",
  "unread": 5,
  "messages": [ ... ]
}
```

---

## INBOX — Point-to-Point Mailbox

### Send a Message

```bash
# Send a routine message
nats pub '$mq9.AI.INBOX.m-uuid-001.normal' '{
  "from": "m-uuid-002",
  "type": "task_result",
  "correlation_id": "req-uuid-001",
  "reply_to": "$mq9.AI.INBOX.m-uuid-002.normal",
  "payload": { ... },
  "ts": 1234567890
}'

# Send an urgent message
nats pub '$mq9.AI.INBOX.m-uuid-001.urgent' '{
  "from": "m-uuid-003",
  "type": "emergency_stop",
  "payload": { ... },
  "ts": 1234567890
}'
```

### Message Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `from` | string | Yes | Sender's mail_id |
| `type` | string | Yes | Message type; defined by the application |
| `correlation_id` | string | No | Request-reply pairing for async request-response patterns |
| `reply_to` | string | No | Reply address; the recipient can PUB directly to this subject |
| `deadline` | int | No | Expected processing deadline (Unix timestamp, milliseconds) |
| `payload` | any | No | Business data |
| `ts` | int | Yes | Send timestamp (milliseconds) |

### Priority Levels

| Priority | Description | Persisted by default | Suggested TTL |
|----------|-------------|---------------------|---------------|
| `urgent` | Critical messages, processed first | true | 86400s |
| `normal` | Routine messages, FIFO | true | 3600s |
| `notify` | Informational, background processing | false | — |

### Receive Messages

When an Agent comes online, it subscribes to its mailbox. The server pushes any buffered offline messages.

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

// Request a mailbox
Message reply = nc.request("$mq9.AI.MAILBOX.CREATE",
    "{\"type\":\"standard\",\"ttl\":3600}".getBytes(),
    Duration.ofSeconds(3));
// reply contains mail_id and token

// Subscribe (recipient)
Dispatcher d = nc.createDispatcher((msg) -> {
    System.out.println("Received: " + new String(msg.getData()));
});
d.subscribe("$mq9.AI.INBOX.m-uuid-001.*");

// Send a message (sender)
nc.publish("$mq9.AI.INBOX.m-uuid-001.normal",
    "{\"from\":\"m-uuid-002\",\"type\":\"task_result\",\"payload\":\"done\",\"ts\":1234567890}".getBytes());

// Pull unread messages (fallback)
Message qReply = nc.request("$mq9.AI.MAILBOX.QUERY.m-uuid-001",
    "{\"token\":\"tok-xxx\"}".getBytes(), Duration.ofSeconds(3));
```

---

## BROADCAST — Public Broadcast

Broadcast requires no CREATE and no prior setup. Publish directly; subscribe directly.

### Publish a Broadcast

```bash
nats pub '$mq9.AI.BROADCAST.{domain}.{event}' '{
  "from": "m-uuid-004",
  "type": "event_type",
  "severity": "high",
  "reply_to": "$mq9.AI.INBOX.m-uuid-004.normal",
  "payload": { ... },
  "ts": 1234567890
}'
```

Broadcasts are not persisted by default. Set `persist=true` explicitly for important broadcasts that must survive subscriber downtime.

`{domain}.{event}` is defined by the application. mq9 only specifies the `$mq9.AI.BROADCAST.` prefix.

### Suggested Domain Names

| domain | Description |
|--------|-------------|
| `system` | System-level events: anomalies, restarts, scaling |
| `task` | Task events: available, completed, failed |
| `capability` | Capability discovery: query, response |
| `data` | Data change events |
| custom | Application-defined domains |

### Subscribe to Broadcasts

```bash
# Subscribe to all events in a single domain
nats sub '$mq9.AI.BROADCAST.system.*'

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
    "{\"task_id\":\"t-001\",\"type\":\"data_analysis\"}".getBytes());

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
| broadcast (default) | — | Memory | Not persisted; override with persist=true |

---

## Message Format

mq9 does not enforce a message format — the payload is arbitrary bytes. JSON is recommended. Suggested fields:

```json
{
  "from": "m-uuid-001",
  "type": "task_result",
  "correlation_id": "req-uuid-001",
  "reply_to": "$mq9.AI.INBOX.m-uuid-001.normal",
  "deadline": 1234567890,
  "payload": { },
  "ts": 1234567890
}
```

---

## Relationship to Native NATS

mq9 is built on top of RobustMQ's NATS protocol layer, adding persistence and priority scheduling for specific subject prefixes:

- Native NATS pub/sub: delivered in real-time; if the subscriber is offline, the message is lost
- mq9 `$mq9.AI.INBOX.*`: written to storage first; messages survive offline periods and are delivered on reconnect or via QUERY

Both can be used together. mq9 only activates its enhanced behavior for messages matching the `$mq9.AI.*` prefix. All other NATS behavior is unchanged. Existing NATS clients need no modification — use the mq9 subjects directly.
