---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  text: "New generation of cloud-native<br/>and AI-native messaging infrastructure."
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
    details: Support MQTT 3.1/3.1.1/5.0, AMQP, RocketMQ Remoting/GRPC, Kafka Protocol, OpenMessaging, JNS, SQS and other mainstream message protocols.
  - title: Layered architecture
    details: Three-tier independent architecture consists of Computing, Storage and Scheduling. Each layer has the ability of cluster deployment and rapid horizontal scaling capacity.
  - title: Plug-in storage
    details: With standalone storage plug-in implementation, you can choose the best plug-in on demand, compatible with traditional on-premise and new cloud-native deployment.
  - title: High cohesion
    details: It provides built-in metadata storage components and distributed journal storage services. All of these ones could be deployed quickly, easily and cohesively.
  - title: Rich functions
    details: Rich functions support sequential messages, dead message messages, transaction messages, idempotent messages, delay messages and other rich message queue functions.

---

<div class="architecture-section">
  <div class="architecture-header">
    <h2>Architecture Overview</h2>
  </div>
  <div class="architecture-container">
    <div class="architecture-image-wrapper">
      <img src="/images/robustmq-architecture.jpg" alt="RobustMQ Architecture" />
      <div class="architecture-overlay">
        <div class="overlay-content">
          <h3>Cloud-Native Design</h3>
          <p>Built for scale, reliability, and performance</p>
        </div>
      </div>
    </div>
  </div>
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
  margin-bottom: 20px;      /* 减少与按钮的距离 */
}

.badges img {
  height: 24px;   /* 调整徽章大小 */
}

/* 优化主标题样式 */
.clip{
  font-size:50px !important;
}

/* 优化副标题样式 */
.text[data-v-72cc4481] {
  font-size: 24px !important;
  font-weight: 600 !important;
  line-height: 1.4 !important;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%) !important;
  -webkit-background-clip: text !important;
  -webkit-text-fill-color: transparent !important;
  background-clip: text !important;
  text-shadow: 0 2px 4px rgba(0,0,0,0.1) !important;
  animation: fadeInUp 1s ease-out !important;
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

/* 优化描述文字样式 */
.tagline {
  font-size: 17px !important;
  font-weight: 400 !important;
  line-height: 1.3 !important;
  color: #4a5568 !important;
  letter-spacing: 0.01em !important;
  font-family: var(--vp-font-family-base) !important;
  text-rendering: optimizeLegibility !important;
  -webkit-font-smoothing: antialiased !important;
  -moz-osx-font-smoothing: grayscale !important;
  opacity: 0.9 !important;
  transition: opacity 0.3s ease !important;
}
.VPButton.brand
{
  background-color:purple !important;
}

/* 调整按钮容器的间距 */
.actions {
  margin-top: -30px !important;
  padding-top: 0 !important;
}
:root {
  --vp-home-hero-name-color: transparent !important;
  --vp-home-hero-name-background: purple !important;

  --vp-home-hero-image-background-image: linear-gradient(-45deg, #bd34fe 50%, #bd34fe 50%) !important;
  --vp-home-hero-image-filter: blur(44px) !important;
}

/* 响应式设计 */
@media (min-width: 640px) {
  :root {
    --vp-home-hero-image-filter: blur(56px) !important;
    --vp-home-hero-name-font-size: 20px !important;
  }
  
  .text[data-v-72cc4481] {
    font-size: 28px !important;
  }
  
  .tagline {
    font-size: 18px !important;
    line-height: 1.4 !important;
  }
}

@media (min-width: 960px) {
  :root {
    --vp-home-hero-image-filter: blur(68px) !important;
  }
  
  .name {
    font-size: 20px !important;
  }
  
  .text[data-v-72cc4481] {
    font-size: 32px !important;
  }
  
  .tagline {
    font-size: 19px !important;
    line-height: 1.45 !important;
    letter-spacing: 0.015em !important;
  }
}

@media (max-width: 640px) {
  .text[data-v-72cc4481] {
    font-size: 20px !important;
    line-height: 1.3 !important;
  }
  
  .tagline {
    font-size: 15px !important;
    line-height: 1.35 !important;
    letter-spacing: 0.005em !important;
  }
}
.VPImage {
    border-radius: 24% !important;
    opacity: 0.8 !important;
    transition: opacity 0.3s ease !important;
}

/* 架构展示区域样式 */
.architecture-section {
  margin: 80px 0;
  padding: 0 20px;
}

.architecture-header {
  text-align: center;
  margin-bottom: 50px;
}

.architecture-header h2 {
  font-size: 2.5rem;
  font-weight: 800;
  margin-bottom: 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  letter-spacing: -0.02em;
  line-height: 1.2;
}

.architecture-header p {
  font-size: 1.125rem;
  color: #64748b;
  max-width: 600px;
  margin: 0 auto;
  line-height: 1.6;
  font-weight: 400;
  padding: 0 20px;
}

.architecture-container {
  max-width: 1200px;
  margin: 0 auto;
  position: relative;
}

.architecture-image-wrapper {
  position: relative;
  border-radius: 24px;
  overflow: hidden;
  box-shadow: 
    0 25px 50px -12px rgba(0, 0, 0, 0.25),
    0 0 0 1px rgba(255, 255, 255, 0.1);
  background: linear-gradient(135deg, #1e293b 0%, #334155 100%);
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.architecture-image-wrapper:hover {
  transform: translateY(-8px) scale(1.02);
  box-shadow: 
    0 32px 64px -12px rgba(0, 0, 0, 0.35),
    0 0 0 1px rgba(255, 255, 255, 0.2);
}

.architecture-image-wrapper img {
  width: 100%;
  height: auto;
  display: block;
  transition: all 0.4s ease;
  filter: brightness(0.9) contrast(1.1);
}

.architecture-image-wrapper:hover img {
  filter: brightness(1) contrast(1.2);
  transform: scale(1.05);
}

.architecture-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: linear-gradient(
    to top,
    rgba(0, 0, 0, 0.8) 0%,
    rgba(0, 0, 0, 0.4) 50%,
    transparent 100%
  );
  padding: 40px 30px 30px;
  opacity: 0;
  transform: translateY(20px);
  transition: all 0.4s ease;
}

.architecture-image-wrapper:hover .architecture-overlay {
  opacity: 1;
  transform: translateY(0);
}

.overlay-content h3 {
  color: white;
  font-size: 1.5rem;
  font-weight: 700;
  margin-bottom: 8px;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
}

.overlay-content p {
  color: rgba(255, 255, 255, 0.9);
  font-size: 1rem;
  margin: 0;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .architecture-header h2 {
    font-size: 2rem;
    margin-bottom: 20px;
  }
  
  .architecture-header p {
    font-size: 1rem;
    padding: 0 16px;
  }
  
  .architecture-section {
    margin: 60px 0;
    padding: 0 16px;
  }
  
  .architecture-overlay {
    padding: 30px 20px 20px;
  }
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
