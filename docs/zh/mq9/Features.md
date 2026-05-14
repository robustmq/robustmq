# 核心功能

## 概述

从功能上看，mq9 对 Agent 来说就是一个高级邮箱：申请一个邮箱地址，发消息给别人，主动拉取自己的消息，确认已处理——其余的由 mq9 在后台保证。

mq9 在普通邮箱基础上提供以下核心能力：

- **pull 消费 + ACK**：客户端主动 FETCH 拉取，ACK 推进消费位点，支持断点续拉
- **优先级**：消息分三级（critical / urgent / normal），紧急消息优先出队
- **消息属性**：key 去重压实、tags 过滤、delay 延迟投递、消息级 TTL
- **Agent 注册与发现**：内置 Agent 注册表，支持全文和语义向量检索
- **TTL**：邮箱有生命周期，到期自动销毁，无需手动清理

---

## 功能详解

### 1. Pull 消费与 ACK

mq9 使用 **pull 模式**：客户端主动调用 `FETCH` 拉取消息，处理完成后调用 `ACK` 确认，broker 推进该消费组的消费位点。

**两种消费模式：**

| 模式 | 使用方式 | 适用场景 |
|------|---------|---------|
| 有状态消费 | 传 `group_name` | broker 记录位点，重连后从断点续拉，适合持续运行的 Worker |
| 无状态消费 | 不传 `group_name` | 每次按 `deliver` 策略独立拉取，不记录位点，适合一次性读取、调试 |

**有状态消费的位点行为：**

| 条件 | 行为 |
|------|------|
| 有位点记录 | 从上次 ACK 位置续拉，`deliver` 策略不生效 |
| 有位点记录 + `force_deliver: true` | 忽略位点，按 `deliver` 重新开始 |
| 无位点记录（首次） | 按 `deliver` 策略定位起点 |

**deliver 起点策略：**

| 值 | 说明 |
|----|------|
| `latest`（默认） | 从当前时刻起只拉新消息 |
| `earliest` | 从 mailbox 最早的消息开始 |
| `from_time` | 从指定时间戳之后开始 |
| `from_id` | 从指定 msg_id 开始（含） |

**消费流程：**

```
客户端 FETCH → broker 返回消息列表 → 客户端处理 → ACK → broker 推进位点
                                                         ↓
                                               下次 FETCH 从此处续拉
```

---

### 2. 优先级系统

每条消息通过 `mq9-priority` header 指定优先级，分为三级：

| 优先级 | Header 值 | 典型场景 |
|--------|----------|---------|
| `critical` | `mq9-priority: critical` | 中止信号、紧急指令、安全事件 |
| `urgent` | `mq9-priority: urgent` | 审批请求、时效性通知 |
| `normal`（默认） | 不填 | 任务分发、结果返回、常规通信 |

**排序保证：**

- 同优先级内：FIFO——消息按发送顺序出队
- 跨优先级：critical 先于 urgent 先于 normal
- 排序由存储层保证，消费方无需自行排序

---

### 3. 消息属性

发送消息时可附加以下属性，通过 NATS header 传递：

| 属性 | Header | 说明 |
|------|--------|------|
| 去重 key | `mq9-key: {key}` | 同 key 的消息只保留最新一条，旧消息被覆盖。适合状态更新类消息（如任务进度） |
| 标签 | `mq9-tags: {tag1},{tag2}` | 逗号分隔，如 `billing,vip`。可通过 QUERY 的 `tags` 字段过滤 |
| 延迟投递 | `mq9-delay: {seconds}` | 消息写入后延迟指定秒数才可见。延迟消息的 `msg_id` 返回 `-1` |
| 消息级 TTL | `mq9-ttl: {seconds}` | 消息在 `发送时间 + ttl` 后自动过期，独立于邮箱 TTL |

**去重 key 使用示例：** 任务处理中持续上报进度，只需关心最新状态：

```
SEND key=status {"status":"running"}   → msg_id=1
SEND key=status {"status":"60%"}       → msg_id=2，旧消息被覆盖
SEND key=status {"status":"done"}      → msg_id=3，只保留这条
QUERY key=status                       → 返回 msg_id=3 这一条
```

---

### 4. TTL 与生命周期

邮箱创建时声明 TTL（生存时间），到期后邮箱及其所有消息自动销毁，无需手动清理：

```json
{"name": "task.queue", "ttl": 3600}
```

**行为规则：**

- TTL 从邮箱创建时开始计时，到期后不可续期
- 没有手动删除邮箱的命令，TTL 是唯一清理机制
- 重复 CREATE 同名邮箱返回错误（`mailbox xxx already exists`），CREATE 不是幂等的
- `ttl: 0` 或省略 ttl 表示永不过期

**消息级 TTL 独立于邮箱 TTL：** 消息可以通过 `mq9-ttl` header 单独设置过期时间，早于邮箱过期自动清理。

---

### 5. 消息查询与删除

**QUERY** — 查询邮箱中当前存储的消息，不影响消费位点：

| 查询方式 | 参数 | 说明 |
|---------|------|------|
| 全量查询 | 不传参数 | 返回所有消息 |
| 按 key | `key: "status"` | 返回该 key 的最新一条 |
| 按标签 | `tags: ["billing", "vip"]` | 返回同时带有所有标签的消息 |
| 按时间 | `since: <unix_ts>` | 返回该时间戳之后的消息 |
| 分页 | `limit: 20` | 最多返回 N 条 |

**DELETE** — 删除邮箱中的指定消息（通过 msg_id）。

---

### 6. Agent 注册与发现

mq9 内置 Agent 注册表，支持三种发现方式：

| 方式 | 参数 | 说明 |
|------|------|------|
| 语义检索 | `semantic: "处理付款并生成发票"` | 向量相似度匹配，理解自然语言意图 |
| 全文检索 | `text: "payment invoice"` | 关键词匹配 |
| 列出全部 | 不传参数 | 返回租户下所有已注册的 Agent |

检索优先级：`semantic` > `text` > 不传。

支持分页（`limit` + `page`，page 从 1 开始）。

**典型流程：**

```
Agent 启动 → REGISTER（携带能力描述）
                  ↓
其他 Agent → DISCOVER（semantic: "找翻译 Agent"）→ 返回匹配列表
                  ↓
            发消息给匹配 Agent 的 mail_address
Agent 关闭 → UNREGISTER
```

注册内容（`payload`）可以是纯文本描述，也可以是 A2A AgentCard JSON 字符串，内容会被同时建立全文索引和向量索引。

---

### 7. mail_address 格式

**字符集**：小写字母（a-z）、数字（0-9）、点（`.`）

**长度**：1 到 128 字符

**规则**：`.` 只能出现在中间，开头和结尾必须是小写字母或数字；不允许连续 `.`

| 合法示例 | 非法示例 |
|---------|---------|
| `task.001` | `task-001`（含连字符） |
| `agent.inbox` | `Task.001`（含大写） |
| `session.20260502` | `.task.001`（点开头） |
| `acme.org.task.queue` | `task..001`（连续点） |

**安全模型：** `mail_address` 的不可猜测性是访问控制的唯一边界。知道 mail_address 即可发消息和订阅；不知道则无从操作——没有 token，没有 ACL。

---

## 与 NATS JetStream 的对比

| | NATS JetStream | mq9 |
|--|---------------|-----|
| 消费模式 | push 或 pull | pull（FETCH + ACK） |
| 消费者状态 | 服务端维护 offset、消费者组、ACK | 服务端维护消费组位点，ACK 推进 |
| 消息过滤 | subject 过滤 | key、tags、since、limit |
| 优先级 | 无内置优先级 | 三级优先级（critical/urgent/normal） |
| Agent 发现 | 无 | 内置，支持向量语义检索 |
| 延迟消息 | 支持 | 支持（mq9-delay header） |
| 消息级 TTL | 支持 | 支持（mq9-ttl header） |
| 接入方式 | 任何 NATS 客户端 | 任何 NATS 客户端（subject 遵循 `$mq9.AI.*` 约定） |
