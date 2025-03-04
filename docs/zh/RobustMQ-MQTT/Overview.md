# 概览
RobustMQ MQTT 是融合型消息队列 RobustMQ 对 MQTT 协议的完整实现，它完整支持 MQTT 3.1/3.1.1/5.0 的全部特性和功能。支持集群化模式部署，单集群可承载百亿级连接，同时支持 TCP、SSL、WebSocket、WebSockets 、QUIC 等多种访问方式。

RobustMQ MQTT 目标是基于 Rust 打造一个高性能、高可用、高可扩展的、支持标准 MQTT 协议的 Broker Server。

## 功能清单
| 特性 | 描述 |
| --- | --- |
| 集群化部署 | Broker 节点无状态部署，单集群最多支持几百上千台Broker 节点 |
| 单机最大连接 | 单机可承载百万连接。 |
| 集群最大连接 | 集群可承载百亿级别的连接。 |
| MQTT 协议 | 完整支持MQTT 3.1/3.1.1/5.0 的所有特性 |
| 网络协议 | 支持TCP、SSL、WebSocket、WebSockets协议接入 |
| 保留消息 | 支持 |
| 遗嘱消息 | 支持 |
| 共享订阅 | 支持 |
| 系统主题 | 支持 |
| 排他订阅 | 支持 |
| 延迟发布 | 支持 |
| 自动订阅 | 支持 |
| 主题重写 | 支持 |
| 通配符订阅 | 支持 |
| Session | 支持 Session，以及 Session 持久化和过期。 |
| 认证 | 支持内置数据库、MySQL、Redis 的密码认证 |
| 授权 | 支持内置数据库、MySQL、Redis的认证实现 |
| 黑名单 | 支持 |
| 连接抖动 | 支持 |
| 消息存储 | 当 Topic 没有订阅时，消息会被自动被丢弃 |
| 离线消息 | 支持基于 Memory、RocksDB、MySQL、Journal Engine、S3、Minio 等存储引擎来存储离线消息 |
| 数据集成 | 支持File、Kafka 的桥接连接器 |
| 指标(Metrics) | 支持集群/Topic等维度的监控指标 |
| Prometheus | 支持 |
| Trace | 支持 |
| 系统主题 | 支持 |
| 慢订阅统计 | 支持 |
| Schema | Json、Protobuf、AVRO |
| QUIC 协议 | 支持 |

## MQTT 5 特性清单
| 特性 | 描述 |
| --- | --- |
| MQTT 发布/订阅 | 支持 |
| 订阅 QOS 0,1,2 | 支持 |
| 发布 QOS 0,1,2 | 支持 |
| 订阅通配符 | 支持 |
| Session | 支持, 支持 Session 持久化和过期。 |
| 保留消息 | 支持 |
| 遗嘱消息 | 支持 |
| 请求/响应（Request/Response） | 支持 |
| 用户属性(User Properties) | 支持 |
| 主题别名(Topic Alias) | 支持 |
| 载荷格式指示与内容类型（Payload Format Indicator & Content Type） | 支持 |
| 共享订阅（Shared Subscriptions） | 支持 |
| 订阅选项（Subscription Options） | 支持 |
| 订阅标识符（Subscription Identifier） | 支持 |
| 保持连接（Keep Alive）  | 支持 |
| 消息过期间隔（Message Expiry Interval） | 支持 |
| 最大报文大小（Maximum Packet Size） | 支持 |

## 架构概述

![image](../../images/doc-image5.png)

如上图所示： RobustMQ MQTT 由 MQTT Broker、Placement Center、Storage Engine 三部分组成。

Placement Center 是 RobustMQ MQTT 集群的元数据管理中心，负责 MQTT 集群的元数据管理、集群的节点管理、集群的故障恢复等等。

MQTT Broker 是完全无状态的节点，MQTT 客户端随机访问一台 Broker 完成消息数据的 Pub/Sub。MQTT Broker 基于 Placement Center 完成节点发现、节点探活，从而完成集群构建。

MQTT 集群通过 Storage Adapter layer 持久化存储消息数据到Storage Engine。

MQTT 集群的元数据存储在 Placement Center Cluster 中。MQTT Broker 支持基于 TCP 的 MQTT 3/4/5 协议的解析和基于 GRPC 协议的集群内部管控和调度。

Placement Center 会运行 MQTT Broker 集群对应的控制器线程。负责 MQTT 集群的调度，比如共享集群的 Leader。

## Dashboard
RobustMQ MQTT Dashboard 正在加紧开发中

## 命令行工具
RobustMQ MQTT 支持 robust-ctl mqtt 工具。详细文档请参考：[robustmq-ctl mqtt](../RobustMQ-Command/Mqtt-Broker.md)