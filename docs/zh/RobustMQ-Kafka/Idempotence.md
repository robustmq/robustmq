# 幂等生产

幂等生产(Idempotent Producer)保证在生产者重试时**同一条消息不会被写入多次**。RobustMQ 完整支持默认幂等 producer(`enable.idempotence=true`,现代 Kafka 客户端的默认值),无需额外配置即可获得"每条消息恰好写一次"的效果。

> **事务不支持**:带 `transactional_id` 的 `InitProducerId` 会被拒绝。幂等是事务的子集,RobustMQ 目前只提供幂等,不提供跨分区/跨会话事务。详见[兼容性与限制](./Compatibility-and-Limitations.md)。

## 工作原理

![RobustMQ Kafka 幂等生产](../../images/kafka-idempotence.svg)

### Producer ID 分配

生产者启动时先发 `InitProducerId`,Broker 分配一个 **producer id** 并返回 epoch 0。

- producer id 在 **Broker 本地单调递增**,属于单节点范畴(非全局分配)。
- epoch 用于区分同一 producer id 的不同"化身",生产者重启后会带更大的 epoch。

### 去重状态

Broker 为每个 `(producer_id, shard)` 维护一份去重状态:

| 字段 | 含义 |
|---|---|
| `epoch` | 已见过的最新 producer epoch |
| `next_seq` | 期望的下一个 base sequence |
| 窗口 | 最近 **5** 个已接受批次的 `(base_seq, base_offset)` |

窗口大小 5 对应 Kafka 默认的 `max.in.flight.requests.per.connection`——即允许最多 5 个在途请求,因此需要记住最近 5 个批次以识别其重发。

### 序列号判定

每个批次带 `base_sequence`。Broker 按如下规则判定(对应源码 `check_producer_sequence`):

| 条件 | 结果 | 返回 |
|---|---|---|
| `epoch` 比已见的更小 | 被 fence(旧化身) | `INVALID_PRODUCER_EPOCH` |
| `epoch` 比已见的更大 | 新化身,重置窗口 | 接受并写入 |
| `base_seq == next_seq` | 正常顺序 | 接受并写入 |
| `base_seq` 命中窗口内某批次 | 重复(重发) | 返回原 `base_offset`,**不重写** |
| 其它(跳号,或落在窗口之外) | 乱序 | `OUT_OF_ORDER_SEQUENCE_NUMBER` |

关键点:**重发命中窗口时返回原本的 base_offset**,不产生新记录,从而实现幂等。写入成功后 `next_seq` 前进到 `last_seq + 1`,并把该批次追加进窗口(超过 5 个则淘汰最旧的)。

## 与生产流程的关系

幂等检查发生在生产写入路径中"大小校验之后、落盘之前"的位置(见[生产者](./Producer.md#写入流程))。非幂等生产者(`producer_id < 0`)跳过整套检查。

## 限制

| 项 | 状态 |
|---|---|
| 幂等生产 | 完整支持 |
| producer id 分配范围 | Broker 本地(单节点),非跨节点全局 |
| 事务(`transactional_id`) | 不支持,`InitProducerId` 拒绝 |
| 跨会话幂等 | 依赖客户端持有的 producer id/epoch,遵循 Kafka 语义 |

## 相关文档

- [生产者](./Producer.md)
- [系统架构](./SystemArchitecture.md)
- [兼容性与限制](./Compatibility-and-Limitations.md)
