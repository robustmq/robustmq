## 1. 整体架构
RobustMQ 是分布式分层架构，由元数据和调度层（Placement Center）、计算层（Multi-protocol computing layer）、存储适配层（Storage Adapter Layer）、存储层（Storage layer） 四个部分组成。是典型的计算、存储、调度分层架构，每一层都具备快速扩缩容能力，从而使达到整个系统具备完整的 Serverless 能力。

![image](https://uploader.shimo.im/f/EzvImtDnVLmiWMp1.png!thumbnail?accessToken=eyJhbGciOiJIUzI1NiIsImtpZCI6ImRlZmF1bHQiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE3NDIzNTc4NTEsImZpbGVHVUlEIjoiRWUzMm1FbGFlZWhaejlBMiIsImlhdCI6MTc0MjM1NzU1MSwiaXNzIjoidXBsb2FkZXJfYWNjZXNzX3Jlc291cmNlIiwicGFhIjoiYWxsOmFsbDoiLCJ1c2VySWQiOjQxNTIyNzgwfQ.6xsFSqx8WnH7_y1NhfiSDDIgc-ayAwqNm6DzeNyV5kk)

- 元数据和调度层 (Plagement Center)： 元数据和调度层负责集群的元数据存储和调度。主要完成:
Broker 集群相关的元数据(Topic、Group、Queue、Partition 等)存储、分发。
Broker 集群的控制、调度，比如集群节点的上下线、配置存储/分发等。
Journal Engine 消费进度(Offset)信息等。

- 协议适配层（Multi-protocol computing layer）:  协议适配层负责各种消息协议的适配及其消息相关的功能实现。代码上每个协议会有其独立的 Broker，比如（RobustMQ MQTT、RobustMQ AMQP 等），功能上它会和各个协议的主流实现对标。协议适配层是完全无状态的的，数据的存储依赖 Placement Center 和 Storage Adapter Layer 来实现。Placement Center 负责元数据的存储，Storage Adapter Layer 负责消息数据的存储。

- 存储适配层（Storage Adapter Layer）: 存储适配层主要负责 RobustMQ 消息数据的高性能、可靠的持久化存储。存储适配层会将多种 MQ 协议中的 Topic/Queue/Partition 统一抽象为 Shard。同时适配不同的底层存储引擎（比如本地文件存储、远程的HDFS、对象存储、自研的存储组件等等），从而实现存储适配层的插件化、可插拔。

- 存储层（Storage layer）： 是指具体的、独立的存储引擎，比如硬盘（云硬盘、机械硬盘），云对象存储（比如 AWS S3），HDFS Cluster， Data Lake Cluster（iceberg，hudi 等）等等。RobustMQ 的存储层支持独立部署和共享存储两种架构。

  - 独立存储：存储层由内置的 RobustMQ Journal Engine 来实现，无需依赖外部组件。底层的存储介质是硬盘（硬盘、SSD、NVME 等）或云硬盘。这种部署方式好处是简单、无额外依赖，系统复杂度和运维成本低，同时具备 Serverless 特性。

  - 共享存储：存储层依赖现有的各种远程存储（比如HDFS、S3、OSS、GCS、Azure Blob、COS、BOS、MinIO...）的完成。这种部署方式的好处是在大规模流量下的成本相比独立部署方案会降低很多，适合在大流量、云架构部署场景下，对成本有强烈诉求的客户。

## 2. 独立部署架构【默认】
独立部署架构是指在不依赖任何外部组件的情况下部署 RobustMQ 的形态。它的存储层由内置的 RobustMQ Journal Engine 来实现，无需依赖外部组件。底层的存储介质是硬盘（硬盘、SSD、NVME 等）或云硬盘。

这种部署方式好处是简单、无额外依赖，系统复杂度和运维成本低，同时具备 Serverless 特性。架构如下：

![image](https://uploader.shimo.im/f/dgwor7moOrJevT6f.png!thumbnail?accessToken=eyJhbGciOiJIUzI1NiIsImtpZCI6ImRlZmF1bHQiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE3NDIzNTc4NTEsImZpbGVHVUlEIjoiRWUzMm1FbGFlZWhaejlBMiIsImlhdCI6MTc0MjM1NzU1MSwiaXNzIjoidXBsb2FkZXJfYWNjZXNzX3Jlc291cmNlIiwicGFhIjoiYWxsOmFsbDoiLCJ1c2VySWQiOjQxNTIyNzgwfQ.6xsFSqx8WnH7_y1NhfiSDDIgc-ayAwqNm6DzeNyV5kk)

如上图所示，独立部署架构由 Placement Center Cluster、Broker Cluster、Journal Engine Cluster 三部分组成，分别负责元数据和调度、协议功能实现、数据存储。

Journal Engine 抽象了Shard的概念，，来承接计算层所有协议（比如 MQTT、AMQP、Kafka、Rocketmq...）的消息数据的持久化存储。也就是说 RobustMQ 可以在不依赖任何外部系统的情况下，独立提供高性能的、稳定的服务。

在高内聚架构中，集群包含Placement Center、Broker、Journal Engine 三个组件，各司其职。不会有外部依赖，好处是精简架构，降低运维成本。

## 3. 共享存储架构
共享存储架构是指依赖外部现成的分布式存储组件来完成数据存储的 RobustMQ 部署形态。它的存储层依赖现有的各种远程存储（比如HDFS、S3、OSS、GCS、Azure Blob、COS、BOS、MinIO...）的完成。

这种部署方式的好处是在大规模流量下的成本相比独立部署方案会降低很多，适合在大流量、云架构部署场景下，对成本有强烈诉求的客户。架构如下：

![image](https://uploader.shimo.im/f/dgwor7moOrJevT6f.png!thumbnail?accessToken=eyJhbGciOiJIUzI1NiIsImtpZCI6ImRlZmF1bHQiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE3NDIzNTc4NTEsImZpbGVHVUlEIjoiRWUzMm1FbGFlZWhaejlBMiIsImlhdCI6MTc0MjM1NzU1MSwiaXNzIjoidXBsb2FkZXJfYWNjZXNzX3Jlc291cmNlIiwicGFhIjoiYWxsOmFsbDoiLCJ1c2VySWQiOjQxNTIyNzgwfQ.6xsFSqx8WnH7_y1NhfiSDDIgc-ayAwqNm6DzeNyV5kk)

如上图所示，共享部署架构由Placement Center Cluster、Broker Cluster和独立分布式存储组件三部分组成。在 RobustMQ 层面，只需要部署Placement Center Cluster 和 Broker Cluster 两个组件。Placement Center Cluster 负责元数据存储，Broker Cluster 通过 Storage Adapter 的交互，抽象 Shard 的概念，调用远程的来完成消息数据的持久化存储。

这种架构的核心在于消息数据的存储完全依赖远程的存储系统来完成，目前使用最多的是 Cloud Object Storage（比如 S3 等）。COS 的核心优势是成本低，因此当业务流量大、部署在公有云环境、对延时不是特别敏感、且对成本有强烈诉求时，这种架构的优势更明显。
