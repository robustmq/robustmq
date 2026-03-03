# Connector 架构

Connector 是 RobustMQ 的数据桥接层，负责将 MQTT Broker 中的消息实时同步到外部数据系统。它运行在 Broker 节点上，由 Meta Service 统一调度和管理。

---

## 整体架构

Connector 采用 **Meta 调度 + Broker 执行** 的分布式架构：

```
┌─────────────────────────────────────────────────┐
│                  Meta Service                    │
│                                                  │
│  ConnectorScheduler                              │
│  ├── check_heartbeat()    心跳超时检测            │
│  └── start_stop_connector_thread()  分配/回收     │
│       └── 按负载均衡将 Connector 分配给 Broker     │
└──────────────────────┬──────────────────────────┘
                       │  gRPC (UpdateCache)
          ┌────────────┼────────────────┐
          ▼            ▼                ▼
   ┌──────────┐  ┌──────────┐    ┌──────────┐
   │ Broker 1 │  │ Broker 2 │    │ Broker N │
   │          │  │          │    │          │
   │ Connector│  │ Connector│    │ Connector│
   │ Thread   │  │ Thread   │    │ Thread   │
   └──────────┘  └──────────┘    └──────────┘
       │              │                │
       ▼              ▼                ▼
   Kafka/Redis/MySQL/ES/MongoDB/File/...
```

---

## 核心流程

### 1. Connector 生命周期

一个 Connector 从创建到运行经历以下阶段：

```
用户创建 Connector (API)
       │
       ▼
Meta Service 持久化 Connector 配置 (status: Idle)
       │
       ▼
ConnectorScheduler 定时扫描 Idle 的 Connector
       │
       ▼
按负载均衡选择 Broker，分配 broker_id (status: Running)
       │
       ▼
Broker 收到 UpdateCache 通知，本地缓存更新
       │
       ▼
Broker 的 start_connectors() 检测到新分配的 Connector
       │
       ▼
根据 ConnectorType 启动对应的 Sink 线程
       │
       ▼
进入 run_connector_loop 消费循环
```

### 2. 消费循环（run_connector_loop）

每个 Connector 线程的核心工作：

```
validate() → 校验配置合法性
     │
init_sink() → 初始化外部连接（如 Kafka Producer、Redis Client）
     │
     ▼
┌──────────────────────────────────┐
│           主循环                  │
│                                  │
│  1. 获取当前消费 offset           │
│  2. 从 Storage 读取一批消息       │
│  3. 调用 send_batch() 写入外部    │
│  4. 成功 → 提交 offset           │
│  5. 失败 → 按策略重试或丢弃       │
│  6. 收到 stop 信号 → cleanup 退出 │
│                                  │
└──────────────────────────────────┘
```

消费循环通过 `select!` 同时监听停止信号和消息读取，保证在需要停止时能及时退出。

### 3. Meta 调度（ConnectorScheduler）

Meta Service 中的 `ConnectorScheduler` 每秒执行两件事：

**心跳检测（check_heartbeat）**：遍历所有 Connector 的心跳时间，如果超时，将该 Connector 标记为 Idle 并清除 broker_id 分配。

**分配与回收（start_stop_connector_thread）**：
- 收集所有 Idle 状态的 Connector
- 计算每个 Broker 的负载（已分配的 Connector 数量）
- 按最少负载原则分配 Connector 到 Broker
- 更新状态为 Running，通过 UpdateCache 通知 Broker

### 4. Broker 侧调度

Broker 上的 `start_connector_thread` 每秒执行两个检查：

**启动检查（start_connectors）**：遍历缓存中的所有 Connector，如果分配给当前 Broker 且线程未运行，根据 `ConnectorType` 启动对应的 Sink 线程。

**回收检查（gc_connectors）**：遍历所有运行中的线程，如果对应的 Connector 已不再分配给当前 Broker（被重新调度或删除），发送停止信号并更新状态为 Idle。

---

## ConnectorSink trait

所有外部数据系统的 Connector 都需要实现 `ConnectorSink` trait：

| 方法 | 说明 |
|------|------|
| `validate()` | 校验连接配置是否合法 |
| `init_sink()` | 初始化外部连接资源（如 Producer、Client） |
| `send_batch()` | 批量发送消息到外部系统 |
| `cleanup_sink()` | 释放连接资源（可选） |

新增一个 Connector 类型只需要：实现 `ConnectorSink`，在 `ConnectorType` 枚举中添加类型，在 `start_thread` 中添加分发逻辑。

---

## 失败处理策略

当 `send_batch` 失败时，根据配置的策略处理：

| 策略 | 行为 |
|------|------|
| `Discard` | 直接丢弃，继续消费下一批 |
| `DiscardAfterRetry` | 重试指定次数后丢弃，每次重试间隔可配置 |
| `DeadMessageQueue` | （规划中）发送到死信队列 |

---

## 心跳与健康检查

Connector 运行时的健康保障采用双层心跳机制：

**Broker 侧**：每次成功读取消息时更新本地心跳时间，由心跳上报线程定期将心跳信息批量上报给 Meta Service。

**Meta 侧**：`ConnectorScheduler` 周期性检查心跳，超时的 Connector 会被重置为 Idle 状态，等待重新调度到其他 Broker。

这套机制保证了当 Broker 宕机或 Connector 异常卡住时，Meta 能自动感知并将 Connector 迁移到健康的 Broker 上继续运行。

---

## Offset 管理

每个 Connector 以 `connector_name` 作为消费组名，独立维护消费进度：

- 每次读取消息时按 Shard 记录最大 offset
- `send_batch` 成功后提交 offset
- 失败时不提交，保证 at-least-once 语义
- Connector 迁移到其他 Broker 后，从上次提交的 offset 继续消费

---

## 支持的外部系统

| 类型 | 说明 |
|------|------|
| Kafka | 写入 Kafka Topic |
| Elasticsearch | 写入 ES Index |
| Redis | 执行 Redis 命令模板 |
| MongoDB | 写入 MongoDB Collection |
| MySQL | 写入 MySQL 表 |
| PostgreSQL | 写入 PostgreSQL 表 |
| RabbitMQ | 发布到 RabbitMQ Exchange |
| Pulsar | 发布到 Pulsar Topic |
| GreptimeDB | 写入 GreptimeDB（行协议） |
| LocalFile | 写入本地文件 |

---

## 代码结构

```
src/connector/src/
├── traits.rs       ConnectorSink trait 定义
├── loops.rs        消费循环（run_connector_loop）、offset 管理
├── core.rs         Broker 侧调度（start/gc connectors）、类型分发
├── manager.rs      运行时状态管理（connector/thread/heartbeat CRUD）
├── heartbeat.rs    心跳上报线程
├── failure.rs      失败处理策略
├── storage/        Meta Service 存储交互（ConnectorStorage、MessageStorage）
├── kafka/          Kafka Sink 实现
├── elasticsearch/  Elasticsearch Sink 实现
├── redis/          Redis Sink 实现
├── mongodb/        MongoDB Sink 实现
├── mysql/          MySQL Sink 实现
├── postgres/       PostgreSQL Sink 实现
├── rabbitmq/       RabbitMQ Sink 实现
├── pulsar/         Pulsar Sink 实现
├── greptimedb/     GreptimeDB Sink 实现
└── file/           LocalFile Sink 实现
```
