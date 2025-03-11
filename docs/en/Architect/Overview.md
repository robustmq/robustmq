# OverView
RobustMQ is a typical distributed layered architecture with separated computation, storage, and scheduling layers. It consists of four parts: the control layer (Placement Center), the computation layer (Multi-protocol computing layer), the storage adapter layer (Storage Adapter Layer), and the independent remote storage layer (Standalone storage engine). Each layer has the capability for rapid scaling, thereby enabling the entire system to possess complete Serverless capabilities.

![image](../../images/doc-image.png)

- Placement Center: The metadata storage and scheduling component of the RobustMQ cluster. It is responsible for cluster-related metadata storage, distribution, and scheduling tasks. For example, the online and offline status of cluster nodes, configuration storage/dissemination, etc.

- Multi-protocol computing layer: The computation layer Broker cluster of the RobustMQ cluster. It is responsible for the adaptation of various message protocols and the implementation of message-related functions. It also writes the received data into the storage layer through the Storage Adapter Layer.

- Storage Adapter Layer: This is the storage adapter component, which unifies various protocol MQ's Topic/Queue/Partition into Shards. It is also responsible for adapting to different storage components, such as local file storage, remote HDFS, object storage, self-developed storage components, etc., thereby persisting Shard data into different storage engines.

- Standalone storage engine: Refers to independent storage engines, such as cloud object storage (e.g., AWS S3), HDFS Cluster, Data Lake Cluster (Iceberg, Hudi, etc.). At the same time, RobustMQ's distributed, segmented storage service, similar to Apache BookKeeper, is the RobustMQ Journal Server. It is responsible for the high-performance, reliable storage of message data and has the ability to scale horizontally without perception.
