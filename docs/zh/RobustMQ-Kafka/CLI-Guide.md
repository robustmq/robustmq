# CLI 操作指南

RobustMQ Kafka 兼容官方 Kafka 命令行工具(`kafka-*.sh`)。本文按工具分类给出常用命令与说明。所有命令以本机 `localhost:9092` 为例。

> 开启 SASL 后,凡是需要连接 broker 的命令都要追加 `--command-config client.properties`(或对应的 `--producer.config` / `--consumer.config`)。SASL 配置示例见 [快速开始](./QuickStart.md#sasl-连接可选)。

## kafka-topics.sh — Topic 管理

```bash
# 创建
kafka-topics.sh --bootstrap-server localhost:9092 \
  --create --topic orders --partitions 6

# 列出
kafka-topics.sh --bootstrap-server localhost:9092 --list

# 查看详情(分区、leader、ISR)
kafka-topics.sh --bootstrap-server localhost:9092 --describe --topic orders

# 扩分区(只能增加)
kafka-topics.sh --bootstrap-server localhost:9092 \
  --alter --topic orders --partitions 12

# 删除
kafka-topics.sh --bootstrap-server localhost:9092 --delete --topic orders
```

> 创建时**不接受** `--replica-assignment`(手动副本分配),副本由存储层自动管理。

## kafka-console-producer.sh — 生产

```bash
# 逐行生产
kafka-console-producer.sh --bootstrap-server localhost:9092 --topic orders

# 带 key(key 与 value 用制表以外的分隔符)
kafka-console-producer.sh --bootstrap-server localhost:9092 --topic orders \
  --property parse.key=true --property key.separator=:

# 开启幂等
kafka-console-producer.sh --bootstrap-server localhost:9092 --topic orders \
  --producer-property enable.idempotence=true
```

## kafka-console-consumer.sh — 消费

```bash
# 从头消费
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic orders --from-beginning

# 以消费组消费,并打印 key
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic orders --group g1 \
  --property print.key=true --property print.offset=true
```

## kafka-consumer-groups.sh — 消费组

```bash
# 列出所有组
kafka-consumer-groups.sh --bootstrap-server localhost:9092 --list

# 查看某组位点与 lag
kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group g1

# 重置位点到最早(需组内无活跃成员)
kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --group g1 --topic orders --reset-offsets --to-earliest --execute

# 删除组
kafka-consumer-groups.sh --bootstrap-server localhost:9092 --delete --group g1
```

## kafka-get-offsets.sh — 查询 offset

```bash
# 查询每个分区的 latest offset
kafka-get-offsets.sh --bootstrap-server localhost:9092 \
  --topic orders --time latest

# earliest / 按时间戳(毫秒)
kafka-get-offsets.sh --bootstrap-server localhost:9092 --topic orders --time earliest
kafka-get-offsets.sh --bootstrap-server localhost:9092 --topic orders --time 1700000000000
```

底层走 `ListOffsets`,支持 earliest / latest / 按时间戳定位。

## kafka-configs.sh — 配置管理

```bash
# 查看 topic 配置
kafka-configs.sh --bootstrap-server localhost:9092 \
  --entity-type topics --entity-name orders --describe

# 修改 topic 配置(增量)
kafka-configs.sh --bootstrap-server localhost:9092 \
  --entity-type topics --entity-name orders \
  --alter --add-config retention.ms=604800000
```

> 大多数配置**可存储但不强制**(见 [兼容性与限制](./Compatibility-and-Limitations.md))。

### SCRAM 用户管理

`kafka-configs.sh` 同时用于管理 SASL/SCRAM 用户凭据(底层是 `AlterUserScramCredentials` / `DescribeUserScramCredentials`):

```bash
# 创建 / 更新 SCRAM-SHA-256 用户 alice
kafka-configs.sh --bootstrap-server localhost:9092 \
  --entity-type users --entity-name alice \
  --alter --add-config 'SCRAM-SHA-256=[iterations=8192,password=alice-secret]'

# 查看用户凭据
kafka-configs.sh --bootstrap-server localhost:9092 \
  --entity-type users --entity-name alice --describe

# 删除凭据
kafka-configs.sh --bootstrap-server localhost:9092 \
  --entity-type users --entity-name alice \
  --alter --delete-config 'SCRAM-SHA-256'
```

## kafka-acls.sh — ACL 管理

```bash
# 授予 alice 对 orders 的读写
kafka-acls.sh --bootstrap-server localhost:9092 \
  --add --allow-principal User:alice \
  --operation Read --operation Write --topic orders

# 查看 ACL
kafka-acls.sh --bootstrap-server localhost:9092 --list
```

> ⚠️ ACL 规则**可增删查,但不参与鉴权强制**——即使配置了拒绝规则,请求仍会被放行。ACL 目前仅作元数据管理用途。

## kafka-delegation-tokens.sh — 委托令牌

委托令牌相关命令必须走已认证连接,因此需要 `--command-config`(SASL):

```bash
# 创建令牌
kafka-delegation-tokens.sh --bootstrap-server localhost:9092 \
  --command-config client.properties \
  --create --max-life-time-period -1 --renewer-principal User:alice

# 查看
kafka-delegation-tokens.sh --bootstrap-server localhost:9092 \
  --command-config client.properties --describe
```

> 令牌为**元数据管理**,增删查改可用,但令牌本身**不参与认证**。

## kafka-cluster.sh — 集群信息

```bash
# 查询 cluster id
kafka-cluster.sh --bootstrap-server localhost:9092 --describe
```

底层走 `DescribeCluster`,返回 cluster id 与 broker 列表;controller 指向当前 meta-service Raft Leader。

## kafka-broker-api-versions.sh — API 版本协商

```bash
kafka-broker-api-versions.sh --bootstrap-server localhost:9092
```

打印 broker 通告的每个 API 及其支持的版本范围——**这是确认某个 API 是否可用的最直接方式**。未通告的 API(如事务、Share Group)不会出现在列表中。逐 API 的对照见 [协议兼容矩阵](./Protocol.md)。

## 延伸阅读

- [快速开始](./QuickStart.md)
- [协议兼容矩阵](./Protocol.md)
- [兼容性与限制](./Compatibility-and-Limitations.md)
