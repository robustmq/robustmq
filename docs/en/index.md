---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: "RobustMQ"
  text: "Next-generation high-performance cloud-native converged message queue."
  tagline: >
   RobustMQ is an open-source middleware message queue project that is 100% implemented in Rust. Its goal is to create a next-generation high-performance cloud-native converged message queue that is compatible with various mainstream message queue protocols and has complete Serverless capabilities based on Rust.
    <div class="badges">
      <img alt="Latest Release" src="https://img.shields.io/github/v/release/robustmq/robustmq?style=flat">
      <img alt="License" src="https://img.shields.io/github/license/robustmq/robustmq?style=flat">
      <img alt="GitHub issues" src="https://img.shields.io/github/issues/robustmq/robustmq?style=flat">
      <img alt="GitHub stars" src="https://img.shields.io/github/stars/robustmq/robustmq?style=flat">
    </div>

  actions:
    - theme: brand
      text: Get Started
      link: /OverView/What-is-RobustMQ
    # - theme: alt
    #   text: API Examples
    #   link: /api-examples
  image:
    src: /logo-large.jpg
    alt: RobustMQ

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
    details: Rich functions support sequential messages, dead message messages, transaction messages, idempotent messages, delay messages and other rich message queue functions.
---
<style>

.badges {
  display: flex;
  justify-content: left;  /* 水平居中 */
  gap: 10px;                /* 徽章之间的间距 */
  margin-top: 10px;
}

.badges img {
  height: 24px;   /* 调整徽章大小 */
}

.clip{
  font-size:50px !important;
}
.text[data-v-72cc4481]
{
  font-size:20px !important;
}
.tagline
{
  font-size:20px !important;
}
.VPButton.brand
{
  background-color:purple !important;
}
:root {
  --vp-home-hero-name-color: transparent !important;
  --vp-home-hero-name-background: purple !important;

  --vp-home-hero-image-background-image: linear-gradient(-45deg, #bd34fe 50%, #bd34fe 50%) !important;
  --vp-home-hero-image-filter: blur(44px) !important;
}

@media (min-width: 640px) {
  :root {
    --vp-home-hero-image-filter: blur(56px) !important;
    --vp-home-hero-name-font-size: 20px !important;
  }
}

@media (min-width: 960px) {
  :root {
    --vp-home-hero-image-filter: blur(68px) !important;
  }
  .name{
    font-size:20px !important;
  }
}
.VPImage {
    border-radius: 24% !important;
    opacity: 0.8 !important;
    transition: opacity 0.3s ease !important;
}
</style>
