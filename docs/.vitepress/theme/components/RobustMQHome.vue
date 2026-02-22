<script setup>
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useData } from 'vitepress'
import { fetchStats } from '../github-stars.js'

const { lang } = useData()
const isZh = computed(() => lang.value === 'zh')

const t = (zh, en) => isZh.value ? zh : en

// GitHub live stats
const ghStars = ref(null)
const ghContributors = ref(null)

// Nav scroll effect
let scrollHandler = null

// Neural network canvas
const neuralCanvas = ref(null)
let animationId = null

function initNeuralNet(canvas) {
  const ctx = canvas.getContext('2d')
  let W = canvas.offsetWidth
  let H = canvas.offsetHeight
  canvas.width = W
  canvas.height = H

  const COUNT = 110
  const MAX_DIST = 170
  const nodes = Array.from({ length: COUNT }, () => ({
    x: Math.random() * W,
    y: Math.random() * H,
    vx: (Math.random() - 0.5) * 0.35,
    vy: (Math.random() - 0.5) * 0.35,
    r: Math.random() * 2 + 1,
    alpha: Math.random() * 0.5 + 0.3,
  }))

  // Active pulses: { fromIdx, toIdx, progress, speed }
  const pulses = []
  const spawnPulse = () => {
    const i = Math.floor(Math.random() * COUNT)
    const j = Math.floor(Math.random() * COUNT)
    if (i === j) return
    const dx = nodes[i].x - nodes[j].x
    const dy = nodes[i].y - nodes[j].y
    if (Math.sqrt(dx * dx + dy * dy) < MAX_DIST) {
      pulses.push({ i, j, t: 0, speed: 0.008 + Math.random() * 0.012 })
    }
  }

  let frame = 0
  function draw() {
    ctx.clearRect(0, 0, W, H)
    frame++

    // Occasionally spawn a pulse
    if (frame % 10 === 0) spawnPulse()

    // Move nodes
    for (const n of nodes) {
      n.x += n.vx; n.y += n.vy
      if (n.x < 0 || n.x > W) n.vx *= -1
      if (n.y < 0 || n.y > H) n.vy *= -1
    }

    // Draw connections
    for (let i = 0; i < COUNT; i++) {
      for (let j = i + 1; j < COUNT; j++) {
        const dx = nodes[i].x - nodes[j].x
        const dy = nodes[i].y - nodes[j].y
        const d = Math.sqrt(dx * dx + dy * dy)
        if (d < MAX_DIST) {
          const a = (1 - d / MAX_DIST) * 0.30
          ctx.beginPath()
          ctx.moveTo(nodes[i].x, nodes[i].y)
          ctx.lineTo(nodes[j].x, nodes[j].y)
          ctx.strokeStyle = `rgba(168,85,247,${a})`
          ctx.lineWidth = 0.8
          ctx.stroke()
        }
      }
    }

    // Draw pulse particles
    for (let p = pulses.length - 1; p >= 0; p--) {
      const pulse = pulses[p]
      pulse.t += pulse.speed
      if (pulse.t >= 1) { pulses.splice(p, 1); continue }
      const ni = nodes[pulse.i], nj = nodes[pulse.j]
      const dx = nj.x - ni.x, dy = nj.y - ni.y
      const d = Math.sqrt(dx * dx + dy * dy)
      if (d >= MAX_DIST) { pulses.splice(p, 1); continue }
      const px = ni.x + dx * pulse.t
      const py = ni.y + dy * pulse.t
      const a = Math.sin(pulse.t * Math.PI) * 0.9
      ctx.beginPath()
      ctx.arc(px, py, 3, 0, Math.PI * 2)
      ctx.fillStyle = `rgba(220,160,255,${a})`
      ctx.fill()
      // glow
      const g = ctx.createRadialGradient(px, py, 0, px, py, 10)
      g.addColorStop(0, `rgba(168,85,247,${a * 0.7})`)
      g.addColorStop(1, 'rgba(168,85,247,0)')
      ctx.beginPath()
      ctx.arc(px, py, 10, 0, Math.PI * 2)
      ctx.fillStyle = g
      ctx.fill()
    }

    // Draw nodes
    for (const n of nodes) {
      ctx.beginPath()
      ctx.arc(n.x, n.y, n.r, 0, Math.PI * 2)
      ctx.fillStyle = `rgba(168,85,247,${n.alpha})`
      ctx.fill()
    }

    animationId = requestAnimationFrame(draw)
  }

  draw()

  const onResize = () => {
    W = canvas.offsetWidth; H = canvas.offsetHeight
    canvas.width = W; canvas.height = H
  }
  window.addEventListener('resize', onResize)
  canvas._cleanup = () => window.removeEventListener('resize', onResize)
}

onMounted(async () => {
  // Neural network
  if (neuralCanvas.value) initNeuralNet(neuralCanvas.value)

  // Nav scroll
  const nav = document.querySelector('.VPNavBar')
  if (nav) {
    scrollHandler = () => nav.classList.toggle('nav-scrolled', window.scrollY > 40)
    window.addEventListener('scroll', scrollHandler, { passive: true })
    scrollHandler()
  }

  // GitHub stats
  try {
    const stats = await fetchStats()
    if (stats.stars) ghStars.value = stats.stars
    if (stats.contributors) ghContributors.value = stats.contributors
  } catch (_) {}
})

onUnmounted(() => {
  if (animationId) cancelAnimationFrame(animationId)
  if (neuralCanvas.value?._cleanup) neuralCanvas.value._cleanup()
  if (scrollHandler) window.removeEventListener('scroll', scrollHandler)
})

const scenarios = computed(() => [
  {
    icon: '⬡',
    color: '#a855f7',
    title: t('AI 训练加速', 'AI Training Acceleration'),
    subtitle: t('消灭 GPU 等待', 'Eliminate GPU Idle Time'),
    points: [
      t('S3/MinIO 直连 + 三级缓存（内存/SSD/S3）', 'S3/MinIO direct connect + 3-tier cache'),
      t('训练数据无需预导入，消除 I/O 瓶颈', 'No data pre-import, eliminates I/O bottleneck'),
      t('百万级轻量 Topic，Agent 独立通道', 'Million-scale Topics for Agent isolation'),
    ]
  },
  {
    icon: '◈',
    color: '#7c3aed',
    title: t('IoT 全链路', 'IoT Full Pipeline'),
    subtitle: t('边缘到云端统一', 'Edge to Cloud Unified'),
    points: [
      t('MQTT 接入 / Kafka 消费，一套系统替代双架构', 'MQTT in / Kafka out, one system replaces two'),
      t('极小内存占用，支持边缘网关部署', 'Tiny memory footprint for edge gateways'),
      t('断网缓存 + 自动同步，覆盖全链路', 'Offline cache + auto-sync full pipeline'),
    ]
  },
  {
    icon: '⬡',
    color: '#c084fc',
    title: t('大数据管道', 'Big Data Pipeline'),
    subtitle: t('Kafka 协议兼容', 'Kafka Protocol Compatible'),
    points: [
      t('完全兼容 Kafka 协议，现有应用零迁移', 'Full Kafka protocol compat, zero migration'),
      t('四模式存储引擎，Topic 级独立配置', '4-mode storage engine, per-Topic config'),
      t('冷数据自动分层到 S3，成本降低 95%', 'Auto-tier cold data to S3, 95% cost reduction'),
    ]
  }
])

const features = computed(() => [
  {
    icon: '⚡',
    title: t('极致性能', 'Extreme Performance'),
    desc: t('Rust 构建，微秒延迟，无 GC 停顿，单节点百万级 QPS', 'Rust built, microsecond latency, zero GC pauses, million QPS'),
  },
  {
    icon: '⚙',
    title: t('双协议统一', 'Dual Protocol'),
    desc: t('MQTT 3.1/3.1.1/5.0 与 Kafka 完全兼容，统一存储层', 'Full MQTT 3.1/3.1.1/5.0 & Kafka compat, unified storage'),
  },
  {
    icon: '◈',
    title: t('插件化存储', 'Pluggable Storage'),
    desc: t('Memory / RocksDB / File Segment / S3，Topic 级粒度配置', 'Memory/RocksDB/FileSegment/S3, per-Topic granularity'),
  },
  {
    icon: '⬡',
    title: t('零外部依赖', 'Zero Dependencies'),
    desc: t('单二进制，内置 Raft 共识，无需 ZooKeeper / etcd', 'Single binary, built-in Raft consensus, no ZooKeeper'),
  },
  {
    icon: '◎',
    title: t('存算分离', 'Compute-Storage Sep.'),
    desc: t('Broker 无状态水平扩展，存储独立演进，弹性自如', 'Stateless Broker scales horizontally, storage evolves independently'),
  },
  {
    icon: '✦',
    title: t('ISR 强一致', 'ISR Consistency'),
    desc: t('ISR 副本机制，Active/Sealed Segment 分层，无迁移扩容', 'ISR replication, Active/Sealed Segment tiering, scale without migration'),
  },
])
</script>

<template>
  <div class="rhome">

    <!-- ── Background ── -->
    <div class="rhome-bg" aria-hidden="true">
      <div class="bg-grid"></div>
      <canvas ref="neuralCanvas" class="bg-neural"></canvas>
      <div class="bg-orb bg-orb-1"></div>
      <div class="bg-orb bg-orb-2"></div>
      <div class="bg-orb bg-orb-3"></div>
      <div class="bg-scan">
        <div class="bg-ripple-1"></div>
        <div class="bg-ripple-2"></div>
      </div>
    </div>

    <!-- ══ HERO ══ -->
    <section class="hero">
      <div class="hero-inner">
        <div class="hero-badge">
          <span class="badge-dot"></span>
          {{ t('目标 Apache 顶级项目 · Rust 构建', 'Targeting Apache TLP · Built with Rust') }}
        </div>

        <h1 class="hero-title">
          <span class="title-word title-robust">Robust</span><span class="title-word title-mq">MQ</span>
        </h1>

        <p class="hero-sub">
          {{ t('下一代 AI 数据通信基础设施', 'Next-Generation AI Data Communication Infrastructure') }}
        </p>

        <p class="hero-desc">
          {{ t(
            '为 AI 训练集群、百万 Agent、IoT 设备与云端之间的数据流动，提供统一、高性能、低成本的通信底座。',
            'A unified, high-performance, low-cost communication substrate for data flowing between AI training clusters, millions of Agents, IoT devices, and the cloud.'
          ) }}
        </p>

        <div class="hero-actions">
          <a class="btn btn-primary" href="/en/OverView/What-is-RobustMQ">
            <span class="btn-glow"></span>
            {{ t('快速开始', 'Get Started') }}
            <span class="btn-arrow">→</span>
          </a>
          <a class="btn btn-ghost" href="https://github.com/robustmq/robustmq" target="_blank" rel="noopener">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/></svg>
            GitHub
          </a>
        </div>

        <div class="hero-stats">
          <a class="stat-item stat-link" href="https://github.com/robustmq/robustmq" target="_blank" rel="noopener">
            <span class="stat-icon">⭐</span>
            <span class="stat-val stat-live" :class="{ 'stat-loading': !ghStars }">{{ ghStars || '···' }}</span>
            <span class="stat-label">{{ t('Star', 'Stars') }}</span>
          </a>
          <div class="stat-divider"></div>
          <a class="stat-item stat-link" href="https://github.com/robustmq/robustmq/graphs/contributors" target="_blank" rel="noopener">
            <span class="stat-icon">👥</span>
            <span class="stat-val stat-live" :class="{ 'stat-loading': !ghContributors }">{{ ghContributors || '···' }}</span>
            <span class="stat-label">{{ t('贡献者', 'Contributors') }}</span>
          </a>
          <div class="stat-divider"></div>
          <div class="stat-item">
            <span class="stat-icon">⚡</span>
            <span class="stat-val">100<span class="stat-unit">µs</span></span>
            <span class="stat-label">{{ t('内存延迟', 'Memory Latency') }}</span>
          </div>
          <div class="stat-divider"></div>
          <div class="stat-item">
            <span class="stat-icon">🚀</span>
            <span class="stat-val">1M<span class="stat-unit">+</span></span>
            <span class="stat-label">{{ t('单节点 QPS', 'Single-Node QPS') }}</span>
          </div>
          <div class="stat-divider"></div>
          <div class="stat-item">
            <span class="stat-icon">📦</span>
            <span class="stat-val">0</span>
            <span class="stat-label">{{ t('外部依赖', 'External Deps') }}</span>
          </div>
        </div>
      </div>
    </section>

    <!-- ══ SCENARIOS ══ -->
    <section class="section scenarios-section">
      <div class="section-inner">
        <div class="section-header">
          <div class="section-tag">{{ t('核心场景', 'Core Scenarios') }}</div>
          <h2 class="section-title">{{ t('为三大场景深度优化', 'Deeply Optimized for Three Scenarios') }}</h2>
          <p class="section-desc">{{ t('不是通用消息队列的简单堆叠，每个场景都经过专项设计', 'Not a generic MQ stack — each scenario is purpose-designed') }}</p>
        </div>

        <div class="scenario-cards">
          <div
            v-for="s in scenarios"
            :key="s.title"
            class="scenario-card"
            :style="{ '--accent': s.color }"
          >
            <div class="sc-icon">{{ s.icon }}</div>
            <h3 class="sc-title">{{ s.title }}</h3>
            <p class="sc-subtitle">{{ s.subtitle }}</p>
            <ul class="sc-points">
              <li v-for="p in s.points" :key="p">
                <span class="sc-bullet"></span>{{ p }}
              </li>
            </ul>
            <div class="sc-glow"></div>
          </div>
        </div>
      </div>
    </section>

    <!-- ══ ARCHITECTURE ══ -->
    <section class="section arch-section">
      <div class="section-inner">
        <div class="section-header">
          <div class="section-tag">{{ t('系统架构', 'Architecture') }}</div>
          <h2 class="section-title">{{ t('三组件，极简边界', 'Three Components, Clear Boundaries') }}</h2>
          <p class="section-desc">{{ t('计算、存储、调度完全分离，每层独立扩展，单二进制交付', 'Compute, storage, and coordination fully separated — each layer scales independently, delivered as a single binary') }}</p>
        </div>

        <div class="arch-visual">
          <!-- Broker requests from Meta -->
          <div class="arch-flow arch-flow-top">
            <div class="flow-line flow-line-h"></div>
            <div class="flow-dot flow-dot-1"></div>
            <div class="flow-dot flow-dot-2"></div>
          </div>

          <div class="arch-nodes">
            <div class="arch-node node-meta">
              <div class="node-icon">◈</div>
              <div class="node-name">Meta Service</div>
              <div class="node-desc">{{ t('元数据 · 协调 · 控制器', 'Metadata · Coordination · Controller') }}</div>
              <div class="node-tech">gRPC · Multi Raft · RocksDB</div>
            </div>

            <div class="arch-connector">
              <div class="conn-line"></div>
              <div class="conn-arrow">↔</div>
              <div class="conn-label">{{ t('集群协调', 'Cluster Sync') }}</div>
            </div>

            <div class="arch-node node-broker">
              <div class="node-icon">⬡</div>
              <div class="node-name">Broker</div>
              <div class="node-desc">{{ t('协议解析 · 消息路由 · 无状态', 'Protocol · Routing · Stateless') }}</div>
              <div class="node-tech">MQTT · Kafka · TCP/TLS/QUIC</div>
            </div>

            <div class="arch-connector">
              <div class="conn-line"></div>
              <div class="conn-arrow">↕</div>
              <div class="conn-label">{{ t('数据读写', 'Data R/W') }}</div>
            </div>

            <div class="arch-node node-storage">
              <div class="node-icon">⚙</div>
              <div class="node-name">Storage Engine</div>
              <div class="node-desc">{{ t('Memory · RocksDB · File Segment', 'Memory · RocksDB · File Segment') }}</div>
              <div class="node-tech">ISR · Tiered Storage · S3</div>
            </div>
          </div>

          <div class="arch-labels">
            <div class="arch-label">
              <span class="label-dot" style="--c:#7c3aed"></span>
              {{ t('角色由配置决定', 'Roles defined by config') }}
            </div>
            <div class="arch-label">
              <span class="label-dot" style="--c:#a855f7"></span>
              {{ t('三个角色单二进制交付', 'All three roles in one binary') }}
            </div>
            <div class="arch-label">
              <span class="label-dot" style="--c:#c084fc"></span>
              {{ t('存算彻底分离', 'Compute-storage fully separated') }}
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- ══ FEATURES ══ -->
    <section class="section features-section">
      <div class="section-inner">
        <div class="section-header">
          <div class="section-tag">{{ t('核心特性', 'Core Features') }}</div>
          <h2 class="section-title">{{ t('从内核开始设计', 'Designed from the Core') }}</h2>
        </div>

        <div class="feature-grid">
          <div
            v-for="f in features"
            :key="f.title"
            class="feature-card"
          >
            <div class="fc-icon">{{ f.icon }}</div>
            <h3 class="fc-title">{{ f.title }}</h3>
            <p class="fc-desc">{{ f.desc }}</p>
          </div>
        </div>
      </div>
    </section>

    <!-- ══ QUICKSTART ══ -->
    <section class="section qs-section">
      <div class="section-inner">
        <div class="section-header">
          <div class="section-tag">{{ t('快速上手', 'Quick Start') }}</div>
          <h2 class="section-title">{{ t('三步启动集群', 'Start a Cluster in 3 Steps') }}</h2>
          <p class="section-desc">{{ t('单二进制，零外部依赖，一条命令拉起节点', 'Single binary, zero external dependencies, one command per node') }}</p>
        </div>

        <div class="qs-terminal">
          <div class="terminal-titlebar">
            <span class="tb-dot tb-red"></span>
            <span class="tb-dot tb-yellow"></span>
            <span class="tb-dot tb-green"></span>
            <span class="tb-title">bash</span>
          </div>
          <div class="terminal-body">
            <div class="t-line">
              <span class="t-comment"># {{ t('一键安装', 'One-line install') }}</span>
            </div>
            <div class="t-line">
              <span class="t-prompt">$</span>
              <span class="t-cmd">curl -sSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash</span>
            </div>
            <div class="t-line t-blank"></div>
            <div class="t-line">
              <span class="t-comment"># {{ t('启动节点', 'Start node') }}</span>
            </div>
            <div class="t-line">
              <span class="t-prompt">$</span>
              <span class="t-cmd">robust-server start</span>
            </div>
            <div class="t-line t-blank"></div>
            <div class="t-line">
              <span class="t-comment"># {{ t('验证集群状态', 'Verify cluster') }}</span>
            </div>
            <div class="t-line">
              <span class="t-prompt">$</span>
              <span class="t-cmd">robust-ctl cluster status</span>
            </div>
            <div class="t-line">
              <span class="t-out">✓ cluster healthy · 1 meta · 1 broker · 1 engine</span>
            </div>
          </div>
        </div>

        <div class="qs-links">
          <a class="qs-link" href="/en/QuickGuide/Quick-Install">
            {{ t('安装指南', 'Installation Guide') }} →
          </a>
          <a class="qs-link" href="/en/QuickGuide/Experience-MQTT">
            {{ t('体验 MQTT', 'Experience MQTT') }} →
          </a>
          <a class="qs-link" href="/en/Architect/Overall-Architecture">
            {{ t('了解架构', 'Learn Architecture') }} →
          </a>
        </div>
      </div>
    </section>

    <!-- ══ FOOTER ══ -->
    <footer class="rhome-footer">
      <div class="footer-inner">
        <div class="footer-brand">
          <span class="footer-logo">RobustMQ</span>
          <span class="footer-tagline">{{ t('技术信仰驱动', 'Driven by Technical Conviction') }}</span>
        </div>
        <div class="footer-links">
          <a href="https://github.com/robustmq/robustmq" target="_blank" rel="noopener">GitHub</a>
          <a href="/en/OverView/What-is-RobustMQ">{{ t('文档', 'Docs') }}</a>
          <a href="/en/Blogs/">{{ t('博客', 'Blog') }}</a>
        </div>
        <p class="footer-note">{{ t('目标成为 Apache 顶级项目 · 永久开源 · 非商业化', 'Aiming for Apache TLP · Forever Open Source · Non-Commercial') }}</p>
      </div>
    </footer>

  </div>
</template>
