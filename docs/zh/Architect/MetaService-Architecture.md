# Meta Service 架构

Meta Service 是 RobustMQ 内置的元数据管理组件，类似于 ZooKeeper 之于 Kafka、NameServer 之于 RocketMQ。不同的是，Meta Service 同时承担了**集群协调、元数据存储、KV 业务数据存储、集群控制器**四个角色，是 RobustMQ 实现零外部依赖的核心。

---

## 技术架构

![meta service arch](../../images/meta-service-arch.png)

**技术栈：gRPC + Multi Raft（openraft）+ RocksDB**

- 节点间通过 gRPC 通信，对外同样通过 gRPC 提供服务
- 基于 [openraft](https://github.com/datafuselabs/openraft) 实现 Multi Raft，保证多节点数据一致性
- RocksDB 负责所有数据的持久化存储（包括 Raft 日志和快照）

---

## 核心职责

| 职责 | 说明 |
|------|------|
| **集群协调** | 节点发现、上下线管理、节点间数据分发 |
| **元数据存储** | Broker 节点信息、Topic 配置、Schema、Connector 配置、Storage Engine 分片元数据 |
| **KV 业务数据** | MQTT Session、保留消息、遗嘱消息、订阅关系、ACL、黑名单等运行时数据 |
| **消费位点** | 消费组的 Offset 提交与管理 |
| **控制器** | Session 过期清理、Last Will 延迟发送、Storage Engine GC、Connector 任务调度 |

---

## Multi Raft 架构

为解决单 Raft 的写入吞吐瓶颈，Meta Service 采用多 Raft 状态机架构（`MultiRaftManager`），目前包含三个独立的 Raft Group，每个 Group 有独立的 Leader 和存储：

| Raft Group | 存储内容 |
|-----------|---------|
| **metadata** | 集群节点信息（ClusterAddNode / DeleteNode）、KV 通用存储、Schema、资源配置、Storage Engine Shard / Segment 元数据 |
| **offset** | 消费组 Offset 提交与管理 |
| **mqtt** | MQTT 业务数据：用户、Topic、Session、保留消息、遗嘱消息、订阅关系、ACL、黑名单、Connector、自动订阅规则、共享订阅组 Leader |

三个 Group 并行工作，互不阻塞。当某类数据写入压力增大时（如高频 Offset 提交），可独立扩展对应 Group 的数量，实现线性扩展。

**Raft 参数配置：**

| 参数 | 值 |
|------|-----|
| heartbeat_interval | 250ms |
| election_timeout_min | 299ms |
| write_timeout | 30s（可配置） |
| 慢写告警阈值 | 1000ms |

---

## 写入路径

```
Broker / Storage Engine
        │  gRPC 调用
        ▼
  Meta Service gRPC Server
        │
        ▼
   MultiRaftManager
  ┌─────┬──────┬──────┐
  │meta │offset│ mqtt │  ← 三个独立 Raft Group
  └──┬──┴──┬───┴──┬───┘
     │     │      │   通过 Raft 共识同步到集群所有节点
     ▼     ▼      ▼
  DataRoute（路由到对应处理逻辑）
        │
        ▼
    RocksDB（持久化）
```

写入时附带超时控制，超过 `write_timeout`（默认 30s）则返回错误；超过 1000ms 记录 warn 日志，便于排查慢写问题。

---

## 数据存储设计

Meta Service 通过 RocksDB 持久化所有数据：

- **Raft 日志**：存储在 RocksDB 中，节点重启后可完整恢复
- **Raft 快照**：定期生成快照压缩日志，加快节点恢复速度
- **业务数据**：通过 DataRoute 路由写入对应的 RocksDB Column Family
- **内存缓存**：CacheManager 维护热数据的内存缓存，降低 RocksDB 读压力；严格控制内存使用，冷数据直接读写 RocksDB

这一设计使 Meta Service 可以支撑百万级 Topic、亿级 Session 等大规模元数据场景，不受内存容量限制。

---

## 控制器（BrokerController）

Meta Service 的 Leader 节点在启动后会运行 `BrokerController`，负责集群的后台调度任务：

| 后台任务 | 说明 |
|---------|------|
| **Session 过期清理** | 定期扫描过期 Session，清理相关数据 |
| **Last Will 延迟发送** | 检测到期的遗嘱消息，触发发送到 Broker |
| **Storage Engine GC** | 清理已删除 Shard / Segment 的残留数据 |
| **Connector 调度** | 管理 MQTT Connector 任务的创建、分配和状态跟踪 |

---

## 启动流程

1. 节点启动，读取配置中的 `meta_addrs` 获取所有 Meta Node 地址
2. 初始化 `MultiRaftManager`，依次创建 `metadata`、`offset`、`mqtt` 三个 Raft Group
3. 通过 gRPC 与所有节点建立连接，按照 Raft 协议完成集群初始化和 Leader 选举
4. Leader 节点启动 `BrokerController`，开始后台调度任务
5. Meta Service 就绪，开始对 Broker 和 Storage Engine 提供 gRPC 服务

---

## 网络层规划

当前使用 gRPC 通信，理论上性能表现良好。在极端场景下（如集群重启时亿级连接同时发起，Broker 对 Meta Service 高频访问），可能出现瓶颈。

针对此设计了两类优化方向：

1. **Batch 语义**：支持批量操作，例如一次调用创建/更新多个 Session，降低 RPC 调用频次（短期优先）
2. **协议替换**（长期规划）：若 gRPC 确实成为瓶颈，可替换为 TCP 或 QUIC

---

## 与 ZooKeeper / etcd 的对比

| 维度 | ZooKeeper | etcd | Meta Service |
|------|-----------|------|--------------|
| 架构 | 单 Leader（ZAB） | 单 Raft | **Multi Raft** |
| 存储 | 全内存 | BoltDB | RocksDB |
| 扩展性 | 受内存限制 | 受单 Raft 限制 | **可独立扩展各 Raft Group** |
| 功能范围 | 元数据协调 | 元数据协调 | 元数据 + KV 存储 + 控制器 |
| 外部依赖 | 是 | 是 | **否（内置）** |
| 消息队列适配 | 通用 | 通用 | **针对 MQ 场景深度定制** |

Meta Service 自研内置的核心动机：通用的 ZooKeeper/etcd 无法针对消息队列的特殊场景进行深度优化。作为内置组件，Meta Service 可以在性能、功能、稳定性上持续迭代，同时保持 RobustMQ 零外部依赖的架构特性。
