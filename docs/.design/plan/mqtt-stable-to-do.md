## 功能覆盖度
1. 离线消息【done】
2. 订阅逻辑优化【done】
3. MQTT 测试用例失败 case 修复【done】
4. 自动订阅【ing】
5. 延时发布【done】
6. 消息验证：确保消息的完整性和合法性
7. 数据集成【done】
   1. sink local file
   2. Kafka 集成
8. 限流【done】
   1. 速率限制
   2. QOS 限制
9. 可观测体系搭建 【done】 需要继续丰富指标
10. ACL/User 【done】
11. Schema: Schema Registry、灵活地消息格式转换：JSON、Avro、Protobuf、Custom codec (HTTP/gRPC)【done】
12. Cloud-Native & K8s 【done】
13. MQTT Over QUIC 【done】

## 测试用例覆盖度
1. 代码测试覆盖度
   1. placement center
   2. MQTT Broker
2. 功能测试覆盖度
   1. TCP、TCPS、WebSocket，WebSockets
   2. 保留消息
   3. 遗嘱消息
   4. 请求相应
   5. 用户属性
   6. 主题别名
   7. 载荷格式指示与内容类型
   8. 共享订阅
   9. 订阅选项
   10. 订阅标识符
   11. 保持连接
   12. 消息过期间隔
   13. 最大报文大小
   14. 错误码
   15. Auth
   16. ACL
   17. 连接抖动
   18. 排他订阅
   19. 延迟发布
   20. 自动订阅
   21. 主题重写
   22. 通配符订阅
3.

## 压测
1. Placement 集群模式 Read/Write 延时和吞吐压测
2. MQTT 发布订阅模式压测
   1. connection
   2. publish
   3. subscribe
   4. qos
3. Journal Engine Node Read/Write 延时和吞吐压测

## 巡检功能
- 简介
- 核心概念
- 系统架构
- 客户端工具使用
- 核心功能
  - 共享订阅
  - 保留消息
  - 遗嘱消息
  - 排他订阅
  - 延迟发布
  - 自动订阅
  - 主题重写
  - 通配符订阅
  - Session 持久化
- GRPC Admin 接口
- Bench 性能压测
- Schema Registry
- 安全
  - 认证
  - 授权
  - 黑名单
  - 连接抖动
- 数据集成
  - Local File
  - Kafka
  - RocketMQ
-  可观测性
   -  指标
   -  trace
   -  集成 promethrus
   -  集成 OpenTelemetry
-  MQTT Over Quic


## MQTT 文档
- 简介
- 核心概念
- 系统架构
- 客户端工具使用
- 核心功能
  - 共享订阅
  - 保留消息
  - 遗嘱消息
  - 排他订阅
  - 延迟发布
  - 自动订阅
  - 主题重写
  - 通配符订阅
  - Session 持久化
- 客户端 SDK
  - 使用 C SDK 连接
  - 使用 Java SDK 连接
  - 使用 Go SDK 连接
  - 使用 Python SDK 连接
  - 使用 Javascript SDK 连接
- GRPC Admin 接口
- Bench 性能压测
- Schema Registry
- 安全
  - 认证
  - 授权
  - 黑名单
  - 连接抖动
- 数据集成
  - Local File
  - Kafka
  - RocketMQ
-  可观测性
   -  指标
   -  trace
   -  集成 promethrus
   -  集成 OpenTelemetry
-  MQTT Over Quic
-
