# Broker 配置

AMQP 相关的 Broker 配置非常精简,目前只有一个端口配置项,其余行为(超时、心跳、认证等)复用整个 RobustMQ 的统一配置。

## 配置项

在配置文件(如 `config/server.toml`)中,AMQP 对应 `[amqp_runtime]` 段:

```toml
[amqp_runtime]
tcp_port = 5672
```

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `tcp_port` | u32 | `5672` | AMQP 协议监听的 TCP 端口,与 RabbitMQ 默认端口一致 |

目前**没有** `tls_port` 字段——AMQP 不支持 TLS,详见 [安全概览](../Security/Overview.md)。

## 与其他协议共用的配置

AMQP Broker 复用了 RobustMQ 统一的以下配置,不需要单独为 AMQP 配置:

- **用户认证数据**:与 MQTT、Kafka 共用同一份用户表,参见 [认证(SASL)](../Security/Authentication-SASL.md)。
- **存储引擎参数**(File Segment 的 segment 大小、清理策略等):在 Broker 级别统一配置,不区分协议,参见 [存储](../Storage.md)。
- **meta-service/Raft 集群配置**:集群拓扑、选举参数等与协议无关。

## 示例:多协议共存的完整片段

```toml
[network]
# 通用网络配置

[mqtt_runtime]
tcp_port = 1883

[kafka_runtime]
tcp_port = 9092

[amqp_runtime]
tcp_port = 5672
```

修改 `tcp_port` 后需要重启 Broker 才能生效,目前没有支持 AMQP 端口的动态热更新。

## 延伸阅读

- [安全概览](../Security/Overview.md)
- [Exchange 与 Queue 相关配置](./ExchangeAndQueueConfig.md)
- [快速开始](../QuickStart.md)
