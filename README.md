<p align="center">
  <picture>
    <img alt="RobustMQ Logo" src="docs/images/robustmq-logo.png" width="300">
  </picture>
</p>
 <h3 align="center">
    Next generation cloud-native converged message queue.
</h3>

## Introduction
RobustMQ is a next-generation high-performance cloud-native converged message queue. The goal is to implement a message queue based on Rust that can be compatible with multiple mainstream message queue protocols and has complete Serveless architecture. 

> Tips: We are still young, please give us some time to grow up. We expect RobustMQ to become the next Apache top-level project in the message queue space.

## Features
- 100% Rust: A message queuing kernel implemented entirely in Rust.
- Multi-protocol: Support MQTT 3.1/3.1.1/5.0, AMQP, RocketMQ Remoting/GRPC, Kafka Protocol, OpenMessing, JNS, SQS and other mainstream message protocols.
- Layered architecture: computing, storage, scheduling independent three-tier architecture, each layer has the ability of cluster deployment, rapid horizontal scaling capacity.
- Plug-in storage: Standalone plug-in storage layer implementation, you can choose the appropriate storage layer according to your needs. It is compatible with traditional and cloud-native architectures, and supports cloud and IDC deployment patterns.
- High cohesion architecture: It provides built-in metadata storage components, distributed Journal storage services, and has the ability to deploy quickly, easily and cohesively.
- Rich functions: support sequential messages, dead message messages, transaction messages, idempotent messages, delay messages and other rich message queue functions.

## Architecture
RobustMQ is a typical distributed layered architecture with separate computing layer, storage layer, and scheduling layer. By the control layer (Placement Center), computing Layer (Multi-protocol computing layer), Storage Adapter layer (Storage Adapter Layer), independent remote storage layer (Standalone storage) engine) consists of four parts. Each layer has the ability to quickly scale up and down, so as to achieve a complete Serverless capability of the whole system.

![image](docs/images/robustmq-architecture.png)

- Plagement Center
  
  The metadata storage and scheduling component of the RobustMQ cluster. It is responsible for cluster-related metadata storage, distribution, scheduling, and so on. Such as cluster node uplinking, configuration storage/distribution, and so on.

- Multi-protocol computing layer

  Broker Cluster, the computing layer of RobustMQ cluster. It is responsible for the adaptation of various messaging protocols and the implementation of message-related functions. The received data is written to the Storage Layer through the Storage Adapter Layer.

- Storage Adapter Layer
  
  Storage adapter layer component, its role to a variety of protocols MQ Topic/Queue/Partition unified abstract Shard. It is also responsible for the adaptation of different storage components, such as local file storage, remote HDFS, object storage, self-developed storage components, and so on. Thus, Shard data can be persistently stored to different storage engines.

- Standalone storage engine
  refers to a standalone storage engine, such as cloud object storage (e.g. AWS S3), HDFS Cluster, Data Lake Cluster (iceberg, hudi, etc.). The RobustMQ is similar to the RobustMQ Journal Server, Apache Bookeeper's distributed, segmented storage service. It is responsible for reliable storage of high-performance message data, and has the ability of rapid horizontal and horizontal expansion without perception.

Click [RobustMQ Tutorial](http://www.robustmq.com/docs/robustmq-tutorial-cn/%e7%ae%80%e4%bb%8b/%e4%bb%80%e4%b9%88%e6%98%af-robustmq/) for detailed system architecture design.

## Planning
RobustMQ has long wanted to support multi-protocol and have a full Serverless architecture. At the same time, we hope to keep the architecture simple while adapting to different deployment scenarios and deployment requirements. To achieve lower deployment, operation and maintenance, and use costs. So there are several stages in the development perspective:

In the first phase, the basic framework of the cluster (such as metadata storage service, storage adaptation layer, bring your own storage layer, etc.) and the functions related to MQTT protocol will be developed. The goal is to complete the RobustMQ architecture and adapt it to the MQTT protocol, and achieve production availability on the MQTT protocol.

Welcome to our development plan.
- [《RobustMQ 2024 Development Plan》](https://github.com/robustmq/robustmq/wiki/RobustMQ-2024-Development-Plan)
- [《RobustMQ Long‐Term Evolution Initiative》](https://github.com/robustmq/robustmq/wiki/RobustMQ-Long%E2%80%90Term-Evolution-Initiative)

> We are still young and development plans can change quickly.

## RobustMQ MQTT
1. Cluster deployment, horizontal unaware expansion.
2. A single machine can carry millions of connections.
3. Support MQTT3.1/3.1.1/5.0 protocol.
4. Supports TCP, SSL, WebSocket, WebSockets protocols.
5. Supports persistent Session storage.
6. Support reserved messages, testament messages, shared subscriptions, etc
7. For the full features, see the [RobustMQ MQTT documentation](docs/en/mqtt-feature.md)

## Get Started
To start the order, you need to start the Placement Center first, and then start the MQTT Broker.
### Binary packages run
#### Stand-alone mode
1. Download .tar.gz
```
$ tar -xzvf robustmq-v0.0.1-release.tar.gz
$ cd robustmq-v0.0.1-release
```

2. Start Placement Center
```
$ bin/robustctl placement-center start
```

3. Start MQTT Broker
```
$ bin/robustctl broker-mqtt start
```

#### Cluster mode
1. Download .tar.gz
```
$ tar -xzvf robustmq-v0.0.1-release.tar.gz
$ cd robustmq-v0.0.1-release
```
2. Start Placement Center
```
$ bin/robustctl placement-center start config/cluster/placement-center/node-1.toml
$ bin/robustctl placement-center start config/cluster/placement-center/node-2.toml
$ bin/robustctl placement-center start config/cluster/placement-center/node-3.toml
```

3. Start MQTT Broker
```
$ bin/robustctl broker-mqtt start config/cluster/mqtt-server/node-1.toml
$ bin/robustctl broker-mqtt start config/cluster/mqtt-server/node-2.toml
$ bin/robustctl broker-mqtt start config/cluster/mqtt-server/node-3.toml
```

### Cargo runs
#### Standalone mode
1. Run standalone by placement-center
```
cargo run --package cmd --bin placement-center -- --conf=config/placement-center.toml
```

2. Run standalone by mqtt-server
```
cargo run --package cmd --bin mqtt-server -- --conf=config/mqtt-server.toml
```

#### Cluster mode
1. Run cluster by placement-center
```
cargo run --package cmd --bin placement-center -- --conf=config/cluster/placement-center/node-1.toml
cargo run --package cmd --bin placement-center -- --conf=config/cluster/placement-center/node-2.toml
cargo run --package cmd --bin placement-center -- --conf=config/cluster/placement-center/node-3.toml
```

2. Run cluster by mqtt-server
```
cargo run --package cmd --bin mqtt-server -- --conf=config/cluster/mqtt-server/node-1.toml
cargo run --package cmd --bin mqtt-server -- --conf=config/cluster/mqtt-server/node-2.toml
cargo run --package cmd --bin mqtt-server -- --conf=config/cluster/mqtt-server/node-3.toml
```

## Development
### Run all test cases
```
make test
```

### Packaging
```
make release
```

## Tests
### MQTT functional tests 
MQTT functionality was tested through the MQTTX tool. MQTTX quick start: https://mqttx.app/zh/docs/get-started.

## Multiple protocols
### RobustMQ AMQP
In the planning

### RobustMQ Kafka
In the planning

### RobustMQ RocketMQ
In the planning

### RobustMQ ...
In the planning

## Contributing
Please refer to contribution [guidelines](docs/en/contribution.md) for more information.

## License
RobustMQ uses the Apache 2.0 license to strike a balance between open contributions and allowing you to use the software however you want



