RobustMQ MQTT是 RobustMQ 适配MQTT协议的完整实现。它支持：
- 集群化部署，水平无感知扩容。
- 单机可承载百万连接。
- 完整支持MQTT3.1/3.1.1/5.0协议。
- 支持TCP、SSL、WebSocket、WebSockets协议。
- 支持 Session，以及 Session 持久化和过期。
- 支持保留消息。
- 支持遗嘱消息。
- 支持共享订阅。
- 支持系统主题。
- 支持 Schema
- 支持数据集成

整体架构如下：
![image](../../images/doc-image5.png)
- MQTT Broker 是无状态的节点，MQTT 客户端随机访问一台 Broker 完成消息数据的 Pub/Sub。
- MQTT Broker 基于 Placement Center 完成节点发现、节点探活，从而完成节点构建。
- MQTT 集群通过 Storage Adapter layer 持久化存储消息数据。MQTT 集群的元数据存储在 Placement Center Cluster 中。
- MQTT Broker 支持基于 TCP 的 MQTT 3/4/5 协议的解析和基于 GRPC 协议的集群内部管控和调度。
- Placement Center 会运行 MQTT Broker 集群对应的控制器线程。负责 MQTT 集群的调度，比如共享集群的 Leader。
