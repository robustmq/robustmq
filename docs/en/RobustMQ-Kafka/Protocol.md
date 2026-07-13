# Protocol Compatibility Matrix

This document lists RobustMQ Kafka's compatibility per API: the supported version range, implementation status, and differences from native Kafka. It is the authoritative reference for deciding whether your client / tool can connect directly.

![RobustMQ Kafka API Coverage](../../images/kafka-api-coverage.svg)

## Status legend

| Icon | Meaning |
|---|---|
| ✅ | **Full**: implemented per protocol semantics, ready to use |
| 🟡 | **Partial**: advertised and accepts requests, but semantics are trimmed (e.g. no-op, not enforced) |
| ⚪ | **Intentionally unsupported**: returns an explicit error instead of crashing; not offered by design |
| ❌ | **Not implemented**: not advertised in `ApiVersions`; clients using it fail fast |

> "Supported versions" is the range RobustMQ advertises in `ApiVersions`; where no version is noted, the API is handled at the version the client negotiates.

## Data plane

| Key | API | Versions | Status | Differences / Notes |
|---|---|---|---|---|
| 0 | Produce | v0–7 | ✅ | Idempotent writes supported; transactional writes rejected; `LogAppendTime` not applied |
| 1 | Fetch | v4–13 | ✅ | Consumer side always returns uncompressed records; no incremental fetch session; `partition_leader_epoch=0`; no `read_committed` |
| 2 | ListOffsets | v0–6 | ✅ | earliest / latest / by timestamp |
| 3 | Metadata | v0–12 | ✅ | Auto-creates topics by default (`auto.create.topics.enable`) |

## Consumer group (classic protocol)

| Key | API | Versions | Status | Differences / Notes |
|---|---|---|---|---|
| 8 | OffsetCommit | — | ✅ | Commit consumed offsets |
| 9 | OffsetFetch | — | ✅ | v8 supports multi-group batch queries |
| 10 | FindCoordinator | v0–4 | ✅ | Returns the coordinator for both group and transaction; transaction fails fast in later APIs |
| 11 | JoinGroup | v0–6 | ✅ | Join a group, trigger rebalance |
| 12 | Heartbeat | — | ✅ | Maintain membership |
| 13 | LeaveGroup | — | ✅ | Voluntarily leave |
| 14 | SyncGroup | — | ✅ | Sync partition assignment |
| 15 | DescribeGroups | — | ✅ | Query group state |
| 16 | ListGroups | — | ✅ | List all groups |
| 42 | DeleteGroups | — | ✅ | Delete groups |
| 47 | OffsetDelete | — | ✅ | Delete committed offsets |

## Consumer group (KIP-848, next generation)

| Key | API | Versions | Status | Differences / Notes |
|---|---|---|---|---|
| 68 | ConsumerGroupHeartbeat | v0–1 | ✅ | Server-side assignment; **no** `subscribed_topic_regex` |
| 69 | ConsumerGroupDescribe | v0–1 | ✅ | Query next-gen consumer groups |

## Idempotent Producer

| Key | API | Versions | Status | Differences / Notes |
|---|---|---|---|---|
| 22 | InitProducerId | v0–3 | ✅ | Idempotent only; with `transactional_id` returns `TRANSACTIONAL_ID_AUTHORIZATION_FAILED` |

## Authentication & handshake

| Key | API | Versions | Status | Differences / Notes |
|---|---|---|---|---|
| 17 | SaslHandshake | v1 | ✅ | Select SASL mechanism (SCRAM-SHA-256 / SCRAM-SHA-512) |
| 18 | ApiVersions | v0–4 | ✅ | First request after connect, negotiates available APIs |
| 36 | SaslAuthenticate | — | ✅ | SASL token exchange |

## Topic / Partition management

| Key | API | Versions | Status | Differences / Notes |
|---|---|---|---|---|
| 19 | CreateTopics | v0–7 | ✅ | Rejects manual replica assignment (replicas managed by storage) |
| 20 | DeleteTopics | — | ✅ | Delete topics |
| 21 | DeleteRecords | — | ✅ | `offset > HW` returns `OffsetOutOfRange` |
| 37 | CreatePartitions | — | ✅ | Add partitions |

## Configuration management

| Key | API | Versions | Status | Differences / Notes |
|---|---|---|---|---|
| 32 | DescribeConfigs | — | ✅ | Query configuration |
| 33 | AlterConfigs | — | ✅ | Modify configuration |
| 44 | IncrementalAlterConfigs | — | ✅ | Incrementally modify configuration |
| 74 | ListConfigResources | — | 🟡 | no-op: advertised but returns no resources |

> Most configs are **storable but not enforced**; see [Compatibility & Limitations](./Compatibility-and-Limitations.md).

## Cluster & operations

| Key | API | Versions | Status | Differences / Notes |
|---|---|---|---|---|
| 60 | DescribeCluster | — | ✅ | Cluster info |
| 75 | DescribeTopicPartitions | v0 | ✅ | Topic partition details |
| 61 | DescribeProducers | — | ❌ | Not advertised |
| 34 | AlterReplicaLogDirs | — | ⚪ | Intentionally unsupported (returns error, does not crash) |
| 35 | DescribeLogDirs | — | ⚪ | Intentionally unsupported |
| 43 | ElectLeaders | — | ⚪ | Intentionally unsupported (leaders managed by storage) |
| 45 | AlterPartitionReassignments | — | ⚪ | Intentionally unsupported (replicas auto-managed) |
| 46 | ListPartitionReassignments | — | ⚪ | Intentionally unsupported |
| 57 | UpdateFeatures | — | ⚪ | Intentionally unsupported |

## Security: ACL / quotas / SCRAM credentials

| Key | API | Versions | Status | Differences / Notes |
|---|---|---|---|---|
| 29 | DescribeAcls | — | ✅ | Query ACLs (**not enforced** for authorization) |
| 30 | CreateAcls | — | ✅ | Create ACLs (not enforced) |
| 31 | DeleteAcls | — | ✅ | Delete ACLs (not enforced) |
| 48 | DescribeClientQuotas | — | ✅ | Query quotas (`client-id` only, **not enforced** for throttling) |
| 49 | AlterClientQuotas | — | ✅ | Modify quotas (not enforced) |
| 50 | DescribeUserScramCredentials | — | ✅ | Query SCRAM credentials |
| 51 | AlterUserScramCredentials | — | ✅ | Add/remove SCRAM credentials (SASL auth validates against these) |

## Delegation tokens

| Key | API | Versions | Status | Differences / Notes |
|---|---|---|---|---|
| 38 | CreateDelegationToken | — | ✅ | Metadata management; tokens **do not participate in auth** |
| 39 | RenewDelegationToken | — | ✅ | Renew (metadata) |
| 40 | ExpireDelegationToken | — | ✅ | Expire (metadata) |
| 41 | DescribeDelegationToken | — | ✅ | Query (metadata) |

## Client telemetry (KIP-714)

| Key | API | Versions | Status | Differences / Notes |
|---|---|---|---|---|
| 71 | GetTelemetrySubscriptions | — | 🟡 | no-op: accepted but no subscription pushed |
| 72 | PushTelemetry | — | 🟡 | no-op: accepted but metrics not processed |

## Transactions (unsupported)

The following APIs are **not advertised in `ApiVersions`**; clients enabling transactions fail fast.

| Key | API | Status |
|---|---|---|
| 24 | AddPartitionsToTxn | ❌ |
| 25 | AddOffsetsToTxn | ❌ |
| 26 | EndTxn | ❌ |
| 28 | TxnOffsetCommit | ❌ |
| 65 | DescribeTransactions | ❌ |
| 66 | ListTransactions | ❌ |

> `FindCoordinator` returns a coordinator for transactions, but the subsequent transaction APIs fail immediately; `InitProducerId` supports idempotent mode only.

## Share Group (KIP-932, unsupported)

All Share Group APIs are **❌ unsupported** (`ShareGroupHeartbeat` / `ShareGroupDescribe` / `ShareFetch` / `ShareAcknowledge` / `DescribeShareGroupOffsets` / `AlterShareGroupOffsets` / `DeleteShareGroupOffsets`, etc.).

## Further reading

- [Core Concepts](./KafkaCoreConcepts.md)
- [Compatibility & Limitations](./Compatibility-and-Limitations.md) — root causes of the differences
- [CLI Guide](./CLI-Guide.md)
