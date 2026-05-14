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

This is a shared environment. Anyone with the subject name can interact with it, so do not send sensitive data. All examples below connect to this server — pass `-s nats://demo.robustmq.com:4222` to each command, or set the `NATS_URL` environment variable once:

```bash
export NATS_URL=nats://demo.robustmq.com:4222
```

---

## Create a Mailbox

A mailbox is the fundamental communication address in mq9. Use `nats request` (request/reply) to create one — the server returns the assigned `mail_address` via NATS reply-to:

```bash
nats request '$mq9.AI.MAILBOX.CREATE' '{"name":"quickstart.demo","ttl":300}'
```

Response:

```json
{"error":"","mail_address":"quickstart.demo"}
```

The `mail_address` is the only access credential. Anyone who knows it can send messages to or fetch messages from this mailbox. Keep it private for private communication.

TTL is set to 300 seconds here for demo convenience. In production, choose a TTL that matches your task's expected lifetime — the mailbox and all its messages are automatically destroyed when TTL expires, with no manual cleanup required.

---

## Send Messages

Send messages to the mailbox, specifying priority via the `mq9-priority` header:

```bash
# Critical — highest priority, processed first; use for abort signals, emergency commands
nats request '$mq9.AI.MSG.SEND.quickstart.demo' \
  --header 'mq9-priority:critical' \
  '{"type":"abort","task_id":"t-001"}'

# Urgent — use for task interrupts, time-sensitive instructions
nats request '$mq9.AI.MSG.SEND.quickstart.demo' \
  --header 'mq9-priority:urgent' \
  '{"type":"interrupt","task_id":"t-002"}'

# Normal (default, no header) — routine communication; use for task dispatch, result delivery
nats request '$mq9.AI.MSG.SEND.quickstart.demo' \
  '{"type":"task","payload":"process dataset A"}'
```

Each send call returns a response including the assigned `msg_id`:

```json
{"error":"","msg_id":1}
```

---

## Fetch Messages (FETCH)

mq9 uses **pull mode**: clients actively call FETCH to retrieve messages, rather than passively waiting for a push.

```bash
nats request '$mq9.AI.MSG.FETCH.quickstart.demo' '{
  "group_name": "my-worker",
  "deliver": "earliest",
  "config": {"num_msgs": 10}
}'
```

The response contains messages sorted by priority (critical → urgent → normal, FIFO within each level):

```json
{
  "error": "",
  "messages": [
    {"msg_id": 1, "payload": "{\"type\":\"abort\",...}", "priority": "critical", "create_time": 1712600001},
    {"msg_id": 2, "payload": "{\"type\":\"interrupt\",...}", "priority": "urgent", "create_time": 1712600002},
    {"msg_id": 3, "payload": "{\"type\":\"task\",...}", "priority": "normal", "create_time": 1712600003}
  ]
}
```

When `group_name` is provided, the broker records the consumption offset. The next FETCH resumes from the last ACK position — no duplicate delivery.

---

## Acknowledge Messages (ACK)

After processing a message, call ACK to advance the consumer group's offset:

```bash
nats request '$mq9.AI.MSG.ACK.quickstart.demo' '{
  "group_name": "my-worker",
  "mail_address": "quickstart.demo",
  "msg_id": 3
}'
```

Response:

```json
{"error":""}
```

The next FETCH after this ACK will return only messages after `msg_id: 3` — already-ACKed messages are not re-delivered.

---

## Query Messages (QUERY)

QUERY inspects messages currently stored in the mailbox without affecting the consumption offset:

```bash
# Full scan
nats request '$mq9.AI.MSG.QUERY.quickstart.demo' '{}'

# Filter by tags (requires mq9-tags header when sending)
nats request '$mq9.AI.MSG.QUERY.quickstart.demo' '{"tags":["urgent"]}'

# By time range with limit
nats request '$mq9.AI.MSG.QUERY.quickstart.demo' '{"since":1712600000,"limit":20}'
```

---

## Delete a Message

To remove a specific message from storage before its mailbox TTL expires:

```bash
nats request '$mq9.AI.MSG.DELETE.quickstart.demo.2' '{}'
```

Subject pattern: `$mq9.AI.MSG.DELETE.{mail_address}.{msg_id}`

---

## Agent Registry and Discovery

mq9 has a built-in Agent registry for publishing and searching Agent capabilities.

**Register an Agent:**

```bash
nats request '$mq9.AI.AGENT.REGISTER' '{
  "name": "demo.translator",
  "payload": "Multilingual translation agent; supports EN/ZH/JA/KO; returns results in real time"
}'
```

**Search by semantic intent:**

```bash
nats request '$mq9.AI.AGENT.DISCOVER' '{
  "semantic": "I need to translate Chinese text into English",
  "limit": 5
}'
```

**Search by keyword:**

```bash
nats request '$mq9.AI.AGENT.DISCOVER' '{
  "text": "translator",
  "limit": 10
}'
```

**Unregister an Agent:**

```bash
nats request '$mq9.AI.AGENT.UNREGISTER' '{"name":"demo.translator"}'
```

---

## Next Steps

- **Protocol** — Full subject reference, request parameters, and message structure: [Protocol Design](./Protocol.md)
- **Features** — Deep dive into FETCH+ACK consumption, priority, message attributes, and TTL lifecycle: [Features](./Features.md)
- **Overview** — Design rationale and canonical Agent scenarios: [Overview](./Overview.md)
