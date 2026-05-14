# mq9 概览

## mq9 是什么

mq9 是 RobustMQ 专门为 AI Agent 设计的通信协议层，与 MQTT、Kafka、NATS、AMQP 并列，共享同一套统一存储架构。

![img](../../images/mq9.jpg)

### 解决什么问题

多 Agent 系统里，Agent 不是服务器——它们是任务驱动的，启动、执行、消亡，随时上下线。Agent A 给 Agent B 发消息，B 不在线，消息直接丢了。每个构建多 Agent 系统的团队都在用临时方案绕过这个问题：

- **Redis pub/sub**：没有持久化，对方不在线消息直接丢
- **Kafka**：topic 需要提前创建和维护，不适合日抛型 Agent
- **自研队列**：每个团队重复造，Agent 之间无法互通

这些方案能用，但都是绕路——**离线送达这件事被当做边界条件手动处理，而不是被基础设施直接保证。**

mq9 直接解决它：**发出去，对方上线自然收到。** 就像人有邮箱——发出去，对方什么时候看是对方的事，消息不会丢。

今天一个系统可能有几十个 Agent，未来可能几百万个。mq9 的设计起点就是这个规模：邮箱按需创建、TTL 自动销毁、Broker 水平扩展。从第一个 Agent 到几百万个，接口不变，运维模型不变。

### 当前能力

已落地：邮箱生命周期（TTL 自动销毁）、三级优先级消息（critical / urgent / normal）、pull 消费（FETCH + ACK 断点续拉）、消息属性（key 去重、tags 过滤、delay 延迟、消息级 TTL）、Agent 注册与发现（全文 + 语义向量检索），以及 Python SDK、LangChain/LangGraph 工具包、MCP Server 接入。

### 未来方向

邮箱解决了"消息能送达"的问题，但 Agent 网络真正成熟后，还需要意图路由（消息自动找到最合适的接收者）、策略拦截（传输层感知语义并执行访问控制）、上下文感知（会话历史随消息流转，减少 Token 重复传递）。

这些方向是 mq9 的演进路线，详见 [发展规划](./Roadmap.md)。背后的判断和思考见 [AI 时代的消息系统应该是什么样的](../Blogs/82.md)。

---

## 定位

mq9 不是通用消息队列，不与 MQTT 或 Kafka 竞争，也不替代它们。它专门针对 **AI Agent 异步通信**这一场景。HTTP 和 A2A 协议解决同步调用问题——调用方必须等待，对方必须在线。mq9 解决异步通信问题——发出去，对方什么时候在线什么时候处理。两者不重叠，不竞争。

### 在 RobustMQ 中的位置

mq9 是 RobustMQ 的第五个原生协议，与 MQTT、Kafka、NATS、AMQP 共享同一套统一存储架构。部署一个 RobustMQ，mq9 的能力全部就位。IoT 设备通过 MQTT 发数据，分析系统通过 Kafka 消费，Agent 通过 mq9 协作——同一个 Broker，同一份存储，零桥接，零复制。

### 在 NATS 生态中的位置

mq9 构建在 NATS 协议之上，但 NATS 只是通信层——客户端与 Broker 之间的传输协议，就像 HTTP 是 Web 的传输协议一样。mq9 的 Broker 是 RobustMQ 用 Rust 完全自研实现的，存储、优先级调度、TTL 管理、pull 消费语义，全部是 RobustMQ 自身的能力，与 NATS Server 没有任何关系。

选择 NATS 协议的原因是务实的：NATS 有覆盖 40+ 语言的官方和社区客户端，AI 领域常用的 Python、Go、JavaScript、Rust 均有成熟实现，mq9 从第一天起就对所有这些语言的开发者开箱即用，无需等待 SDK 覆盖。NATS 的 request/reply 原语恰好覆盖 mq9 所需的全部通信模式。

在语义层面，mq9 介于 NATS Core 和 JetStream 之间，但面向 Agent 场景做了专门优化：pull 消费 + ACK 位点、三级优先级调度、消息属性（key/tags/delay/ttl）、内置 Agent 注册表。这些是 mq9 专属能力，JetStream 没有对等实现。

---

## 核心抽象：邮箱

mq9 只有一个核心抽象：**邮箱（MAILBOX）**。

为什么是邮箱？因为 mq9 把 Agent 当做人。人与人之间最自然的异步沟通方式是邮箱——你写好发出去，对方什么时候看是对方的事，你不用等着，消息也不会丢。Agent 之间的通信本质上是同一种场景：发出去，对方上线自然收到。邮箱这个语义是最直观的映射。

顺着这个类比往下走：

- **地址**：每个邮箱有一个 `mail_address`，就是它的通信地址。地址在创建时指定（如 `task.queue`），不可猜测即安全边界——知道 `mail_address` 就能发消息和拉取，不知道则无从操作。

- **信件**：发给邮箱的每条消息可附加属性——优先级（critical / urgent / normal）通过 `mq9-priority` header 指定；去重 key 通过 `mq9-key` 指定；过滤标签通过 `mq9-tags` 指定；延迟投递通过 `mq9-delay` 指定；消息级 TTL 通过 `mq9-ttl` 指定。

- **取件**：客户端主动 FETCH 拉取消息，处理完后 ACK 推进消费位点。下次 FETCH 从断点续拉，不会重复消费。传 `group_name` 时 broker 记录位点（有状态消费）；不传时每次独立消费（无状态消费）。

- **信箱寿命**：邮箱在创建时声明 TTL，到期自动销毁，所有未取的消息随之清理。不需要手动关闭，任务结束就忘掉，系统自己负责清理。

---

## 操作一览

| 操作 | Subject | 说明 |
|------|---------|------|
| 创建邮箱 | `$mq9.AI.MAILBOX.CREATE` | 创建邮箱，name 自定义，ttl 声明生命周期 |
| 发消息 | `$mq9.AI.MSG.SEND.{mail_address}` | 优先级通过 `mq9-priority` header 指定 |
| 拉取消息 | `$mq9.AI.MSG.FETCH.{mail_address}` | pull 模式，支持有状态/无状态消费 |
| 确认消息 | `$mq9.AI.MSG.ACK.{mail_address}` | 推进消费组位点，支持断点续拉 |
| 查询消息 | `$mq9.AI.MSG.QUERY.{mail_address}` | 按 key/tags/since 查询，不影响位点 |
| 删除消息 | `$mq9.AI.MSG.DELETE.{mail_address}.{msg_id}` | 删除指定消息 |
| 注册 Agent | `$mq9.AI.AGENT.REGISTER` | 注册 Agent 及其能力描述 |
| 注销 Agent | `$mq9.AI.AGENT.UNREGISTER` | 注销 Agent |
| 上报状态 | `$mq9.AI.AGENT.REPORT` | Agent 心跳/状态上报 |
| 发现 Agent | `$mq9.AI.AGENT.DISCOVER` | 全文或语义向量检索 Agent |

**三个优先级：**

| 级别 | Header 值 | 典型场景 |
|------|----------|---------|
| `critical`（最高）| `mq9-priority: critical` | 中止信号、紧急指令、安全事件 |
| `urgent`（紧急）| `mq9-priority: urgent` | 审批请求、时效性通知 |
| `normal`（默认）| 不填 | 任务分发、结果返回、常规通信 |

---

## 设计原则

**pull 消费 + ACK**：客户端主动 FETCH 拉取，ACK 推进消费位点，支持断点续拉。消息不会因为消费者暂时离线而丢失，重连后从上次 ACK 处继续。

**mail_address 不绑定 Agent 身份**：mq9 只认 `mail_address`，不认 `agent_id`。一个 Agent 可以为不同任务申请不同的邮箱，用完不管，TTL 自动清理。通道级设计，不是身份级设计。

**不创建新概念**：request/reply 复用 NATS 原生语义，位点管理类似 Kafka consumer group，消息属性通过 NATS header 传递。不引入私有传输格式。

**Broker 完全自研**：NATS 只是传输协议。存储、优先级调度、TTL 管理、消费位点、Agent 注册表——全部是 RobustMQ 用 Rust 自研实现的能力，运行在 RobustMQ 统一存储层之上。

**单机即可，按需升级**：单机部署满足大量需求，一行命令启动。需要高可用时切集群，接口不变，Agent 无感知。
