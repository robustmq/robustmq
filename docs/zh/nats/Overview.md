# RobustMQ NATS 概览

## NATS 是什么

NATS 是一个轻量级、高性能的开源消息系统，由 Derek Collison 创建，是 CNCF 孵化项目，采用 Apache 2.0 许可。NATS 的设计哲学是极简：基于 TCP 的纯文本协议，少数几个命令，单个二进制文件部署，零外部依赖。

NATS 定位是"连接技术"（connective technology），适用于云、边缘、IoT 设备之间的通信。核心特点是低延迟（微秒级）、高吞吐（单节点可达千万级消息/秒）、极轻量（空载内存 10～20MB）。

NATS 有两层架构：

- **Core NATS**：纯 pub/sub，不提供持久化存储，at-most-once 投递，极致性能
- **JetStream**：持久化层，提供 Stream、Consumer、at-least-once / exactly-once 语义

---

## RobustMQ 支持 NATS 的原因

在 RobustMQ 支持的所有协议中，NATS 是最适合 AI Agent 和边缘计算场景的，有几个原因：

**文本协议，AI 原生友好。** NATS 是纯文本协议，`PUB`、`SUB`、`MSG` 就是几个简单的文本命令。AI Agent 可以直接生成和解析这种格式，不需要二进制编解码，甚至可以直接通过 TCP 连接与 RobustMQ 交互，零依赖。

**Subject 命名空间，天然适合能力路由。** NATS 的 Subject 采用 `.` 分隔的层级结构，支持 `*` 和 `>` 通配符，天然适合组织和路由 Agent 能力：

```
ai.agent.translation.en-to-zh    # 翻译 Agent
ai.agent.code.review              # 代码审查 Agent
ai.tool.search.web                # Web 搜索工具
```

**Queue Group，无配置负载均衡。** 多个相同能力的 Agent 加入同一个 Queue Group，NATS 自动分配请求，不需要额外的负载均衡器。

**Request-Reply，天然适配 Agent 模式。** AI Agent 的核心交互是"发请求，等响应"，NATS 的 Request-Reply 正好是这个语义。

**边缘场景天然适配。** 单二进制、零依赖、极低内存占用，NATS 是边缘设备上最合适的消息中间件。配合 mq9 的邮箱语义，可以直接在边缘实现离线消息缓存和上线后的按优先级投递。

---

## RobustMQ 的 NATS 实现

### 统一存储

RobustMQ 的 NATS 实现与 MQTT、Kafka、AMQP 使用同一套存储基础设施，但各
协议处理器仍然会根据租户、名称和 `TopicSource` 解析 Topic。NATS Subject
不会自动等同于 MQTT Topic、Kafka Topic 或 AMQP Queue。只有在处理器解析出
兼容的映射且协议语义允许时，才可以跨协议访问；系统没有为所有协议提供
默认的无桥接互通。

存储层支持三种引擎，NATS 根据场景选择：

| 存储引擎 | 特性 | 适用场景 |
|---------|------|---------|
| Memory | 纯内存，微秒级延迟 | NATS Core Subject Topic |
| RocksDB | 持久化，TTL 自动清理 | mq9 Subject Topic，以及按配置使用 RocksDB 的 AMQP/MQTT Topic |
| File Segment | 大吞吐，顺序写入 | Kafka Topic，以及配置为 Segment 存储的其他 Topic |

NATS Core Subject 和 mq9 邮箱是逻辑 Topic。在当前创建路径中，每个 Topic
都会生成自己的 `storage_name_list`；虽然辅助代码会创建 `$sys.inner.*`
Topic，但后续 `with_partition(...)` 会重新生成 Subject Topic 的 Shard
名称，因此 Subject 当前不会使用该辅助 Shard 池。

### 支持范围

| 功能 | 状态 |
|------|------|
| Pub/Sub | ✅ 已支持 |
| Request/Reply | ✅ 已支持 |
| Queue Group（竞争消费）| ✅ 已支持 |
| Subject 通配符（`*` 和 `>`）| ✅ 已支持 |
| 消息 Header（HPUB/HMSG）| ✅ 已支持 |
| 连接鉴权 | ✅ 已支持（启用鉴权时使用用户名密码） |
| TLS | ✅ 已支持 |
| JetStream | 🚧 开发中 |

### mq9：基于 NATS 的 AI Agent 通信协议

RobustMQ 在 NATS 协议之上，定义了 `$mq9.AI.*` 命名空间，专为 AI Agent 异步通信设计。mq9 不是新协议，而是在 NATS Subject 上定义的语义约定，所有 NATS 客户端直接可用。

mq9 的设计参考了 JetStream 的扩展方式：不引入新的协议指令，所有操作通过标准 NATS pub/sub/req-reply 完成，服务端对 `$mq9.AI.*` 前缀的消息启用持久化和优先级调度。

详见 [mq9 概览](https://mq9.robustmq.com/zh/docs/overview)。

---

## 与原生 NATS 的兼容性

RobustMQ 当前实现的 NATS Core 指令子集包括：`INFO`、`CONNECT`、`PUB`、
`HPUB`、`SUB`、`UNSUB`、`MSG`、`HMSG`、`PING`、`PONG`、`+OK` 和 `-ERR`。
标准客户端可以使用这些操作直接连接，但这不等同于完整的 NATS Core 功能
集。Token、JWT 和 NKey 字段会被解析以保持线协议兼容，但启用鉴权时实际
仍通过用户名密码校验；JetStream 尚未实现。

```bash
# 使用官方 NATS CLI 直接连接 RobustMQ
nats pub "hello.world" "test message" --server nats://localhost:4222
nats sub "hello.world" --server nats://localhost:4222
```

---

## 在 RobustMQ 中的定位

RobustMQ 是多协议统一消息引擎，NATS 是其中一个原生协议，与 MQTT、Kafka、AMQP 并列。

共享存储层使兼容的跨协议视图成为可能，但映射是显式的，并非自动互通。
例如，NATS Core 的订阅发现当前只选择 `TopicSource::NATS` 的 Topic，不会
把所有 MQTT、Kafka 或 AMQP Topic 自动当作 NATS Subject。若业务需要跨协议
访问，必须使用双方处理器都支持的 Topic/租户映射。

这在边缘场景和 AI 场景中有实际价值：IoT 设备通过 MQTT 上报数据，边缘 Agent 通过 NATS/mq9 协调任务，分析系统通过 Kafka 消费——同一个 broker，同一份存储，零运维成本。
