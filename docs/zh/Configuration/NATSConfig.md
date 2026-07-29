# NATS 配置说明

> 本文档描述 RobustMQ NATS（含 mq9）协议的所有配置项。全局/基础配置请参考 [Broker 配置说明](BROKER.md)；其他协议配置见 [MQTT 配置](MQTTConfig.md)、[Kafka 配置](KafkaConfig.md)、[AMQP 配置](AMQPConfig.md)。

## [nats_runtime]

NATS Core / mq9 协议服务配置。

```toml
[nats_runtime]
tcp_port = 4222
tls_port = 4223
ws_port = 4080
wss_port = 4443
max_payload = 1048576
auth_required = false
ping_interval = 60
ping_max = 3
ping_send_chunk = 10000
core_shard_num = 10
push_thread_num = 1
push_queue_thread_num = 10
mq9_mailbox_default_ttl = 86400
```

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `tcp_port` | `u32` | `4222` | NATS TCP 监听端口 |
| `tls_port` | `u32` | `4223` | NATS TLS 监听端口 |
| `ws_port` | `u32` | `4080` | NATS WebSocket 监听端口 |
| `wss_port` | `u32` | `4443` | NATS WebSocket Secure 监听端口 |
| `max_payload` | `u64` | `1048576` (1 MB) | 单条消息最大 payload 大小（字节） |
| `auth_required` | `bool` | `false` | 是否要求客户端认证 |
| `ping_interval` | `u64` | `60` | 服务端主动发送 PING 的间隔（秒） |
| `ping_max` | `u64` | `3` | 最大未回应 PING 次数，超过后断开连接 |
| `ping_send_chunk` | `usize` | `10000` | 发送 PING 时每批处理的连接数 |
| `core_shard_num` | `usize` | `10` | 内部核心分片数量 |
| `push_thread_num` | `usize` | `1` | 直接推送线程数（每个 bucket 一个线程） |
| `push_queue_thread_num` | `usize` | `10` | 队列推送线程数（每个队列组 bucket 一个线程） |
| `mq9_mailbox_default_ttl` | `u64` | `86400` | mq9 Mailbox 默认 TTL（秒），客户端未指定时使用 |

## 延伸阅读

- [Broker 配置说明](BROKER.md) — 全局/基础配置
- [RobustMQ NATS 概览](../nats/Overview.md)
- [JetStream](../nats/JetStream.md)
