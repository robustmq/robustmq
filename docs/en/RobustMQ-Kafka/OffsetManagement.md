# Offset Management

An offset records how far a consumer group has read in each partition. RobustMQ **persists offsets in the meta layer** and provides the full Kafka offset-management surface: commit, fetch, reset, delete, and lag computation. This page covers the relevant APIs and the `kafka-consumer-groups.sh` commands.

## Commit and Fetch

| API | Purpose | Notes |
|---|---|---|
| `OffsetCommit` | Commit offsets | A group writes a partition's consumption progress to meta (`commitSync` is a synchronous commit) |
| `OffsetFetch` | Fetch committed offsets | v8+ supports **querying multiple groups in one batched request** |

Offsets are persisted in the meta layer, so a group can resume from its committed offset after a restart or membership change (see [Consumer Group](./ConsumerGroup.md)).

## Reset Offsets

`kafka-consumer-groups.sh --reset-offsets` moves a group's offsets to a chosen position:

| Target | Meaning |
|---|---|
| `--to-earliest` | Reset to the earliest available offset |
| `--to-latest` | Reset to the latest offset |
| `--to-offset <n>` | Reset to a specific offset |
| `--shift-by <n>` | Move the current offset forward/back by n (negative to rewind) |

Execution modes:

| Mode | Behavior |
|---|---|
| `--dry-run` | Preview the intended changes only, nothing is persisted |
| `--execute` | Actually commit the changes |

## Delete Offsets

`kafka-consumer-groups.sh --delete-offsets` deletes a group's committed offsets for a given topic/partition. Afterward the partition has no committed offset, and the next consumption starts per `auto.offset.reset` (see [Consumer](./Consumer.md#starting-offset-auto-offset-reset)).

## Lag

`kafka-consumer-groups.sh --describe` shows per-partition progress:

| Column | Meaning |
|---|---|
| `CURRENT-OFFSET` | The group's committed offset |
| `LOG-END-OFFSET` | The partition's latest offset (its end) |
| `LAG` | The lag = `LOG-END-OFFSET − CURRENT-OFFSET` |

A larger lag means consumption is further behind production.

## Persistence

All offsets are persisted in the Raft-based **meta-service**, managed in the same layer as cluster dynamic config and ACL/SCRAM/quotas (see [System Architecture](./SystemArchitecture.md)). The coordinator (Raft leader) handles offset reads and writes.

## Related

- [Consumer Group (Classic Protocol)](./ConsumerGroup.md)
- [Next-Gen Consumer Group (KIP-848)](./ConsumerGroupNext.md)
- [Consumer](./Consumer.md)
- [System Architecture](./SystemArchitecture.md)
