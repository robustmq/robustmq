# DeleteRecords

`DeleteRecords` **advances a partition's low watermark**: it marks records before a given offset as unreadable and lets their space be reclaimed. It deletes a log prefix, not consumer offsets (deleting committed group offsets is a separate path — don't conflate the two).

> Internally in RobustMQ the "low watermark" is the shard's `earliest_offset`; the read layer exposes it as `start_offset`, and the Kafka protocol calls it `low_watermark` — all the same value.

![DeleteRecords low-watermark truncation](../../images/kafka-delete-records.svg)

## beforeOffset semantics

Each partition carries a `beforeOffset`:

| `beforeOffset` | Behavior | Returns |
|---|---|---|
| `N > 0` | delete records with offset `< N` | new low watermark = achieved value |
| `-1` | delete up to the **high watermark** (all currently readable data) | new low watermark |
| `0` | no-op: nothing deleted, returns the current low watermark | success (error_code 0) |
| `< -1` (other negatives) | invalid | `OFFSET_OUT_OF_RANGE` |
| `> high watermark` | out of range | `OFFSET_OUT_OF_RANGE` |
| unknown topic/partition | — | `UNKNOWN_TOPIC_OR_PARTITION` |

> Note the difference between `0` and `< -1`: `beforeOffset = 0` is a successful no-op; other negatives such as `-2` return `OFFSET_OUT_OF_RANGE`, not a no-op.

## How it runs

1. The broker resolves each partition's target offset: `-1` resolves to that partition's high watermark; `> HW` is flagged out of range.
2. The target goes to the storage layer, where it is first **clamped to latest**; if `target <= earliest` (already deleted or lower), it returns the current low watermark without deleting.
3. Otherwise it deletes records in `[earliest, target)` and moves `earliest_offset` forward to `target` — advancing the low watermark.

### Physical-reclaim granularity

The low watermark advances **exactly** to `target`, but on-disk space is reclaimed at **segment granularity**:

- the File Segment engine only deletes **sealed segments that fall entirely below `target`**;
- it **never deletes the active segment**;
- if `target` lands inside a segment that must be kept, that segment is retained whole — records `>= target` stay on disk, and those below the new low watermark simply become unreadable.

So the logical readable range shrinks immediately, while disk is freed only once an entire sealed segment sits below the low watermark.

## Consumer behavior after deletion

After deletion, consumers read from the **new low watermark**. If a consumer requests an offset now below the new low watermark, it gets `OFFSET_OUT_OF_RANGE` and must relocate per its `auto.offset.reset` policy (typically to the earliest readable offset).

## Relationship with retention

`DeleteRecords` advances the low watermark **manually and immediately**; time/size-based **retention** does the same thing **automatically in the background** (removing expired or oversized old segments and moving the low watermark forward). Both advance the same low watermark and compose: retention handles steady-state cleanup, while `DeleteRecords` is for on-demand truncation (e.g. compliance deletes, fast space reclaim).

## CLI example

```bash
# Delete records with offset < 1000 in partition 0 of topic orders
kafka-delete-records.sh --bootstrap-server localhost:9092 \
  --offset-json-file delete.json
```

`delete.json`:

```json
{
  "partitions": [
    { "topic": "orders", "partition": 0, "offset": 1000 }
  ],
  "version": 1
}
```

> Segment structure and the low-watermark mechanism are covered in [Storage Engine](./Storage.md).
