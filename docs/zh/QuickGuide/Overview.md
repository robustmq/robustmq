## 概述
RobustMQ 长期预计支持多种消息队列协议。当前已支持 MQTT 协议，即：RobustMQ MQTT.

## RobustMQ MQTT

### 部署模式
RobustMQ MQTT 单机和集群两种部署模式。
- 单机模式：启动单机模式的 MQTT Server，其中 Placement Center 和 MQTT Server 都是单机运行。
- 集群模式：启动集群模式的 MQTT Server，其中 Placement Center 和 MQTT Server 都是多节点集群模式运行。其中 Placement Center 默认三节点，MQTT Broker 节点数量不限制。

## 运行方式
1. Cargo 运行： 下载源代码，然后执行 cargo run 命令运行 MQTT Server。该方式适用于开发调试。
2. 二进制包运行：下载或编译二进制包，然后执行二进制包运行 MQTT Server。该方式适用于生产环境。
3. Docker 运行：即下载或编译 Docker 镜像，然后执行 Docker 镜像运行 MQTT Server。该方式适用于生产环境。
4. K8s 运行：即在 K8s 集群中运行 MQTT Server。该方式适用于生产环境。
   
> 建议：在开发调试阶段，我们一般用 Cargo 运行。在生产环境我们一般推荐用 docker 或 K8s 方式运行，因为这样可以方便的进行扩容和缩容。同时我们也支持二进制包运行。

### 快速启动
- [编译二进制安装包【可选】](mqtt/Build.md)
- [二进制运行-单机模式](mqtt/Run-Standalone-Mode.md)
- [二进制运行-集群模式](mqtt/Run-Cluster-Mode.md)
- [Docker 运行](mqtt/Run-Docker-Mode.md)
- [K8s 运行](mqtt/Run-K8S-Mode.md)