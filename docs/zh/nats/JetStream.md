# JetStream

> 🚧 **开发中** — JetStream 功能正在实现中，本文档描述的是 JetStream 的核心概念和 RobustMQ 的支持计划。

## JetStream 是什么

JetStream 是 NATS 的持久化层，在 Core NATS 的 pub/sub 基础上增加了消息存储、消费者管理和投递保障。Core NATS 是 at-most-once（消息丢了就丢了），JetStream 是 at-least-once 乃至 exactly-once。

两者的核心区别：

| | Core NATS | JetStream |
|-|-----------|-----------|
| 持久化 | 否，纯内存 | 是，消息写入 Stream |
| 投递语义 | at-most-once | at-least-once / exactly-once |
| 消费者状态 | 无 | 有（Consumer，保存消费进度）|
| 离线消息 | 丢失 | 保留，上线后重放 |
| 回放 | 不支持 | 支持（从任意 offset 重放）|
| 适用场景 | 实时推送、低延迟 | 事件溯源、审计日志、可靠投递 |

---

## 核心概念

### Stream

Stream 是 JetStream 的存储单元。一个 Stream 绑定一组 Subject，所有发布到这些 Subject 的消息都会被持久化到 Stream 中。

```bash
# 创建一个 Stream，绑定 orders.> 下所有消息
nats stream add ORDERS \
  --subjects "orders.>" \
  --storage file \
  --retention limits \
  --max-age 24h
```

Stream 的核心配置：

| 配置项 | 说明 |
|--------|------|
| `subjects` | 绑定的 Subject，支持通配符 |
| `storage` | 存储类型：`file`（持久化）或 `memory`（内存）|
| `retention` | 保留策略：`limits`（按大小/时间）、`interest`（有消费者时保留）、`workqueue`（消费后删除）|
| `max-age` | 消息最大保留时间 |
| `max-msgs` | Stream 最大消息条数 |
| `max-bytes` | Stream 最大字节数 |

### Consumer

Consumer 是消费者的视图，保存在 Stream 上的消费进度。每个 Consumer 独立追踪自己消费到了哪条消息。

```bash
# 创建一个 Push Consumer（服务端主动推送）
nats consumer add ORDERS payments-service \
  --filter "orders.created" \
  --deliver all \
  --ack explicit

# 创建一个 Pull Consumer（客户端主动拉取）
nats consumer add ORDERS audit-log \
  --filter "orders.>" \
  --deliver all \
  --pull
```

Consumer 类型：

| 类型 | 说明 |
|------|------|
| **Push Consumer** | 服务端主动推送消息到指定 Subject，类似订阅 |
| **Pull Consumer** | 客户端主动 fetch，适合批量处理和流量控制 |

Consumer 的起始位置（deliver policy）：

| 选项 | 说明 |
|------|------|
| `all` | 从 Stream 最早的消息开始 |
| `new` | 只接收创建 Consumer 之后的新消息 |
| `last` | 从最后一条消息开始 |
| `by_start_time` | 从指定时间之后的消息开始 |
| `by_start_sequence` | 从指定序号开始 |

### ACK 机制

JetStream 的消息投递需要客户端显式 ACK。未 ACK 的消息会在超时后重新投递。

```bash
# ACK 类型
Ack         # 正常确认，消息处理完成
Nak         # 否定确认，请求立即重新投递
InProgress  # 告知服务端正在处理，重置 ACK 超时
Term        # 终止，不再重新投递
```

---

## 与 Core NATS 和 mq9 的关系

RobustMQ 的三种 NATS 使用模式各有定位：

| | Core NATS | JetStream | mq9 |
|-|-----------|-----------|-----|
| **持久化** | 否 | 是 | 是 |
| **消费者状态** | 无 | 有（offset）| 无（store-first push）|
| **优先级** | 无 | 无 | 三级（critical/urgent/normal）|
| **TTL** | 无 | 按 Stream 配置 | 按邮箱配置 |
| **适合场景** | 实时推送、低延迟 | 事件流、审计、可靠投递 | AI Agent 异步通信 |
| **目标用户** | 通用 pub/sub | 数据管道、微服务 | AI Agent |

三者可以在同一个 RobustMQ 实例上并存，按场景选择：

- 需要极致低延迟、不在意丢消息 → **Core NATS**
- 需要消息不丢、支持回放、有消费者进度 → **JetStream**
- AI Agent 之间的异步通信、离线投递、优先级 → **mq9**

---

## 当前状态

JetStream 目前正在开发中。已完成：

- Stream 和 Consumer 的概念设计与存储层映射
- Pull Consumer 基础能力

待完成：

- Push Consumer
- ACK 与重投递机制
- exactly-once 语义
- Stream 管理 API（NATS CLI 兼容）

进度更新请关注 [GitHub Milestones](https://github.com/robustmq/robustmq/milestones)。
