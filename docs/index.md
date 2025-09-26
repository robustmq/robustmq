---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  text: "New generation of cloud-native and AI-native messaging infrastructure"
  tagline: |
    RobustMQ is a next-generation cloud-native message queue that is 100% implemented in Rust, specifically redesigned for the AI era and cloud-native environments. Through multi-protocol unification (MQTT/Kafka/AMQP), compute-storage separation architecture, and pluggable storage, it provides high-performance messaging infrastructure with microsecond-level latency and zero GC pauses, supporting Serverless elastic scaling.
    
    <div class="hero-badges">
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

<div class="console-section">
  <div class="console-header">
    <h2>Management Console</h2>
  </div>
  <div class="console-container">
    <div class="console-image-wrapper">
      <img src="/images/console-logo.png" alt="RobustMQ Console" />
      <div class="console-overlay">
        <div class="overlay-content">
          <h3>Web Management</h3>
          <p>Easy monitoring and configuration through web interface</p>
        </div>
      </div>
    </div>
  </div>
</div>

<div class="web-ui-section">
  <div class="web-ui-header">
    <h2>Web Dashboard</h2>
  </div>
  <div class="web-ui-container">
    <div class="web-ui-image-wrapper">
      <img src="/images/web-ui.png" alt="RobustMQ Web Dashboard" />
      <div class="web-ui-overlay">
        <div class="overlay-content">
          <h3>Real-time Monitoring</h3>
          <p>Comprehensive dashboard for system monitoring and management</p>
        </div>
      </div>
    </div>
  </div>
</div>

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
  font-size: 28px !important;
  font-weight: 600 !important;
  line-height: 1.4 !important;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%) !important;
  -webkit-background-clip: text !important;
  -webkit-text-fill-color: transparent !important;
  background-clip: text !important;
  text-shadow: 0 2px 4px rgba(0,0,0,0.1) !important;
  /* animation: fadeInUp 1s ease-out !important; */
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
  font-size: 16px !important;
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
    font-size: 32px !important;
  }
  
  .tagline {
    font-size: 17px !important;
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
    font-size: 36px !important;
  }
  
  .tagline {
    font-size: 18px !important;
    line-height: 1.45 !important;
    letter-spacing: 0.015em !important;
  }
}

@media (max-width: 640px) {
  .text[data-v-72cc4481] {
    font-size: 24px !important;
    line-height: 1.3 !important;
  }
  
  .tagline {
    font-size: 14px !important;
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

/* Web UI展示区域样式 */
.web-ui-section {
  margin: 80px 0;
  padding: 0 20px;
}

.web-ui-header {
  text-align: center;
  margin-bottom: 50px;
}

.web-ui-header h2 {
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

.web-ui-header p {
  font-size: 1.125rem;
  color: #64748b;
  max-width: 600px;
  margin: 0 auto;
  line-height: 1.6;
  font-weight: 400;
  padding: 0 20px;
}

.web-ui-container {
  max-width: 800px;
  margin: 0 auto;
  position: relative;
}

.web-ui-image-wrapper {
  position: relative;
  border-radius: 24px;
  overflow: hidden;
  box-shadow:
    0 25px 50px -12px rgba(0, 0, 0, 0.25),
    0 0 0 1px rgba(255, 255, 255, 0.1);
  background: linear-gradient(135deg, #1e293b 0%, #334155 100%);
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  text-align: center;
  padding: 20px;
}

.web-ui-image-wrapper:hover {
  transform: translateY(-8px) scale(1.02);
  box-shadow:
    0 32px 64px -12px rgba(0, 0, 0, 0.35),
    0 0 0 1px rgba(255, 255, 255, 0.2);
}

.web-ui-image-wrapper img {
  max-width: 100%;
  height: auto;
  display: block;
  margin: 0 auto;
  transition: all 0.4s ease;
  filter: brightness(0.9) contrast(1.1);
  border-radius: 16px;
}

.web-ui-image-wrapper:hover img {
  filter: brightness(1) contrast(1.2);
  transform: scale(1.05);
}

.web-ui-overlay {
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

.web-ui-image-wrapper:hover .web-ui-overlay {
  opacity: 1;
  transform: translateY(0);
}

/* Web UI区域响应式设计 */
@media (max-width: 768px) {
  .web-ui-header h2 {
    font-size: 2rem;
    margin-bottom: 20px;
  }
  
  .web-ui-header p {
    font-size: 1rem;
    padding: 0 16px;
  }
  
  .web-ui-section {
    margin: 60px 0;
    padding: 0 16px;
  }
  
  .web-ui-image-wrapper {
    padding: 15px;
  }
  
  .web-ui-overlay {
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
  /* animation: fadeInUp 1.2s ease-out 0.5s both; */
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

/* 首页特定的布局优化 */
.VPHome .VPContent {
  overflow-x: hidden;
}

/* 确保首页元素不会超出边界 */
.architecture-section,
.console-section,
.web-ui-section {
  max-width: 100vw;
  overflow-x: hidden;
}
</style>
