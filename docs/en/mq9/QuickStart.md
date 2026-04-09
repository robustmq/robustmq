# Quick Start

This guide walks you through mq9's core operations against the public demo server using the NATS CLI. No account, no configuration, no SDK — just a terminal.

---

## Prerequisites

Install the [NATS CLI](https://docs.nats.io/using-nats/nats-tools/nats_cli). It is the only tool required to interact with mq9.

---

## Connect to the Public Server

The RobustMQ demo server is available at:

```
nats://demo.robustmq.com:4222
```

This is a shared environment. Anyone with the subject name can subscribe to it, so do not send sensitive data. All examples below connect to this server implicitly — pass `-s nats://demo.robustmq.com:4222` to each command, or set the `NATS_URL` environment variable once:

```bash
export NATS_URL=nats://demo.robustmq.com:4222
```

---

## Create a Mailbox

A mailbox is the fundamental communication address in mq9. Use `nats req` (request/reply) to create one — the server returns the assigned `mail_id` via NATS reply-to:

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":60}'
```

Response:

```json
{"mail_id":"m-7f3a1c9e2b"}
```

The `mail_id` is the only access credential. Anyone who knows it can send messages to or subscribe from this mailbox. Keep it private for private communication.

TTL is set to 60 seconds here for demo convenience. In production, choose a TTL that matches your task's expected lifetime — the mailbox and all its messages are automatically destroyed when TTL expires, with no manual cleanup required.

---

## Send Messages at Different Priorities

mq9 supports three priority levels: `critical`, `urgent`, and `normal` (default, no suffix). The subject pattern for sending is:

```
$mq9.AI.MAILBOX.MSG.{mail_id}.{priority}   # for urgent or critical
$mq9.AI.MAILBOX.MSG.{mail_id}              # for normal (default, no suffix)
```

Replace `m-7f3a1c9e2b` with the `mail_id` from the previous step:

```bash
# Critical — highest priority, processed first; use for abort signals, emergency commands, security events
nats pub '$mq9.AI.MAILBOX.MSG.m-7f3a1c9e2b.critical' '{"type":"abort","task_id":"t-001"}'

# Urgent — use for task interrupts, time-sensitive instructions
nats pub '$mq9.AI.MAILBOX.MSG.m-7f3a1c9e2b.urgent'   '{"type":"interrupt","task_id":"t-002"}'

# Normal (default, no suffix) — routine communication; use for task dispatch and result delivery
nats pub '$mq9.AI.MAILBOX.MSG.m-7f3a1c9e2b' '{"type":"task","payload":"process dataset A"}'
```

Sending is fire-and-forget (`nats pub`). The server writes each message to storage immediately. The sender does not wait for the subscriber to be online.

> **Wildcard publishing is prohibited.** The subject `$mq9.AI.MAILBOX.MSG.*.*` is rejected by the server. You must specify an exact `mail_id` when sending.

---

## Subscribe and Receive

Subscribe to receive messages:

```bash
# Subscribe to all priorities (critical, urgent, and normal)
nats sub '$mq9.AI.MAILBOX.MSG.m-7f3a1c9e2b.*'
```

All messages sent above are pushed immediately upon subscribing — regardless of whether the subscriber was online when they were sent. This is mq9's **store-first semantics**: every subscription begins by delivering all non-expired stored messages in priority order (critical before urgent before normal, FIFO within the same level), then switches to real-time delivery for new messages.

This means there is no difference between subscribing before or after messages are sent. An Agent that reconnects after a network outage receives everything it missed automatically.

To subscribe to a single priority level only:

```bash
nats sub '$mq9.AI.MAILBOX.MSG.m-7f3a1c9e2b.urgent'
nats sub '$mq9.AI.MAILBOX.MSG.m-7f3a1c9e2b.critical'
```

---

## List Message Metadata

To inspect what messages are currently stored in a mailbox without consuming them, send a request to the LIST subject:

```bash
nats req '$mq9.AI.MAILBOX.LIST.m-7f3a1c9e2b' '{}'
```

Response:

```json
{
  "messages": [
    {"msg_id": "msg-001", "priority": "critical", "ts": 1712600001},
    {"msg_id": "msg-002", "priority": "urgent",   "ts": 1712600002},
    {"msg_id": "msg-003", "priority": "normal",   "ts": 1712600003}
  ]
}
```

The response returns `msg_id`, `priority`, and `ts` (Unix timestamp) for each stored message. Payloads are not included — LIST is for inspection, not retrieval. Use the `msg_id` values to target specific messages for deletion.

---

## Delete a Message

To remove a specific message from storage before its mailbox TTL expires:

```bash
nats req '$mq9.AI.MAILBOX.DELETE.m-7f3a1c9e2b.msg-002' '{}'
```

This is useful in competitive consumption workflows where a worker wants to explicitly acknowledge completion by removing the task message. The subject pattern is:

```
$mq9.AI.MAILBOX.DELETE.{mail_id}.{msg_id}
```

---

## Create a Public Mailbox

A public mailbox has a user-defined `mail_id` — the name you choose becomes the address. Use this for shared task queues or capability announcements where multiple parties need to discover the address without out-of-band coordination.

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{
  "ttl": 3600,
  "public": true,
  "name": "task.queue",
  "desc": "Shared worker task queue"
}'
```

Response:

```json
{"mail_id":"task.queue"}
```

The `mail_id` is exactly the `name` you provided. The mailbox is automatically registered to `$mq9.AI.PUBLIC.LIST` and becomes discoverable by anyone subscribing to that system address. When the TTL expires, it is automatically removed from the list.

CREATE is idempotent: if a mailbox named `task.queue` already exists, this call returns success without resetting its TTL. This makes it safe to call at worker startup without worrying about overwriting an existing mailbox.

---

## Queue Group (Competitive Consumption)

When multiple workers subscribe to the same mailbox with the same queue group name, mq9 delivers each message to exactly one worker. Open two terminals and run the same command in each:

**Terminal 1:**

```bash
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers
```

**Terminal 2:**

```bash
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers
```

Now send several messages:

```bash
nats pub '$mq9.AI.MAILBOX.MSG.task.queue' '{"task":"job-1"}'
nats pub '$mq9.AI.MAILBOX.MSG.task.queue' '{"task":"job-2"}'
nats pub '$mq9.AI.MAILBOX.MSG.task.queue' '{"task":"job-3"}'
```

Each message appears in exactly one terminal — the broker distributes them across the queue group. Workers can join or leave the group at any time; the queue adjusts automatically with no configuration change.

Because mq9 uses store-first delivery, if a worker crashes before deleting a task message, the message remains in storage and is re-delivered when any group member reconnects. Combine a public mailbox with a queue group for a zero-config, crash-tolerant task queue.

---

## Next Steps

- **Protocol** — Full subject reference, request parameters, message structure, and storage tier details: [Protocol](./Protocol.md)
- **Features** — Deep dive into priority semantics, store-first delivery, TTL lifecycle, and competitive consumption: [Features](./Features.md)
- **Overview** — Design rationale, positioning relative to NATS Core and JetStream, and eight canonical Agent scenarios: [Overview](./Overview.md)
