---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  text: "新一代云原生和AI原生消息基础设施"
  tagline: |
    RobustMQ 是 100% 用 Rust 实现的新一代云原生消息队列，专为 AI 时代和云原生环境重新设计。通过多协议统一（MQTT/Kafka/AMQP）、存算分离架构和插件化存储，提供微秒级延迟、零 GC 停顿的高性能消息基础设施，支持 Serverless 弹性扩缩容。
    
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
  - title: 🦀 Rust 高性能内核
    details: 完全用 Rust 实现的消息队列内核，零 GC 停顿、内存安全、微秒级延迟，为 AI 应用提供极致性能保障。
  - title: 🔌 多协议统一平台
    details: 原生支持 MQTT、Kafka、AMQP 等主流协议，一次部署多协议可用，避免系统割裂，降低运维复杂度。
  - title: ☁️ 存算分离架构
    details: 采用 Broker、Journal、Meta Service 三层独立设计，无状态计算层支持 Serverless 弹性扩缩容，存储层独立扩展。
  - title: 💾 插件化存储引擎
    details: 支持内存、SSD、对象存储等多种后端，智能分层存储，根据业务场景灵活选择，大幅降低存储成本。
  - title: 🚀 AI 原生优化
    details: 专为 AI 时代设计，支持海量数据流处理、实时推理场景，微秒级延迟满足 AI 应用的严苛性能要求。
  - title: 🌐 云原生友好
    details: 单二进制部署、K8s Operator 支持、可视化管理界面，真正实现云原生时代的简化运维和快速部署。

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

<div class="console-section">
  <div class="console-header">
    <h2>管理控制台</h2>
  </div>
  <div class="console-container">
    <div class="console-image-wrapper">
      <img src="/images/console-logo.png" alt="RobustMQ Console" />
      <div class="console-overlay">
        <div class="overlay-content">
          <h3>Web 管理</h3>
          <p>通过 Web 界面轻松监控和配置</p>
        </div>
      </div>
    </div>
  </div>
</div>

<div class="footer-message">
  <p>很高兴有机会让你看到不一样的作品</p>
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

/* Console展示区域样式 */
.console-section {
  margin: 80px 0;
  padding: 0 20px;
}

.console-header {
  text-align: center;
  margin-bottom: 50px;
}

.console-header h2 {
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

.console-header p {
  font-size: 1.125rem;
  color: #64748b;
  max-width: 600px;
  margin: 0 auto;
  line-height: 1.6;
  font-weight: 400;
  padding: 0 20px;
}

.console-container {
  max-width: 800px;
  margin: 0 auto;
  position: relative;
}

.console-image-wrapper {
  position: relative;
  border-radius: 24px;
  overflow: hidden;
  box-shadow:
    0 25px 50px -12px rgba(0, 0, 0, 0.25),
    0 0 0 1px rgba(255, 255, 255, 0.1);
  background: linear-gradient(135deg, #1e293b 0%, #334155 100%);
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  text-align: center;
  padding: 40px;
}

.console-image-wrapper:hover {
  transform: translateY(-8px) scale(1.02);
  box-shadow:
    0 32px 64px -12px rgba(0, 0, 0, 0.35),
    0 0 0 1px rgba(255, 255, 255, 0.2);
}

.console-image-wrapper img {
  max-width: 100%;
  height: auto;
  display: block;
  margin: 0 auto;
  transition: all 0.4s ease;
  filter: brightness(0.9) contrast(1.1);
  border-radius: 16px;
}

.console-image-wrapper:hover img {
  filter: brightness(1) contrast(1.2);
  transform: scale(1.05);
}

.console-overlay {
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

.console-image-wrapper:hover .console-overlay {
  opacity: 1;
  transform: translateY(0);
}

/* Console区域响应式设计 */
@media (max-width: 768px) {
  .console-header h2 {
    font-size: 2rem;
    margin-bottom: 20px;
  }
  
  .console-header p {
    font-size: 1rem;
    padding: 0 16px;
  }
  
  .console-section {
    margin: 60px 0;
    padding: 0 16px;
  }
  
  .console-image-wrapper {
    padding: 20px;
  }
  
  .console-overlay {
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
  box-shadow:none;
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
  box-shadow:0 8px 25px rgba(102, 126, 234, 0.3);
  font-weight: 600;
  font-size: 0.9rem;
  letter-spacing: 0.5px;
  transition: all 0.3s ease;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.footer-brand:hover {
  transform: translateY(-2px);
  box-shadow:0 12px 35px rgba(102, 126, 234, 0.4);
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
