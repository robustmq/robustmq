# Core Features

## Overview

From a functional perspective, mq9 looks like a mailbox to an Agent — a mailbox with advanced capabilities.

Agents don't need to think about protocol details. Just treat it like email: claim a mailbox address, send messages to other addresses, subscribe to your own address to receive messages. Everything else — no message loss, priority ordering, automatic expiry cleanup — is handled by mq9 in the background.

mq9 adds three capabilities on top of a basic mailbox:

- **TTL**: Mailboxes have a lifecycle; they auto-destroy on expiry with no manual cleanup needed
- **Priority**: Three message levels (critical / urgent / normal); urgent messages are delivered first
- **Public mailboxes**: Support for discoverable addresses that any Agent can find and send to — ideal for task queues and broadcast scenarios

These three capabilities cover the vast majority of communication patterns in multi-Agent systems. Details below.

---

## Feature Reference

### 1. Mailbox Types

mq9 provides two mailbox types. The type is chosen at creation time and cannot be changed.

| | Private | Public |
|---|---|---|
| `mail_address` | Server-generated UUID (not guessable) | User-defined string (e.g. `task.queue`) |
| Discovery | Not discoverable — only parties who already know the `mail_address` can interact | Automatically registered to `$mq9.AI.PUBLIC.LIST`; discoverable by anyone |
| Use case | Point-to-point messaging, task result delivery, private replies | Task queues, broadcast channels, capability announcements |

**Security model:** The unguessability of the `mail_address` is the only access control boundary. There are no bearer tokens, no ACL entries, no auth headers. A party that knows the `mail_address` can send messages and subscribe; a party that does not know it cannot interact with the mailbox in any way. For private mailboxes, treat the `mail_address` as a secret shared only with intended participants.

Public mailboxes trade that opacity for discoverability. The name is the address — choose it to be meaningful and self-describing (e.g. `vision.results`, `task.queue`) rather than opaque.

---

### 2. Priority System

Every message is sent to one of three priority levels: `critical`, `urgent`, or `normal` (default, no suffix). Priority is encoded in the subject:

```
$mq9.AI.MAILBOX.MSG.{mail_address}.critical   # highest priority
$mq9.AI.MAILBOX.MSG.{mail_address}.urgent     # urgent
$mq9.AI.MAILBOX.MSG.{mail_address}            # default (normal), no suffix
```

**Ordering guarantees:**

- Within the same priority level: FIFO — messages are delivered in the order they were sent.
- Across priority levels: `critical` is delivered before `urgent`, which is delivered before `normal`. This ordering is enforced by the storage layer; subscribers receive messages in correct order without any client-side sorting.

**Storage per priority level:**

| Priority | Storage backend | Persistence |
|----------|----------------|-------------|
| `critical` | RocksDB | Persisted — survives broker restarts |
| `urgent` | RocksDB | Persisted — survives broker restarts |
| `normal` (default) | Memory | Not persisted — lost on broker restart; resend if necessary |

**Practical guidance:**

- `critical` — Abort signals, emergency commands, security events. These must arrive first and must not be lost.
- `urgent` — Task interrupts, time-sensitive instructions. These need prioritized delivery and must not be lost.
- `normal` (default) — Task dispatch, result delivery, approval requests. The default level for most Agent-to-Agent communication; acceptable to lose on restart.

---

### 3. Store-First Delivery

mq9's delivery model differs from standard pub/sub. The sequence for every incoming message is:

1. Message arrives at the broker.
2. Written to storage (RocksDB or Memory, per priority).
3. If any subscribers are currently connected: also pushed to them in real time.
4. If no subscribers are connected: message waits in storage.
5. When a subscriber connects: all non-expired stored messages are pushed immediately, in priority order, then real-time delivery continues.

This has two important consequences:

**Offline agents do not lose messages.** An Agent that is restarting, updating, or temporarily unreachable will receive all messages sent during its absence the moment it reconnects and subscribes. No polling, no replay request, no special recovery path.

**Subscribing is equivalent to querying.** There is no separate QUERY or FETCH command. Every subscription begins by delivering the full set of non-expired stored messages. The server tracks no read/unread state and no per-consumer position. Subscribing again from a new connection replays all non-expired messages from the beginning.

**Comparison with related systems:**

| System | Persistence | Consumer state | Replay model |
|--------|------------|----------------|--------------|
| NATS Core | None — offline messages are lost | None | None |
| NATS JetStream | Full stream persistence | Per-consumer offsets, consumer groups, acknowledgments | Configurable replay from any offset |
| mq9 | TTL-bounded persistence per mailbox | None | All non-expired messages on every subscribe |

mq9 occupies the space between Core and JetStream. It adds enough persistence to handle the offline Agent case without introducing the stream, consumer, offset, and acknowledgment machinery that JetStream requires. The tradeoff is that mq9 has no "pick up where I left off" semantics — see [No Server-Side Consumer State](#7-no-server-side-consumer-state) for details.

---

### 4. TTL and Lifecycle

TTL (time-to-live) is the sole lifecycle mechanism for mailboxes. It is declared at creation:

```json
{"ttl": 3600, "public": false}
```

**Behavior:**

- TTL starts counting from the moment the mailbox is created.
- When TTL expires: the mailbox is automatically destroyed and all stored messages are cleaned up. No manual intervention required.
- There is no DELETE mailbox command. The intended pattern is: create a mailbox for a task, use it, and ignore it — TTL handles the cleanup.
- TTL cannot be changed after creation. It is fixed by the first successful `MAILBOX.CREATE` call.

**CREATE is idempotent.** If you call `MAILBOX.CREATE` with a name that already exists (relevant for public mailboxes), the server returns success without modifying the existing mailbox or resetting its TTL. This design supports two patterns:

- **Ensure-before-send:** Workers call CREATE at startup to guarantee the mailbox exists before publishing. If the mailbox was created by a previous worker run, this is a no-op.
- **Reconnect resilience:** Agents can call CREATE on reconnect without risk of overwriting a live mailbox or its messages.

**Design intent:** mq9 mailboxes are designed to be ephemeral. Create one per task, per session, or per Agent instance. When the task ends, do nothing — the mailbox expires. This avoids the operational burden of maintaining a registry of live mailboxes or issuing cleanup commands.

---

### 5. Competitive Consumption (Queue Groups)

Multiple subscribers can compete for messages on the same mailbox by joining a queue group. Each message is delivered to exactly one member of the group.

```bash
# Worker 1 (receives all priority messages)
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers

# Worker 2
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers
```

All subscribers using the same queue group name (`workers` above) share message delivery. The broker routes each message to one member using load balancing. The group name is arbitrary — any string works.

**Dynamic membership:** Workers can join or leave the queue group at any time. The broker adjusts routing immediately with no configuration change or coordinator involvement.

**Crash tolerance:** Because mq9 uses store-first delivery, a message is not removed from storage by the act of delivery alone. If a worker receives a message and crashes before it deletes the message, the message remains in storage. It will be re-delivered when any group member reconnects. Workers that require at-least-once processing should explicitly delete (`$mq9.AI.MAILBOX.DELETE.{mail_address}.{msg_id}`) the message after successful completion.

**Recommended pattern:** Combine a public mailbox with a queue group for a zero-config distributed task queue. The mailbox name serves as the queue address, discoverable via `$mq9.AI.PUBLIC.LIST`. Workers subscribe with `--queue` on startup. No queue configuration, no broker-side consumer group definition, no coordinator.

---

### 6. Idempotent Create

`MAILBOX.CREATE` is safe to call multiple times for the same mailbox name. The behavior:

- If the mailbox does not exist: created with the specified TTL.
- If the mailbox already exists: returns success. The original TTL is preserved, not reset.

This property makes CREATE safe to use in the following scenarios:

- **Worker startup initialization:** Each worker instance calls CREATE for its public task queue mailbox at startup. Only the first call creates it; subsequent calls are no-ops. No coordination between workers needed.
- **Reconnect after disconnect:** An Agent that reconnects can re-issue CREATE for its mailboxes without risk of losing stored messages or shortening TTL.
- **Multiple producers:** Several Agents sending to the same public mailbox may each call CREATE independently before sending. The first caller creates the mailbox; the rest observe the no-op. All producers can proceed without knowing which one "owns" the mailbox.

Note that idempotency applies to the mailbox identity, not the TTL value. If a second CREATE call specifies a different TTL, the original TTL is still preserved. To change TTL, let the mailbox expire and create a new one.

---

### 7. No Server-Side Consumer State

The mq9 server tracks zero consumer state. There are no offsets, no consumer groups, no acknowledgment sequences, and no "last delivered" pointers.

**What this means in practice:**

- Every subscription delivers all non-expired messages from the beginning of the mailbox's history.
- Two separate connections subscribing to the same mailbox both receive the full message set.
- There is no way to subscribe and receive only "new" messages — every subscriber starts from the oldest non-expired message.
- Unsubscribing and resubscribing replays all non-expired messages again.

**Tradeoff:** The protocol is significantly simpler. There is no negotiation of consumer position, no acknowledgment flow, and no server-side bookkeeping that scales with the number of consumers. This matches the target workload: AI Agents typically process all messages in a mailbox when they wake up, rather than needing to resume from a specific point.

**For deduplication:** If a subscriber must avoid processing the same message twice across reconnects, use the per-message `msg_id` field. The application tracks which `msg_id` values it has already processed; on reconnect, it skips messages with known `msg_id`s. The server provides `msg_id` as part of every delivered message; the tracking responsibility lies with the client.

This is the explicit counterpart to JetStream's consumer offset model: simpler server, slightly more work for clients that need deduplication.
