# mq9 概览

## mq9 是什么

mq9 是 RobustMQ 专门为 AI Agent 通信设计的协议层，与 MQTT、Kafka、NATS、AMQP 并列，生长于 RobustMQ 的底层统一存储架构之上。

![img](../../images/mq9.jpg)

今天，当 Agent A 给 Agent B 发消息，B 不在线，消息直接丢了。没有标准机制保证"我发出去的消息，对方上线后能收到"。每个构建多 Agent 系统的团队，都在用自己的临时方案绕过这个问题——Redis pub/sub、轮询数据库、自研任务队列。能用，但都是绕路。

mq9 直接解决这个问题：**发出去，对方上线自然收到。**

就像人有邮箱——发出去，对方什么时候看是对方的事，消息不会丢。Agent 之间也需要这种机制。mq9 就是为这个场景设计的基础设施层。

---

## 定位

mq9 不是通用消息队列，不与 MQTT 或 Kafka 竞争，也不替代它们。它专门针对 **AI Agent 异步通信**这一场景：

- Agent 是临时的——任务完成就消亡，随时上下线，不能假设对方一直在线
- Agent 之间的通信关系是网状的——发布方不知道谁在监听，消费方不知道消息从哪来
- Agent 通信需要轻量的生命周期管理——创建即申请，到期自动销毁，不需要手动清理

HTTP 和 A2A 协议解决同步调用问题——调用方必须等待，对方必须在线。mq9 解决异步通信问题——发出去，对方什么时候在线什么时候处理，两者不重叠，不竞争。

### 在 RobustMQ 中的位置

mq9 是 RobustMQ 的第五个原生协议，与 MQTT、Kafka、NATS、AMQP 共享同一套统一存储架构。部署一个 RobustMQ，mq9 的能力全部就位。IoT 设备通过 MQTT 发数据，分析系统通过 Kafka 消费，Agent 通过 mq9 协作——同一个 broker，同一份存储，零桥接，零复制。

### 在 NATS 生态中的位置

mq9 介于 Core pub/sub 和 JetStream 之间。Core NATS 太轻——没有持久化，离线就丢，做不了邮箱。JetStream 太重——stream、consumer、offset、replay，一整套对标 Kafka 的语义，为 Agent 发个消息不需要这些。mq9 在 pub/sub 的基础上加了持久化、优先级、TTL 自动管理，但不引入 stream、consumer group、offset 这些重概念。

---

## 三个命令字

mq9 的全部协议只有三个命令字：

| 命令字 | 说明 |
|--------|------|
| `MAILBOX.CREATE` | 创建邮箱或广播频道。INBOX 返回 mail_id + token，BROADCAST 返回成功 |
| `INBOX` | 点对点消息投递，持久化，三级优先级（urgent / normal / notify） |
| `BROADCAST` | 公共广播，持久化，任何人可发可订，支持通配符和 queue group |

没有 QUERY——每次订阅全量推送所有未过期消息，订阅本身就是查询。没有 DELETE——TTL 自动清理。

---

## 两个核心抽象

### 邮箱（INBOX）

私人的。有 mail_id、有 token，别人往里发，只有主人能收。创建时声明 TTL，到期自动销毁。

邮箱不需要显式删除，TTL 到期自动销毁，消息随邮箱一起清理。

### 广播频道（BROADCAST）

公共的。无 mail_id、无 token，任何人可以创建，任何人可以发，任何人可以订阅。创建者不独占它。

广播频道也持久化、也有 TTL。离线的订阅者上线后也能收到未过期的消息。

### 系统公告板

mq9 内置三个永久广播频道，系统启动时自动创建，TTL 永不过期，不可删除：

| 公告板 | 用途 |
|--------|------|
| `$mq9.AI.BROADCAST.system.capability` | 能力公告——我能做什么 |
| `$mq9.AI.BROADCAST.system.status` | 状态公告——我还活着，负载多少 |
| `$mq9.AI.BROADCAST.system.channel` | 频道公告——我创建了一个新的广播频道 |

Agent 上线后订阅 `$mq9.AI.BROADCAST.system.*`，就能发现整个网络。不需要注册中心。

---

## 为什么现有方案不够

| 方案 | 核心问题 |
|------|---------|
| HTTP | 同步调用，对方必须在线，Agent 频繁上下线的假设根本不成立 |
| Redis pub/sub | 没有持久化，对方不在线消息直接丢，无法保证送达 |
| Kafka | 为大批量数据流设计，没有临时邮箱概念，没有优先级队列，topic 创建需要管理员操作，Agent 是日抛型的，Kafka 的设计假设是资源长期存在 |
| RabbitMQ | AMQP 模型灵活，但性能天花板和单机架构是硬伤，社区注意力不在 Agent 场景 |
| NATS Core | 没有持久化，离线就丢 |
| JetStream | stream、consumer、offset 等概念太重，不是为邮箱场景设计的 |
| 自研队列 | 每个团队重复造轮子，实现各不相同，无法互通 |

这些方案有一个共同的缺陷：**都假设对方在线，或需要手动处理离线情况**。Agent 频繁上下线的特点，在所有现有方案里都是需要绕过去的问题，不是被直接解决的问题。

---

## 八个核心场景

| 场景 | 协议组合 | 核心优势 |
|------|---------|---------|
| 子 Agent 完成任务，异步通知主 Agent | MAILBOX.CREATE + INBOX.normal | 主 Agent 不需要阻塞等待，消息离线不丢 |
| 主 Agent 感知所有子 Agent 状态 | BROADCAST + 通配符订阅 | 新 Agent 加入自动感知，TTL 过期自动感知消亡，零维护 |
| 任务广播，多个 Worker 竞争消费 | BROADCAST + queue group | 竞争消费零配置，无 rebalance，Worker 动态增减 |
| Agent 发现异常，广播告警 | BROADCAST + 持久化 | 发布方不需要维护订阅列表，离线 handler 上线后也能收到 |
| 云端给离线边缘 Agent 发指令 | INBOX urgent/normal | 原生优先级，边缘断网不丢消息，联网后按优先级处理 |
| 人机混合工作流 | INBOX 双向 + correlation_id | 人类和 Agent 用完全相同的协议，无需额外审批服务 |
| Agent A 向 Agent B 提问，B 可能不在线 | INBOX + reply_to + correlation_id | B 不在线请求不丢失，A 不阻塞，异步 request-reply |
| Agent 注册能力，其他 Agent 发现它 | BROADCAST.system.capability | 去中心化，无需注册中心，TTL 自动清理 |

---

## 设计原则

**不需要新 SDK**：mq9 基于 NATS 协议构建。任何 NATS 客户端——Go、Python、Rust、Java、JavaScript——都可以直接作为 mq9 客户端使用，无需引入新依赖。`$mq9.AI.*` 的命名空间设计让协议本身成为文档，看到 subject 就懂语义。

**mail_id 不绑定 Agent 身份**：mq9 只认 mail_id，不认 agent_id。一个 mail_id 是一个通信通道，一个 Agent 可以为不同任务申请不同的 mail_id，用完不管，TTL 自动清理。这是通道级的设计，不是身份级的。

**先存储后推送**：消息到达先写存储层，再尝试推送给在线订阅者。在线走实时路径，不在线消息在存储里等着，下次订阅时全量推送所有未过期消息。

**不创建新概念**：ACK 复用 NATS 原生 reply-to，订阅复用 NATS 原生 sub 语义，竞争消费复用 NATS 原生 queue group。

**存储按需选择**：运行在 RobustMQ 统一存储层之上，提供三种存储能力：Memory（协调信号，丢了重发）、RocksDB（临时持久化，邮箱默认选择，TTL 到期自动清理）、File Segment（长期持久化，审计日志）。不是所有消息都值得三副本，也不是所有消息都可以丢。

**单机即可，按需升级**：单机部署满足大量需求，一行命令启动，无需集群。需要高可用时切集群，接口不变，Agent 无感知。
