# RoadMap
## 2024
In 2024: Improve RobustMQ MQTT and enhance the basic capabilities of RobustMQ.
- Improve the functionality of RobustMQ MQTT.
- Enhance the stability and performance of the metadata service Placement Center.
- Conduct load testing on RobustMQ MQTT and optimize its performance.
- Conduct load testing on Placement Center and optimize its performance.
- Improve the test cases for RobustMQ MQTT and Placement Center.
- Improve the official website and technical documentation of RobustMQ.
- Recruit partners interested in Rust or middleware to jointly build a remarkable basic software.

## 2025 H1
### Overview

### Details
#### Placement Center
> Goal: The cluster mode of Placement Center runs stably, conduct performance load testing, and improve monitoring metrics.

Detailed tasks:
1. Improve cluster capabilities to enable stable operation in cluster mode.
   1. In cluster mode, the interfaces provided by GRPC run stably.
   2. In cluster mode, when the leader switches, the interfaces provided by GRPC run stably.
   3. In cluster mode, when the leader switches, the Controller of non - leader nodes pauses, and the Controller of the leader node runs.
2. Conduct performance load testing on the Placement Center service.
3. Add monitoring metrics for Placement Center.

#### Robust MQTT
> Goal: Complete the first phase, ensuring the stable operation of RobustMQ MQTT in cluster mode. Optimize the current functional code, improve test cases, and add some functions.

Cluster mode:
1. Ensure the stable operation of Robust MQTT in multi - node mode.
2. Run in multi - node mode in Docker in cluster mode.
3. Conduct performance load testing on MQTT TCP and improve it.

Test cases:
1. Improve unit tests for code related to publishing.
2. Improve unit tests for code related to subscribing.

Functions:
1. Optimize the subscription process, automatically discard messages without subscriptions, and support offline messages.
2. Support Journals for offline messages.
3. Support MySQL for offline messages.
4. Support Redis for offline messages.
5. Implement the MQTT data integration framework.
6. Support Kafka in MQTT data integration.
7. Implement the MQTT - supported Schema framework.
8. Enable auto - subscription.
9. Handle connection jitter.
10. Improve Metrics.
11. Integrate with opentelemetry.
12. Develop a rate - limiting module (for request count, connection count, traffic, etc.).
13. Improve the Auth module.

Cli (Command - line):
1. Standardize the output format of command - line content.
2. Improve MQTT - related Admin interfaces and capabilities.

Dashboard:
1. Build the framework of the Dashboard.
2. Link the Dashboard with the Admin - related interfaces of the MQTT Broker.

#### Journal Engine
> Goal: Ensure the stable operation of Journal Engine in single - node mode.

Functions:
1. Develop the stable version code of the core functions of the first version of Journal Engine.
2. Improve the core processes in single - machine mode.
   1. Data reading and writing.
   2. Operations (create, read, update, delete) on Shard/Segment.
   3. Conversion of Segment Status.
3. Improve the Journal Engine Client.
   1. Support core capabilities of data reading and writing.
4. Ensure stable operation in single - machine mode.
5. Design the ISR mode for multiple replicas.

#### Storage Adapter
> Goal: Support Journal Engine, MySQL, Redis, and explore support for remote object storage (such as Aws S3).

Functions:
1. Support Journal.
2. Support MySQL.
3. Support Redis.
4. Support Aws S3.

## 2025 H2
