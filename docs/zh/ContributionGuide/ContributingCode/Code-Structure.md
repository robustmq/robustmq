# RobustMQ 代码结构说明

## 概述

RobustMQ 是一个用 Rust 构建的高性能、多协议消息队列系统。本文档详细介绍了项目的代码组织结构，帮助开发者快速理解和参与项目开发。

## 项目架构

RobustMQ 采用分布式分层架构，主要由以下四个层次组成：

- **元数据和调度层 (Meta Service)**：负责集群元数据存储和调度
- **协议适配层 (Multi-protocol Computing Layer)**：支持多种消息协议的适配
- **存储适配层 (Storage Adapter Layer)**：提供可插拔的存储抽象
- **存储层 (Storage Layer)**：具体的存储引擎实现

![RobustMQ Architecture](../../../images/robustmq-architecture.jpg)

## 源码目录结构

### 根目录结构

```
robustmq/
├── src/                    # 源代码目录
├── docs/                   # 项目文档
├── config/                 # 配置文件模板
├── docker/                 # Docker 相关文件
├── example/                # 示例和测试脚本
├── scripts/                # 构建和部署脚本
├── tests/                  # 集成测试
├── Cargo.toml             # Rust 工作空间配置
└── README.md              # 项目说明
```

## 核心模块详解

### 1. 服务器组件 (Server Components)

#### `src/broker-server/`
**功能**：主要的 Broker 服务器实现
- **职责**：协调各个协议 Broker，提供统一的服务入口
- **核心文件**：
  - `cluster_service.rs` - 集群服务管理
  - `grpc.rs` - gRPC 服务接口
  - `metrics.rs` - 性能指标收集

#### `src/meta-service/`
**功能**：元数据服务 (Meta Service)
- **职责**：集群元数据存储、节点管理、故障恢复
- **核心模块**：
  - `core/` - 核心缓存和控制器
  - `raft/` - Raft 共识算法实现
  - `storage/` - 元数据存储引擎
  - `controller/` - 各协议控制器

#### `src/journal-server/`
**功能**：日志存储服务器
- **职责**：消息持久化存储、索引管理
- **核心模块**：
  - `segment/` - 存储段管理
  - `index/` - 索引管理
  - `core/` - 核心存储逻辑

### 2. 协议实现 (Protocol Brokers)

#### `src/mqtt-broker/`
**功能**：MQTT 协议实现
- **职责**：支持 MQTT 3.x/4.x/5.x 协议
- **核心模块**：
  - `handler/` - MQTT 消息处理器
  - `subscribe/` - 订阅管理
  - `security/` - 认证和授权
  - `bridge/` - 消息桥接功能
  - `observability/` - 可观测性支持

#### `src/kafka-broker/`
**功能**：Kafka 协议实现
- **职责**：兼容 Kafka 协议的消息处理
- **状态**：开发中

#### `src/amqp-broker/`
**功能**：AMQP 协议实现
- **职责**：支持 AMQP 协议
- **状态**：开发中

### 3. 管理和工具 (Management & Tools)

#### `src/admin-server/`
**功能**：Web 管理控制台后端
- **职责**：提供 HTTP API 用于集群管理
- **核心模块**：
  - `mqtt/` - MQTT 相关管理接口
  - `cluster/` - 集群管理接口
  - `server.rs` - HTTP 服务器实现

#### `src/cli-command/`
**功能**：命令行工具
- **职责**：提供集群管理和运维命令
- **支持的命令**：
  - MQTT 管理命令
  - 集群管理命令
  - Journal 管理命令

#### `src/cli-bench/`
**功能**：性能测试工具
- **职责**：提供各种性能测试场景
- **测试类型**：
  - MQTT 发布/订阅测试
  - KV 存储测试
  - Raft 一致性测试

### 4. 核心库和工具 (Core Libraries)

#### `src/common/`
**功能**：通用组件库
- **base/** - 基础工具和类型定义
- **config/** - 配置管理
- **metadata-struct/** - 元数据结构定义
- **network-server/** - 网络服务器框架
- **rocksdb-engine/** - RocksDB 存储引擎封装
- **metrics/** - 指标收集框架
- **security/** - 安全相关组件

#### `src/protocol/`
**功能**：协议定义和编解码
- **职责**：定义各种协议的数据结构和编解码逻辑
- **支持协议**：
  - MQTT (3.x/4.x/5.x)
  - Kafka
  - AMQP
  - RobustMQ 内部协议

#### `src/storage-adapter/`
**功能**：存储适配器
- **职责**：提供统一的存储接口，支持多种存储后端
- **支持的存储**：
  - 内存存储
  - RocksDB
  - MySQL
  - S3
  - Journal Server

### 5. 专用功能模块 (Specialized Modules)

#### `src/delay-message/`
**功能**：延迟消息处理
- **职责**：实现消息延迟投递功能

#### `src/schema-register/`
**功能**：消息模式注册
- **职责**：消息格式验证和模式管理
- **支持格式**：JSON、Avro、Protobuf

#### `src/message-expire/`
**功能**：消息过期处理
- **职责**：清理过期消息

#### `src/grpc-clients/`
**功能**：gRPC 客户端库
- **职责**：提供各种 gRPC 服务的客户端实现
- **连接池管理**：优化连接复用

#### `src/journal-client/`
**功能**：Journal 客户端
- **职责**：提供 Journal Server 的客户端接口

## 模块依赖关系

### 依赖层次

```
应用层:     broker-server, admin-server, cli-*
           ↓
协议层:     mqtt-broker, kafka-broker, amqp-broker
           ↓
服务层:     meta-service, journal-server
           ↓
适配层:     storage-adapter, grpc-clients
           ↓
基础层:     common/*, protocol
```

### 核心依赖关系

1. **broker-server** 依赖所有协议 broker
2. **协议 broker** 依赖 meta-service 和 storage-adapter
3. **meta-service** 使用 Raft 算法进行一致性保证
4. **storage-adapter** 提供统一的存储抽象
5. **common** 模块被所有其他模块依赖

## 配置和部署

### 配置文件结构

```
config/
├── server.toml              # 主配置文件
├── server.toml.template     # 配置模板
├── server-tracing.toml      # 链路追踪配置
├── version.ini              # 版本信息
└── certs/                   # TLS 证书
```

### 构建产物

```
target/
├── debug/                   # 调试版本
├── release/                 # 发布版本
└── bin/                     # 可执行文件
    ├── robust-server        # 主服务器
    ├── robust-ctl          # 命令行工具
    └── robust-bench        # 性能测试工具
```

## 开发指南

### 添加新协议支持

1. 在 `src/` 下创建新的协议目录 (如 `new-protocol-broker/`)
2. 在 `src/protocol/` 中定义协议结构
3. 实现协议的编解码逻辑
4. 在 `src/broker-server/` 中集成新协议
5. 添加相应的测试和文档

### 扩展存储后端

1. 在 `src/storage-adapter/` 中实现新的存储适配器
2. 实现 `StorageAdapter` trait
3. 在配置文件中添加新存储类型的配置选项
4. 添加集成测试

### 添加管理功能

1. 在 `src/admin-server/` 中添加新的 HTTP 接口
2. 在 `src/cli-command/` 中添加对应的命令行命令
3. 更新相关文档

## 测试结构

### 单元测试
每个模块都包含 `tests/` 目录，用于模块级别的单元测试。

### 集成测试
`tests/` 目录包含端到端的集成测试：
- 集群功能测试
- 协议兼容性测试
- 性能测试

### 性能测试
使用 `src/cli-bench/` 工具进行性能测试和基准测试。

## 监控和可观测性

### 指标收集
- `src/common/metrics/` - 指标收集框架
- Prometheus 指标导出
- 自定义指标支持

### 链路追踪
- OpenTelemetry 集成
- 分布式链路追踪
- 性能分析支持

### 日志管理
- 结构化日志
- 多级别日志输出
- 日志轮转支持

## 总结

RobustMQ 采用模块化设计，各组件职责清晰，便于开发和维护。通过理解这个代码结构，开发者可以：

1. 快速定位相关功能代码
2. 了解模块间的依赖关系
3. 按照既定模式添加新功能
4. 进行有效的问题排查和性能优化

建议新贡献者从 `src/common/` 和 `src/protocol/` 开始了解，然后深入特定的协议实现。
