---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  text: "新一代云原生和AI原生消息基础设施"
  tagline: |
    RobustMQ 是一个 100% 用 Rust 实现的中间件消息队列领域的开源项目。它的的目标是基于Rust 打造兼容多种主流消息队列协议、具备完整 Serverless 能力的下一代高性能云原生融合型消息队列。
    
    <div class="badges">
      <img alt="Latest Release" src="https://img.shields.io/github/v/release/robustmq/robustmq?style=flat">
      <img alt="License" src="https://img.shields.io/github/license/robustmq/robustmq?style=flat">
      <img alt="GitHub issues" src="https://img.shields.io/github/issues/robustmq/robustmq?style=flat">
      <img alt="GitHub stars" src="https://img.shields.io/github/stars/robustmq/robustmq?style=flat">
    </div>

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
    details: 支持 MQTT 3.1/3.1.1/5.0、AMQP、RocketMQ Remoting/GRPC、Kafka 协议、OpenMessaging、JNS、SQS 等主流消息协议。
  - title: 分层架构
    details: 三层独立架构，包括 Computing、Storage 和 Scheduling。每一层都具备集群部署能力和快速水平扩容能力。
  - title: 插件存储
    details: 通过独立存储插件实施，您可以按需选择最佳插件，与传统本地部署和新的云原生部署兼容。
  - title: 高内聚力
    details: 它提供内置的元数据存储组件和分布式日志存储服务。所有这些都可以快速、轻松和有凝聚力地部署。
  - title: 功能丰富
    details: 功能丰富：支持顺序消息、死消息、事务消息、幂等消息、延时消息等丰富的消息队列功能。

---

<div class="architecture-section">
  <div class="architecture-header">
    <h2>架构概览</h2>
  </div>
  <div class="architecture-container">
    <div class="architecture-image-wrapper">
      <img src="/images/robustmq-architecture.jpg" alt="RobustMQ Architecture" />
      <div class="architecture-overlay">
        <div class="overlay-content">
          <h3>云原生设计</h3>
          <p>为规模、可靠性和性能而构建</p>
        </div>
      </div>
    </div>
  </div>
</div>

<div class="footer-message">
  <p>很高兴有机会让你看到不一样的作品。</p>
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
  font-size:55px !important;
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
  
  .text[data-v-72cc4481] {
    font-size: 30px !important;
  }
}

@media (min-width: 960px) {
  :root {
    --vp-home-hero-image-filter: blur(68px) !important;
  }
  .name{
    font-size:20px !important;
  }
  
  .text[data-v-72cc4481] {
    font-size: 35px !important;
  }
}

@media (max-width: 640px) {
  .text[data-v-72cc4481] {
    font-size: 22px !important;
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
