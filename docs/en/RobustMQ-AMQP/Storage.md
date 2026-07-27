# Storage

AMQP queue messages are stored using the exact same underlying engine as Kafka and MQTT — **File Segment** — with no separate storage implementation built for AMQP. This is a direct expression of the "one data copy, multiple protocol views" architecture.

## A Unified Storage Driver

All three protocol brokers — AMQP, Kafka, MQTT — write through the same `StorageDriverManager`, which bottoms out in the `storage-engine` crate's File Segment engine. A queue is, at the storage layer, a shard: messages are appended sequentially to a log and located by offset.

This means:

- AMQP's durability guarantees, on-disk layout, and compaction/cleanup strategy are identical to Kafka/MQTT's — the same operational and monitoring know-how applies.
- There are no AMQP-specific storage configuration knobs — storage tuning (segment size, flush policy, etc.) is configured once at the broker level, independent of protocol.

## Publish Means Persisted

In confirm mode, there's no race between persistence and acknowledgement: `Basic.Publish` first buffers the message in memory (`PendingPublish`); the actual write fully awaits `StorageDriverManager::write` returning success before the broker builds and sends `Basic.Ack`/`Basic.Nack`. In other words, as soon as a client receives `Basic.Ack`, the message is genuinely on disk — there's no "ack now, write asynchronously later" path.

## Capabilities Not Yet Implemented

The following storage-related features common to message-queue systems are **not implemented** in RobustMQ AMQP today:

| Feature | Status | Notes |
|---|---|---|
| Message TTL / expiration | ❌ | No mechanism to set an expiration time on a queue or message |
| Max queue length | ❌ | No queue capacity cap, and no overflow-drop/reject policy |
| Dead-letter exchange (DLX) | ❌ | Rejected or discarded messages are simply deleted, never forwarded to a dead-letter exchange |
| Priority queues | ❌ | No priority-based scheduling |
| Lazy queues | ❌ | No memory-vs-disk two-tier storage distinction; every message goes through File Segment uniformly |

## Further Reading

- [Queue Purge & Delete](./QueuePurgeAndDelete.md)
- [Core Concepts](./AMQPCoreConcepts.md)
- [System Architecture](./SystemArchitecture.md)
- [Compatibility & Limitations](./Compatibility-and-Limitations.md)
