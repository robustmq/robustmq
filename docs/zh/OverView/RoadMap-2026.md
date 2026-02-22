# RoadMap 2026

## 总体目标

当前版本为 **0.3.0**。2026 年计划发布 **2～3 个大版本**，版本号从 0.4.0 起，预计推进至 0.5.0 或 0.6.0。

全年核心主题是**代码质量、性能与稳定性**，以实现以下三个里程碑：

| 里程碑 | 目标 |
|--------|------|
| MQTT Broker 生产可用 | 集群稳定运行、压测达标、具备完整的运维和可观测能力 |
| Kafka 基本功能可用 | 核心读写路径打通，主流 Kafka 客户端可直接接入 |
| AI MQ 探索落地 | 对象存储数据源、智能缓存层、AI Agent 通信 Channel 初步可用 |

---

## v0.4.0（预计 2026 年 5 月）

> 目标：MQTT Broker 达到生产可用标准，完成大规模代码重构和性能调优。

### MQTT Broker

#### 性能与稳定性
1. 完成一轮全面的代码重构，消除技术债务
2. 压测 MQTT 集群模式（连接数、发布/订阅吞吐、延迟），制定基线
3. 修复压测发现的所有阻塞性 Bug
4. 连接层优化：减少不必要的内存分配，降低 P99 延迟
5. 消息路由层优化：共享订阅分发性能提升

#### 可靠性
1. 完善 QoS 1/2 消息确认链路，修复边界条件
2. Session 持久化（RocksDB）可靠性验证和测试覆盖
3. 集群节点宕机/重启后，在线 Session 无感恢复

#### 可观测性
1. 完善 Prometheus Metrics 指标覆盖（连接、发布、订阅、存储各维度）
2. Grafana Dashboard 完善，提供开箱即用的监控模板
3. 结构化日志：统一日志格式，支持按 ClientID / Topic 过滤

#### 安全
1. TLS/mTLS 双向认证稳定化
2. ACL 规则引擎性能优化
3. JWT 认证支持

#### 测试
1. 单元测试覆盖率目标 ≥ 60%
2. 集成测试：覆盖 MQTT 3.x / 5.0 全协议流程
3. 混沌测试：节点宕机、网络分区场景下的行为验证

### Meta Service
1. Multi Raft 选举和日志复制稳定性验证
2. 压测 Meta Service gRPC 接口，优化热点路径
3. 快照和日志压缩机制完善

### Storage Engine（Journal）
1. 集群模式（多副本 ISR）稳定运行
2. Segment 文件管理：过期清理、容量告警
3. Journal Client 重连和重试机制完善

### 文档 & 社区
1. 完善生产部署文档（K8s、Docker Compose 集群模式）
2. 发布性能基准报告
3. 新增贡献者指南，降低社区参与门槛

---

## v0.5.0（预计 2026 年 9 月）

> 目标：Kafka 基本功能可用，AI MQ 初步形态，MQTT 能力持续增强。

### Kafka 基本功能
1. **Producer 协议**：Produce 请求解析、消息写入 Journal Engine
2. **Consumer 协议**：Fetch 请求、Offset 管理、Consumer Group 基础支持
3. **Topic 管理**：CreateTopics / DeleteTopics / DescribeTopics
4. **兼容性验证**：Java Kafka Client、kafka-console-producer/consumer 直连测试
5. Admin API 支持 Kafka Topic 的查询和管理

### AI MQ（探索阶段）
1. **对象存储数据源**：Topic 直接挂载 S3 / MinIO 路径，支持顺序读取
2. **智能缓存层（原型）**：三层缓存（内存 → SSD → 对象存储）框架搭建
3. **AI Agent Channel**：基于 MQTT 共享订阅，实现 Agent 间隔离通信的最佳实践和示例
4. 发布 AI MQ 技术设计文档，收集社区反馈

### MQTT Broker（能力增强）
1. 规则引擎：增强 SQL 过滤能力，支持更多下游目标（HTTP Webhook、Kafka、GreptimeDB）
2. 数据桥接：完善 Kafka、MySQL、PostgreSQL Bridge 的稳定性
3. 慢订阅检测和告警
4. 连接数、消息速率、流量的精细化限流
5. MQTT over QUIC（预研）

### 运维工具
1. CLI 工具：支持集群状态诊断、连接查询、Topic 统计
2. HTTP Admin API：补全缺失接口，提供完整的 OpenAPI 文档
3. Dashboard：告警配置、规则引擎可视化配置

---

## v0.6.0（预计 2026 年 12 月）

> 目标：Kafka 功能持续完善，AI MQ 功能增强。

### Kafka 功能持续完善
1. Consumer Group Rebalance 完整实现
2. 事务 Producer 基础支持
3. Kafka Streams 和 Kafka Connect 兼容性测试

### AI MQ（功能增强）
1. 智能预加载：基于访问模式预测，提前将数据从对象存储加载到内存/SSD
2. 多租户 AI Channel：基于 Namespace 隔离，支持多个 AI 训练任务并发使用
3. GPU 友好的批量读取接口（大 Batch 低延迟）

### MQTT Broker（生态完善）
1. MQTT Bridge 支持更多协议（Redis Streams、RabbitMQ）
2. 插件化扩展框架（用户自定义认证、自定义路由规则）
3. 多数据中心/异地容灾方案（预研）

---

## 长期协议规划（2026 年后）

RobustMQ 的架构从设计之初就支持多协议扩展，Broker 层的协议处理是插件化的。以下协议在架构上已有规划，长期会逐步支持，但 **2026 年内不在计划范围**——当前阶段专注于把 MQTT 和 Kafka 做好：

| 协议 | 状态 | 说明 |
|------|------|------|
| MQTT 3.x / 5.0 | ✅ 已支持 | 核心协议，持续完善 |
| Kafka | 🚧 开发中 | 2026 年逐步成型 |
| AMQP | 📅 长期规划 | 架构上支持，暂无短期计划 |
| RocketMQ | 📅 长期规划 | 架构上支持，暂无短期计划 |

---

## 全年通用目标

以下目标贯穿全年所有版本：

| 类别 | 目标 |
|------|------|
| 代码质量 | 每个版本进行一轮代码审查和重构，消除 Clippy 警告 |
| 测试 | 持续提升单测和集成测试覆盖率 |
| 文档 | 每个功能附带中英文文档，同步更新 |
| 社区 | 每季度发布 RobustMQ 进展博客，维护 Good First Issue 列表 |
| 发布节奏 | 每个大版本配套二进制包 + Docker 多架构镜像自动发布 |

---

## 版本节奏预览

```
2026 年 5 月  ──►  v0.4.0  MQTT 生产可用
2026 年 9 月  ──►  v0.5.0  Kafka 基本功能可用 + AI MQ 初步成型
2026 年 12 月 ──►  v0.6.0  Kafka 功能持续完善 + AI MQ 功能增强
```
