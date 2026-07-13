# 消费组(经典协议)

消费组(Consumer Group)让多个消费者协同消费一个 topic:每个分区在同一时刻只被组内一个成员消费,成员增减时自动**再均衡**(rebalance)。本文介绍**经典协议**——基于 `JoinGroup` / `SyncGroup` / `Heartbeat` 的全链路。新一代 KIP-848 协议见[新一代消费组协议](./ConsumerGroupNext.md)。

## 协调器

消费组的协调器(Coordinator)就是当前 **meta-service Raft Leader**。客户端先用 `FindCoordinator` 定位协调器所在节点,再向它发起后续请求。若请求发到非协调器节点,返回 `NOT_COORDINATOR` 让客户端重定向。

## 全链路

![RobustMQ Kafka 经典消费组](../../images/kafka-group-rebalance.svg)

| 步骤 | API | 说明 |
|---|---|---|
| 1. 定位协调器 | `FindCoordinator` | 找到 group 对应的协调器节点 |
| 2. 首轮加入 | `JoinGroup` | 首次不带 member id,协调器返回 `MEMBER_ID_REQUIRED` 并分配一个 member id |
| 3. 再次加入 | `JoinGroup` | 带上 member id 重新加入;协调器选出一个成员作为 **leader**,把成员列表发给它 |
| 4. 同步分配 | `SyncGroup` | **leader 客户端**计算分区分配方案并上报;Broker 只做**中继**,把每个成员各自的分配下发回去 |
| 5. 维持成员 | `Heartbeat` | 周期心跳保活;协调器发起再均衡时返回 `REBALANCE_IN_PROGRESS` 提示重新加入 |
| 6. 离开 | `LeaveGroup` | 主动离组,触发再均衡 |

> 关键设计:**分配方案由客户端 leader 计算,Broker 只中继**。具体的分配策略(assignor,如 range、roundrobin)完全由客户端决定,Broker 不参与计算。

## 位点提交与续读

成员在消费过程中通过 `OffsetCommit` 提交位点(`commitSync` 即同步提交并等待确认)。位点持久化在 meta 层,因此:

- 组内**新成员**接手某分区时,从该分区**已提交位点**继续读,不重复也不丢(位点缺失时按 `auto.offset.reset`)。
- 位点的查询、重置、lag 计算见[位点管理](./OffsetManagement.md)。

## 再均衡

成员集合变化时触发再均衡,分区在新成员间重新切分:

| 场景 | 结果 |
|---|---|
| 双消费者订阅同一 topic | 分区在两者间切分(如各占一半) |
| 一个成员离开 | 其分区重新分配给剩余成员 |
| 新成员加入 | generation 递增,重新分配 |

再均衡期间,协调器通过心跳返回 `REBALANCE_IN_PROGRESS`,各成员重新 `JoinGroup` → `SyncGroup` 完成新一轮分配。

## 相关命令与文档

- 命令行:`kafka-consumer-groups.sh --describe/--list` 查看组状态与 lag。
- [位点管理](./OffsetManagement.md)
- [新一代消费组协议(KIP-848)](./ConsumerGroupNext.md)
- [消费者](./Consumer.md)
- [系统架构](./SystemArchitecture.md)
