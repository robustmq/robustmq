# RobustMQ 整体架构

## 组件构成

RobustMQ 由三个核心组件构成，通过单一二进制文件交付，由配置中的 `roles` 字段决定启用哪些组件：

```toml
roles = ["meta", "broker", "engine"]
```

| 组件 | 职责 |
|------|------|
| **Meta Service** | 集群元数据管理、节点协调、集群控制器 |
| **Broker** | 多协议解析与消息处理（MQTT、Kafka、NATS、AMQP、mq9） |
| **Storage Engine** | 内置存储引擎，提供 Memory / RocksDB / File Segment 三种后端 |

三个组件可单机全启，也可按节点独立部署。

![RobustMQ 整体架构](../../images/arch_overall.png)

---

## Broker 分层结构

Broker 是无状态的协议处理层，内部按以下层次组织：

| 层次 | 说明 |
|------|------|
| 网络层 | TCP / TLS / WebSocket / WSS / QUIC |
| 协议层 | MQTT / Kafka / NATS / AMQP / mq9 协议解析 |
| 协议逻辑层 | 各协议独立的业务模块（mqtt-broker、kafka-broker 等） |
| 消息通用逻辑层 | 消息收发、过期、延迟发布、安全认证、Schema 校验、监控指标 |
| Storage Adapter | Shard 抽象层，将写操作路由到对应存储后端 |

Broker 本身不持久化任何数据，所有状态存储在 Meta Service 或 Storage Engine 中。

---

## 节点间通信

每个节点启动时会初始化三类 Server：

| Server | 协议 | 用途 |
|--------|------|------|
| Inner gRPC Server | gRPC | 节点间内部通信（Meta ↔ Broker ↔ Storage Engine） |
| Admin HTTP Server | HTTP | 对外运维接口（REST API） |
| Prometheus Server | HTTP | 指标采集接口 |

---

## 启动顺序

节点内各模块按固定顺序初始化：

1. **配置加载**：读取 `config.toml`，解析 `roles` 字段确定本节点启用哪些组件
2. **日志系统**：初始化 tracing subscriber，所有后续日志依赖此步骤
3. **gRPC Server 启动**：Inner gRPC Server、Admin HTTP Server、Prometheus Server 按序绑定端口
4. **Meta Service 初始化**（如启用）：初始化 MultiRaftManager，依次创建三个 Raft Group，完成 Leader 选举
5. **Storage Engine 初始化**（如启用）：创建 I/O Worker Pool，挂载已有 Segment 文件，向 Meta Service 注册节点
6. **Broker 初始化**（如启用）：连接 Meta Service，加载 Topic / Shard 缓存，启动协议处理器
7. **就绪**：对外接受客户端连接

![节点启动顺序](../../images/arch_startup_order.png)

---

## 部署模式

**单机模式**：三个组件运行在同一个进程内，适合开发、测试、边缘场景。

**分布式模式**：三个组件分别部署在不同节点，独立扩展。典型配置：

- Meta Service：3 或 5 节点（Raft 奇数节点）
- Broker：按流量水平扩展，无状态
- Storage Engine：按存储容量扩展

![部署模式](../../images/arch_overall_deploy.png)
