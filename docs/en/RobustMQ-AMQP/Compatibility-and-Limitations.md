# Compatibility & Limitations

This page summarizes where RobustMQ's AMQP implementation stands relative to standard AMQP 0-9-1 (using RabbitMQ as the reference), to help you evaluate migration feasibility. For a per-method breakdown, see [Protocol Support](./Protocol.md).

## Fully Supported

- Connection/Channel lifecycle management (open, close, heartbeats, `Channel.Flow`).
- All four exchange types — direct, fanout, topic, headers — plus exchange-to-exchange binding chains.
- Queue declaration (including `passive`, returning real `message_count`/`consumer_count`), binding, purge, and delete (with `if-unused`/`if-empty` checks).
- `Basic.Publish`/`Basic.Get`/`Basic.Consume`/`Basic.Cancel`.
- `Basic.Ack`/`Nack`/`Reject`/`Recover`/`Recover-Async`, with automatic requeue on disconnect.
- `Confirm.Select` and real Publisher Confirms (acked only after the message is persisted).
- Exclusive consumption (`exclusive`) is genuinely enforced.
- SASL PLAIN authentication, sharing RobustMQ's unified user store.

## Partially Supported / Limited

| Feature | Limitation |
|---|---|
| `Basic.Qos` prefetch | Strictly enforced only when the consumer is on the same node as the queue leader; best-effort across nodes. `prefetch-size` (byte-based) has no effect |
| `durable` / `auto-delete` | Saved, but currently don't affect actual persistence or auto-deletion behavior (durable and non-durable behave identically) |
| `no-local` | Declaring it doesn't filter out messages published by the same connection |
| `Tx.*` (transactions) | Only replies with acknowledgement frames — no real atomic commit. Don't use for reliability guarantees; use Publisher Confirms instead |
| `internal` exchange | Saved, but doesn't currently prevent clients from publishing directly to an internal exchange |

## Not Implemented

- **ACL / operation authorization**: once authenticated, there's no permission check on any exchange/queue operation.
- **TLS/SSL**: the AMQP port only accepts plaintext TCP.
- **AMQPLAIN** SASL mechanism: only PLAIN is supported.
- **Dead-letter queues (DLX)**: rejected/discarded messages are simply deleted, never forwarded.
- **Message/queue TTL**: `x-message-ttl`, `x-expires`, etc. have no effect.
- **Queue capacity limits**: `x-max-length`, `x-max-length-bytes` have no effect.
- **Priority queues**: `x-max-priority` has no effect.
- **Lazy queues**: every message goes through File Segment storage uniformly, with no memory/disk two-tier distinction.
- **Federation / Shovel / mirrored-queue clustering** and other RabbitMQ ecosystem plugin features: none of these are implemented — RobustMQ's high availability comes from the Raft metadata layer and the File Segment storage engine itself, a fundamentally different mechanism, so RabbitMQ-plugin-style configuration doesn't carry over.

## Migration Advice

If you're migrating from RabbitMQ to RobustMQ:

1. Check whether your application relies on policy-style `arguments` (TTL, DLX, priority, capacity limits) — these currently need to be reimplemented or worked around at the application layer.
2. Check whether you rely on fine-grained permission control (vhost-level or exchange/queue-level ACLs) — currently you need to compensate with network-layer isolation.
3. Check whether you rely on non-durable objects disappearing after a restart — that assumption doesn't hold today.
4. If consumers are spread across multiple nodes and you strictly rely on precise prefetch throttling, be aware that the cross-node case is currently best-effort.

## Further Reading

- [Protocol Support](./Protocol.md)
- [Security Overview](./Security/Overview.md)
- [Exchange & Queue Declare Arguments](./Configuration/ExchangeAndQueueConfig.md)
- [Roadmap](./Roadmap.md)
