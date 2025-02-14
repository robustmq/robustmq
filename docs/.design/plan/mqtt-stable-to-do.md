## 功能覆盖度
1. 离线消息【done】
2. 订阅逻辑优化【done】
3. MQTT 测试用例失败 case 修复
4. 自动订阅
5. 数据集成
   1. sink local file
   3. Kafka 集成
6. 限流
   1. 速率限制
   2. QOS 限制
7. 可观测体系搭建
8. ACL/User
9. Schema: Schema Registry、灵活地消息格式转换：JSON、Avro、Protobuf、Custom codec (HTTP/gRPC)
10. 消息验证：确保消息的完整性和合法性
11. 规则引擎
12. Flow 设计器
13. 故障排查
14. Cloud-Native & K8s
15. 边缘计算
16. MQTT Over QUIC
17. 多协议网关

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