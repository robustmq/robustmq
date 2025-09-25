## Logo

![image](/images/robustmq-logo.png)

## Vision
To become the next-generation cloud-native and AI-native messaging infrastructure.

## Background & Motivation: Why Build RobustMQ

Traditional message queues face new challenges in the AI era: protocol fragmentation requiring multiple system maintenance, latency jitter unable to meet real-time AI demands, compute-storage coupling struggling with Serverless support, and high costs lacking elastic scaling.

RobustMQ was born specifically redesigned for the AI era and cloud-native environments.

---

## Core Features

**Design Philosophy**: AI-Ready, Cloud-Native, Multi-Protocol Unification, Compute-Storage Separation, Pluggable Storage

### 🦀 Rust High-Performance Core
- Zero-cost abstractions, memory safety, no GC pauses
- Tokio-based async runtime, supporting million-level concurrent connections
- Microsecond-level latency, meeting real-time AI application requirements

### 🔌 Multi-Protocol Unified Platform
- **MQTT** (1883/8083) - IoT devices, real-time communication
- **Kafka** (9092) - Big data stream processing, AI training
- **AMQP** (5672) - Enterprise integration, microservices
- One deployment, multi-protocol availability

### ☁️ Compute-Storage Separation Architecture
- **Broker Server**: Stateless protocol processing, Serverless support
- **Meta Service**: Raft-based high-availability metadata management
- **Journal Server**: Pluggable storage, supporting local files, S3, HDFS, etc.

### 💾 Pluggable Storage
- Memory storage: Microsecond latency, ultimate performance
- SSD storage: Millisecond latency, high-frequency access
- Object storage: Second-level latency, ultra-low cost
- WAL mechanism support, ensuring consistency

---

## Development Roadmap

- **2025 Q4**: MQTT protocol production-ready, release version 0.2.0
- **2026**: Enhance Kafka compatibility, support AI training data pipelines
- **Long-term Goal**: Become an Apache top-level project

---

## Core Advantages

- **🚀 Ultimate Performance**: Rust zero-cost abstractions, microsecond latency, zero GC pauses
- **🔌 Multi-Protocol Unification**: MQTT/Kafka/AMQP integration, avoiding system fragmentation
- **☁️ Compute-Storage Separation**: Stateless compute layer, supporting Serverless and elastic scaling
- **💾 Pluggable Storage**: Memory/SSD/Object storage intelligent tiering
- **🎯 AI-Native**: Optimized for AI workflows, supporting massive data and real-time inference
- **🌐 Cloud-Native**: Containerization, K8s Operator, visual management
- **🤝 Open Source Driven**: Apache 2.0 license, active global community

---

📌 **RobustMQ** — Next-generation cloud-native and AI-native messaging infrastructure

---

## 📖 Learn More

Want to dive deeper into RobustMQ's technical details and design philosophy? Read our comprehensive blog article:
**[Redefining Cloud-Native Message Queues with Rust](../Blogs/01.md)**

