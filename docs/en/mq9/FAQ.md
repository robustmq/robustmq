# FAQ

---

**Do I need a special SDK to use mq9?**

No. Any NATS client works directly — Go, Python, Rust, JavaScript, Java, .NET, or the NATS CLI. mq9 is a subject naming convention layered on top of NATS. The RobustMQ SDK adds typed wrappers and async patterns, but it is entirely optional. If you can send a NATS request or publish to a subject, you can use mq9.

---

**What happens if the recipient is offline when I send a message?**

The message is written to storage immediately on the server. When the recipient subscribes — even minutes or hours later — all non-expired messages are pushed immediately in priority order. Store-first delivery is guaranteed regardless of whether the sender and receiver are online at the same time.

---

**What is the difference between `list` and `subscribe`?**

`list` (`MAILBOX.LIST`) returns a metadata snapshot: `msg_id`, `priority`, and `ts` — **no payload**. Use it to inspect queue depth or retrieve specific `msg_id` values for targeted deletion. `subscribe` (`MAILBOX.MSG.*`) delivers full messages including payload, continuously, as they arrive. The two are complementary: `list` to inspect, `subscribe` to consume.

---

**Can I update a mailbox's TTL after creation?**

No. TTL is fixed at creation time. CREATE is idempotent — calling it again with the same name returns success but does not change the existing TTL or reset the expiration clock. To change a TTL, the mailbox must be allowed to expire and then recreated with the new value.

---

**What happens when a mailbox expires?**

The mailbox and all its messages are automatically destroyed. No client-side cleanup is needed. If a subscriber is currently connected to that mailbox's subject, it is silently disconnected. There is no notification sent to subscribers on expiration.

---

**Can multiple agents write to the same mailbox?**

Yes. Any agent that knows the `mail_id` can publish to it. There is no sender allowlist or ownership restriction. For private mailboxes, access control is achieved by keeping the `mail_id` secret. For public mailboxes, any agent that knows the name can publish.

---

**What does subscribing twice to the same mailbox do?**

Each subscription independently receives all non-expired messages. There is no deduplication across subscriptions — both will receive the same messages. If you want each message delivered to exactly one subscriber, use a queue group (`--queue <group-name>` in the NATS CLI, or the equivalent in your client library).

---

**How does priority work when a subscriber reconnects?**

On subscribe, the server pushes all stored messages in priority order: `critical` first, then `urgent`, then `normal`. Within the same priority level, FIFO order is preserved. This means a reconnecting agent receives its most critical messages first regardless of the order in which they were originally sent.

---

**Is mq9 a replacement for MQTT or Kafka?**

No. mq9 is purpose-built for AI Agent async communication. MQTT is the right choice for IoT telemetry and device messaging. Kafka is the right choice for high-throughput event streaming and data pipelines. mq9 solves the mailbox problem: ephemeral agents, offline-tolerant delivery, lightweight TTL lifecycle. All three protocols can run simultaneously on one RobustMQ deployment with zero bridging required.

---

**How large can a message payload be?**

mq9 inherits NATS message size limits. The default maximum payload is 8 MB per message, which is configurable in the RobustMQ server settings. For large binary transfers (models, datasets, files), store the data externally and pass a reference URL or object key in the mq9 message payload.

---

**What is the difference between private and public mailboxes?**

Private mailboxes use a server-generated UUID as the `mail_id`. Because the ID is not guessable, only agents that were explicitly given the `mail_id` can send to or subscribe from it. Public mailboxes use a user-defined name as the `mail_id` — any agent that knows the name can interact. Use private mailboxes for point-to-point communication between known parties; use public mailboxes for shared task queues, service endpoints, and capability announcements.

---

**Can I use mq9 without RobustMQ? Can I use it with a plain NATS server?**

No. mq9 store-first semantics, priority ordering, TTL auto-cleanup, and `PUBLIC.LIST` are implemented inside the RobustMQ server. A plain NATS server does not support any of these features. The NATS client library is used as the transport layer, but the server must be RobustMQ.

---

**What error codes should I handle?**

| Code | Meaning | When it occurs |
|------|---------|----------------|
| 400 | Bad request | Missing required fields (e.g. `name` for a public mailbox CREATE) |
| 403 | Forbidden | Wildcard `mail_id` subscription (`$mq9.AI.MAILBOX.MSG.*.*`) |
| 404 | Not found | Mailbox or message does not exist |
| 409 | Conflict | Public mailbox name already taken by a different mailbox |
| 410 | Gone | Mailbox TTL has expired |

---

**How is mq9 different from NATS JetStream?**

JetStream adds stream persistence to NATS — it is a full Kafka-comparable system with named streams, durable consumers, message sequences, and replay. mq9 is lighter: mailbox TTL, three-level priority, store-first delivery, and no stream or consumer concepts. JetStream is the better fit for large-scale event sourcing, audit logs, and offset-based replay. mq9 is the better fit for ephemeral agent-to-agent messaging where TTL lifecycle and minimal setup matter more than offset tracking or stream replay.
