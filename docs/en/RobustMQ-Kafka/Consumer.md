# Consumer

Consumers read messages from topic partitions by offset through the `Fetch` API. RobustMQ is compatible with standard Kafka consumers: both subscription styles, offset-reset policy, long-poll fetching, and full round-tripping of key/value/headers. The group coordination flow is covered in [Consumer Group](./ConsumerGroup.md).

## Two Subscription Styles

| Style | API semantics | Notes |
|---|---|---|
| `assign(partitions)` | Manual partition assignment | The client picks which partitions to read, does not join a group, no rebalance |
| `subscribe(topics)` | Join a consumer group | The coordinator assigns partitions; supports rebalance (see [Consumer Group](./ConsumerGroup.md)) |

## Starting Offset: auto.offset.reset

When a consumer has **no committed offset** for a partition (first-time consumption, or the offset expired), `auto.offset.reset` decides where to start:

| Value | Start point |
|---|---|
| `earliest` | From the earliest available offset |
| `latest` | From the latest offset (only messages written afterward) |

If a committed offset exists, consumption resumes from it and `auto.offset.reset` does not apply. Offset commit and management are covered in [Offset Management](./OffsetManagement.md).

## Fetch Flow

![RobustMQ Kafka fetch flow](../../images/kafka-fetch-flow.svg)

Handling of one `Fetch` request:

1. **Decode** — the protocol layer parses the request; Fetch v12+ identifies topics by topic_id (UUID), which the broker reverse-resolves to a topic and echoes back in the response.
2. **Read by offset** — the storage layer locates data via the offset→position index and reads decoded records.
3. **Long poll** — if too little data is readable, the broker waits (see below).
4. **Reframe RecordBatch** — decoded records are reassembled into a Kafka `RecordBatch` for the response.

## Long Poll

A `Fetch` request carries two control parameters:

| Parameter | Meaning |
|---|---|
| `min_bytes` | Minimum bytes the caller wants returned |
| `max_wait` | Maximum time to wait |

RobustMQ's long poll is a single **wait-then-reread**: if the first read does not satisfy `min_bytes`, it waits up to `max_wait` and reads once more, then returns. This avoids busy-spinning when there is no new data while still responding promptly to data that arrives within `max_wait`.

## Message Round-Trip

Each record a consumer reads keeps its full fields:

| Field | Notes |
|---|---|
| key | Returned as-is |
| value | Returned as-is |
| headers | Returned as-is |
| offset | The continuous offset assigned by the storage layer |

> **Compression**: regardless of the producer's compression, `Fetch` currently **always returns uncompressed batches** (see [Producer · Compression](./Producer.md#compression)).

## Related

- [Consumer Group](./ConsumerGroup.md)
- [Next-Gen Consumer Group (KIP-848)](./ConsumerGroupNext.md)
- [Offset Management](./OffsetManagement.md)
- [Producer](./Producer.md)
- [System Architecture](./SystemArchitecture.md)
