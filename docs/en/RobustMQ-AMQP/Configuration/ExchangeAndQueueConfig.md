# Exchange & Queue Declare Arguments

Besides name/type, `Exchange.Declare` and `Queue.Declare` carry a set of declare arguments. Here's exactly how RobustMQ handles each one.

## Exchange.Declare Arguments

| Argument | Honored? | Notes |
|---|---|---|
| `type` | ✅ | Determines the routing algorithm — direct/fanout/topic/headers all supported |
| `passive` | ✅ | Checks existence only, without creating; returns `404 NOT_FOUND` if absent |
| `durable` | 🟡 | Saved, but currently behaves identically to `durable=false` (see notes below) |
| `auto-delete` | 🟡 | Saved, but doesn't currently trigger "auto-delete once the last binding is removed" |
| `internal` | 🟡 | Saved, but currently doesn't block clients from publishing directly to an internal exchange |
| `arguments` | ❌ | Stored verbatim as a key-value table, but **never parsed or acted on** — e.g. a common argument like `alternate-exchange` is not read or executed |

## Queue.Declare Arguments

| Argument | Honored? | Notes |
|---|---|---|
| `passive` | ✅ | Checks existence only, returning **real** `message_count`/`consumer_count`; returns `404 NOT_FOUND` if absent |
| `durable` | 🟡 | Saved, but currently behaves identically to `durable=false` |
| `exclusive` | ✅ | Declaring a queue exclusive prevents other connections from accessing it |
| `auto-delete` | 🟡 | Saved, but doesn't currently trigger "auto-delete once the last consumer disconnects" |
| `arguments` | ❌ | Stored verbatim as a key-value table, but **never parsed or acted on** |

## Common Policy Arguments in `arguments`

The policy-style arguments commonly used by RabbitMQ clients are currently **entirely inert** in RobustMQ — they're stored as plain key-value pairs in metadata, nothing more:

- `x-message-ttl` — per-message TTL, has no effect
- `x-expires` — per-queue TTL, has no effect
- `x-max-length` / `x-max-length-bytes` — queue capacity cap, has no effect
- `x-dead-letter-exchange` / `x-dead-letter-routing-key` — dead-letter routing, has no effect (RobustMQ has no dead-letter queue implementation)
- `x-max-priority` — priority queues, has no effect

Declaring these arguments won't cause an error (they're silently accepted and stored), but the corresponding policy behavior simply won't happen. Don't design a migration that depends on these arguments actually taking effect.

## About `durable`

Regardless of whether `durable` is `true` or `false`, Exchange/Queue metadata is persisted the same way — restarting the broker does not clear objects declared as non-durable (`durable=false`). This differs from RabbitMQ (where non-durable objects vanish after a restart). If your application logic depends on "a non-durable queue disappears after a restart," that assumption does not hold on RobustMQ today.

## Further Reading

- [Core Concepts](../AMQPCoreConcepts.md)
- [Storage](../Storage.md)
- [Compatibility & Limitations](../Compatibility-and-Limitations.md)
