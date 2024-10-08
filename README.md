<p align="center">
  <picture>
    <img alt="RobustMQ Logo" src="docs/images/robustmq-logo.png" width="300">
  </picture>
</p>
 <h3 align="center">
    Next generation cloud-native converged message queue.
</h3>

##  🚀 Introduction
RobustMQ is a next-generation high-performance cloud-native converged message queue. The goal is to implement a message queue based on Rust that can be compatible with multiple mainstream message queue protocols and has complete Serveless architecture. 

Official documentation:
- [《English Version》](http://www.robustmq.com/docs/robustmq-tutorial/introduction/)
- [《中文版》](http://www.robustmq.com/docs/robustmq-tutorial-cn/%e7%ae%80%e4%bb%8b/%e4%bb%80%e4%b9%88%e6%98%af-robustmq/)

> Tips: We are still young, please give us some time to grow up. We expect RobustMQ to become the next Apache top-level project in the message queue space.

## 💡 Features
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

## Planning
RobustMQ has long wanted to support multi-protocol and have a full Serverless architecture. At the same time, we hope to keep the architecture simple while adapting to different deployment scenarios and deployment requirements. To achieve lower deployment, operation and maintenance, and use costs. So there are several stages in the development perspective:

In the first phase, the basic framework of the cluster (such as metadata storage service, storage adaptation layer, bring your own storage layer, etc.) and the functions related to MQTT protocol will be developed. The goal is to complete the RobustMQ architecture and adapt it to the MQTT protocol, and achieve production availability on the MQTT protocol.

Welcome to our development plan.
- [《RobustMQ 2024 Development Plan》](https://github.com/robustmq/robustmq/wiki/RobustMQ-2024-Development-Plan)
- [《RobustMQ Long‐Term Evolution Initiative》](https://github.com/robustmq/robustmq/wiki/RobustMQ-Long%E2%80%90Term-Evolution-Initiative)

> We are still young and development plans can change quickly.

## RobustMQ MQTT
RobustMQ MQTT is RobustMQ's complete implementation of the MQTT protocol. The goal is to build a high-performance, full-featured message queuing MQTT product in Rust that can be deployed in clusters. The final target for the feature is EMQX Enterprise Edition.

### Features
1. **Cluster deployment**: A single cluster supports thousands of Broker nodes, supporting unaware smooth horizontal scaling capabilities.
2. **Full protocol support**: All features of MQTT3.1, 3.1.1, 5.0 protocols are supported
3. **High performance**: A single machine supports millions of connections and high concurrent message throughput.
4. **Multiple communication protocols**: Support TCP, TCP SSL, WebSocket, WebSocket SSL, QUIC, HTTP and other access methods.
5. **Plug-in storage**: Support offline messages, support a variety of message persistence storage engines.
6. **Fully functional**: It supports basic functions such as testamential messages and reserved messages, as well as all features of EMQX Broker Enterprise Edition. For the full features, see the [RobustMQ MQTT documentation](docs/en/mqtt-feature.md)

### Get Started
To start the order, you need to start the Placement Center first, and then start the MQTT Broker.
#### Binary packages run
##### Download .tar.gz
```
$ wget https://github.com/robustmq/robustmq/releases/download/v0.1.0-beta/robustmq-apple-mac-arm64-0.1.0-beta.tar.gz
$ tar -xzvf robustmq-apple-mac-arm64-0.1.0-beta.tar.gz
$ cd robustmq-apple-mac-arm64-0.1.0-beta
```

##### Start the Placement Center and MQTT Server for a node
```
$ bin/robust-server placement-center start config/placement-center.toml
$ bin/robust-server mqtt-server start config/mqtt-server.toml
```

##### Start the Placement Center and MQTT Server for 3 nodes
```
# Start Placement Center
$ bin/robust-server placement-center start example/mqtt-cluster/placement-center/node-1.toml
$ bin/robust-server placement-center start example/mqtt-cluster/placement-center/node-2.toml
$ bin/robust-server placement-center start cexample/mqtt-cluster/placement-center/node-3.toml

# Start MQTT Broker
$ bin/robust-server mqtt-server start example/mqtt-cluster/mqtt-server/node-1.toml
$ bin/robust-server mqtt-server start example/mqtt-cluster/mqtt-server/node-2.toml
$ bin/robust-server mqtt-server start example/mqtt-cluster/mqtt-server/node-3.toml
```

#### Cargo runs
##### Quickly launch the sample cluster
```
$ git clone https://github.com/robustmq/robustmq.git
$ cd roubustmq

# start cluster
$ sh example/mqtt-cluster/start.sh

# stop cluster
$ sh example/mqtt-cluster/stop.sh
```

##### Start the Placement Center and MQTT Server for a node
```
$ cargo run --package cmd --bin placement-center -- --conf=config/placement-center.toml
$ cargo run --package cmd --bin mqtt-server -- --conf=config/mqtt-server.toml
```

##### Start the Placement Center and MQTT Server for 3 node
```
# Start Placement Center
$cargo run --package cmd --bin placement-center -- --conf=example/mqtt-cluster/placement-center/node-1.toml
$cargo run --package cmd --bin placement-center -- --conf=example/mqtt-cluster/placement-center/node-2.toml
$cargo run --package cmd --bin placement-center -- --conf=example/mqtt-cluster/placement-center/node-3.toml

# Start MQTT Broker
cargo run --package cmd --bin mqtt-server -- --conf=example/mqtt-cluster/mqtt-server/node-1.toml
cargo run --package cmd --bin mqtt-server -- --conf=example/mqtt-cluster/mqtt-server/node-2.toml
cargo run --package cmd --bin mqtt-server -- --conf=example/mqtt-cluster/mqtt-server/node-3.toml
```


#### Run all test cases
You need to install the cargo-nextes command first. Please refer to documentation[《Integration testing》](http://www.robustmq.com/docs/robustmq-tutorial-cn/%e7%b3%bb%e7%bb%9f%e6%9e%b6%e6%9e%84/%e6%b5%8b%e8%af%95%e7%94%a8%e4%be%8b/)
```
make test
```

#### Packaging
Follow the "make help" prompts to build packages for different platforms

```
FWR3KG21WF:robustmq bytedance$ make help

Usage:
  make <target>

Build Mac Release
  build-mac-release               Build mac version robustmq.

Build Linux Release
  build-linux-release             Build linux version robustmq.

Build Win Release
  build-win-release               Build win version robustmq.

Build Arm Release
  build-arm-release               Build arm version robustmq.
  test                            Integration testing for Robustmq
  clean                           Clean the project.
  help                            Display help messages.
```

####  MQTT functional tests 
MQTT functionality was tested through the MQTTX tool. MQTTX quick start: https://mqttx.app/zh/docs/get-started.

## RobustMQ AMQP
In the planning

## RobustMQ Kafka
In the planning

## RobustMQ RocketMQ
In the planning

## RobustMQ ...
In the planning

## Contributing

### Contribution Guidelines
Please refer to [contribution guidelines](http://www.robustmq.com/docs/robustmq-tutorial-cn/%e8%b4%a1%e7%8c%ae%e6%8c%87%e5%8d%97/) for more information.

### Contact us
- Slack
Join [RobustMQ Slack](https://join.slack.com/t/robustmq/shared_invite/zt-2r7sccx50-DIrAlYOETp3xhsX1Qsr79A)

- Wechat Group
If you're interested in contributing to this project or discussing development topics, scan the QR Code to join our WeChat group for real-time discussions and collaboration.
<div align="center"> 
  <img src="docs/images/WechatGroup.png" alt="WeChat QR Code" width=200 />
</div>

- Personal wechat
Wechat group QR code will be updated regularly. If the QR code expires, the developer's personal wechat can be added.
<div align="center"> 
  <img src="docs/images/wechat.jpg" alt="WeChat QR Code" width=200 />
</div>

## License
RobustMQ uses the Apache 2.0 license to strike a balance between open contributions and allowing you to use the software however you want



