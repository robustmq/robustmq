# Storage

AMQP queue messages use the unified `StorageDriverManager`, but the physical
backend is selected by the queue Topic configuration. Queue topics created by
the current AMQP implementation use **`EngineRocksDB`**; Kafka topics use
`EngineSegment`, while MQTT topics use their own topic configuration. The
protocols share storage infrastructure, not one mandatory physical engine or
one automatic protocol namespace.

## A Unified Storage Driver

AMQP, Kafka, and MQTT write through the same `StorageDriverManager`, which
selects a driver from each Topic's storage type. An AMQP queue is represented
by a `TopicSource::AMQP` Topic and its backing shard; records are written and
read through the adapter rather than through an AMQP-specific storage API.

This means:

- AMQP's durability and cleanup behavior follow the selected backend and Topic
  configuration; they are not identical to Kafka's File Segment layout.
- There is no separate AMQP storage API. Backend-specific settings are applied
  through the unified Topic and storage configuration.

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
| Lazy queues | ❌ | No RabbitMQ-style memory-vs-disk two-tier queue mode; the queue uses its configured backend uniformly |

## Further Reading

- [Queue Purge & Delete](./QueuePurgeAndDelete.md)
- [Core Concepts](./AMQPCoreConcepts.md)
- [System Architecture](./SystemArchitecture.md)
- [Compatibility & Limitations](./Compatibility-and-Limitations.md)
