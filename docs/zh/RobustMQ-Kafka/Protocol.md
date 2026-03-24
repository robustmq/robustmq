# RobustMQ Kafka 协议支持

本文档列出 RobustMQ 作为 Kafka Broker 需要支持的 Kafka 协议 API，以及各 API 的优先级和说明。

参考文档：
- [Kafka Protocol Guide (4.2)](https://kafka.apache.org/42/design/protocol/)
- [kafka-protocol crate 0.17.0](https://docs.rs/kafka-protocol/0.17.0)

---

## 一、核心数据面（必须支持）

| API Key | API 名称 | 说明 | 已支持 |
|---------|----------|------|--------|
| 0 | **Produce** | 生产者写消息 | ❌ |
| 1 | **Fetch** | 消费者拉消息 | ❌ |
| 2 | **ListOffsets** | 查询 topic/partition 的 offset（earliest / latest / by timestamp） | ❌ |
| 3 | **Metadata** | 客户端启动时获取集群拓扑、topic / partition / broker 信息 | ❌ |

---

## 二、Consumer Group 管理（必须支持）

| API Key | API 名称 | 说明 | 已支持 |
|---------|----------|------|--------|
| 8 | **OffsetCommit** | 提交消费 offset | ❌ |
| 9 | **OffsetFetch** | 查询已提交 offset | ❌ |
| 10 | **FindCoordinator** | 找 Group / Transaction Coordinator 所在 broker | ❌ |
| 11 | **JoinGroup** | 加入消费组，触发 rebalance | ❌ |
| 12 | **Heartbeat** | 消费者心跳，维持 group 成员资格 | ❌ |
| 13 | **LeaveGroup** | 主动离开消费组 | ❌ |
| 14 | **SyncGroup** | rebalance 后同步分区分配结果 | ❌ |
| 15 | **DescribeGroups** | 查询 consumer group 状态 | ❌ |
| 16 | **ListGroups** | 列出所有 consumer group | ❌ |
| 42 | **DeleteGroups** | 删除 consumer group | ❌ |
| 47 | **OffsetDelete** | 删除已提交的 offset | ❌ |

---

## 三、连接与认证（必须支持）

| API Key | API 名称 | 说明 | 已支持 |
|---------|----------|------|--------|
| 17 | **SaslHandshake** | SASL 认证握手，选择认证机制 | ❌ |
| 18 | **ApiVersions** | 客户端连接后第一个请求，协商支持的 API 版本 | ❌ |
| 36 | **SaslAuthenticate** | SASL token 交换（SaslHandshake v1+ 使用） | ❌ |

---

## 四、Topic / Partition 管理（必须支持）

| API Key | API 名称 | 说明 | 已支持 |
|---------|----------|------|--------|
| 19 | **CreateTopics** | 创建 topic | ❌ |
| 20 | **DeleteTopics** | 删除 topic | ❌ |
| 21 | **DeleteRecords** | 删除 partition 中指定 offset 之前的消息 | ❌ |
| 37 | **CreatePartitions** | 增加 partition 数量 | ❌ |

---

## 五、配置管理（必须支持）

| API Key | API 名称 | 说明 | 已支持 |
|---------|----------|------|--------|
| 32 | **DescribeConfigs** | 查询 topic / broker 配置 | ❌ |
| 33 | **AlterConfigs** | 修改配置 | ❌ |
| 44 | **IncrementalAlterConfigs** | 增量修改配置（推荐替代 AlterConfigs） | ❌ |

---

## 六、事务支持（Exactly-Once，按需支持）

| API Key | API 名称 | 说明 | 已支持 |
|---------|----------|------|--------|
| 22 | **InitProducerId** | 获取 producer id（幂等生产 / 事务必须） | ❌ |
| 24 | **AddPartitionsToTxn** | 将 partition 加入事务 | ❌ |
| 25 | **AddOffsetsToTxn** | 将 consumer offset 加入事务 | ❌ |
| 26 | **EndTxn** | 提交或回滚事务 | ❌ |
| 28 | **TxnOffsetCommit** | 事务内提交 offset | ❌ |
| 65 | **DescribeTransactions** | 查询事务状态 | ❌ |
| 66 | **ListTransactions** | 列出所有事务 | ❌ |

---

## 七、ACL 权限控制（可选）

| API Key | API 名称 | 说明 | 已支持 |
|---------|----------|------|--------|
| 29 | **DescribeAcls** | 查询 ACL 规则 | ❌ |
| 30 | **CreateAcls** | 创建 ACL 规则 | ❌ |
| 31 | **DeleteAcls** | 删除 ACL 规则 | ❌ |

---

## 八、Quota 配额管理（可选）

| API Key | API 名称 | 说明 | 已支持 |
|---------|----------|------|--------|
| 48 | **DescribeClientQuotas** | 查询客户端配额 | ❌ |
| 49 | **AlterClientQuotas** | 修改客户端配额 | ❌ |
| 50 | **DescribeUserScramCredentials** | 查询 SCRAM 认证信息 | ❌ |
| 51 | **AlterUserScramCredentials** | 修改 SCRAM 认证信息 | ❌ |

---

## 九、Delegation Token 认证（可选）

| API Key | API 名称 | 说明 | 已支持 |
|---------|----------|------|--------|
| 38 | **CreateDelegationToken** | 创建委托令牌 | ❌ |
| 39 | **RenewDelegationToken** | 续期委托令牌 | ❌ |
| 40 | **ExpireDelegationToken** | 使委托令牌过期 | ❌ |
| 41 | **DescribeDelegationToken** | 查询委托令牌信息 | ❌ |

---

## 十、客户端遥测（可选，KIP-714）

| API Key | API 名称 | 说明 | 已支持 |
|---------|----------|------|--------|
| 71 | **GetTelemetrySubscriptions** | 获取客户端遥测订阅配置 | ❌ |
| 72 | **PushTelemetry** | 客户端上报遥测指标 | ❌ |
| 74 | **ListConfigResources** | 列出配置资源 | ❌ |

---

## 十一、运维管理（可选）

| API Key | API 名称 | 说明 | 已支持 |
|---------|----------|------|--------|
| 23 | **OffsetForLeaderEpoch** | follower / consumer 用于 leader epoch 校验 | ❌ |
| 34 | **AlterReplicaLogDirs** | 迁移日志目录 | ❌ |
| 35 | **DescribeLogDirs** | 查询 log 存储目录信息 | ❌ |
| 43 | **ElectLeaders** | 手动触发 leader 选举 | ❌ |
| 45 | **AlterPartitionReassignments** | 重新分配 partition 副本 | ❌ |
| 46 | **ListPartitionReassignments** | 查询 partition 重分配进度 | ❌ |
| 57 | **UpdateFeatures** | 更新 broker feature flag | ❌ |
| 60 | **DescribeCluster** | 查询集群信息 | ❌ |
| 61 | **DescribeProducers** | 查询活跃 producer 信息 | ❌ |
| 75 | **DescribeTopicPartitions** | 查询 topic partition 详情 | ❌ |

---

## 十二、新一代 Consumer Group 协议（可选，KIP-848）

| API Key | API 名称 | 说明 | 已支持 |
|---------|----------|------|--------|
| 68 | **ConsumerGroupHeartbeat** | 新 consumer group 协议心跳 | ❌ |
| 69 | **ConsumerGroupDescribe** | 新 consumer group 查询 | ❌ |

---

## 十三、Share Group（可选，KIP-932，Kafka 4.0+）

Share Group 是 Kafka 4.0 引入的新消费模型，支持消息的共享消费（非独占分区）：

| API Key | API 名称 | 说明 | 已支持 |
|---------|----------|------|--------|
| 76 | **ShareGroupHeartbeat** | Share Group 心跳 | ❌ |
| 77 | **ShareGroupDescribe** | 查询 Share Group 状态 | ❌ |
| 78 | **ShareFetch** | Share Group 拉取消息 | ❌ |
| 79 | **ShareAcknowledge** | Share Group 确认消息 | ❌ |
| 90 | **DescribeShareGroupOffsets** | 查询 Share Group offset | ❌ |
| 91 | **AlterShareGroupOffsets** | 修改 Share Group offset | ❌ |
| 92 | **DeleteShareGroupOffsets** | 删除 Share Group offset | ❌ |

---

## 十四、不需要支持（KRaft / 集群内部通信）

以下 API 仅用于 Kafka 集群内部（副本管理、KRaft 选举、Controller 通信），RobustMQ 不需要实现：

| API Key | API 名称 | 说明 |
|---------|----------|------|
| 4 | LeaderAndIsr | broker 间副本管理 |
| 5 | StopReplica | broker 间停止副本 |
| 6 | UpdateMetadata | Controller 广播元数据 |
| 7 | ControlledShutdown | broker 有序下线 |
| 27 | WriteTxnMarkers | Transaction Coordinator 写入 broker，内部 |
| 52 | Vote | KRaft 投票 |
| 53 | BeginQuorumEpoch | KRaft |
| 54 | EndQuorumEpoch | KRaft |
| 55 | DescribeQuorum | KRaft quorum 状态 |
| 56 | AlterPartition | KRaft ISR 变更 |
| 58 | Envelope | Controller 请求转发 |
| 59 | FetchSnapshot | KRaft 日志快照 |
| 62 | BrokerRegistration | KRaft broker 注册 |
| 63 | BrokerHeartbeat | KRaft broker 心跳 |
| 64 | UnregisterBroker | KRaft broker 注销 |
| 67 | AllocateProducerIds | KRaft Controller 分配 Producer ID |
| 70 | ControllerRegistration | KRaft Controller 注册 |
| 73 | AssignReplicasToDirs | 内部副本目录分配 |
| 80 | AddRaftVoter | KRaft |
| 81 | RemoveRaftVoter | KRaft |
| 82 | UpdateRaftVoter | KRaft |
| 83 | InitializeShareGroupState | Share Group 状态内部存储 |
| 84 | ReadShareGroupState | Share Group 状态内部读取 |
| 85 | WriteShareGroupState | Share Group 状态内部写入 |
| 86 | DeleteShareGroupState | Share Group 状态内部删除 |
| 87 | ReadShareGroupStateSummary | Share Group 状态摘要内部读取 |

---

## 实现路线图

### 第一阶段：标准客户端可用

实现以下 API 后，标准 Kafka 生产者 / 消费者客户端可正常工作：

```text
ApiVersions(18) → Metadata(3) → SaslHandshake(17) / SaslAuthenticate(36)
→ Produce(0) / Fetch(1) / ListOffsets(2)
→ FindCoordinator(10) → JoinGroup(11) / SyncGroup(14) / Heartbeat(12) / LeaveGroup(13)
→ OffsetCommit(8) / OffsetFetch(9)
→ CreateTopics(19) / DeleteTopics(20) → DescribeConfigs(32)
```

共约 **20 个 API**。

### 第二阶段：事务支持

在第一阶段基础上，增加事务相关 API（22、24、25、26、28、65、66）。

### 第三阶段：管控完整性

补全 ACL、Quota、运维管理类 API。
