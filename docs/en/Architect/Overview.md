## 1. Architecture
RobustMQ is a distributed layered architecture, It consists of four parts: metadata and scheduling layer (Placement Center), computing Layer (Multi-protocol computing layer), Storage Adapter layer (Storage Adapter Layer) and storage layer (storage layer). It is a typical computing, storage, scheduling layered architecture, each layer has a rapid expansion capacity, so that the whole system has a complete Serverless capabilities.

![image](https://uploader.shimo.im/f/EzvImtDnVLmiWMp1.png!thumbnail?accessToken=eyJhbGciOiJIUzI1NiIsImtpZCI6ImRlZmF1bHQiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE3NDIzNTc4NTEsImZpbGVHVUlEIjoiRWUzMm1FbGFlZWhaejlBMiIsImlhdCI6MTc0MjM1NzU1MSwiaXNzIjoidXBsb2FkZXJfYWNjZXNzX3Jlc291cmNlIiwicGFhIjoiYWxsOmFsbDoiLCJ1c2VySWQiOjQxNTIyNzgwfQ.6xsFSqx8WnH7_y1NhfiSDDIgc-ayAwqNm6DzeNyV5kk)

- Metadata and Scheduling Center: The metadata and scheduling layer is responsible for cluster metadata storage and scheduling. Main achievements:
Broker cluster metadata (Topic, Group, Queue, Partition, etc.) is stored and distributed.
Broker cluster control, scheduling, such as cluster nodes on and off line, configuration storage/distribution, etc.
Journal Engine consumes Offset information, etc.

- Multi-protocol computing layer: The protocol adaptation layer is responsible for the adaptation of various message protocols and the implementation of message-related functions. Code will have its own Broker for each protocol, e.g. (RobustMQ MQTT, RobustMQ AMQP, etc.), which will functionally match the mainstream implementations of each protocol. The protocol Adapter Layer is completely stateless, and the Storage of data depends on the Placement Center and Storage Adapter Layer. The Placement Center is responsible for the Storage of metadata, and the Storage Adapter Layer is responsible for the storage of message data.

- Storage Adapter Layer: The storage adapter layer is mainly responsible for the RobustMQ message data's high-performance and reliable persistent storage. The storage adaptation layer abstracts Topic/Queue/Partition in various MQ protocols into Shard. At the same time, it ADAPTS to different underlying storage engines (such as local file storage, remote HDFS, object storage, self-developed storage components, etc.), so as to realize the plug-in and pluggable storage adaptation layer.

- Storage layer: refers to concrete, individual storage engines such as hard drives (cloud drives, mechanical drives), cloud object stores (e.g. AWS S3), HDFS Cluster, Data Lake Cluster (iceberg, hudi, etc.), and so on. The RobustMQ storage layer supports both standalone and shared storage architectures.

  - Independent storage: The storage layer is implemented by the built-in RobustMQ Journal Engine without relying on external components. The underlying storage medium is a hard drive (hard disk, SSD, NVME, etc.) or a cloud drive. The benefits of this deployment method are simple, no additional dependencies, low system complexity and operation and maintenance costs, and Serverless features.

  - Shared storage: The storage layer relies on various existing remote storage (e.g. HDFS, S3, OSS, GCS, Azure Blob, COS, BOS, MinIO...) The completion of. The advantage of this deployment method is that the cost under large-scale traffic will be much lower than the independent deployment scheme, which is suitable for customers with strong demands for cost under large traffic and cloud architecture deployment scenarios.

## 2. Standalone deployment architecture [Default]

The standalone deployment architecture refers to the form in which RobustMQ is deployed without relying on any external components. Its storage layer is implemented by the built-in RobustMQ Journal Engine without relying on external components. The underlying storage medium is a hard drive (hard disk, SSD, NVME, etc.) or a cloud drive.

The benefits of this deployment method are simple, no additional dependencies, low system complexity and operation and maintenance costs, and Serverless features. The architecture is as follows:

![image](https://uploader.shimo.im/f/dgwor7moOrJevT6f.png!thumbnail?accessToken=eyJhbGciOiJIUzI1NiIsImtpZCI6ImRlZmF1bHQiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE3NDIzNTc4NTEsImZpbGVHVUlEIjoiRWUzMm1FbGFlZWhaejlBMiIsImlhdCI6MTc0MjM1NzU1MSwiaXNzIjoidXBsb2FkZXJfYWNjZXNzX3Jlc291cmNlIiwicGFhIjoiYWxsOmFsbDoiLCJ1c2VySWQiOjQxNTIyNzgwfQ.6xsFSqx8WnH7_y1NhfiSDDIgc-ayAwqNm6DzeNyV5kk)

As shown in the figure above, the independent deployment architecture consists of Placement Center Cluster, Broker Cluster, and Journal Engine Cluster, which are responsible for metadata and scheduling, protocol function implementation, and data storage, respectively.

Journal Engine abstracts the concept of Shard, to carry all the protocols of the compute layer (such as MQTT, AMQP, Kafka, Rocketmq...). For persistent storage of message data. This means that RobustMQ provides a robust, high-performance service without any external system.

In the high cohesion architecture, the cluster contains three components: Placement Center, Broker and Journal Engine, each performing its own duties. There are no external dependencies, and the benefits are a leaner architecture and lower operational costs.

## 3. Shared memory architecture
The shared storage architecture refers to RobustMQ, which relies on external off-the-shelf distributed storage components for data storage. Its storage layer relies on various existing remote storage (such as HDFS, S3, OSS, GCS, Azure Blob, COS, BOS, MinIO...). The completion of.

The advantage of this deployment method is that the cost under large-scale traffic will be much lower than the independent deployment scheme, which is suitable for customers with strong demands for cost under large traffic and cloud architecture deployment scenarios. The architecture is as follows:


![image](https://uploader.shimo.im/f/dgwor7moOrJevT6f.png!thumbnail?accessToken=eyJhbGciOiJIUzI1NiIsImtpZCI6ImRlZmF1bHQiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE3NDIzNTc4NTEsImZpbGVHVUlEIjoiRWUzMm1FbGFlZWhaejlBMiIsImlhdCI6MTc0MjM1NzU1MSwiaXNzIjoidXBsb2FkZXJfYWNjZXNzX3Jlc291cmNlIiwicGFhIjoiYWxsOmFsbDoiLCJ1c2VySWQiOjQxNTIyNzgwfQ.6xsFSqx8WnH7_y1NhfiSDDIgc-ayAwqNm6DzeNyV5kk)

As shown in the figure above, the shared deployment architecture consists of three parts: Placement Center Cluster, Broker Cluster and independent distributed storage component. At the RobustMQ level, only two components, Placement Center Cluster and Broker Cluster, need to be deployed. Placement Center Cluster is responsible for metadata Storage. Broker Cluster abstracts the concept of Shard through the interaction of Storage Adapter, and calls remote shard to complete the persistent storage of message data.

The core of this architecture is that the Storage of message data completely depends on the remote storage system to complete, and the most commonly used is Cloud Object Storage (such as S3, etc.). The core advantage of COS is low cost, so when the business flow is large, deployed in the public cloud environment, not particularly sensitive to delay, and there is a strong demand for cost, the advantages of this architecture are more obvious.
