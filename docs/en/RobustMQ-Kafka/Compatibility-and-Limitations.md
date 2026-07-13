# Compatibility & Limitations

This document honestly lists what RobustMQ Kafka **supports / partially supports / does not support**, with **root causes**. Almost every limitation traces back to one design principle: **Kafka and MQTT share the same protocol-neutral storage** (see [System Architecture](./SystemArchitecture.md)). For per-API versions and status, see the [Protocol Compatibility Matrix](./Protocol.md).

## Overview

| Capability | Status |
|---|---|
| Produce / Consume / Offsets | ✅ Supported |
| Idempotent Producer | ✅ Supported |
| Classic / KIP-848 consumer groups | ✅ Supported |
| Topic / partition / config management | ✅ Supported |
| SASL/SCRAM authentication | ✅ Supported |
| Metadata / DescribeCluster | ✅ Supported |
| Delegation tokens | ✅ Supported (metadata only) |
| Fetch compression / incremental fetch session / `read_committed` | 🟡 Partial |
| Config enforcement | 🟡 Partial (stored, not enforced) |
| ACL / quotas | 🟡 Partial (manageable, not enforced) |
| Transactions | ❌ Not supported |
| Share Group (KIP-932) | ❌ Not supported |
| Replica reassignment / log dirs / manual leader election | ⚪ Intentionally unsupported |

## Fully supported ✅

- **Data plane**: `Produce` (with idempotence), `Fetch` (long polling), `ListOffsets`.
- **Consumer groups**: classic protocol (client-side assignment) and KIP-848 (server-side assignment) side by side.
- **Topic management**: create / delete / add partitions, auto-create on by default.
- **Configuration management**: `DescribeConfigs` / `AlterConfigs` / `IncrementalAlterConfigs`.
- **Authentication**: SASL/SCRAM (SCRAM-SHA-256 / SCRAM-SHA-512); SCRAM user credentials manageable via `kafka-configs.sh`.
- **Metadata**: `Metadata` / `DescribeCluster` / `DescribeTopicPartitions`.

## Partially supported 🟡

### Fetch returns no compression, no incremental session

- The consumer side always returns **uncompressed** records; `partition_leader_epoch` is always `0`; incremental fetch sessions are not supported; the `read_committed` isolation level is not supported.
- **Root cause**: storage keeps protocol-neutral decoded records, not Kafka's as-is compressed batches, so zero-copy and compression pass-through are impossible and `Fetch` must reframe a `RecordBatch`; with no transactions there is no `read_committed`.

### Configs stored but not enforced

- Most topic / broker configs can be **written and read back**, but are **not necessarily enforced** at runtime (e.g. certain retention / cleanup policies).
- **Root cause**: storage and lifecycle are managed by the unified RobustMQ kernel with its own mechanisms, not a one-to-one mapping of Kafka's config semantics.

### ACLs and quotas manageable but not enforced

- `DescribeAcls` / `CreateAcls` / `DeleteAcls` and `DescribeClientQuotas` / `AlterClientQuotas` all work; rules are stored and queryable.
- But **ACLs are not enforced** for authorization and **quotas are not enforced** for throttling (quotas currently support the `client-id` dimension only).
- **Root cause**: the enforcement paths for authorization and throttling are not yet wired in; only metadata management is provided today.

### Delegation tokens: metadata only

- `CreateDelegationToken` / `Renew` / `Expire` / `Describe` work for token metadata management, but the **token itself does not participate in authentication**.

### Client telemetry is a no-op

- `GetTelemetrySubscriptions` / `PushTelemetry` are accepted but push no subscription and process no metrics.

## Not supported ❌

### Transactions (Exactly-Once)

- The transaction APIs (`AddPartitionsToTxn` / `AddOffsetsToTxn` / `EndTxn` / `TxnOffsetCommit` / `DescribeTransactions` / `ListTransactions`) are **not advertised in `ApiVersions`**.
- `InitProducerId` supports **idempotent mode only**; with a `transactional_id` it returns `TRANSACTIONAL_ID_AUTHORIZATION_FAILED`.
- `FindCoordinator` returns a coordinator for transactions, but the subsequent transaction APIs fail immediately.
- **Impact**: do not set `transactional.id` on the client; `enable.idempotence=true` (idempotence) is fine.
- **Root cause**: transactions need a transaction log, transaction markers (`WriteTxnMarkers`), and a `read_committed` read path — machinery not yet integrated with the current protocol-neutral storage implementation.

### Share Group (KIP-932)

- All Share Group APIs (`ShareGroupHeartbeat` / `ShareGroupDescribe` / `ShareFetch` / `ShareAcknowledge` / `*ShareGroupOffsets`) are unsupported.
- **Root cause**: Share Group is a new consumption model introduced in Kafka 4.0 that relies on a dedicated shared-state store, not yet implemented.

## Intentionally unsupported ⚪

The following **storage / replica operations** APIs return an explicit error (rather than crashing), because those responsibilities are managed automatically by the RobustMQ storage layer and are not exposed for manual operation:

| API | Notes |
|---|---|
| `AlterReplicaLogDirs` / `DescribeLogDirs` | Log directories managed by the storage layer |
| `ElectLeaders` | Leader election is automatic via storage + Raft |
| `AlterPartitionReassignments` / `ListPartitionReassignments` | Replica placement is auto-managed; no manual reassignment |
| `UpdateFeatures` | No broker feature-flag updates |
| `DescribeProducers` | Not advertised |

- **Root cause**: replica placement, leader election, and log directories are decided automatically by the storage engine and the Raft metadata layer; exposing these manual operations conflicts with RobustMQ's self-managing model.

## Key differences from native Kafka

| Dimension | Native Kafka | RobustMQ |
|---|---|---|
| Storage unit | Compressed RecordBatch as-is (zero-copy) | Protocol-neutral decoded records, reframed on Fetch |
| Controller / Coordinator | KRaft / ZooKeeper | meta-service Raft leader |
| Multi-protocol | Kafka only | Kafka / MQTT share the same data |
| Transactions | Supported | Not supported |
| ACL / quotas | Enforced | Manageable, not enforced |
| Replica / leader ops | Manually controllable | Auto-managed by the storage layer |

## Further reading

- [System Architecture](./SystemArchitecture.md)
- [Protocol Compatibility Matrix](./Protocol.md)
- [Core Concepts](./KafkaCoreConcepts.md)
