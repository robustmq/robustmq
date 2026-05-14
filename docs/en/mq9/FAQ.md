# FAQ

## Do I need a special SDK to use mq9?

No. Any NATS client works out of the box — Go, Python, Rust, JavaScript, Java, .NET, or the NATS CLI. mq9 is a subject naming convention defined on top of NATS; all operations use NATS request/reply. The RobustMQ SDK provides typed wrappers and async patterns, but is entirely optional.

---

## What happens if the recipient is offline when I send a message?

The message is written to server-side storage immediately. The recipient can call FETCH at any time — even minutes or hours later — and all non-expired messages are returned in priority order. Messages are not lost due to the recipient being offline.

---

## What is the difference between FETCH and QUERY?

`FETCH` (`$mq9.AI.MSG.FETCH.*`) is a consumption operation. When used with `group_name`, the broker records the consumption offset; ACK advances it, and the next FETCH resumes from where the last one left off — no duplicate delivery.

`QUERY` (`$mq9.AI.MSG.QUERY.*`) is an inspection operation. It returns messages currently stored in the mailbox **without affecting the consumption offset**. Results can be filtered by key, tags, or since timestamp. Two consecutive QUERYs return the same result (assuming no new messages), making it suitable for debugging and state inspection.

---

## Can I change a mailbox's TTL after creation?

No. TTL is fixed at creation time and cannot be changed or renewed. Calling CREATE again with the same name returns an error (`mailbox xxx already exists`). To change the TTL, wait for the mailbox to expire and create a new one with the desired value.

---

## What happens when a mailbox expires?

The mailbox and all its messages are automatically destroyed with no client-side cleanup required. Consumer group offsets associated with the mailbox are also cleared. No notification is sent to clients when a mailbox expires.

---

## Can multiple Agents write to the same mailbox?

Yes. Any Agent that knows the `mail_address` can send messages — there is no sender allowlist or ownership restriction. Private mailboxes rely on keeping the `mail_address` confidential for access control; public mailboxes can be written to by any Agent that knows the name.

---

## How do multiple workers compete to consume the same mailbox?

Multiple workers use the **same `group_name`** when calling FETCH. The broker ensures each message is picked up by exactly one worker (via offset advancement). Each worker calls FETCH independently, processes the message, and ACKs — after which the broker advances the offset so no other worker receives the same message.

Workers can join or leave at any time; the offset is maintained by the broker with no client-side coordination needed.

---

## Which msg_id should I pass to ACK?

Pass the `msg_id` of the **last message** in the current FETCH batch. The broker advances the consumer group offset to this msg_id; the next FETCH resumes from here. There is no need to ACK each message individually — a single ACK confirms the entire batch.

---

## How does priority work after a reconnect?

**Stateful consumption** (with `group_name`): on reconnect, FETCH resumes from the last ACK position and returns unconsumed messages in priority order (`critical` → `urgent` → `normal`), FIFO within each level.

**Stateless consumption** (without `group_name`): each FETCH applies the `deliver` policy independently; no offset is recorded.

---

## Is mq9 a replacement for MQTT or Kafka?

No. mq9 is designed specifically for AI Agent async communication. MQTT is the right choice for IoT telemetry and device messaging. Kafka is the right choice for high-throughput event streams and data pipelines. mq9 solves the Agent mailbox problem: ephemeral channels, offline-tolerant delivery, lightweight TTL lifecycle. All three protocols can run on the same RobustMQ deployment with zero bridging.

---

## How large can a message payload be?

There is currently no hard limit. For very large binary transfers (models, datasets, files), the recommended pattern is to store the data in an external object store and pass a reference URL or object key in the mq9 message body — keeping messages lightweight.

---

## Can mq9 be used without RobustMQ? What about a regular NATS server?

No. mq9's message persistence, priority ordering, TTL auto-cleanup, consumption offset management, and Agent registry are all implemented inside the RobustMQ server. A regular NATS server does not support these features. NATS client libraries are used as the transport layer, but the server must be RobustMQ.

---

## What errors should I handle?

All responses include an `error` field. An empty string means success; a non-empty string is the error description. Common errors:

| Error message | Cause |
|--------------|-------|
| `mailbox xxx already exists` | CREATE called with a name that already exists |
| `mailbox not found` | Mailbox does not exist or has expired |
| `message not found` | The specified msg_id does not exist or has expired |
| `invalid mail_address` | mail_address format is invalid (uppercase, hyphens, etc.) |
| `agent not found` | UNREGISTER or REPORT called with an unregistered Agent name |

---

## What is the difference between mq9 and NATS JetStream?

JetStream adds streaming persistence to NATS — it is a full Kafka-like system with named streams, durable consumers, message sequences, and replay. mq9 is optimized for Agent workloads: FETCH+ACK pull consumption, three-tier priority, message attributes (key/tags/delay/ttl), and a built-in Agent registry — without streams or stream management. JetStream is better suited for large-scale event sourcing, audit logging, and offset-based replay; mq9 is better suited for lightweight Agent-to-Agent async communication where TTL lifecycle and zero configuration matter more than complex stream management.
