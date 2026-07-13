# 生产者

生产者(Producer)通过 `Produce` API 把消息写入 topic 分区。RobustMQ 兼容标准 Kafka 生产者:acks 确认语义、客户端侧分区路由、批量写入都按 Kafka 协议行为工作。本文介绍消息进入 RobustMQ 后的写入路径与相关限制。

## 写入流程

![RobustMQ Kafka 生产流程](../../images/kafka-produce-flow.svg)

一次 `Produce` 请求在 Broker 内的处理顺序:

1. **解码**:协议层解出 `ProduceRequest`,包含若干 topic/分区,每个分区携带一个 record 批次。
2. **校验**:核心层校验 acks 取值,再校验批次大小不超过 `max.message.bytes`。
3. **幂等检查**:若批次带 producer id(幂等生产者),按序列号去重(见[幂等生产](./Idempotence.md))。
4. **解批入库**:把批次解开为**协议中立的 record**,交由 File Segment 存储引擎追加写。
5. **分配 offset**:offset 由存储层在写入时分配,同一批次内连续递增。
6. **响应**:每个分区返回其 `base_offset`(该批次第一条消息的 offset)。`acks=0` 时不返回响应。

## acks 确认语义

`acks` 决定 Broker 在何时向生产者确认写入。

| acks | 含义 | 是否返回响应 |
|---|---|---|
| `-1`(`all`) | 等待副本按存储层策略持久化后确认 | 是 |
| `1` | Leader 写入后即确认 | 是 |
| `0` | 不等待确认,发完即忘 | **否** |

> `acks=0` 时生产者不期待任何响应,Broker 也不回包;因此该模式下无法感知写入失败。

## 分区路由

分区选择发生在**客户端侧**,Broker 只按请求中已指定的分区写入:

| 场景 | 客户端行为 |
|---|---|
| 指定了 key | 对 key 哈希,固定映射到同一分区(保证同 key 有序) |
| 没有 key | 在分区间分散(轮询 / 粘性,取决于客户端实现) |

## 批量与 offset 连续性

一次 `Produce` 请求的一个分区可携带**多条 record**。写入时存储层为整批分配**连续的 offset**,批次内消息顺序与 offset 递增顺序一致。响应里的 `base_offset` 指向批次第一条,后续消息的 offset 依次为 `base_offset + 1`、`base_offset + 2` ……

## 消息大小限制

单个批次的大小上限由 `max.message.bytes` 控制,默认 **1048588** 字节(1 MiB + 批次头开销,与 Kafka 默认一致)。超过上限的批次在解码/写入前即被拒绝,返回 `MESSAGE_TOO_LARGE`。

## 压缩

生产侧压缩**全部支持**:客户端可用 gzip、snappy、lz4、zstd 压缩批次,Broker 在解批时解压。

> **限制**:由于存储保存的是解码后的 record,消费侧 `Fetch` 目前**固定返回未压缩批次**,不还原生产时的压缩编码。

## 已知限制

| 项 | 状态 |
|---|---|
| LogAppendTime | 暂未应用(消息时间戳按客户端 CreateTime) |
| 事务性生产 | 不支持(见[幂等生产](./Idempotence.md)与[兼容性与限制](./Compatibility-and-Limitations.md)) |
| Fetch 侧压缩 | 固定返回未压缩 |

## 相关文档

- [幂等生产](./Idempotence.md)
- [消费者](./Consumer.md)
- [系统架构](./SystemArchitecture.md)
- [协议兼容矩阵](./Protocol.md)
