# 多协议架构

RobustMQ 在同一个 Broker 进程内支持 MQTT、Kafka、NATS、AMQP、mq9 五种协议。各协议共享同一套运行时、存储层和集群协调组件。

---

## 分层结构

![多协议分层架构](../../images/arch_multiprotocol_layers.png)

---

## 各层职责

### 网络层

监听各协议端口，处理连接建立和 TLS 握手，将字节流交给协议层解析。支持五种接入方式：TCP、TLS、WebSocket、WebSocket Secure、QUIC。

### 协议层

各协议独立的编解码逻辑。负责将字节流解析为协议帧，将协议帧序列化为字节流。不包含任何业务逻辑。

| 协议 | 状态 |
|------|------|
| MQTT 3.1 / 3.1.1 / 5.0 | 生产可用 |
| Kafka | 开发中 |
| NATS | 开发中 |
| AMQP | 规划中 |
| mq9 | 开发中 |

### 协议逻辑层

各协议独立的业务模块，处理协议特有的会话管理、订阅、消费组等逻辑。不同协议的 crate 之间无依赖关系。

### 消息通用逻辑层

跨协议共享的业务能力，由各协议逻辑层调用：

| 能力 | 说明 |
|------|------|
| 消息过期 | 按 Topic 配置的 TTL 清理消息 |
| 延迟发布 | 指定时间后投递的消息 |
| 安全认证 | 用户名密码、TLS 证书、Token 验证 |
| ACL | 资源级别的读写权限控制 |
| Schema 校验 | 消息 payload 格式校验 |
| 监控指标 | Prometheus 指标采集 |

### Storage Adapter

将各协议的存储概念（Topic / Partition / Queue）统一抽象为 Shard，路由到对应存储后端。Broker 无需感知底层存储类型和分布，详见 [StorageAdapter-Architecture.md](./StorageAdapter-Architecture.md)。

---

## 共享的运行时

所有协议共享：

- **Meta Service 连接**：集群协调、元数据读写共用同一个 gRPC 连接池
- **Storage Engine 连接**：消息读写共用同一套存储层，数据写入一次
- **Raft 状态机**：集群一致性由同一套 Multi Raft 保障
- **监控采集**：Prometheus 指标统一从同一个端口暴露

---

## 协议隔离原则

各协议逻辑层之间无 crate 依赖，独立演进。协议特有概念（MQTT 的 QoS、Kafka 的 Consumer Group、NATS 的 Subject）不向其他层泄漏。共享能力（认证、Schema、监控）通过独立 crate 提供，各协议按需引入。

这样设计的结果是：新增一种协议不会改动已有协议的代码，已有协议的变更也不影响其他协议。

## 协议间消息路由

当前各协议的消息存储在各自的 Shard 中，协议间不做自动路由。跨协议消息共享通过 Storage Adapter 在 Shard 层面实现：不同协议的消费者可以订阅同一个 Shard（即同一个 Topic/Partition 映射），实现跨协议消费同一份数据。

---

## 新增协议的接入范围

增加一种新协议需要实现：

1. **协议层**：编解码逻辑，解析协议帧
2. **协议逻辑层**：协议特有的会话、订阅、消费等业务逻辑
3. **网络层注册**：在对应端口注册协议处理器

不需要修改：Storage Adapter、Storage Engine、Meta Service、通用逻辑层。
