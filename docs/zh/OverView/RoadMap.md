## 2024
2024 年：完善 RobustMQ MQTT, 完善 RobustMQ 的基础能力。
- 完善 RobustMQ MQTT 的功能
- 提高元数据服务 Placement Center 的稳定性和性能
- 压测 RobustMQ MQTT，并优化性能
- 压测 Placement Center，并优化性能
- 完善 RobustMQ MQTT 和 Placement Center 的测试用例
- 完善 RobustMQ 的官网和技术文档
- 寻找对 Rust 或中间件感兴趣的小伙伴，一起打造一个牛逼的基础软件

## 2025 H1
### 概述

### 明细
#### Placement Center
> 目标： Placement Center 集群模式稳定运行、压测性能、完善监控指标

详细工作：
1. 完善集群能力，能运行稳定的集群模式
   1. 集群模式下，GRPC 提供的接口稳定运行
   2. 集群模式下，Leader 切换，GRPC 提供的接口稳定运行
   3. 集群模式下，Leader 切换，非 Leader 节点的 Controller 暂停，Leader 节点运行 Controller
2. 压测 Placement Center 服务的性能
3. 添加 Placement Center 的监控指标
   
#### Robust MQTT 
> 目标：完成第一阶段，RobustMQ MQTT 集群模式稳定运行。优化当前功能代码、完善测试用例、增加部分功能。

集群模式：
1. Robust MQTT 多节点模式稳定运行
2. 集群模式下docker 多节点运行
3. 压测 MQTT TCP 的性能，并完善

测试用例：
1. 完善发布相关代码的单元测试
2. 完善订阅相关代码的单元测试

功能：
1. 订阅流程优化，无订阅自动丢弃消息，支持离线消息
2. 离线消息支持Journal
3. 离线消息支持 MySQL
4. 离线消息支持 Redis
5. MQTT 数据集成框架实现
6. MQTT 数据集成支持 Kafka
7. MQTT 支持 Schema 框架实现
8. 自动订阅
9. 连接抖动
10. 完善 Metrics
11. 接入opentelemetry
12. 限流模块开发（请求数、连接数、流量等等）
13. Auth 模块完善

Cli(命令行)：
1. 完善命令行输出内容的统一格式
2. 完善 MQTT 相关 Admin 接口和能力

Dashboard:
1. 完成 Dashboard 的框架搭建
2. 完成 Dashboard 和 MQTT Broker 的 Admin 相关接口联动

#### Journal Engine 
> 目标：Journal Engine 单节点模式稳定运行

功能:
1. Journal Engine 第一版核心功能稳定版本代码
2. 单机模式核心流程完善
   1. 数据读写
   2. Shard/Segment 增删改查
   3. Segment Status 转换
3. Journal Engine Client 完善
   1. 支持数据读写核心能力
4. 稳定的单机模式运行
5. 设计多副本的 ISR 模式

#### Storage Adapter
> 目标：支持 Journal Engine、MySQL、Redis，探索支持远程对象存储(比如 Aws S3)

功能：
1. 支持 Journal
2. 支持 MySQL
3. 支持 Redis
4. 支持 Aws S3
   
## 2025 H2