# Placement-Center
## Overview
Placement Center (hereinafter referred to as PC) is a metadata storage component implemented based on Raft and RocksDB. From a storage perspective, it is a high-performance distributed KV storage service. By definition, it is the metadata storage, control, and scheduling center for Robust Broker and Robust Journal clusters. The architecture is as follows:
![image](../../images/doc-image1.png)

- PC is a cluster composed of the Raft protocol, completing Leader elections, switches, data consistency, etc., according to the definition of the Raft protocol.
- Placement Center Node (hereinafter referred to as PCN) is the running node of the PC cluster. Upon startup, it elects Leaders and Followers according to the Raft protocol, and automatic Leader switching will occur when a Node fails.
- The storage layer of a single PCN is implemented based on RocksDB. It combines with the Raft protocol to achieve distributed and reliable data storage.
- Data writes in PC are completed by the Leader, and reads can be completed by both the Leader and Followers.
- After data is written to the Leader node, it is first written to the local RocksDB for persistent storage and then distributed to multiple Followers according to the Raft protocol.
- Data reads can be performed from Leader or Follower nodes, directly reading relevant data from local caches and RocksDB.
- Placement Center (PC) supports two protocols, GRPC and HTTP. GRPC is the default data stream protocol responsible for data writing and reading. The HTTP protocol is mainly used for operations and acquisition related to cluster management, cluster status, and monitoring information.
- The Leader node runs Controller Threads related to control and scheduling, which monitor Robust Broker and Robust Journal clusters and execute necessary scheduling actions.
- Placement Center (PC) supports both single-node operation and cluster operation modes. The cluster mode is recommended to have a minimum of 3 nodes, and the number of nodes should be odd.
