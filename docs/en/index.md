---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: "RobustMQ"
  text: "New generation of cloud-native and AI-native messaging infrastructure"
  tagline: >
    RobustMQ is a next-generation cloud-native message queue that is 100% implemented in Rust, specifically redesigned for the AI era and cloud-native environments. Through multi-protocol unification (MQTT/Kafka/AMQP), compute-storage separation architecture, and pluggable storage, it provides high-performance messaging infrastructure with microsecond-level latency and zero GC pauses, supporting Serverless elastic scaling.
    <div class="badges">
      <img alt="Latest Release" src="https://img.shields.io/github/v/release/robustmq/robustmq?style=flat">
      <img alt="License" src="https://img.shields.io/github/license/robustmq/robustmq?style=flat">
      <img alt="GitHub issues" src="https://img.shields.io/github/issues/robustmq/robustmq?style=flat">
      <img alt="GitHub stars" src="https://img.shields.io/github/stars/robustmq/robustmq?style=flat">
    </div>

  actions:
    - theme: brand
      text: Get Started
      link: /en/OverView/What-is-RobustMQ
    # - theme: alt
    #   text: API Examples
    #   link: /api-examples
  image:
    src: /logo-large.jpg
    alt: RobustMQ

features:
  - title: 🦀 Rust High-Performance Kernel
    details: A message queue kernel implemented entirely in Rust, with zero GC pauses, memory safety, and microsecond-level latency, providing ultimate performance guarantee for AI applications.
  - title: 🔌 Multi-Protocol Unified Platform
    details: Native support for MQTT, Kafka, AMQP and other mainstream protocols. Deploy once, multiple protocols available, avoiding system fragmentation and reducing operational complexity.
  - title: ☁️ Compute-Storage Separation Architecture
    details: Three-tier independent design with Broker, Journal, and Meta Service. Stateless compute layer supports Serverless elastic scaling, with independent storage layer expansion.
  - title: 💾 Pluggable Storage Engine
    details: Supports multiple backends including memory, SSD, and object storage. Intelligent tiered storage allows flexible selection based on business scenarios, significantly reducing storage costs.
  - title: 🚀 AI-Native Optimization
    details: Specifically designed for the AI era, supporting massive data stream processing and real-time inference scenarios, with microsecond-level latency meeting stringent AI application performance requirements.
  - title: 🌐 Cloud-Native Friendly
    details: Single binary deployment, K8s Operator support, and visual management interface, truly achieving simplified operations and rapid deployment in the cloud-native era.
---

<div class="footer-message">
  <p>Glad to have the opportunity to show you something different</p>
</div>

<div class="website-footer">
  <p>RobustMQ Website</p>
</div>

<div class="footer-brand">
  <span>@RobustMQ</span>
</div>
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
  font-size:24px !important;
}
.tagline
{
  font-size:16px !important;
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

/* 添加动画效果 */
@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 底部消息样式 */
.footer-message {
  text-align: center;
  margin: 80px 0 60px;
  padding: 0 20px;
}

.footer-message p {
  font-size: 1.8rem;
  color: #2d3748;
  font-weight: 700;
  line-height: 1.2;
  max-width: none;
  margin: 0 auto;
  padding: 0;
  background: none;
  border: none;
  box-shadow: none;
  position: relative;
  animation: fadeInUp 1.2s ease-out 0.5s both;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  text-shadow: 0 2px 4px rgba(0,0,0,0.1);
  white-space: nowrap;
}

.footer-message p::before {
  content: "";
  position: absolute;
  top: -10px;
  left: 50%;
  transform: translateX(-50%);
  width: 60px;
  height: 3px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 2px;
}

.footer-message p::after {
  content: "";
  position: absolute;
  bottom: -15px;
  left: 50%;
  transform: translateX(-50%);
  width: 40px;
  height: 2px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 1px;
  opacity: 0.6;
}

/* 网站底部样式 */
.website-footer {
  text-align: center;
  margin: 40px 0 20px;
  padding: 0 20px;
}

.website-footer p {
  font-size: 0.875rem;
  color: #94a3b8;
  font-weight: 400;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  margin: 0;
  opacity: 0.8;
}

/* 固定底部品牌标识 */
.footer-brand {
  position: fixed;
  bottom: 20px;
  right: 20px;
  z-index: 1000;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 12px 20px;
  border-radius: 25px;
  box-shadow: 0 8px 25px rgba(102, 126, 234, 0.3);
  font-weight: 600;
  font-size: 0.9rem;
  letter-spacing: 0.5px;
  transition: all 0.3s ease;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.footer-brand:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 35px rgba(102, 126, 234, 0.4);
  background: linear-gradient(135deg, #5a67d8 0%, #6b46c1 100%);
}

.footer-brand span {
  display: flex;
  align-items: center;
  gap: 6px;
}

.footer-brand span::before {
  content: "🚀";
  font-size: 1rem;
}

/* 移动端调整 */
@media (max-width: 768px) {
  .footer-brand {
    bottom: 15px;
    right: 15px;
    padding: 10px 16px;
    font-size: 0.85rem;
  }
}
</style>
