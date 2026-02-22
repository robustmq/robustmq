---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  text: "下一代AI、IoT、大数据统一的通信基础设施"
  tagline: |
    让数据在AI训练集群、百万Agent、IoT设备和云端之间，以最优路径、最低延迟、最小成本自由流动。
    RobustMQ 用 Rust 构建，完全兼容 Kafka 和 MQTT 协议，通过对象存储智能缓存、百万级轻量 Topic、共享订阅和多模式存储引擎，一套系统替代 MQTT + Kafka 双架构。

  actions:
    - theme: alt
      text: ⭐ GitHub Stars
      link: https://github.com/robustmq/robustmq
    - theme: alt
      text: 👥 Contributors
      link: https://github.com/robustmq/robustmq/graphs/contributors
    - theme: alt
      text: 📦 Version
      link: https://github.com/robustmq/robustmq/releases
    - theme: brand
      text: 🚀 快速开始
      link: /zh/OverView/What-is-RobustMQ
  image:
    src: /logo-large.jpg
    alt: RobustMQ

features:
  - title: 🚀 极致性能
    details: 基于 Rust 构建，微秒级延迟，无 GC 停顿，单节点百万级 QPS，极小内存占用支持边缘部署。
  - title: 🔌 双协议统一
    details: 完全兼容 MQTT 3.1/3.1.1/5.0 和 Kafka 协议，统一存储层实现 MQTT in / Kafka out，一套系统替代双架构，零迁移成本。
  - title: 🎯 AI 训练加速
    details: 对象存储（S3/MinIO）直连，三层智能缓存（内存/SSD/S3），训练数据无需预导入，消除 I/O 瓶颈，极大提升 GPU 利用率。
  - title: 🤖 Agent 通信
    details: 单集群支持百万级轻量 Topic，每个 Agent 独立通道，细粒度隔离与监控，成本按 Agent 可追溯。
  - title: 🔄 弹性消费
    details: 共享订阅模式突破 Kafka "并发度 = Partition 数量"的限制，GPU 训练节点随时扩缩容，无需修改 Topic 配置。
  - title: 💾 智能存储引擎
    details: 内存/混合/持久/分层四模式，Topic 级独立配置，热数据极速访问，冷数据自动分层到 S3，性能与成本兼顾。
  - title: 🌐 边缘到云端
    details: 极小内存占用，从边缘网关到云端集群统一部署，断网缓存 + 自动同步覆盖 IoT 全链路。
  - title: 🛠️ 极简部署
    details: 单二进制文件，零外部依赖，内置 Raft 共识，开箱即用，运维成本极低。

---

<div class="architecture-section">
  <div class="architecture-header">
    <h2>系统架构</h2>
    <p>Meta Service 管元数据，无状态 Broker 处理协议，插件化 Storage Engine 负责持久化。三组件固定架构，边界清晰，各自独立扩展。</p>
  </div>
  <div class="architecture-container">
    <div class="architecture-image-wrapper">
      <img src="/images/robustmq-architecture.jpg" alt="RobustMQ Architecture" />
      <div class="architecture-overlay">
        <div class="overlay-content">
          <h3>Meta Service · Broker · Storage Engine</h3>
          <p>存算分离，无状态 Broker 水平扩展，存储层独立演进</p>
        </div>
      </div>
    </div>
  </div>
</div>

<div class="console-section">
  <div class="console-header">
    <h2>Web 管理控制台</h2>
    <p>可视化管理连接、Topic、订阅、规则引擎和监控指标，内置 Grafana 集成，开箱即用。</p>
  </div>
  <div class="console-container">
    <div class="console-image-wrapper">
      <img src="/images/console-logo.png" alt="RobustMQ Console" />
      <div class="console-overlay">
        <div class="overlay-content">
          <h3>Dashboard</h3>
          <p>实时连接、消息、存储全览</p>
        </div>
      </div>
    </div>
  </div>
</div>

<div class="web-ui-section">
  <div class="web-ui-header">
    <h2>实时监控</h2>
  </div>
  <div class="web-ui-container">
    <div class="web-ui-image-wrapper">
      <img src="/images/web-ui.jpg" alt="RobustMQ Web Dashboard" />
      <div class="web-ui-overlay">
        <div class="overlay-content">
          <h3>Grafana + Prometheus</h3>
          <p>连接数、吞吐、延迟、存储全维度可观测</p>
        </div>
      </div>
    </div>
  </div>
</div>

<div class="footer-message">
  <p>技术信仰驱动，目标成为 Apache 顶级项目</p>
</div>

<div class="website-footer">
  <p>RobustMQ Website</p>
</div>

<div class="footer-brand">
  <span>@RobustMQ</span>
</div>

---

<style>

/* 徽章样式已移至 BadgeSection 组件中 */


.VPHero .name .clip,
.VPHomeHero .name .clip,
h1.name .clip,
.clip {
  font-size: 36px !important;
  font-weight: 700 !important;
  line-height: 1.2 !important;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #f093fb 100%) !important;
  -webkit-background-clip: text !important;
  -webkit-text-fill-color: transparent !important;
  background-clip: text !important;
  text-shadow: 0 4px 8px rgba(102, 126, 234, 0.3) !important;
  letter-spacing: -0.02em !important;
  animation: titleGlow 3s ease-in-out infinite alternate !important;
  display: inline-block !important;
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

/* 主标题渐变动画 */
@keyframes titleGlow {
  0% {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #f093fb 100%);
    text-shadow: 0 4px 8px rgba(102, 126, 234, 0.3);
  }
  50% {
    background: linear-gradient(135deg, #764ba2 0%, #f093fb 50%, #667eea 100%);
    text-shadow: 0 6px 12px rgba(118, 75, 162, 0.4);
  }
  100% {
    background: linear-gradient(135deg, #f093fb 0%, #667eea 50%, #764ba2 100%);
    text-shadow: 0 4px 8px rgba(240, 147, 251, 0.3);
  }
}

@media (min-width: 640px) {
  :root {
    --vp-home-hero-image-filter: blur(56px) !important;
    --vp-home-hero-name-font-size: 20px !important;
  }

  .VPHero .name .clip,
  .VPHomeHero .name .clip,
  h1.name .clip,
  .clip {
    font-size: 40px !important;
  }

  .text[data-v-72cc4481] {
    font-size: 30px !important;
  }
}

@media (min-width: 960px) {
  :root {
    --vp-home-hero-image-filter: blur(68px) !important;
  }

  .VPHero .name .clip,
  .VPHomeHero .name .clip,
  h1.name .clip,
  .clip {
    font-size: 44px !important;
  }

  .name{
    font-size:20px !important;
  }

  .text[data-v-72cc4481] {
    font-size: 35px !important;
  }
}

@media (max-width: 640px) {
  .VPHero .name .clip,
  .VPHomeHero .name .clip,
  h1.name .clip,
  .clip {
    font-size: 28px !important;
    line-height: 1.1 !important;
  }

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
