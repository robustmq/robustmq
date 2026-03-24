# RobustMQ Kafka Protocol Support

This document lists the Kafka protocol APIs that RobustMQ needs to support as a Kafka Broker, along with their priority and description.

References:
- [Kafka Protocol Guide (4.2)](https://kafka.apache.org/42/design/protocol/)
- [kafka-protocol crate 0.17.0](https://docs.rs/kafka-protocol/0.17.0)

---

## 1. Core Data Plane (Required)

| API Key | API Name | Description | Supported |
|---------|----------|-------------|-----------|
| 0 | **Produce** | Producer writes messages | ❌ |
| 1 | **Fetch** | Consumer fetches messages | ❌ |
| 2 | **ListOffsets** | Query topic/partition offsets (earliest / latest / by timestamp) | ❌ |
| 3 | **Metadata** | Fetch cluster topology, topic / partition / broker info on startup | ❌ |

---

## 2. Consumer Group Management (Required)

| API Key | API Name | Description | Supported |
|---------|----------|-------------|-----------|
| 8 | **OffsetCommit** | Commit consumed offsets | ❌ |
| 9 | **OffsetFetch** | Fetch committed offsets | ❌ |
| 10 | **FindCoordinator** | Find the Group / Transaction Coordinator broker | ❌ |
| 11 | **JoinGroup** | Join a consumer group, triggers rebalance | ❌ |
| 12 | **Heartbeat** | Consumer heartbeat to maintain group membership | ❌ |
| 13 | **LeaveGroup** | Voluntarily leave a consumer group | ❌ |
| 14 | **SyncGroup** | Sync partition assignment after rebalance | ❌ |
| 15 | **DescribeGroups** | Query consumer group state | ❌ |
| 16 | **ListGroups** | List all consumer groups | ❌ |
| 42 | **DeleteGroups** | Delete consumer groups | ❌ |
| 47 | **OffsetDelete** | Delete committed offsets | ❌ |

---

## 3. Connection & Authentication (Required)

| API Key | API Name | Description | Supported |
|---------|----------|-------------|-----------|
| 17 | **SaslHandshake** | SASL authentication handshake, selects mechanism | ❌ |
| 18 | **ApiVersions** | First request after connection, negotiates supported API versions | ❌ |
| 36 | **SaslAuthenticate** | SASL token exchange (used with SaslHandshake v1+) | ❌ |

---

## 4. Topic / Partition Management (Required)

| API Key | API Name | Description | Supported |
|---------|----------|-------------|-----------|
| 19 | **CreateTopics** | Create topics | ❌ |
| 20 | **DeleteTopics** | Delete topics | ❌ |
| 21 | **DeleteRecords** | Delete records before a given offset in a partition | ❌ |
| 37 | **CreatePartitions** | Increase partition count | ❌ |

---

## 5. Configuration Management (Required)

| API Key | API Name | Description | Supported |
|---------|----------|-------------|-----------|
| 32 | **DescribeConfigs** | Query topic / broker configuration | ❌ |
| 33 | **AlterConfigs** | Modify configuration | ❌ |
| 44 | **IncrementalAlterConfigs** | Incrementally modify configuration (recommended over AlterConfigs) | ❌ |

---

## 6. Transaction Support (Exactly-Once, Optional)

| API Key | API Name | Description | Supported |
|---------|----------|-------------|-----------|
| 22 | **InitProducerId** | Obtain producer id (required for idempotent / transactional produce) | ❌ |
| 24 | **AddPartitionsToTxn** | Add partitions to a transaction | ❌ |
| 25 | **AddOffsetsToTxn** | Add consumer offsets to a transaction | ❌ |
| 26 | **EndTxn** | Commit or abort a transaction | ❌ |
| 28 | **TxnOffsetCommit** | Commit offsets within a transaction | ❌ |
| 65 | **DescribeTransactions** | Query transaction state | ❌ |
| 66 | **ListTransactions** | List all transactions | ❌ |

---

## 7. ACL Access Control (Optional)

| API Key | API Name | Description | Supported |
|---------|----------|-------------|-----------|
| 29 | **DescribeAcls** | Query ACL rules | ❌ |
| 30 | **CreateAcls** | Create ACL rules | ❌ |
| 31 | **DeleteAcls** | Delete ACL rules | ❌ |

---

## 8. Quota Management (Optional)

| API Key | API Name | Description | Supported |
|---------|----------|-------------|-----------|
| 48 | **DescribeClientQuotas** | Query client quotas | ❌ |
| 49 | **AlterClientQuotas** | Modify client quotas | ❌ |
| 50 | **DescribeUserScramCredentials** | Query SCRAM credential info | ❌ |
| 51 | **AlterUserScramCredentials** | Modify SCRAM credentials | ❌ |

---

## 9. Delegation Token Authentication (Optional)

| API Key | API Name | Description | Supported |
|---------|----------|-------------|-----------|
| 38 | **CreateDelegationToken** | Create a delegation token | ❌ |
| 39 | **RenewDelegationToken** | Renew a delegation token | ❌ |
| 40 | **ExpireDelegationToken** | Expire a delegation token | ❌ |
| 41 | **DescribeDelegationToken** | Query delegation token info | ❌ |

---

## 10. Client Telemetry (Optional, KIP-714)

| API Key | API Name | Description | Supported |
|---------|----------|-------------|-----------|
| 71 | **GetTelemetrySubscriptions** | Get client telemetry subscription config | ❌ |
| 72 | **PushTelemetry** | Client pushes telemetry metrics | ❌ |
| 74 | **ListConfigResources** | List configuration resources | ❌ |

---

## 11. Operations & Administration (Optional)

| API Key | API Name | Description | Supported |
|---------|----------|-------------|-----------|
| 23 | **OffsetForLeaderEpoch** | Used by follower / consumer for leader epoch validation | ❌ |
| 34 | **AlterReplicaLogDirs** | Migrate log directories | ❌ |
| 35 | **DescribeLogDirs** | Query log storage directory info | ❌ |
| 43 | **ElectLeaders** | Manually trigger leader election | ❌ |
| 45 | **AlterPartitionReassignments** | Reassign partition replicas | ❌ |
| 46 | **ListPartitionReassignments** | Query partition reassignment progress | ❌ |
| 57 | **UpdateFeatures** | Update broker feature flags | ❌ |
| 60 | **DescribeCluster** | Query cluster information | ❌ |
| 61 | **DescribeProducers** | Query active producer information | ❌ |
| 75 | **DescribeTopicPartitions** | Query topic partition details | ❌ |

---

## 12. Next-Generation Consumer Group Protocol (Optional, KIP-848)

| API Key | API Name | Description | Supported |
|---------|----------|-------------|-----------|
| 68 | **ConsumerGroupHeartbeat** | New consumer group protocol heartbeat | ❌ |
| 69 | **ConsumerGroupDescribe** | New consumer group query | ❌ |

---

## 13. Share Group (Optional, KIP-932, Kafka 4.0+)

Share Group is a new consumption model introduced in Kafka 4.0, supporting shared message consumption (non-exclusive partitions):

| API Key | API Name | Description | Supported |
|---------|----------|-------------|-----------|
| 76 | **ShareGroupHeartbeat** | Share Group heartbeat | ❌ |
| 77 | **ShareGroupDescribe** | Query Share Group state | ❌ |
| 78 | **ShareFetch** | Share Group fetch messages | ❌ |
| 79 | **ShareAcknowledge** | Share Group acknowledge messages | ❌ |
| 90 | **DescribeShareGroupOffsets** | Query Share Group offsets | ❌ |
| 91 | **AlterShareGroupOffsets** | Modify Share Group offsets | ❌ |
| 92 | **DeleteShareGroupOffsets** | Delete Share Group offsets | ❌ |

---

## 14. Not Required (KRaft / Internal Cluster Communication)

The following APIs are used only for Kafka internal cluster communication (replica management, KRaft elections, Controller communication). RobustMQ does not need to implement these:

| API Key | API Name | Description |
|---------|----------|-------------|
| 4 | LeaderAndIsr | Inter-broker replica management |
| 5 | StopReplica | Inter-broker stop replica |
| 6 | UpdateMetadata | Controller broadcasts metadata |
| 7 | ControlledShutdown | Orderly broker shutdown |
| 27 | WriteTxnMarkers | Transaction Coordinator writes to broker (internal) |
| 52 | Vote | KRaft voting |
| 53 | BeginQuorumEpoch | KRaft |
| 54 | EndQuorumEpoch | KRaft |
| 55 | DescribeQuorum | KRaft quorum state |
| 56 | AlterPartition | KRaft ISR change |
| 58 | Envelope | Controller request forwarding |
| 59 | FetchSnapshot | KRaft log snapshot |
| 62 | BrokerRegistration | KRaft broker registration |
| 63 | BrokerHeartbeat | KRaft broker heartbeat |
| 64 | UnregisterBroker | KRaft broker deregistration |
| 67 | AllocateProducerIds | KRaft Controller allocates Producer IDs |
| 70 | ControllerRegistration | KRaft Controller registration |
| 73 | AssignReplicasToDirs | Internal replica directory assignment |
| 80 | AddRaftVoter | KRaft |
| 81 | RemoveRaftVoter | KRaft |
| 82 | UpdateRaftVoter | KRaft |
| 83 | InitializeShareGroupState | Share Group state internal storage |
| 84 | ReadShareGroupState | Share Group state internal read |
| 85 | WriteShareGroupState | Share Group state internal write |
| 86 | DeleteShareGroupState | Share Group state internal delete |
| 87 | ReadShareGroupStateSummary | Share Group state summary internal read |

---

## Implementation Roadmap

### Phase 1: Standard Client Compatibility

After implementing the following APIs, standard Kafka producer / consumer clients will work correctly:

```text
ApiVersions(18) → Metadata(3) → SaslHandshake(17) / SaslAuthenticate(36)
→ Produce(0) / Fetch(1) / ListOffsets(2)
→ FindCoordinator(10) → JoinGroup(11) / SyncGroup(14) / Heartbeat(12) / LeaveGroup(13)
→ OffsetCommit(8) / OffsetFetch(9)
→ CreateTopics(19) / DeleteTopics(20) → DescribeConfigs(32)
```

Approximately **20 APIs** in total.

### Phase 2: Transaction Support

Add transaction-related APIs on top of Phase 1 (22, 24, 25, 26, 28, 65, 66).

### Phase 3: Full Management Coverage

Complete ACL, Quota, and operations management APIs.
