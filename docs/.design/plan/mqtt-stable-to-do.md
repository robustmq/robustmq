## MQTT 功能完善
1. 离线消息【done】
2. 订阅逻辑优化【done】
3. MQTT 测试用例失败 case 修复【done】
4. 自动订阅【ing】
5. 延时发布【done】
6. 消息验证：确保消息的完整性和合法性【done】
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
14. 主题监控

## 测试用例覆盖度
1. 代码测试覆盖度(单元测试和集成测试)
   1. meta service
   2. MQTT Broker
2. 功能测试覆盖度
   1. 连接建立/关闭(TCP/TCPS/WebSocket/WebSockets,QUIC)
   2. Pub/Sub
   3. 离线消息（开启关闭）
   4. Session相关（持久化）
   5. QOS 0,1,2
   6. 保留消息
   7. 遗嘱消息
   8. 请求/响应
   9.  用户属性
   10. 主题别名
   11. 载荷格式指示与内容类型
   12. 共享订阅
   13. 订阅选项
   14. 订阅标识符
   15. 保持连接
   16. 消息过期间隔
   17. 最大报文大小
   18. 错误码
   19. Auth
   20. ACL
   21. 通配符订阅

3. MQTT 高级特性
   1.  连接抖动
   2.  排他订阅
   3.  延迟发布
   4.  自动订阅
   5.  主题重写
   6.  Schema Registry
   7.  Connectgor
   8.  认证
   9.  授权
   10. 黑名单
   11. 连接抖动
   12. 限流
   13. 数据集成
     - Local File
     - Kafka
   14. 可观测性
      -  Metrics
      -  集成 promethrus
      -  主题监控
      -  慢订阅统计
      -  Trace
   15. Schema
   16. 消息 CRC 验证

## 压测
1. Placement 集群模式 Read/Write 延时和吞吐压测
2. MQTT 发布订阅模式压测
   1. connection
   2. publish
   3. subscribe
   4. qos
3. Journal Engine Node Read/Write 延时和吞吐压测

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
