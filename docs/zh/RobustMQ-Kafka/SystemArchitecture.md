# 系统架构

RobustMQ 的 Kafka 能力并不是一个独立的 Kafka 服务,而是构建在 RobustMQ 统一内核之上的一层 **Kafka 协议兼容层**。它复用 RobustMQ 的网络框架、File Segment 存储引擎与基于 Raft 的元数据服务,对外呈现为一个标准的 Kafka Broker——原生 Kafka 客户端、官方命令行工具都可以直接连接。

其最重要的设计取向是**"一份数据、多协议视图"**:Kafka 与 MQTT 共享同一份 topic 存储和元数据,因此存储层保存的是**协议中立的、已解码的消息记录**,而不是某个协议私有的字节格式。这一取向决定了后文诸多实现选择(例如为什么不采用 Kafka 的"原样存储压缩批次")。

## 分层架构

![RobustMQ Kafka 系统架构](../../images/kafka-architecture.svg)

自上而下分为五层:

### 1. 客户端层

任何标准 Kafka 客户端都可接入:Java `kafka-clients`(Producer / Consumer / AdminClient)、官方命令行工具(`kafka-topics.sh`、`kafka-console-producer.sh`、`kafka-consumer-groups.sh` 等)、以及 librdkafka 系列客户端。客户端通过 `ApiVersions` 与 Broker 协商可用的 API 及版本,再据此收发请求。

### 2. 协议层(`src/protocol`)

基于 `kafka-protocol` 0.17 完成 Kafka 线上协议的**编解码**。要点:

- **响应头版本按 API 逐个派生**:Kafka 各 API 变为 flexible(带 tagged fields)的版本不同,不能一刀切。协议层依据每个响应类型自身的 `header_version(api_version)` 计算头版本,否则会给非 flexible 的响应(如 Produce v7)多写一个字节,导致客户端解析错位。
- **Handler 分发**(`command.rs`):按 API key 将请求路由到对应处理函数,当前覆盖 66 个 Kafka API。
- **网络层**:请求按连接分片,保证同一连接上的响应严格保序(Kafka 客户端依赖 correlation_id 配对,但保序可避免一类竞态)。

### 3. 核心处理层(`src/kafka-broker`)

Kafka 语义的实现所在:

- **数据面**:`Produce`(含幂等)、`Fetch`(长轮询)、`ListOffsets`。
- **消费组**:经典协议(`FindCoordinator` / `JoinGroup` / `SyncGroup` / `Heartbeat` / `LeaveGroup`)与新一代 KIP-848 协议(`ConsumerGroupHeartbeat`,服务端分配)并存。
- **元数据与管理**:`Metadata`、`DescribeCluster`、Topic 创建/删除/扩分区、配置读写。
- **安全**:SASL/SCRAM 认证、ACL、客户端配额、委托令牌。

### 4. 存储层(File Segment 引擎)

通过 `StorageDriverManager` 访问 File Segment 引擎:

- **段(Segment)**:追加写,达到阈值后 seal 并 scroll 到新段。
- **offset→position 索引 + mmap 读**:按 offset 定位物理位置。
- **ISR 多副本**:多副本复制,`leader_epoch` 做 fencing。

存储保存的是解码后的记录:Kafka `Produce` 写入时把批次解开入库,`Fetch` 时再重组为 `RecordBatch`。因此同一个 topic 也能被 MQTT 等其它协议读写——这正是多协议互通的基础。

### 5. 元数据层(meta-service · Raft)

集群元数据由基于 Raft 的 meta-service 管理:

- **Controller / Coordinator 即 Raft Leader**:Kafka 语义中的 controller id、消费组协调器、事务协调器都定位到当前 Raft Leader(经约 3 秒 TTL 缓存的 gRPC 查询)。
- **节点注册表**:各节点注册其对外地址(见 [广告地址机制](./Operations/AdvertisedListeners.md)),供 `Metadata`/`FindCoordinator` 告知客户端。
- **动态配置与位点**:集群动态配置(如 `auto.create.topics.enable`)、消费组位点、ACL/SCRAM/配额均持久化在此并可热更新。

## 一次请求的走向

- **数据面(Produce / Fetch)**:协议层解码 → 核心层处理(幂等校验、大小校验)→ 存储层写入/读取 → 组装响应。写入时由存储层分配连续 offset。
- **元数据 / 协调 / 认证 / 配置**:协议层解码 → 核心层处理 → 读写 Raft 元数据层。只有当前节点是 Raft Leader(coordinator)时才承担协调职责,否则返回 `NOT_COORDINATOR` 让客户端重定向。

## 与原生 Kafka 的关键差异

| 维度 | 原生 Kafka | RobustMQ |
|---|---|---|
| 存储单元 | 原样存储压缩后的 RecordBatch(Fetch zero-copy) | 协议中立的解码记录,Fetch 时重组 |
| Controller | KRaft / ZooKeeper | meta-service Raft Leader |
| 多协议 | 仅 Kafka | Kafka / MQTT 共享同一份数据 |
| 事务 | 支持 | 暂不支持(见[兼容性与限制](./Compatibility-and-Limitations.md)) |

> 关于逐 API 的支持状态,见 [协议兼容矩阵](./Protocol.md)。
