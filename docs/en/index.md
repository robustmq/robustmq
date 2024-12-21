---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: "RobustMQ"
  text: "Next-generation high-performance cloud-native converged message queue."
  tagline: RobustMQ is an open-source project in the middleware message queue space implemented 100% in Rust. Its goal is to build a next-generation high-performance cloud-native converged message queue based on Rust that is compatible with a variety of mainstream message queue protocols and has full serverless capabilities.
  actions:
    - theme: brand
      text: Get Started
      link: /en/Introduction/What-is-RobustMQ
    # - theme: alt
    #   text: API Examples
    #   link: /api-examples

features:
  - title: 100% Rust
    details: A message queuing kernel implemented entirely in Rust, which is the amazing language to build software with stunning performance, reliability and productivity.
  - title: Multi-protocol
    details: Support MQTT 3.1/3.1.1/5.0, AMQP, RocketMQ Remoting/GRPC, Kafka Protocol, OpenMessing, JNS, SQS and other mainstream message protocols.
  - title: Layered architecture
    details: Three-tier independent architecture consists of Computing, Storage and Scheduling. Each layer has the ability of cluster deployment and rapid horizontal scaling capacity.
  - title: Plug-in storage
    details: With standalone storage plug-in implementation, you can choose the best plug-in on demand, compatible with traditional on-premise and new cloud-native deployment.
  - title: High cohesion
    details: It provides built-in metadata storage components and distributed journal storage services. All of these ones could be deployed quickly, easily and cohesively.
  - title: Rich functions
    details: support sequential messages, dead message messages, transaction messages, idempotent messages, delay messages and other rich message queue functions.
---
