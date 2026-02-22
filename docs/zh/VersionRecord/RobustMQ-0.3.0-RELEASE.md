# RobustMQ 0.3.0 RELEASE

<p align="center">
  <img src="../../images/robustmq-logo.png" alt="RobustMQ Logo" width="200">
</p>

**发布日期**：2025年2月  
**版本**：0.3.0  
**GitHub**：[https://github.com/robustmq/robustmq](https://github.com/robustmq/robustmq)

> **注意**：0.3.0 仍处于早期阶段，暂不建议在生产环境使用。我们计划在 0.4.0 版本（预计 2025 年 5 月）达到生产可用标准。欢迎提前试用并反馈问题。

---

## 版本亮点

RobustMQ 0.3.0 是一个重要的里程碑版本——不只是功能迭代，而是对整体定位和架构的一次重新梳理。

从 0.3.0 开始，RobustMQ 的定位变得更加清晰：**下一代 AI、IoT、大数据统一的通信基础设施**。通过 MQTT 和 Kafka 双协议、百万级 Topic、对象存储（S3/MinIO）直连、多模式存储引擎、智能数据缓存，为 AI 训练、Agent 通信、IoT 设备、大数据处理提供高性能、低成本、稳定的通信基础设施。

---

## 架构重新设计

0.3.0 对整体架构做了重新设计，系统由三个职责清晰的组件构成。

### Meta Service
负责集群元数据管理和协调。所有节点状态、Topic 配置、客户端会话信息均存储于此，通过自研的 **Multi Raft** 机制保证一致性和高可用。

### Broker
负责协议处理和请求路由。Broker 是无状态的，只处理客户端连接、协议解析、消息路由，不持有任何持久化数据。存算分离设计让 Broker 可随时水平扩展，加节点无需数据迁移。

### Storage Engine
负责数据持久化，支持三种存储引擎：

| 引擎 | 特点 | 适用场景 |
|------|------|----------|
| Memory | 纯内存，微秒级延迟 | 低延迟、临时数据 |
| RocksDB | 统一 KV 存储 | 百万级 Topic，通用持久化 |
| File Segment | 顺序写入，高吞吐 | Kafka 场景，大数据流 |

上层协议与存储引擎之间是插件化接口，未来可扩展更多存储后端，核心架构不变。

### Multi Raft
Meta Service 使用自研的 Multi Raft 实现，支持多个独立的 Raft Group，不同类型的元数据由不同的 Raft Group 管理，避免单一 Raft 的性能瓶颈，是支撑后续大规模部署的重要基础设施。

---

## MQTT Broker 核心功能

0.3.0 的 MQTT Broker 在功能完整性上达到了重要节点，核心功能覆盖了生产级 MQTT Broker 的主要场景：

- **完整协议支持**：MQTT 3.1 / 3.1.1 / 5.0，包含连接/断开、Keep-alive、遗嘱消息、保留消息、QoS 0/1/2
- **会话管理**：Session 持久化与恢复
- **订阅模式**：共享订阅、主题重写与过滤
- **安全认证**：用户名密码认证、ACL 权限控制
- **消息特性**：离线消息存储、延迟消息
- **规则引擎**：基础规则引擎功能

---

## 代码质量：多轮重构与 Bug 修复

0.3.0 背后是大量不可见的工程投入：

**架构重构**
- 连接管理层从单一实现重构为可扩展的抽象层
- 存储引擎从耦合设计拆分为插件化接口
- gRPC 客户端层加入重试机制和超时控制
- Handler 处理层加入每线程独立监控指标

**稳定性修复**
- 修复多个高并发场景下的 race condition
- 修复会话恢复逻辑的边界 bug
- 修复大量连接下的内存增长问题

**性能优化**
- 优化连接建立的关键路径，减少不必要的 gRPC 调用
- Handler 处理超时机制，避免任务卡死导致吞吐崩塌

---

## 生态工具完善

### Grafana + Prometheus
内置完整的监控指标体系，覆盖连接数、消息吞吐、Handler 延迟、gRPC 耗时、存储引擎读写、队列积压等核心维度。提供开箱即用的 Grafana Dashboard，部署后直接导入即可查看完整运行状态。

### Command CLI（robust-ctl）
完善命令行管理工具，支持集群状态查询、Topic 管理、客户端会话查看、用户权限配置等日常运维操作。

### HTTP API
提供完整的 REST API，覆盖集群管理、Topic 操作、用户管理、规则引擎配置等，方便与现有运维平台和自动化脚本集成。

### Bench CLI（robust-bench）
内置压测工具，支持 MQTT 连接、发布、订阅多种压测模式，可指定并发数、消息速率、Payload 大小、持续时长等参数，方便部署后快速验证集群性能和稳定性。

### RobustMQ Dashboard
提供 Web 管理控制台，集群概览、节点状态、Topic 列表、客户端连接、规则引擎配置均可在界面上直接操作，降低运维门槛。

### 官网与文档
重新梳理了官网结构和文档体系，覆盖快速上手、架构介绍、配置参考、API 手册、压测指南等核心内容，文档与代码同步维护。

---

## 安装与使用

### 一键安装

```bash
# 安装最新版本（默认安装到 ~/robustmq-v0.3.0，并创建 ~/robustmq 符号链接）
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

### 启动服务

```bash
# 启动 Placement Center（Meta Service）
robust-server placement-center start

# 启动 MQTT Broker
robust-server mqtt-server start
```

### 快速验证

```bash
# 安装 MQTTX CLI（https://mqttx.app/zh/docs/cli）
# 订阅
mqttx sub -h localhost -p 1883 -t "test/topic"

# 发布（另开终端）
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"
```

Web 控制台：访问 `http://localhost:3000`（Dashboard）

完整文档：[快速上手指南](../QuickGuide/Quick-Install.md)

### Docker

```bash
docker pull ghcr.io/robustmq/robustmq:v0.3.0
docker pull ghcr.io/robustmq/robustmq:latest
```

### 源码编译

```bash
git clone https://github.com/robustmq/robustmq.git
cd robustmq
git checkout v0.3.0
cargo build --release
```

---

## 后续计划

| 方向 | 内容 |
|------|------|
| 性能与稳定性 | MQTT 连接建立路径优化、Raft 写入吞吐提升、存储引擎深度调优 |
| MQTT 完善 | 规则引擎完整实现、Webhook 集成、运维 API 补全 |
| AI MQ | Topic 直连对象存储（S3/MinIO）、三层智能缓存、预测式预加载 |
| Kafka 协议 | Kafka 完整协议实现，兼容 Flink、Spark、Kafka Connect |

详见 [2026 年 RoadMap](../OverView/RoadMap-2026.md)。

---

## 支持与反馈

- **GitHub**：[https://github.com/robustmq/robustmq](https://github.com/robustmq/robustmq)
- **Issues**：[https://github.com/robustmq/robustmq/issues](https://github.com/robustmq/robustmq/issues)
- **Discussions**：[https://github.com/robustmq/robustmq/discussions](https://github.com/robustmq/robustmq/discussions)
- **文档**：[https://robustmq.com](https://robustmq.com)

---

**RobustMQ 团队**  
2025年2月
