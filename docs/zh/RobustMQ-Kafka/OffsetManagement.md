# 位点管理

位点(offset)记录消费组在每个分区读到了哪里。RobustMQ 把位点**持久化在 meta 层**,提供提交、查询、重置、删除以及 lag 计算等完整的 Kafka 位点管理能力。本文覆盖对应的 API 与 `kafka-consumer-groups.sh` 命令行操作。

## 提交与查询

| API | 作用 | 说明 |
|---|---|---|
| `OffsetCommit` | 提交位点 | 消费组把某分区的消费进度写入 meta(`commitSync` 为同步提交) |
| `OffsetFetch` | 查询已提交位点 | v8+ 支持**一次请求批量查询多个 group** |

位点持久化在 meta 层,消费组重启或成员变更后可从已提交位点续读(见[消费组](./ConsumerGroup.md))。

## 重置位点(reset-offsets)

`kafka-consumer-groups.sh --reset-offsets` 用于把消费组位点改到指定位置:

| 目标 | 含义 |
|---|---|
| `--to-earliest` | 重置到最早可用 offset |
| `--to-latest` | 重置到最新 offset |
| `--to-offset <n>` | 重置到指定 offset |
| `--shift-by <n>` | 在当前位点上前移/后移 n(负数回退) |

执行模式:

| 模式 | 行为 |
|---|---|
| `--dry-run` | 仅预览将要变更的结果,不落库 |
| `--execute` | 实际提交变更 |

## 删除位点(delete-offsets)

`kafka-consumer-groups.sh --delete-offsets` 删除某消费组在指定 topic/分区上的已提交位点。删除后该分区无已提交位点,下次消费按 `auto.offset.reset` 起始(见[消费者](./Consumer.md#起始位点auto-offset-reset))。

## Lag(消费滞后)

`kafka-consumer-groups.sh --describe` 展示每个分区的消费进度:

| 列 | 含义 |
|---|---|
| `CURRENT-OFFSET` | 消费组已提交的位点 |
| `LOG-END-OFFSET` | 分区当前最新 offset(末端) |
| `LAG` | 滞后量 = `LOG-END-OFFSET − CURRENT-OFFSET` |

lag 越大表示消费越落后于生产。

## 持久化

所有位点都持久化在基于 Raft 的 **meta-service** 中,与集群动态配置、ACL/SCRAM/配额同层管理(见[系统架构](./SystemArchitecture.md))。协调器(Raft Leader)负责位点的读写。

## 相关文档

- [消费组(经典协议)](./ConsumerGroup.md)
- [新一代消费组协议(KIP-848)](./ConsumerGroupNext.md)
- [消费者](./Consumer.md)
- [系统架构](./SystemArchitecture.md)
