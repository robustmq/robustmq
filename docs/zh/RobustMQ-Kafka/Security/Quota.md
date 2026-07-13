# 客户端配额

RobustMQ Kafka 实现了客户端配额的管理接口(`AlterClientQuotas` / `DescribeClientQuotas`),可设置与查询配额并持久化到 meta-service。

> **现状(务必留意):配额可设置、查询并持久化,但尚未接入限流强制。** Produce / Fetch 路径不读取配额,设置后**不会真正限流**;配额虽已加载进 Broker 缓存,但没有任何调用点据此对生产/消费做节流。配额目前是"可管理的元数据"。

## 支持范围

| 维度 | 支持情况 |
|---|---|
| 实体类型 | **仅 `client-id`**;其它(如 `user`)返回 `InvalidRequest` |
| 实体维度 | 仅单维度实体;多维度返回 `InvalidRequest` |
| 配额键 | `producer_byte_rate`、`consumer_byte_rate` |
| 键取值 | 非删除操作要求值为正且有限,否则 `InvalidRequest` |
| 实体名 | 不可为空,不可为保留名 `__default__` |

## CLI 示例

```bash
# 为 client-id = svc-a 设置生产/消费限速(字节/秒)
kafka-configs.sh --bootstrap-server localhost:9092 \
  --alter --entity-type clients --entity-name svc-a \
  --add-config 'producer_byte_rate=1048576,consumer_byte_rate=2097152'

# 查询
kafka-configs.sh --bootstrap-server localhost:9092 \
  --describe --entity-type clients --entity-name svc-a

# 删除某项
kafka-configs.sh --bootstrap-server localhost:9092 \
  --alter --entity-type clients --entity-name svc-a \
  --delete-config 'producer_byte_rate'
```

`DescribeClientQuotas` 的过滤匹配支持 `EXACT`(精确)、`DEFAULT`(默认)、`ANY`(任意)三种匹配方式,并可选 `strict`。

配额经 Raft 持久化并广播到各 Broker 缓存(见 [安全总览](./Overview.md#数据持久化路径))。

## 限制小结

| 限制 | 说明 |
|---|---|
| 未强制 | 不参与限流,设置后不会节流生产/消费 |
| 实体类型 | 仅 `client-id`,不支持 user / user+client-id |
| 配额键 | 仅字节速率两项,无请求速率(`request_percentage`)等 |
