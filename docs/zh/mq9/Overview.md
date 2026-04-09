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
- Agent 之间的通信关系是动态的——发布方不知道谁在监听，消费方不知道消息从哪来
- Agent 通信需要轻量的生命周期管理——创建即申请，到期自动销毁，不需要手动清理

HTTP 和 A2A 协议解决同步调用问题——调用方必须等待，对方必须在线。mq9 解决异步通信问题——发出去，对方什么时候在线什么时候处理，两者不重叠，不竞争。

### 在 RobustMQ 中的位置

mq9 是 RobustMQ 的第五个原生协议，与 MQTT、Kafka、NATS、AMQP 共享同一套统一存储架构。部署一个 RobustMQ，mq9 的能力全部就位。IoT 设备通过 MQTT 发数据，分析系统通过 Kafka 消费，Agent 通过 mq9 协作——同一个 broker，同一份存储，零桥接，零复制。

### 在 NATS 生态中的位置

mq9 介于 Core pub/sub 和 JetStream 之间。Core NATS 太轻——没有持久化，离线就丢，做不了邮箱。JetStream 太重——stream、consumer、offset、replay，一整套对标 Kafka 的语义，为 Agent 发个消息不需要这些。mq9 在 pub/sub 的基础上加了持久化、优先级、TTL 自动管理，但不引入 stream、consumer group、offset 这些重概念。

---

## 核心概念：邮箱

mq9 只有一个核心抽象：**邮箱（MAILBOX）**。

邮箱是 Agent 的通信地址。临时的，TTL 驱动生命周期，到期自动销毁。Agent 为每个任务申请一个邮箱，拿到一个 mail_id，这就是它在这个任务里的通信地址。任务结束，邮箱自动过期清理。

**mail_id 不可猜测即安全边界。** mail_id 是系统生成的不可猜测字符串。知道 mail_id 就能发消息、能订阅。不知道 mail_id 就无从操作。没有 token，没有 ACL。

邮箱分两种：

| | 私有邮箱 | 公开邮箱 |
|---|---|---|
| mail_id | 系统生成，不可猜测 | 用户自定义，有意义的名字 |
| 可发现性 | 不公开，只有知道 mail_id 的 Agent 能找到 | 自动注册到 PUBLIC.LIST，任何 Agent 可发现 |
| 适合场景 | 点对点私信、任务结果回传 | 任务队列、公共频道、能力公告 |

公开邮箱的 mail_id 本身就是地址，起一个有意义的名字——`task.queue`、`analytics.result`——比 UUID 更容易被其他 Agent 发现和理解。

---

## 三件事

mq9 的全部操作只有三件事：

| 操作 | Subject | 说明 |
|------|---------|------|
| 创建邮箱 | `$mq9.AI.MAILBOX.CREATE` | 创建私有或公开邮箱 |
| 发消息（默认）| `$mq9.AI.MAILBOX.{mail_id}` | 默认优先级（normal），无后缀 |
| 发消息（urgent）| `$mq9.AI.MAILBOX.{mail_id}.urgent` | 紧急优先级 |
| 发消息（critical）| `$mq9.AI.MAILBOX.{mail_id}.critical` | 最高优先级 |
| 订阅邮箱（非默认）| `$mq9.AI.MAILBOX.{mail_id}.*` | 订阅 urgent 和 critical 消息 |
| 列出消息 | `$mq9.AI.MAILBOX.LIST.{mail_id}` | 返回消息元数据（不含消息体） |
| 删除消息 | `$mq9.AI.MAILBOX.DELETE.{mail_id}.{msg_id}` | 删除指定消息 |

**三个优先级：**

| 级别 | 语义 | 典型场景 |
|------|------|---------|
| critical | 最高优先级，立即处理 | 中止信号、紧急指令、安全事件 |
| urgent | 紧急 | 任务中断、时效性指令 |
| normal（默认，无后缀）| 常规通信 | 任务分发、结果返回、审批请求 |

---

## 发现公开邮箱

`$mq9.AI.PUBLIC.LIST` 是系统内置地址，broker 维护，不接受用户写入，TTL 永不过期。

`public: true` 的邮箱创建后自动注册，TTL 到期自动移除。Agent 无需手动维护注册表。

```bash
nats sub '$mq9.AI.PUBLIC.LIST'
```

订阅即全量推送——当前所有公开邮箱立即推送，后续新增和移除实时推送。不需要注册中心，PUBLIC.LIST 本身就是目录。

---

## 当前能力状态

mq9 的基础通信层已完整落地：

- **邮箱生命周期** — 私有和公开邮箱、TTL 到期自动销毁、幂等 CREATE
- **三级优先级消息** — critical / urgent / normal，先存储后推送，离线容错
- **竞争消费** — NATS 队列组，动态成员，容灾任务队列
- **公开发现** — PUBLIC.LIST 用于去中心化能力公告
- **六种语言 SDK** — Python（已完整实现）、Go、JavaScript、Java、Rust、C#（已搭建框架）
- **LangChain & LangGraph 集成** — `langchain-mq9` 工具包，6 个工具
- **MCP Server 支持** — 通过 JSON-RPC 2.0 接入 Dify、Claude 等 AI 生态

---

## 未来方向

mq9 的下一步，是在基础邮箱通信之上，逐步建立语义理解、智能路由和意图审计的能力：

| 阶段 | 目标 |
|------|------|
| **第一阶段：语义服务发现** | PUBLIC.LIST 支持语义搜索，Agent 描述需求即可找到能力匹配的对方，无需提前知道名称 |
| **第二阶段：语义路由** | 消息无需指定目标，发送方描述意图，mq9 自动路由给最合适的 Agent |
| **第三阶段：意图感知策略** | 消息经过策略引擎，语义不合规则在传输层直接拦截，构建基础设施层安全边界 |
| **第四阶段：上下文感知** | mq9 感知 Agent 之间的会话历史，消息自动携带上下文，减少 Token 重复传递 |

详细规划见 [发展规划](./Roadmap.md)，背后的判断和思考见 [AI 时代的消息系统应该是什么样的](../Blogs/82.md)。

---

## 为什么现有方案不够

| 方案 | 核心问题 |
|------|---------|
| HTTP | 同步调用，对方必须在线，Agent 频繁上下线的假设根本不成立 |
| Redis pub/sub | 没有持久化，对方不在线消息直接丢，无法保证送达 |
| Kafka | 为大批量数据流设计，topic 创建需要管理员操作，Agent 是日抛型的，Kafka 的设计假设是资源长期存在 |
| RabbitMQ | AMQP 模型灵活，但性能天花板和单机架构是硬伤 |
| NATS Core | 没有持久化，离线就丢 |
| JetStream | stream、consumer、offset 等概念太重，不是为邮箱场景设计的 |
| 自研队列 | 每个团队重复造轮子，实现各不相同，无法互通 |

这些方案有一个共同的缺陷：**都假设对方在线，或需要手动处理离线情况**。Agent 频繁上下线的特点，在所有现有方案里都是需要绕过去的问题，不是被直接解决的问题。

---

## 设计原则

**不需要新 SDK**：mq9 基于 NATS 协议构建。任何 NATS 客户端——Go、Python、Rust、Java、JavaScript——都可以直接作为 mq9 客户端使用，无需引入新依赖。`$mq9.AI.*` 的命名空间设计让协议本身成为文档，看到 subject 就懂语义。

**mail_id 不绑定 Agent 身份**：mq9 只认 mail_id，不认 agent_id。一个 Agent 可以为不同任务申请不同的 mail_id，用完不管，TTL 自动清理。这是通道级的设计，不是身份级的。

**先存储后推送**：消息到达先写存储层，再尝试推送给在线订阅者。在线走实时路径，不在线消息在存储里等着，下次订阅时全量推送所有未过期消息。

**不创建新概念**：订阅复用 NATS 原生 sub 语义，竞争消费复用 NATS 原生 queue group，reply-to 复用 NATS 原生机制。

**存储按需选择**：运行在 RobustMQ 统一存储层之上，提供三种存储能力：Memory（协调信号，丢了重发）、RocksDB（临时持久化，邮箱默认选择，TTL 到期自动清理）、File Segment（长期持久化，审计日志）。

**单机即可，按需升级**：单机部署满足大量需求，一行命令启动，无需集群。需要高可用时切集群，接口不变，Agent 无感知。
