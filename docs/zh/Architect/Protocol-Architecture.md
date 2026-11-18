# 协议编解码架构

本文档说明内部 `src/protocol` crate 的实现。它面向贡献者；面向用户的协议行为应以 MQTT、Kafka、NATS 和 AMQP 各自的文档为准。

## 职责范围

该 crate 负责以下协议的线上数据类型和编解码：

- MQTT 3.1、3.1.1 和 5.0
- Kafka 请求和响应
- AMQP 0-9-1 帧
- NATS Core 命令和消息
- 内部 Storage Engine 协议

协议业务行为仍由对应的 broker crate 负责。协议 crate 不执行 Topic 路由、认证决策、订阅管理或存储操作。

## 统一分发

`src/protocol/src/codec.rs` 中的 `RobustMQCodec` 包装各协议 codec，并通过统一的 `tokio_util::codec::{Decoder, Encoder}` 接口工作。解码结果使用 `RobustMQCodecWrapper` 表示：

| 枚举变体 | 内容 |
|---|---|
| `MQTT` | `MqttPacketWrapper`，包含协商出的 MQTT 版本 |
| `KAFKA` | `KafkaPacketWrapper` |
| `AMQP` | 一个 `AMQPFrame` |
| `NATS` | `NatsPacket` |
| `StorageEngine` | `StorageEnginePacket` |

已知协议的网络接入器使用 `RobustMQCodec::new_with_protocol` 创建 codec。MQTT 监听器使用未预设版本的 codec，由第一个 `CONNECT` 报文选择 MQTT 3、4 或 5。未预设协议的 codec 不会对 Kafka、AMQP、NATS 或 Storage Engine 流量执行通用自动识别。

## 协议标识

`RobustMQProtocol` 是内部路由标识，不是线上协议注册表。其数值用于连接元数据和内部报文包装：

| 协议 | 内部值 |
|---|---:|
| MQTT3 / MQTT4 / MQTT5 | 3 / 4 / 5 |
| Kafka | 0 |
| Storage Engine | 10 |
| AMQP | 11 |
| NATS | 12 |

`from_u8` 是兼容性回退逻辑：它识别 MQTT4、MQTT5 和 NATS，其余数值都映射为 MQTT3。因此不能把它当作 `to_u8` 对所有协议值的可逆转换。

## 帧处理和半包读取

所有 codec 都会保留输入缓冲区中的不完整字节，在尚未收到完整帧或完整命令时返回 `Ok(None)`。AMQP 和 Kafka 使用带长度字段的二进制帧；NATS 使用 CRLF 结束的控制行，并为 `PUB`、`HPUB`、`MSG` 和 `HMSG` 追加 payload 与结尾 CRLF；MQTT 使用固定报头和可变长度的 remaining length 字段。

## 错误处理

直接使用协议专用 codec 时，会返回该协议自己的错误。通过 `RobustMQCodec` 分发时，MQTT、Kafka、AMQP 和 Storage Engine 的解码失败目前会在分发路径中视为没有解码出报文；NATS 的解码错误会转换为 `CommonError` 并向上传递，以便关闭连接，而不是让连接一直等待更多字节。需要精确协议错误时，应直接使用对应的协议 codec。

## 边界

该 crate 定义线上数据表示和序列化方式。解码后的报文如何处理、接受、拒绝、持久化、确认或投递，由各 broker crate 决定。修改协议语义时，除了更新本文档，还应同步更新对应的协议兼容文档和测试。
