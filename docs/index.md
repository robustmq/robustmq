---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: "RobustMQ"
  text: "下一代高性能云原生融合消息队列"
  tagline: RobustMQ 是一个 100% 用 Rust 实现的中间件消息队列领域的开源项目。它的的目标是基于Rust 打造兼容多种主流消息队列协议、具备完整 Serverless 能力的下一代高性能云原生融合型消息队列。
  actions:
    - theme: brand
      text: Get Started
      link: /zh/OverView/What-is-RobustMQ
    # - theme: alt
    #   text: API Examples
    #   link: /api-examples
  image:
    src: /logo-large.jpg
    alt: RobustMQ

features:
  - title: 100% Rust
    details: 完全用 Rust 实现的消息队列内核，这是构建具有惊人性能、可靠性和生产力的软件的神奇语言。
  - title: 多协议
    details: 支持 MQTT 3.1/3.1.1/5.0、AMQP、RocketMQ Remoting/GRPC、Kafka 协议、OpenMessing、JNS、SQS 等主流消息协议。
  - title: 分层架构
    details: 三层独立架构，包括 Computing、Storage 和 Scheduling。每一层都具备集群部署能力和快速水平扩容能力。
  - title: 插件存储
    details: 通过独立存储插件实施，您可以按需选择最佳插件，与传统本地部署和新的云原生部署兼容。
  - title: 高内聚力
    details: 它提供内置的元数据存储组件和分布式日志存储服务。所有这些都可以快速、轻松和有凝聚力地部署。
  - title: 功能丰富
    details: 功能丰富：支持顺序消息、死消息、事务消息、幂等消息、延时消息等丰富的消息队列功能。
---

<style>
.clip{
  font-size:50px;
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
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: purple;

  --vp-home-hero-image-background-image: linear-gradient(-45deg, #bd34fe 50%, #47caff 50%);
  --vp-home-hero-image-filter: blur(44px);
}

@media (min-width: 640px) {
  :root {
    --vp-home-hero-image-filter: blur(56px);
    --vp-home-hero-name-font-size: 20px;
  }
}

@media (min-width: 960px) {
  :root {
    --vp-home-hero-image-filter: blur(68px);
  }
  .name{
    font-size:20px;
  }
}
</style>
