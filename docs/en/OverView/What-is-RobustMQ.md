## 一、RobustMQ
### 1.1 Logo
![image](https://uploader.shimo.im/f/edIiOkJ79eEBngLX.png!thumbnail?accessToken=eyJhbGciOiJIUzI1NiIsImtpZCI6ImRlZmF1bHQiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE3NDIzNTc4NTEsImZpbGVHVUlEIjoiRWUzMm1FbGFlZWhaejlBMiIsImlhdCI6MTc0MjM1NzU1MSwiaXNzIjoidXBsb2FkZXJfYWNjZXNzX3Jlc291cmNlIiwicGFhIjoiYWxsOmFsbDoiLCJ1c2VySWQiOjQxNTIyNzgwfQ.6xsFSqx8WnH7_y1NhfiSDDIgc-ayAwqNm6DzeNyV5kk)

### 1.2 Definition
Next-generation high performance cloud-native converged message queues.

### 1.3 Vision
Use Rust to build a high-performance, stable, fully functional message queue that is compatible with a variety of mainstream message queue protocols, and has complete Serverless architecture.

### 1.4 Features
- 100% Rust: A message queuing kernel implemented entirely in Rust.
- Multi-protocol: Support MQTT 3.1/3.1.1/5.0, AMQP, RocketMQ Remoting/GRPC, Kafka Protocol, OpenMessing, JNS, SQS and other mainstream message protocols.
- Layered architecture: computing, storage, scheduling independent three-tier architecture, each layer has the ability of cluster deployment, rapid horizontal scaling capacity.
- Plug-in storage: Standalone plug-in storage layer implementation, you can choose the appropriate storage layer according to your needs. It is compatible with traditional and cloud-native architectures, and supports cloud and IDC deployment patterns.
- High cohesion architecture: It provides built-in metadata storage components, distributed Journal storage services, and has the ability to deploy quickly, easily and cohesively.
- Rich functions: support sequential messages, dead message messages, transaction messages, idempotent messages, delay messages and other rich message queue functions.

### 1.5 Why

Explore the application of Rust in the field of message queue, and build a message queue product with better performance, functionality and stability based on Rust.
- Build a message queue that is compatible with all major message queuing protocols to reduce usage, learning, and maintenance costs.
- Create a full Serverless message queue to reduce resource costs.
- Build a message queue that can meet the needs of various scenarios.


## 二、Long term planning

RobustMQ vision is multi-protocol support and a fully Serverless architecture. The overall development effort is heavy, so it is divided into several phases:

- Phase 1: Development of the cluster infrastructure (such as metadata storage service, storage adaptation layer, self-contained storage layer, etc.).

- Phase 2: Complete the development of MQTT protocol-related functions, with the goal of building RobustMQ, verifying its feasibility, and adapting the MQTT protocol. Functionally align EMQX and finally implement RobustMQ MQTT for production availability.

- Phase 3: We will start development of Kafka protocol-related features, align them functionally, and make Kafka RobustMQ production-ready for Kafka.

- Phase 4: initiates development of protocols such as AMQP/RocketMQ.

>  We are currently in the second phase, with the goal of completing the RobustMQ MQTT and making the RobustMQ MQTT production-ready by mid-2025.
