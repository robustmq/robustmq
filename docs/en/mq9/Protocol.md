# mq9 Protocol

## Protocol Foundation

mq9 is built on the NATS protocol. All mq9 operations are standard NATS pub/sub/req-reply calls, with Subject names following the `$mq9.` prefix convention.

Any NATS client library can use mq9 directly — no additional SDK required.

## Subject Naming

```
$mq9.AI.<operation>.<params...>
```

| Subject | Type | Description |
|---------|------|-------------|
| `$mq9.AI.MAILBOX.CREATE` | req/reply | Create an Agent identity and mailbox |
| `$mq9.AI.INBOX.<agent_id>.<priority>` | pub/sub | Deliver a message to an Agent's mailbox |
| `$mq9.AI.BROADCAST.<domain>.<event>` | pub/sub | Broadcast an event |
| `$mq9.AI.STATUS.<agent_id>` | pub/sub | Agent status reporting |

## Mailbox Protocol

### Create a Mailbox

On first use, an Agent requests an identity via req/reply:

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{}'
```

Response:

```json
{
  "agent_id": "agt-uuid-001",
  "token": "tok-xxxxxxxx"
}
```

`agent_id` is a globally unique identifier used in all subsequent communication.

### Deliver a Message

Send a message to an Agent's mailbox. If the recipient is offline, the message is persisted and waits:

```bash
nats pub '$mq9.AI.INBOX.<agent_id>.<priority>' '<payload>'
```

`<priority>` values:

| Priority | Subject | Description |
|----------|---------|-------------|
| `urgent` | `$mq9.AI.INBOX.agt-001.urgent` | Highest priority, processed first when the Agent comes online |
| `normal` | `$mq9.AI.INBOX.agt-001.normal` | Routine tasks, FIFO |
| `notify` | `$mq9.AI.INBOX.agt-001.notify` | Informational, no persistence guarantee |

Examples:

```bash
# Send an urgent stop command
nats pub '$mq9.AI.INBOX.agt-001.urgent' \
  '{"type":"stop","reason":"anomaly detected"}'

# Send a routine task
nats pub '$mq9.AI.INBOX.agt-001.normal' \
  '{"from":"agt-002","task_id":"t-001","payload":"process dataset A"}'

# Send a heartbeat notification
nats pub '$mq9.AI.INBOX.agt-001.notify' \
  '{"type":"heartbeat","ts":1710000000}'
```

### Receive Messages

When an Agent comes online, it subscribes to its mailbox. The `*` wildcard matches all priority levels:

```bash
# Receive messages at all priority levels
nats sub '$mq9.AI.INBOX.agt-001.*'

# Receive only urgent messages
nats sub '$mq9.AI.INBOX.agt-001.urgent'
```

## Broadcast Protocol

### Publish a Broadcast

```bash
nats pub '$mq9.AI.BROADCAST.<domain>.<event>' '<payload>'
```

`domain` is the business domain, `event` is the event type. Both support wildcard subscriptions.

Examples:

```bash
# Broadcast a task available event
nats pub '$mq9.AI.BROADCAST.task.available' \
  '{"task_id":"t-001","type":"analysis","priority":"high"}'

# Broadcast a system anomaly
nats pub '$mq9.AI.BROADCAST.system.anomaly' \
  '{"source":"monitor-agent","level":"critical"}'
```

### Subscribe to Broadcasts

```bash
# Subscribe to all events in the task domain
nats sub '$mq9.AI.BROADCAST.task.*'

# Subscribe to anomaly events across all domains
nats sub '$mq9.AI.BROADCAST.*.anomaly'

# Subscribe to all broadcasts
nats sub '$mq9.AI.BROADCAST.>'
```

### Competing Consumers (Queue Group)

When multiple Worker Agents compete for the same tasks, use a NATS queue group to ensure each message is handled by exactly one Worker:

```bash
nats sub '$mq9.AI.BROADCAST.task.available' --queue workers
```

## Status Protocol

Agents can report their own status on the status Subject, allowing other Agents to observe their lifecycle:

```bash
# Report online
nats pub '$mq9.AI.STATUS.agt-001' '{"status":"online","ts":1710000000}'

# Report offline
nats pub '$mq9.AI.STATUS.agt-001' '{"status":"offline"}'
```

An orchestrator can observe all sub-Agent states with a wildcard subscription:

```bash
nats sub '$mq9.AI.STATUS.*'
```

## Message Format

mq9 does not enforce a message format — the payload is arbitrary bytes. JSON is recommended. Suggested fields:

```json
{
  "from": "agt-uuid-001",
  "task_id": "t-001",
  "type": "task|result|status|heartbeat",
  "payload": "...",
  "ts": 1710000000
}
```

## Relationship to Native NATS

mq9 is built on top of RobustMQ's NATS protocol layer, adding persistence and priority scheduling for specific Subject prefixes:

- Native NATS pub/sub: delivered in real-time; if the subscriber is offline, the message is lost
- mq9 `$mq9.AI.INBOX.*`: persisted to the storage layer; messages survive offline periods

Both can be used together. mq9 only activates persistence and priority scheduling for messages matching the `$mq9.AI.INBOX.*` prefix. All other NATS behavior remains unchanged.
