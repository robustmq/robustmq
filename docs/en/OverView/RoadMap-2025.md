# RoadMap

## 2024

2024: Improve RobustMQ MQTT and enhance the basic capabilities of RobustMQ.

- Enhance the functionality of RobustMQ MQTT.
- Improve the stability and performance of the metadata service, Meta Service.
- Conduct stress tests on RobustMQ MQTT and optimize performance.
- Conduct stress tests on Meta Service and optimize performance.
- Improve test cases for RobustMQ MQTT and Meta Service.
- Enhance the official website and technical documentation of RobustMQ.
- Look for friends interested in Rust or middleware to build a great foundational software together.

## 2025 H1

### Meta Service

**Goal:** Stable operation of Meta Service in cluster mode, performance testing, and completion of monitoring metrics.

#### Features
1. Observability
   - Add monitoring metrics for Meta Service.

#### Improvements
1. Improve cluster capabilities to ensure stable operation in cluster mode.
   - Ensure stable operation of GRPC interfaces in cluster mode.
   - Ensure stable operation of GRPC interfaces during Leader switching.
   - During Leader switching, pause the Controller on non-Leader nodes and run the Controller on the Leader node.
2. Conduct performance testing on the Meta Service service.

### Robust MQTT

**Goal:** Complete the first phase, ensuring stable operation of RobustMQ MQTT in cluster mode. Optimize existing functional code, improve test cases, and add some features.

#### Features
MQTT
1. Offline messages
   - Support for Journal
   - Support for MySQL
   - Support for Redis
2. Data integration
   - Implementation of a data integration framework
   - Support for Kafka
3. Schema
   - Framework implementation
4. Subscription
   - Optimize subscription process to automatically discard messages without subscriptions
   - Automatic subscription
5. Observability
   - Improve Metrics
   - Integrate with otel
6. Develop a throttling module (for request count, connection count, traffic, etc.)
7. Other
   - Address connection jitter

Dashboard
1. Complete the framework for the Dashboard.
2. Implement linkage between the Dashboard and MQTT Broker's Admin-related interfaces.

#### Testing
1. Improve unit tests for publishing-related code.
2. Improve unit tests for subscription-related code.

#### Improvements
Cluster mode:
1. Ensure stable operation of Robust MQTT in multi-node mode.
2. Support Docker multi-node operation in cluster mode.
3. Conduct performance testing on MQTT TCP and improve it.
4. Complete the Auth module.

CLI:
1. Improve the unified format of command-line output.
2. Enhance MQTT-related Admin interfaces and capabilities.

### Journal Engine

**Goal:** Stable operation of Journal Engine in single-node mode.

#### Features
1. Stable version of the first version of the core functionality of Journal Engine.
2. Complete the core processes of single-node mode.
   1. Data read and write
   2. Add, delete, modify, and query Shard/Segment
   3. Segment Status transition
3. Improve Journal Engine Client.
   1. Support for core data read and write capabilities.
4. Stable single-node operation.
5. Design a multi-replica ISR mode.

### Storage Adapter

**Goal:** Support for Journal Engine, MySQL, Redis, and explore support for remote object storage (e.g., Aws S3).

#### Features
1. Support for Journal.
2. Support for MySQL.
3. Support for Redis.
4. Support for Aws S3.
