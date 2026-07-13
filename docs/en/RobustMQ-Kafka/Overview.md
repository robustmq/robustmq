# Overview

RobustMQ Kafka is a **Kafka protocol compatibility layer** built on top of the unified RobustMQ kernel. It is not a standalone Kafka distribution, but a protocol implementation that lets the standard Kafka ecosystem connect directly to RobustMQ. Native Kafka clients (Java `kafka-clients`, `librdkafka`) and the official command-line tools (`kafka-*.sh`) connect directly, on the default port `9092`.

## Design principle: one data, multiple protocol views

The core design principle of RobustMQ Kafka is **"one data, multiple protocol views"**: Kafka and MQTT share the same topic storage and metadata. The storage layer therefore keeps **protocol-neutral, decoded message records** rather than Kafka's private compressed batch format. This principle drives many implementation choices (for example, why Kafka's "store batches as-is + zero-copy" model is not used).

Key facts:

- **Controller / Coordinator = the meta-service Raft leader**: no ZooKeeper or KRaft. The controller, group coordinator, and transaction coordinator in Kafka semantics all resolve to the current Raft leader, located via a gRPC lookup cached with a ~3s TTL.
- **Storage engine = File Segment**: segment append / seal / scroll, `offset → position` index with mmap reads, ISR multi-replica with `leader_epoch` fencing.
- **Decode on write**: `Produce` unpacks the RecordBatch into the store on write, and `Fetch` reframes records back into a `RecordBatch`, so the same topic can be read and written by other protocols too.

See [System Architecture](./SystemArchitecture.md) for details.

## Capability summary

| Capability | Status | Notes |
|---|---|---|
| Produce / Consume / Offsets | ✅ | `Produce` / `Fetch` / `ListOffsets` / `OffsetCommit` / `OffsetFetch` |
| Idempotent Producer | ✅ | Producer ID allocation + sequence dedup (last-5 sliding window + epoch fencing) |
| Classic consumer groups | ✅ | `FindCoordinator` / `JoinGroup` / `SyncGroup` / `Heartbeat` / `LeaveGroup` |
| KIP-848 consumer groups | ✅ | `ConsumerGroupHeartbeat` (server-side assignment); no `subscribed_topic_regex` |
| Topic management | ✅ | Create / delete / add partitions; auto-create on by default |
| Configuration management | ✅ | `DescribeConfigs` / `AlterConfigs` / `IncrementalAlterConfigs` |
| SASL / SCRAM authentication | ✅ | SCRAM-SHA-256 / SCRAM-SHA-512 |
| ACL / quotas | 🟡 | Manageable, but **not enforced** for authorization or throttling |
| Delegation tokens | ✅ | Metadata management (tokens do not participate in auth) |
| Metadata / DescribeCluster | ✅ | Cluster topology, brokers, topic / partition info |
| Fetch compression | 🟡 | Consumer side always returns **uncompressed** records |
| Transactions | ❌ | Not advertised, not supported (reasons below) |
| Share Group (KIP-932) | ❌ | Not supported |

> For per-API versions and differences see the [Protocol Compatibility Matrix](./Protocol.md); for the full "supported / partial / unsupported" list with reasons see [Compatibility & Limitations](./Compatibility-and-Limitations.md).

## Quick start

After starting a single node, use the official CLI to create a topic, produce, and consume:

```bash
# Create a topic
kafka-topics.sh --bootstrap-server localhost:9092 \
  --create --topic quickstart --partitions 3

# Produce (type a few lines, then Ctrl-C)
kafka-console-producer.sh --bootstrap-server localhost:9092 --topic quickstart

# Consume from the beginning
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic quickstart --from-beginning
```

Full steps (including a Java `kafka-clients` example and SASL connection) are in [Quick Start](./QuickStart.md).

## Documentation map

| Document | Contents |
|---|---|
| [System Architecture](./SystemArchitecture.md) | Five-layer architecture, request flow, key differences from native Kafka |
| [Core Concepts](./KafkaCoreConcepts.md) | Topic / Partition / Offset, Record, consumer groups, Coordinator, Segment |
| [Protocol Compatibility Matrix](./Protocol.md) | Per-API supported versions, status, and differences |
| [Quick Start](./QuickStart.md) | Single-node startup, CLI and minimal Java client examples |
| [CLI Guide](./CLI-Guide.md) | Complete guide to the official kafka CLI against RobustMQ |
| [Compatibility & Limitations](./Compatibility-and-Limitations.md) | Supported / partial / unsupported list and root causes |
