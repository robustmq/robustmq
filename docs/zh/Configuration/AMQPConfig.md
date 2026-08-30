# AMQP 配置说明

> 本文档描述 RobustMQ AMQP 协议的所有配置项。全局/基础配置请参考 [Broker 配置说明](BROKER.md)；其他协议配置见 [MQTT 配置](MQTTConfig.md)、[Kafka 配置](KafkaConfig.md)、[NATS 配置](NATSConfig.md)。

## [amqp_runtime]

AMQP 协议服务配置。目前只有端口配置，没有 TLS 端口或动态配置项——详见 [AMQP 兼容性与限制](../RobustMQ-AMQP/Compatibility-and-Limitations.md)。

```toml
[amqp_runtime]
tcp_port = 5672
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `tcp_port` | `u32` | `5672` | AMQP 协议 TCP 监听端口，与 RabbitMQ 默认端口一致 |

## 延伸阅读

- [Broker 配置说明](BROKER.md) — 全局/基础配置
- [RobustMQ AMQP 核心概念](../RobustMQ-AMQP/AMQPCoreConcepts.md)
- [AMQP 兼容性与限制](../RobustMQ-AMQP/Compatibility-and-Limitations.md)
