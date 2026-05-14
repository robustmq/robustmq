# 常见问题

## 使用 mq9 需要特殊 SDK 吗？

不需要。任何 NATS 客户端都可以直接使用——Go、Python、Rust、JavaScript、Java、.NET 或 NATS CLI。mq9 是在 NATS 之上定义的 subject 命名约定，所有操作均通过 NATS request/reply 完成。RobustMQ SDK 提供类型化封装和异步模式，但完全是可选的。

---

## 发送消息时接收方不在线会怎样？

消息立即写入服务端存储。接收方随时可以调用 FETCH 拉取——哪怕几分钟或几小时后——所有未过期消息会按优先级顺序返回。消息不会因接收方离线而丢失。

---

## FETCH 和 QUERY 有什么区别？

`FETCH`（`$mq9.AI.MSG.FETCH.*`）是消费操作，配合 `group_name` 时 broker 会记录消费位点，ACK 后推进，下次 FETCH 从断点续拉，不重复消费。

`QUERY`（`$mq9.AI.MSG.QUERY.*`）是查询操作，返回邮箱中当前存储的消息，**不影响消费位点**，可按 key、tags、since 过滤。两次 QUERY 返回相同结果（消息没有变化时），适合调试和状态检视。

---

## 可以在创建后修改邮箱的 TTL 吗？

不可以。TTL 在创建时固定，不可更改，也不可续期。重复以相同名称 CREATE 会返回错误（`mailbox xxx already exists`）。要更改 TTL，必须等邮箱过期后以新值重新创建。

---

## 邮箱过期后会发生什么？

邮箱及其所有消息自动销毁，无需客户端清理。消费组的位点记录也随之清除。过期时不会向客户端发送任何通知。

---

## 可以有多个 Agent 向同一邮箱写入吗？

可以。任何知道 `mail_address` 的 Agent 都能发送消息，没有发送方白名单或所有权限制。私有邮箱通过保密 `mail_address` 实现访问控制；公开邮箱则任何知道名称的 Agent 都能发送。

---

## 多个 Worker 如何竞争消费同一邮箱？

多个 Worker 使用**相同的 `group_name`** 调用 FETCH，broker 保证每条消息只被其中一个 Worker 拿到（通过消费位点推进实现）。Worker 各自独立调用 FETCH，处理完后 ACK，broker 推进位点后该消息不会再被其他 Worker 重复消费。

Worker 可以随时加入或退出，位点由 broker 维护，无需客户端协调。

---

## ACK 的 msg_id 应该传哪个值？

传本次 FETCH 返回的**最后一条消息**的 `msg_id`。broker 会将该 group 的消费位点推进到此 msg_id，之后的 FETCH 从这里续拉。不需要对每条消息单独 ACK——一次 ACK 确认本批所有消息。

---

## 重连后优先级如何工作？

有状态消费（传 `group_name`）：重连后调用 FETCH，broker 从上次 ACK 位置续拉，按优先级顺序（`critical` → `urgent` → `normal`）返回未消费消息，同优先级内保持 FIFO 顺序。

无状态消费（不传 `group_name`）：每次 FETCH 按 `deliver` 策略独立拉取，不记录位点。

---

## mq9 是 MQTT 或 Kafka 的替代品吗？

不是。mq9 专门为 AI Agent 异步通信设计。MQTT 是 IoT 遥测和设备消息的正确选择。Kafka 是高吞吐量事件流和数据管道的正确选择。mq9 解决 Agent 邮箱问题：临时通道、离线容错投递、轻量 TTL 生命周期。三种协议可以在同一个 RobustMQ 部署上同时运行，零桥接。

---

## 消息体可以有多大？

目前暂无硬性限制。对于超大二进制传输（模型、数据集、文件），建议将数据存储在外部对象存储，在 mq9 消息体中传递引用 URL 或对象键，以保持消息轻量。

---

## 不用 RobustMQ 能用 mq9 吗？普通 NATS 服务器可以吗？

不可以。mq9 的消息持久化、优先级排序、TTL 自动清理、消费位点管理和 Agent 注册表均在 RobustMQ 服务端内部实现。普通 NATS 服务器不支持这些功能。NATS 客户端库用作传输层，但服务端必须是 RobustMQ。

---

## 需要处理哪些错误？

所有响应均包含 `error` 字段，为空字符串表示成功，非空字符串为错误描述。常见错误：

| 错误描述 | 触发原因 |
|---------|---------|
| `mailbox xxx already exists` | 重复 CREATE 同名邮箱 |
| `mailbox not found` | 邮箱不存在或已过期 |
| `message not found` | 指定 msg_id 的消息不存在或已过期 |
| `invalid mail_address` | mail_address 格式不合法（含大写、连字符等） |
| `agent not found` | UNREGISTER 或 REPORT 时 Agent 名称不存在 |

---

## mq9 和 NATS JetStream 有什么区别？

JetStream 为 NATS 添加了流式持久化——是一个完整的类 Kafka 系统，包含命名流、持久化消费者、消息序列和重放功能。mq9 针对 Agent 场景做了专门优化：FETCH+ACK pull 消费、三级优先级、消息属性（key/tags/delay/ttl）、内置 Agent 注册表，没有流或 stream 概念。JetStream 更适合大规模事件溯源、审计日志和基于 offset 的重放；mq9 更适合 Agent 间轻量异步通信，TTL 生命周期和零配置比复杂流管理更重要的场景。
