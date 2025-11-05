# 系统架构

![image](../../images/mqtt-broker-arch.png)

如上图所示： RobustMQ MQTT 由 MQTT Broker、Meta Service、Storage Engine 三部分组成。

MQTT Broker 是完全无状态的节点，基于 Meta Service 完成节点发现、节点探活，从而完成集群构建。MQTT 客户端随机访问一台 Broker 完成消息数据的 Pub/Sub。MQTT Broker 支持基于 TCP 的 MQTT 3/4/5 协议的解析和基于 GRPC 协议的集群内部管控和调度。

MQTT 集群的元数据存储在 Meta Service Cluster 中。Meta Service 是 RobustMQ MQTT 集群的元数据管理中心，负责 MQTT 集群的元数据管理、集群的节点管理、集群的故障恢复等等。Meta Service 会运行 MQTT Broker 集群对应的控制器线程。负责 MQTT 集群的调度，比如共享集群的 Leader。

MQTT 集群通过 Storage Adapter layer 持久化存储消息数据到Storage Engine。
