# Protocol Design

---

## 1. Protocol Foundation

mq9 is a semantic convention defined on top of NATS subjects, running on RobustMQ's unified storage layer. All mq9 operations are standard NATS pub/sub or request/reply calls — no extra protocol layer or SDK required.

### Why NATS

mq9 chose NATS as its transport protocol rather than building a custom one, for three reasons:

- **Complete client ecosystem**: NATS has official and community clients covering 40+ languages. Python, Go, JavaScript, and Rust — the languages most common in AI — all have mature implementations. Choosing NATS means mq9 works out of the box for developers in all these languages from day one, with no need to wait for SDK coverage.

- **Semantic fit**: NATS pub/sub and request/reply primitives cover all the communication patterns mq9 needs — fire-and-forget publishing, active subscription, synchronous requests. The subject namespace mechanism provides a natural way to express mq9's mailbox address system.

- **Broker is fully self-developed**: Using the NATS protocol does not mean using NATS Server. mq9's Broker is implemented by RobustMQ in Rust, running on RobustMQ's unified storage layer. Storage, priority scheduling, TTL management, and store-first delivery semantics are all RobustMQ's own capabilities. NATS is only the communication protocol between the client and the Broker — like HTTP is to the Web.

| Operation type | NATS primitive | Description |
|---------------|---------------|-------------|
| Create mailbox | `nats req` (request/reply) | Client sends a request; server returns result via reply-to |
| Send message | `nats pub` (fire-and-forget) | Fire and forget; no acknowledgement required |
| Subscribe | `nats sub` (subscribe) | Subscription triggers full replay, then real-time push |
| List messages | `nats req` (request/reply) | Query message metadata in a mailbox |
| Delete message | `nats req` (request/reply) | Delete a specific message |

Message encoding: request and response bodies are JSON, UTF-8. Standard NATS connection, default port `4222`.

---

## 2. Subject Naming

All mq9 subjects use `$mq9.AI` as a prefix. The server enables persistence and priority scheduling for messages under this prefix; all other NATS subjects are unaffected.

```text
$mq9.AI
  └── MAILBOX
        ├── CREATE                                   # Create a mailbox
        ├── MSG.{mail_address}                            # Send/subscribe (normal, default, no suffix)
        ├── MSG.{mail_address}.urgent                     # Send/subscribe urgent messages
        ├── MSG.{mail_address}.critical                   # Send/subscribe highest-priority messages
        ├── MSG.{mail_address}.*                          # Subscribe to all priorities
        ├── LIST.{mail_address}                           # List mailbox message metadata
        └── DELETE.{mail_address}.{msg_id}                # Delete a specific message
```

| Subject | NATS primitive | Description |
|---------|---------------|-------------|
| `$mq9.AI.MAILBOX.CREATE` | request | Create a private or public mailbox |
| `$mq9.AI.MAILBOX.MSG.{mail_address}` | pub/sub | Send/subscribe normal priority messages (default, no suffix) |
| `$mq9.AI.MAILBOX.MSG.{mail_address}.urgent` | pub/sub | Send/subscribe urgent priority messages |
| `$mq9.AI.MAILBOX.MSG.{mail_address}.critical` | pub/sub | Send/subscribe critical priority messages |
| `$mq9.AI.MAILBOX.MSG.{mail_address}.*` | sub | Subscribe to **all** priorities (critical + urgent + normal) |
| `$mq9.AI.MAILBOX.LIST.{mail_address}` | request | List metadata for all messages in a mailbox |
| `$mq9.AI.MAILBOX.DELETE.{mail_address}.{msg_id}` | request | Delete a specific message from a mailbox |
| `$mq9.AI.PUBLIC.LIST` | sub | Discover all public mailboxes, system-managed |

### mail_address Format

Private mailbox `mail_address` values are server-generated, in the format `mail-{nanoid}-{nanoid}`, for example:

```text
mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag
```

Public mailbox `mail_address` values are user-defined via the `name` field at creation time. Dot-separated names are supported, e.g. `task.queue`, `vision.results`.

`mail_address` unguessability is the security boundary: knowing the `mail_address` lets you send and subscribe; without it, there is no way to interact with the mailbox. No token, no ACL.

---

## 3. Core Concepts

**Mailbox** — mq9's fundamental communication address. Created via `MAILBOX.CREATE`, which returns a `mail_address`. Not bound to Agent identity — one Agent can create different mailboxes for different tasks.

**TTL** — The lifetime declared at mailbox creation. When it expires, the mailbox and all its messages are automatically destroyed with no manual cleanup needed. `MAILBOX.CREATE` is idempotent: calling it again with the same name silently returns success; TTL is fixed by the first creation.

**Priority** — Every message belongs to one of three priority levels, encoded in the subject suffix:

| Priority | Subject form | Typical use |
|----------|-------------|-------------|
| `critical` | `MSG.{mail_address}.critical` | Abort signals, emergency commands, security events |
| `urgent` | `MSG.{mail_address}.urgent` | Task interrupts, time-sensitive instructions |
| `normal` (default) | `MSG.{mail_address}` (no suffix) | Task dispatch, result delivery, routine communication |

Same-priority messages are FIFO; across priorities, critical comes before urgent before normal. Ordering is guaranteed by the storage layer — consumers need not sort themselves.

**Store-first, then push** — Messages are written to storage on arrival. If a subscriber is online, they are also pushed in real time. If offline, messages wait in storage. Each new subscription triggers a full replay of all non-expired messages, then switches to real-time push. Agents that reconnect never miss messages.

**No server-side consumer state** — The server does not track read/unread status, does not maintain per-consumer position, has no ACK, no offset. Deduplication is the client's responsibility, using `msg_id`.

**msg_id** — A unique identifier (uint64) assigned by the server to each message. Used for client-side deduplication and for DELETE operations.

---

## 4. Commands

### 4.1 Create Mailbox (MAILBOX.CREATE)

Subject: `$mq9.AI.MAILBOX.CREATE` · NATS primitive: request/reply

#### CREATE Request Parameters

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `ttl` | uint64 | No | server default | Mailbox lifetime in seconds; mailbox and messages auto-destroyed on expiry |
| `public` | bool | No | `false` | Whether to make the mailbox public; `true` auto-registers it to `$mq9.AI.PUBLIC.LIST` |
| `name` | string | Required for public | — | Custom `mail_address` for a public mailbox; dot-separated names supported (e.g. `task.queue`) |
| `desc` | string | No | `""` | Mailbox description; recommended for public mailboxes, included in PUBLIC.LIST pushes |

#### CREATE Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `mail_address` | string | Mailbox identifier. Private mailboxes use `mail-{nanoid}-{nanoid}` format; public mailboxes use the `name` value from the request |
| `is_new` | bool | `true` if this call created a new mailbox; `false` if it already existed (idempotent return) |

#### CREATE Examples

Create a private mailbox:

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":3600}'
```

```json
{"mail_address":"mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag","is_new":true}
```

Create a public mailbox:

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 86400,
  "public": true,
  "name": "task.queue",
  "desc": "Shared worker task queue"
}'
```

```json
{"mail_address":"task.queue","is_new":true}
```

Idempotent call (mailbox already exists):

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":86400,"public":true,"name":"task.queue"}'
```

```json
{"mail_address":"task.queue","is_new":false}
```

> CREATE is idempotent. If the mailbox already exists, it returns success with `is_new: false`; the original TTL is preserved. Workers can safely call this at startup without checking whether the mailbox exists.

---

### 4.2 Send Message (MAILBOX.MSG pub)

Subject: `$mq9.AI.MAILBOX.MSG.{mail_address}[.{priority}]` · NATS primitive: publish

Knowing the `mail_address` is enough to send — no authorization required. Messages are written to storage and return immediately; the sender does not wait for subscribers to be online.

#### MSG-pub Subject Encoding

| Priority | Subject |
|----------|---------|
| normal (default) | `$mq9.AI.MAILBOX.MSG.{mail_address}` (no suffix) |
| urgent | `$mq9.AI.MAILBOX.MSG.{mail_address}.urgent` |
| critical | `$mq9.AI.MAILBOX.MSG.{mail_address}.critical` |

> Wildcard publishing is prohibited. Subjects like `$mq9.AI.MAILBOX.MSG.*.*` will be rejected by the server. Publishing requires an exact `mail_address`.

#### MSG-pub Response Fields

Publish is fire-and-forget with no response body. If sent using request mode:

| Field | Type | Description |
|-------|------|-------------|
| `msg_id` | uint64 | Server-assigned unique identifier for this message; used for subsequent DELETE operations |

#### MSG-pub Message Body (Recommended Structure)

mq9 does not enforce message body format — payload is any byte sequence. The following JSON convention is recommended:

| Field | Type | Description |
|-------|------|-------------|
| `from` | string | Sender's `mail_address` |
| `type` | string | Message type, application-defined (e.g. `task_dispatch`, `task_result`, `abort`) |
| `correlation_id` | string | Links to the original request; used in request-reply patterns |
| `reply_to` | string | Reply address; the recipient sends their response to this `mail_address` |
| `payload` | object | Message content; mq9 does not parse this |
| `ts` | int64 | Send timestamp (Unix seconds) |

#### MSG-pub Examples

```bash
# critical — highest priority
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.critical' \
  '{"type":"abort","task_id":"t-001","ts":1712600001}'

# urgent
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.urgent' \
  '{"type":"interrupt","task_id":"t-002","ts":1712600002}'

# normal — default, no suffix
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag' \
  '{"type":"task","payload":{"job":"process dataset A"},"reply_to":"mail-sender-001-abc","ts":1712600003}'
```

---

### 4.3 Subscribe (MAILBOX.MSG sub)

Subject: `$mq9.AI.MAILBOX.MSG.{mail_address}[.{priority}|.*]` · NATS primitive: subscribe

#### MSG-sub Subscription Patterns

| Subject | Meaning |
|---------|---------|
| `$mq9.AI.MAILBOX.MSG.{mail_address}.*` | Subscribe to **all** priorities (critical + urgent + normal) |
| `$mq9.AI.MAILBOX.MSG.{mail_address}.critical` | critical only |
| `$mq9.AI.MAILBOX.MSG.{mail_address}.urgent` | urgent only |
| `$mq9.AI.MAILBOX.MSG.{mail_address}` | normal only (no suffix) |

> `.*` matches all priorities. When the server receives a `.*` subscription, it pushes messages from all three priorities — critical, urgent, and normal — to the subscriber. This is the standard way to subscribe to a full mailbox.

#### MSG-sub Push Semantics

Each subscription triggers the following sequence:

1. All non-expired stored messages are pushed immediately, in order: critical → urgent → normal, FIFO within each priority
2. After the replay completes, the subscription switches to real-time push: new messages are delivered immediately as they arrive

The server does not track consumption state. Re-subscribing replays all non-expired messages from the beginning.

#### MSG-sub Examples

```bash
# Subscribe to all priorities (recommended)
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.*'

# critical only
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.critical'

# urgent only
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.urgent'

# normal only
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag'

# Competing consumers (Queue Group) — each message delivered to exactly one worker
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers
```

---

### 4.4 List Messages (MAILBOX.LIST)

Subject: `$mq9.AI.MAILBOX.LIST.{mail_address}` · NATS primitive: request/reply

View metadata for messages currently stored in a mailbox without consuming them (messages remain in storage).

#### LIST Request Parameters

Request body is an empty JSON object: `{}`

#### LIST Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `mail_address` | string | Mailbox identifier |
| `messages` | array | List of message metadata |

Messages array elements:

| Field | Type | Description |
|-------|------|-------------|
| `msg_id` | uint64 | Server-assigned message identifier; used for DELETE |
| `payload` | string | Message body content (raw string) |
| `priority` | string | Priority level: `critical`, `urgent`, or `normal` |
| `header` | bytes \| null | NATS message headers (optional) |
| `create_time` | uint64 | Message write timestamp (Unix seconds) |

#### LIST Examples

```bash
nats req '$mq9.AI.MAILBOX.LIST.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag' '{}'
```

```json
{
  "mail_address": "mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag",
  "messages": [
    {
      "msg_id": 1001,
      "payload": "{\"type\":\"abort\",\"task_id\":\"t-001\"}",
      "priority": "critical",
      "header": null,
      "create_time": 1712600001
    },
    {
      "msg_id": 1002,
      "payload": "{\"type\":\"interrupt\",\"task_id\":\"t-002\"}",
      "priority": "urgent",
      "header": null,
      "create_time": 1712600002
    },
    {
      "msg_id": 1003,
      "payload": "{\"type\":\"task\",\"payload\":{\"job\":\"process dataset A\"}}",
      "priority": "normal",
      "header": null,
      "create_time": 1712600003
    }
  ]
}
```

---

### 4.5 Delete Message (MAILBOX.DELETE)

Subject: `$mq9.AI.MAILBOX.DELETE.{mail_address}.{msg_id}` · NATS primitive: request/reply

Delete a specific message from mailbox storage. In competing consumer scenarios, workers explicitly delete a message after successfully processing it to avoid redelivery.

#### DELETE Request Parameters

Request body is an empty JSON object: `{}`

The `mail_address` and `msg_id` are encoded in the subject. Parsing rule: the token after the last `.` is the `msg_id`; everything before it is the `mail_address`. For example, `task.queue.1003` parses as `mail_address=task.queue`, `msg_id=1003`.

#### DELETE Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `deleted` | bool | `true` if successfully deleted; `false` if the message does not exist or has already expired |

#### DELETE Examples

```bash
nats req '$mq9.AI.MAILBOX.DELETE.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.1002' '{}'
```

```json
{"deleted":true}
```

Public mailbox (mail_address contains dots):

```bash
nats req '$mq9.AI.MAILBOX.DELETE.task.queue.1003' '{}'
```

```json
{"deleted":true}
```

---

### 4.6 Discover Public Mailboxes (PUBLIC.LIST)

Subject: `$mq9.AI.PUBLIC.LIST` · NATS primitive: subscribe

A system-managed address maintained by the Broker; does not accept user writes. Mailboxes created with `public: true` are automatically registered on creation and removed when their TTL expires.

Subscribing triggers a full push of all current public mailboxes immediately, followed by real-time updates as mailboxes are added or expire.

#### PUBLIC.LIST Push Format

When a mailbox is created:

| Field | Type | Description |
|-------|------|-------------|
| `event` | string | Fixed value: `"created"` |
| `mail_address` | string | Public mailbox identifier |
| `desc` | string | Description provided at creation |
| `ttl` | uint64 | Lifetime in seconds |

When a mailbox expires:

| Field | Type | Description |
|-------|------|-------------|
| `event` | string | Fixed value: `"expired"` |
| `mail_address` | string | Identifier of the expired mailbox |

#### PUBLIC.LIST Examples

```bash
nats sub '$mq9.AI.PUBLIC.LIST'
```

```json
{"event":"created","mail_address":"task.queue","desc":"Shared worker task queue","ttl":86400}
{"event":"created","mail_address":"vision.results","desc":"Vision processing results","ttl":3600}
{"event":"expired","mail_address":"vision.results"}
```

---

## 6. Relationship to Native NATS

mq9 is built on top of RobustMQ's NATS protocol layer, using subject naming conventions and server-side enhancements:

| | NATS Core | mq9 |
|--|----------|-----|
| Persistence | None — messages lost if subscriber is offline | TTL-bounded persistence, priority-tiered storage |
| Consumer state | None | None (by design) |
| Message ordering | Not guaranteed across subjects | Within a mail_address: priority-ordered; within a priority: FIFO |
| Access method | Any NATS client | Any NATS client; subjects follow the `$mq9.AI.*` convention |

mq9 only enables enhanced logic for subjects under the `$mq9.AI.*` prefix; all other NATS behavior is unchanged. Existing NATS clients need no modification — use mq9 subjects directly.
