# 概览
RobustMQ MQTT 是RobustMQ 对 MQTT 协议的完整实现。它已 100% 支持标准的MQTT 协议。因此可兼容业界所有MQTT 命令行、SDK、Web 工具等。因此业务使用RobustMQ MQTT，完全不需改动。百分百兼容当前MQTT的生态。 

从功能上看，已对齐 EMQX 90% 以上的功能。所有MQTT 相关核心功能已100%覆盖。

## RobustMQ MQTT 特性列表

| 特性 | 描述 |
| --- | --- |
| 集群化部署 | Broker 节点无状态部署，单集群最多支持几百上千台Broker 节点 |
| 单机最大连接 | 单机可承载百万连接。 |
| 集群最大连接 | 集群可承载百亿级别的连接。 |
| MQTT 协议 | 完整支持MQTT 3.1/3.1.1/5.0 的所有特性 |
| 网络协议 | 支持TCP、SSL、WebSocket、WebSockets、Quic协议接入 |
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
| 保留消息 | 支持 |
| 遗嘱消息 | 支持 |
| 共享订阅 | 支持 |
| 系统主题 | 支持 |
| 排他订阅 | 支持 |
| 延迟发布 | 支持 |
| 自动订阅 | 支持 |
| 主题重写 | 支持 |
| 通配符订阅 | 支持 |
|  系统主题 | 支持 |
| 慢订阅统计 | 支持 |
| Session | 支持 Session，以及 Session 持久化和过期。 |
| 认证 | 支持内置数据库、MySQL、Redis 的密码认证 |
| 授权 | 支持内置数据库、MySQL、Redis的认证实现 |
| 黑名单 | 支持 |
| 连接抖动 | 支持 |
| 数据集成 | 支持File、Kafka 的桥接连接器 |
| Schema | Json、Protobuf、AVRO |
| 消息存储 | 当 Topic 没有订阅时，消息会被自动被丢弃 |
| 离线消息 | 支持基于 Memory、RocksDB、MySQL、Journal Engine、S3、Minio 等存储引擎来存储离线消息 |
| 指标(Metrics) | 支持集群/Topic等维度的监控指标 |
| Prometheus | 支持 |
| Trace | 支持 |

## EMQX 特性对比
| 特性 | EMQX |RobustMQ MQTT |
| --- | --- |--- |
| 集群化部署 | 支持 | 支持 |

## Dashboard
RobustMQ Dashboard 已完成对RobustMQ MQTT 协议功能的支持。详细文档请参考：


## 命令行工具
RobustMQ MQTT 支持 robust-ctl mqtt 工具。详细文档请参考：[robustmq-ctl mqtt](../RobustMQ-Command/CLI_COMMON.md)
