# Storage Engine

RobustMQ Kafka persists messages through the unified **File Segment storage engine**. The most fundamental difference from native Kafka: the storage layer keeps **protocol-neutral, decoded records**, not Kafka's private compressed byte batches. This lets the same data be read and written by multiple protocols — Kafka and MQTT — over one store.

![RobustMQ Kafka storage layout](../../images/kafka-storage.svg)

## Segments

Each topic-partition (internally a *shard*) is split into a series of **append-only** segment files, at `{data_fold}/{shard}/{segment_no}.msg`:

- **Append-only**: the active segment is written sequentially with `append`.
- **Scroll & seal**: a new segment is created and the old one sealed when triggered. The trigger is a **combination of two gates**, not simply "full → scroll":
  - the written offset reaches a scroll interval (checked every 10,000 offsets), **and**
  - the current segment size exceeds ~**90%** of `max_segment_size`.
  When both hold, the engine requests the next segment from meta-service (with retry); the new segment starts at `previous_end + 1` and the old one is sealed as immutable.
- **Segment size cap**: per-shard `max_segment_size` (default **1 GiB**); the cluster-level default comes from the storage config key `max_segment_size`.

Each record's on-disk layout (big-endian) is a fixed 24-byte header plus three variable sections:

```
offset(u64) | total_len(u32) | metadata_len(u32) | metadata
            | protocol_data_len(u32) | protocol_data | data_len(u32) | data
```

## offset → position index and mmap reads

Reads locate data in two levels:

1. **Segment-range index**: first map the offset to a specific segment (`SegmentOffsetIndex`, offset-range → segment number).
2. **In-segment position index**: each segment maintains a persisted `offset → byte position` index (in RocksDB). A read does a **floor seek** (the nearest entry not greater than the target offset) to get a start position.

With the start position, the engine **mmaps** the whole segment file (`memmap2`) and scans forward from there, skipping records with `offset < start_offset`, until it hits `max_size` (data bytes) or `max_record`. After a write flush the mmap cache is invalidated so the next read re-maps to the latest data.

## Batch writes and offset assignment

On a batch write the engine reads the shard's current latest offset as the starting point and assigns **contiguous offsets in input-list order** (`offset, offset+1, …`), then saves `max_offset + 1` as the new latest. Therefore:

- offsets within a batch are strictly contiguous and match input order;
- the Produce response base offset is the first offset assigned to the batch.

> A past bug built the response from a `HashMap`, scrambling the base offset; it is now fixed by sorting on offset, with a regression test locking the order.

## ISR replication and leader_epoch fencing

- **Multi-replica replication**: writes go to the segment leader first; a non-leader forwards the write to the leader. With `acks=all` (-1), if the ISR size is below `min_in_sync_replicas` the write returns `NotEnoughReplicas`, and it waits for the high watermark to advance past this write.
- **leader_epoch fencing**: each leadership change bumps an epoch, recording `(epoch, start_offset)` in a persisted `LeaderEpochCache`. `OffsetsForLeaderEpoch` uses it: a request epoch below the current one → `Fenced`, above → `Unknown`, equal → returns that epoch's end offset as the follower's truncation point, preventing log divergence on split-brain.

## Protocol-neutral storage: unpack on write, reframe on read

This is the crux of the difference from native Kafka:

- **Produce (write)**: the `RecordBatch` is fully decoded and flattened into individual records before storage (producer identity is taken from the batch's first record for idempotence checks).
- **Fetch (read)**: records read from storage are **re-encoded** into a **v2, uncompressed** `RecordBatch` returned to the client.

## Differences from native Kafka, and why

| Aspect | Native Kafka | RobustMQ |
|---|---|---|
| Storage unit | stores the compressed RecordBatch as-is | protocol-neutral decoded records |
| Fetch path | zero-copy, ships the raw batch | reads records, reframes into a v2 uncompressed RecordBatch |
| Compression | preserves client compression | does not retain the original compressed batch |
| Multi-protocol | Kafka only | Kafka / MQTT share one store |

**Why this design?** RobustMQ's goal is "one data, multiple protocol views." If it stored Kafka's private compressed batches as-is, protocols like MQTT could not read the same data. Reframing per Fetch is a deliberate trade — it buys multi-protocol interop at the cost of Kafka's zero-copy and compression pass-through.

> Related: low-watermark advancement and record deletion in [DeleteRecords](./DeleteRecords.md); overall layering in [System Architecture](./SystemArchitecture.md).
