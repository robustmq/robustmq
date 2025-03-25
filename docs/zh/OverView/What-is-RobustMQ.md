## 一、RobustMQ
### 1.1 Logo
![image](https://uploader.shimo.im/f/edIiOkJ79eEBngLX.png!thumbnail?accessToken=eyJhbGciOiJIUzI1NiIsImtpZCI6ImRlZmF1bHQiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE3NDIzNTc4NTEsImZpbGVHVUlEIjoiRWUzMm1FbGFlZWhaejlBMiIsImlhdCI6MTc0MjM1NzU1MSwiaXNzIjoidXBsb2FkZXJfYWNjZXNzX3Jlc291cmNlIiwicGFhIjoiYWxsOmFsbDoiLCJ1c2VySWQiOjQxNTIyNzgwfQ.6xsFSqx8WnH7_y1NhfiSDDIgc-ayAwqNm6DzeNyV5kk)

### 1.2 定义
下一代高性能云原生融合型消息队列。
> Next-generation high performance cloud-native converged message queues.

### 1.3 愿景

用 Rust 打造一个高性能的、高可用、功能齐全的、兼容多种主流消息队列协议的、架构上具备完整 Serverless 能力的消息队列。

> Use Rust to build a high-performance, stable, fully functional message queue that is compatible with a variety of mainstream message queue protocols, and has complete Serverless architecture.

### 1.4 特点
- 100% Rust：完全基于 Rust 语言实现的消息队列引擎。
- 多协议：支持MQTT 3.1/3.1.1/5.0、AMQP、Kafka Protocol、RocketMQ Remoting/GRPC、OpenMessing、JNS、SQS 等主流消息协议。
- 分层架构：计算、存储、调度完全独立的三层架构，职责清晰、独立。
- Serverless：所有组件均具备分布式集群化部署，快速扩缩容的能力。
- 插件式存储：独立插件式的存储层实现，支持独立部署和共享存储两种架构。
- 功能齐全：完全对齐协议对应的社区主流 MQ 产品的功能和能力。

### 1.5 为什么
探索 Rust 在消息队列领域的应用，基于 Rust 打造一个在性能、功能、稳定性方面表现更优秀的消息队列产品。
- 打造一个能够兼容各种主流消息队列协议的消息队列，从而降低使用、学习、运维成本。
- 打造一个具备完整 Serverless 能力的消息队列，从而降低资源成本。
- 打造一个能够满足各种场景需求的消息队列。


## 二、长期规划
RobustMQ 的愿景是支持多协议，并在架构上具备完整的Serverless能力。整体开发工作量大，因此分为几个阶段：

- 第一阶段：主要会完成集群的基础框架的（比如元数据存储服务、存储适配层、自带存储层等）的开发。

- 第二阶段：完成 MQTT 协议相关功能的开发，目标是完成 RobustMQ 整体架构的搭建、验证可行性，并完成MQTT协议适配。功能上对齐 EMQX ，最终实现RobustMQ MQTT 生产可用。

- 第三阶段：会启动 Kafka 协议相关功能的开发，功能上对齐，实现Kafka RobustMQ Kafka 生产可用。

- 第四阶段：会启动 AMQP/RocketMQ 等协议的开发。

> 当前所处的是第二阶段，目标是在 2025 年 年中完善 RobustMQ MQTT，实现 RobustMQ MQTT Release 版本发布。
