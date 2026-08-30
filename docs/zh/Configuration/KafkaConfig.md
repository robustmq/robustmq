# Kafka 配置说明

> 本文档描述 RobustMQ Kafka 协议的所有配置项。全局/基础配置请参考 [Broker 配置说明](BROKER.md)；其他协议配置见 [MQTT 配置](MQTTConfig.md)、[AMQP 配置](AMQPConfig.md)、[NATS 配置](NATSConfig.md)。

## [kafka_runtime]

Kafka 协议服务配置。

```toml
[kafka_runtime]
tcp_port = 9092
max_fetch_bytes = 4194304
max_message_bytes = 1048588
max_describe_topic_partitions = 2000
auto_create_topics_enable = true

[kafka_runtime.sasl]
enabled = false
mechanisms = ["SCRAM-SHA-256", "SCRAM-SHA-512"]
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `tcp_port` | `u32` | `9092` | Kafka 协议 TCP 监听端口 |
| `max_fetch_bytes` | `u32` | `4194304` (4 MB) | 单个 Fetch 响应里每个分区最多返回的字节数上限，不受客户端请求的 `max_bytes`/`partition_max_bytes` 影响 |
| `max_message_bytes` | `u32` | `1048588` | 单个 Produce 记录批次的最大大小（字节），对应 Kafka 的 `message.max.bytes`/`max.message.bytes`，超过会被拒绝并返回 `MESSAGE_TOO_LARGE` |
| `max_describe_topic_partitions` | `u32` | `2000` | 单次 `DescribeTopicPartitions` 响应最多返回的分区数上限，不受客户端 `response_partition_limit` 影响 |
| `auto_create_topics_enable` | `bool` | `true` | 是否在生产/消费未知 Topic 时自动创建（可通过集群动态配置覆盖，`config_type` 仍为 `KafkaDynamic`） |

**[kafka_runtime.sasl] SASL 认证配置：**

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enabled` | `bool` | `false` | 是否启用 SASL 认证；为 `false` 时连接无需认证即可接受，握手/认证处理逻辑保持空转 |
| `mechanisms` | `array` | `["SCRAM-SHA-256", "SCRAM-SHA-512"]` | Broker 对外提供的 SASL 机制列表 |

## 延伸阅读

- [Broker 配置说明](BROKER.md) — 全局/基础配置
- [RobustMQ Kafka 核心概念](../RobustMQ-Kafka/KafkaCoreConcepts.md)
- [RobustMQ Kafka SASL/SCRAM 认证](../RobustMQ-Kafka/Security/Authentication-SASL-SCRAM.md)
