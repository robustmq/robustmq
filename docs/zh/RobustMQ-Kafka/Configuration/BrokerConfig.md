# Broker 运行时配置

RobustMQ Kafka 的配置分为两类:

- **Broker 运行时静态配置**:写在 `config/server.toml`,进程启动时加载,改动需重启节点生效。本文覆盖这一类。
- **集群动态配置**:持久化在 meta 层,可在线热改,跨重启保留。见 [动态配置机制](./DynamicConfig.md)。

Kafka 相关的静态配置集中在 `[kafka_runtime]` 段(SASL 在其子段 `[kafka_runtime.sasl]`)。所有项都有默认值,未配置时按默认值运行。

## `[kafka_runtime]`

| 配置项 | 默认值 | 说明 |
|---|---|---|
| `tcp_port` | `9092` | Kafka 协议监听端口。也是节点注册到 meta 的广告地址端口(见 [广告地址机制](../Operations/AdvertisedListeners.md))。 |
| `max_fetch_bytes` | `4194304`(4 MiB) | 单个分区在一次 `Fetch` 响应中可返回的字节上限。无论客户端 `max_bytes` / `partition_max_bytes` 请求多大,都不会超过此值。 |
| `max_message_bytes` | `1048588`(约 1 MiB) | 单个 `Produce` record batch 的大小上限。超过则拒绝并返回 `MESSAGE_TOO_LARGE`,与 Kafka 的 `message.max.bytes` / topic `max.message.bytes` 语义一致。 |
| `max_describe_topic_partitions` | `2000` | 单个 `DescribeTopicPartitions` 响应最多返回的分区数,无论客户端 `response_partition_limit` 请求多大。 |

## `[kafka_runtime.sasl]`

| 配置项 | 默认值 | 说明 |
|---|---|---|
| `enabled` | `false` | 为 `false` 时连接无需认证,SASL 握手 / 认证处理器保持静默。设为 `true` 开启 SASL 认证。 |
| `mechanisms` | `["SCRAM-SHA-256", "SCRAM-SHA-512"]` | Broker 对外提供的 SASL 机制列表,通过 `SaslHandshake` 告知客户端。 |

## 配置示例

```toml
[kafka_runtime]
tcp_port = 9092
max_fetch_bytes = 4194304
max_message_bytes = 1048588
max_describe_topic_partitions = 2000

[kafka_runtime.sasl]
enabled = true
mechanisms = ["SCRAM-SHA-256", "SCRAM-SHA-512"]
```

## 关于对外广告地址

Kafka 客户端拿到的 broker 地址并不是 `tcp_port` 单独决定的,而是 **`broker_ip`(顶层配置)+ `tcp_port`** 组合成的广告地址。`broker_ip` 未设置时使用本机自动探测的 IP。在 Docker / Kubernetes / NAT 环境中,若广告地址对客户端不可达,连接会失败。详见 [广告地址机制](../Operations/AdvertisedListeners.md)。

## 相关配置

- 顶层的 `broker_ip`、`roles`、`grpc_port`、`http_port` 等属于全局节点配置,影响所有协议。见 [部署](../Operations/Deployment.md)。
- Topic 级参数(如 `retention.ms`、`cleanup.policy`)属于动态配置,通过 Kafka 协议读写,见 [Topic 配置](./TopicConfig.md)。
