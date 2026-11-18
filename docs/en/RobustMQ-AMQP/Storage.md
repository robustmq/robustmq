# Storage

AMQP queue messages use the shared `StorageDriverManager` and `EngineStorageAdapter`,
but the current AMQP queue-creation path selects **`EngineRocksDB`**. Kafka topics
currently select `EngineSegment`, while MQTT topics select `EngineRocksDB`. The
protocols therefore share the adapter and storage-engine interfaces, but they do not
automatically share one physical File Segment data copy.

## A Unified Storage Driver

AMQP, Kafka, and MQTT write through the same `StorageDriverManager`, which routes to
the configured storage engine for each Topic. An AMQP queue is represented by an
internal Topic and one or more Shards; the current queue path provisions RocksDB
shards, and messages are located by offset through the storage adapter.

This means:

- AMQP's persistence and cleanup behavior follows the selected RocksDB shard configuration, not Kafka's File Segment layout.
- Queue storage still uses the shared topic/shard configuration path; storage settings such as retention and record limits are carried in the Topic configuration.

## Publisher Confirm Semantics

In confirm mode, a routed `Basic.Publish` first buffers the message in memory (`PendingPublish`); the broker awaits each target queue's `StorageDriverManager::write` result before it builds and sends `Basic.Ack`/`Basic.Nack`. For a routed publish, `Basic.Ack` means the configured storage write succeeded on the handling node; it is not a cross-node replica acknowledgement. An unroutable publish is currently acknowledged even though no queue write occurs (and `mandatory=true` also receives `Basic.Return`).

## Capabilities Not Yet Implemented

The following storage-related features common to message-queue systems are **not implemented** in RobustMQ AMQP today:

| Feature | Status | Notes |
|---|---|---|
| Message TTL / expiration | ❌ | No mechanism to set an expiration time on a queue or message |
| Max queue length | ❌ | No queue capacity cap, and no overflow-drop/reject policy |
| Dead-letter exchange (DLX) | ❌ | Rejected or discarded messages are simply deleted, never forwarded to a dead-letter exchange |
| Priority queues | ❌ | No priority-based scheduling |
| Lazy queues | ❌ | No memory-vs-disk two-tier storage distinction; every message goes through the configured RocksDB queue storage path |

## Further Reading

- [Queue Purge & Delete](./QueuePurgeAndDelete.md)
- [Core Concepts](./AMQPCoreConcepts.md)
- [System Architecture](./SystemArchitecture.md)
- [Compatibility & Limitations](./Compatibility-and-Limitations.md)
