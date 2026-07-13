# 路线图

RobustMQ Kafka 已经能让标准 Kafka 客户端端到端跑通,协议覆盖面较广;同时仍有一批行为在持续深化(尤其是"配置 / ACL / 配额的强制生效"与"事务")。本页给出诚实的现状。

![RobustMQ Kafka 路线图](../../images/kafka-roadmap.svg)

## 已完成

| 能力 | 说明 |
|---|---|
| 数据面 | `Produce` / `Fetch`(长轮询)/ `ListOffsets` |
| 幂等生产 | 幂等 producer(去重,避免重试重复写入) |
| 消费组 | 经典协议 + 新一代 KIP-848(服务端分配)并存 |
| Topic 管理 | 创建 / 删除 / 扩分区 |
| 配置读写 | `DescribeConfigs` / `AlterConfigs` / `IncrementalAlterConfigs`(存取 + 回显) |
| 认证 | SASL / SCRAM(SHA-256 / SHA-512) |
| ACL / 配额 | 元数据的存取(尚未强制) |
| 委托令牌 | Delegation Token 创建 / 续期 / 过期 / 查询 |
| CLI 兼容 | 官方 `kafka-*.sh` 工具可用 |
| 集群元数据 | `Metadata` / `DescribeCluster` / `DescribeTopicPartitions` |

## 进行中 / 规划中

| 能力 | 说明 |
|---|---|
| Fetch 压缩 | `Fetch` 响应的压缩编码 |
| 增量 fetch session | incremental fetch session,减少全量元数据传输 |
| `leader_epoch` 语义 | 更完整的 leader epoch fencing / 校验 |
| 配置强制生效 | 让已存储的 topic 配置(如 `retention.ms`、`cleanup.policy`)真正驱动引擎行为 |
| ACL / 配额强制 | 从"仅存元数据"到运行时强制 |
| 事务 | Exactly-once 事务语义 |
| Share Group | KIP-932 共享消费模型 |
| 落盘压缩 | 存储层的 record 压缩 |

## 如何理解"已存取但未强制"

RobustMQ 的许多管控能力遵循"元数据先行"的节奏:接口、持久化与回显先就绪(客户端可正常读写、CLI 可正常展示),行为强制生效随后补齐。因此:

- **配置**:`AlterConfigs` 能写、`DescribeConfigs` 能正确回显动态来源,但多数键当前只存储、尚未全部改变引擎行为。见 [Topic 配置](./Configuration/TopicConfig.md)。
- **ACL / 配额**:规则可存取,但运行时强制尚在规划。

相关文档:[系统架构](./SystemArchitecture.md) · [协议兼容矩阵](./Protocol.md) · [Topic 配置](./Configuration/TopicConfig.md)。
