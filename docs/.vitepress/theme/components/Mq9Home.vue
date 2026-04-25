<script setup>
import { computed, onMounted, onUnmounted } from 'vue'
import { useData } from 'vitepress'

const { lang } = useData()
const isZh = computed(() => lang.value === 'zh')
const t = (zh, en) => isZh.value ? zh : en

onMounted(() => {
  document.body.classList.add('mq9-layout')
})
onUnmounted(() => {
  document.body.classList.remove('mq9-layout')
})

const primitives = computed(() => [
  {
    icon: '📬',
    title: t('邮箱', 'Mailbox'),
    subtitle: t('点对点异步投递', 'Point-to-point async delivery'),
    desc: t(
      '创建邮箱，拿到 mail_address。发件方直接投到对方 mail_address，不需要知道对方在不在线，消息等着，对方上线全量收到。私有邮箱 mail_address 系统生成不可猜测，公开邮箱 mail_address 用户自定义。',
      'Create a mailbox, get a mail_address. Send to the recipient\'s mail_address — no need to know if they\'re online. Messages wait; the recipient gets all of them when they come back. Private mailbox mail_addresss are unguessable; public mailbox mail_addresss are user-defined.'
    ),
    color: '#a855f7',
    code: `# Create a mailbox
nats pub '$mq9.AI.MAILBOX.CREATE' '{"ttl":3600}'
# → {"mail_address":"m-001"}

# Send (offline-safe, default priority)
nats pub '$mq9.AI.MAILBOX.m-001' \\
  '{"msg_id":"msg-1","type":"task_result","ts":1234}'

# Subscribe (all priorities)
nats sub '$mq9.AI.MAILBOX.m-001.*'`,
  },
  {
    icon: '📡',
    title: t('公开邮箱', 'Public Mailbox'),
    subtitle: t('任意可发可订的公共频道', 'Public channel — anyone can publish or subscribe'),
    desc: t(
      '创建 public 邮箱，mail_address 用户自定义，自动注册到 PUBLIC.LIST。任何人知道 mail_address 即可发可订。支持 queue group 竞争消费。PUBLIC.LIST 是公开邮箱的发现地址，无需注册中心。',
      'Create a public mailbox with a user-defined mail_address — auto-registered to PUBLIC.LIST. Anyone can publish or subscribe. Supports queue group competing consumers. PUBLIC.LIST is the discovery address — no registry service needed.'
    ),
    color: '#7c3aed',
    code: `# Create a public mailbox
nats pub '$mq9.AI.MAILBOX.CREATE' \\
  '{"ttl":86400,"public":true,"name":"task.queue"}'
# → {"mail_address":"task.queue"}

# Publish (default priority, no suffix)
nats pub '$mq9.AI.MAILBOX.task.queue' \\
  '{"msg_id":"t-001","type":"analysis"}'

# Compete via queue group
nats sub '$mq9.AI.MAILBOX.task.queue.*' --queue workers

# Discover public mailboxes
nats sub '$mq9.AI.PUBLIC.LIST'`,
  },
  {
    icon: '⚡',
    title: t('优先级', 'Priority'),
    subtitle: t('紧急消息先处理', 'Critical messages first'),
    desc: t(
      '三个优先级：critical（最高）、urgent（紧急）、normal（默认，无后缀）。critical 和 urgent 持久化存储，normal 使用内存。边缘设备离线 8 小时后上线，critical 先于 urgent 先于 normal 处理。',
      'Three priority levels: critical (highest), urgent, normal (default, no suffix). critical and urgent are persisted to RocksDB; normal uses memory. An edge device offline for 8 hours reconnects and gets critical messages first, then urgent, then normal.'
    ),
    color: '#6d28d9',
    code: `# critical — highest, persisted, processed first
nats pub '$mq9.AI.MAILBOX.m-001.critical' \\
  '{"msg_id":"c-1","type":"emergency_stop"}'

# urgent — high priority, persisted
nats pub '$mq9.AI.MAILBOX.m-001.urgent' \\
  '{"msg_id":"u-1","type":"interrupt"}'

# normal — default, no suffix, memory storage
nats pub '$mq9.AI.MAILBOX.m-001' \\
  '{"msg_id":"n-1","type":"task"}'`,
  },
])

const scenarios = computed(() => [
  {
    num: '01',
    title: t('子 Agent 通知主 Agent', 'Sub-Agent notifies Orchestrator'),
    desc: t('子 Agent 完成任务，发结果到主 Agent 邮箱。主 Agent 不需要阻塞等待，空了来取。', 'Sub-Agent sends results to the orchestrator\'s mailbox. The orchestrator picks up results when ready — no blocking.'),
  },
  {
    num: '02',
    title: t('感知所有子 Agent 状态', 'Monitor all Agent states'),
    desc: t('Worker 创建公开邮箱定期上报状态，主 Agent 订阅 PUBLIC.LIST 发现所有 Worker。TTL 过期自动感知消亡，不需要注册或注销。', 'Workers create public mailboxes to report status. The orchestrator discovers them via PUBLIC.LIST. TTL expiry auto-signals death — no registration needed.'),
  },
  {
    num: '03',
    title: t('任务队列竞争消费', 'Task queue with competing consumers'),
    desc: t('创建公开邮箱作为任务队列，Worker 用 queue group 竞争消费，每条任务只被一个 Worker 处理。离线 Worker 上线后也能收到未过期任务。', 'Create a public mailbox as a task queue; Workers use queue group to compete. Each task is handled by exactly one Worker. Offline Workers receive non-expired tasks after reconnecting.'),
  },
  {
    num: '04',
    title: t('异常告警广播', 'Anomaly alert broadcast'),
    desc: t('创建公开邮箱发异常事件，订阅者自行响应。离线的 handler 上线后也能收到未过期的告警。发布方不需要维护订阅列表。', 'Create a public mailbox and publish anomaly events. Subscribers respond independently. Offline handlers receive non-expired alerts after reconnecting. Publisher needs no subscriber list.'),
  },
  {
    num: '05',
    title: t('边缘 Agent 离线积压', 'Edge Agent offline buffering'),
    desc: t('云端给边缘 Agent 邮箱发指令，边缘断网消息持久化等待，联网后按优先级处理——critical 先于 urgent 先于 normal。', 'Cloud sends instructions to an edge Agent\'s mailbox. Messages persist during outage and are processed by priority on reconnect — critical before urgent before normal.'),
  },
  {
    num: '06',
    title: t('人机混合工作流', 'Human-in-the-loop workflows'),
    desc: t('Agent 发审批请求到人类审批员邮箱，审批员处理后发结果回 Agent 邮箱。人和 Agent 用完全相同的协议，流程不中断。', 'Agent sends approval requests to a human\'s mailbox; the human replies to the Agent\'s mailbox. Humans and Agents use the exact same protocol — the workflow is uninterrupted.'),
  },
  {
    num: '07',
    title: t('异步 Request-Reply', 'Async Request-Reply'),
    desc: t('Agent A 发请求到 Agent B 邮箱，带 reply_to 和 correlation_id。B 上线后处理并回复到 A 邮箱。离线不丢，A 不阻塞。', 'Agent A sends a request to Agent B\'s mailbox with reply_to and correlation_id. B replies to A\'s mailbox when online. Nothing lost if offline; A doesn\'t block.'),
  },
  {
    num: '08',
    title: t('能力注册与发现', 'Capability registration and discovery'),
    desc: t('Agent 启动时创建公开邮箱声明能力，自动注册到 PUBLIC.LIST，其他 Agent 订阅即可感知整个网络。去中心化，零额外服务。', 'Agents create public mailboxes to declare capabilities at startup — auto-registered to PUBLIC.LIST. Subscribers sense the entire network. Decentralized — zero extra services.'),
  },
])
</script>

<template>
  <div class="mq9-page">

    <!-- ── HERO ── -->
    <section class="mq9-hero">
      <div class="mq9-hero-bg" aria-hidden="true">
        <div class="mq9-orb mq9-orb-1"></div>
        <div class="mq9-orb mq9-orb-2"></div>
      </div>
      <div class="mq9-hero-inner">
        <div class="mq9-badge">
          <span class="mq9-badge-dot"></span>
          {{ t('RobustMQ 第五个原生协议层', 'RobustMQ\'s Fifth Native Protocol') }}
        </div>

        <h1 class="mq9-title">
          <span class="mq9-title-name">mq9</span>
        </h1>

        <p class="mq9-title-sub">{{ t('AI Agent 通信层', 'Communication Layer for AI Agents') }}</p>

        <p class="mq9-hero-desc">
          {{ t(
            'Agent 之间的消息，发出去就不会丢。mq9 是专为 AI Agent 设计的邮箱系统——离线照样收，优先级自动排，任何 NATS 客户端直连。',
            'Messages between Agents are never lost. mq9 is a mailbox system built for AI Agents — offline delivery, automatic priority ordering, any NATS client connects directly.'
          ) }}
        </p>

        <div class="mq9-hero-actions">
          <a class="mq9-btn-primary" href="https://mq9.robustmq.com/" target="_blank" rel="noopener">
            {{ t('快速开始', 'Get Started') }} →
          </a>
          <a class="mq9-btn-ghost" :href="t('/zh/mq9/Overview', '/en/mq9/Overview')">
            {{ t('查看文档', 'Documentation') }}
          </a>
          <a class="mq9-btn-ghost" href="https://github.com/robustmq/robustmq" target="_blank" rel="noopener">
            GitHub
          </a>
        </div>

        <div class="mq9-hero-note">
          {{ t('基于 NATS 协议 · 支持 NATS 客户端 / RobustMQ SDK / LangChain / MCP Server', 'Built on NATS · NATS clients / RobustMQ SDK / LangChain / MCP Server') }}
        </div>
      </div>
    </section>

    <!-- ── PROBLEM ── -->
    <section class="mq9-section">
      <div class="mq9-section-inner">
        <div class="mq9-problem">
          <div class="mq9-problem-text">
            <h2 class="mq9-section-title">{{ t('今天的问题', 'The Problem Today') }}</h2>
            <p>{{ t('人和人之间有飞书、钉钉、邮件。我发出去，你空了来看，不需要同时在线。', 'People have email, Slack, WeChat. You send a message, the recipient reads it when available — no need to be online at the same time.') }}</p>
            <p>{{ t('Agent 和 Agent 之间呢？', 'What about Agent to Agent?') }}</p>
            <p class="mq9-problem-highlight">{{ t('今天，Agent A 给 Agent B 发消息，B 不在线，消息直接丢了。每个团队都在用 Redis pub/sub、轮询数据库、自研队列绕过这个问题。能用，但都是绕路。', 'Today, Agent A sends a message to Agent B. B is offline — the message is gone. Every team works around this with Redis pub/sub, database polling, or homegrown queues. It works, but it\'s a workaround.') }}</p>
            <p class="mq9-solution-line">{{ t('mq9 直接解决它：发出去，对方上线自然收到。', 'mq9 solves it directly: send a message, the recipient gets it when they come online.') }}</p>
          </div>
          <div class="mq9-problem-compare">
            <div class="mq9-compare-item mq9-compare-bad">
              <div class="mq9-compare-label">{{ t('今天', 'Today') }}</div>
              <div class="mq9-compare-content">
                <div class="mq9-flow-node">Agent A</div>
                <div class="mq9-flow-arrow mq9-flow-bad">→ <span>{{ t('消息丢失', 'message lost') }}</span></div>
                <div class="mq9-flow-node mq9-offline">Agent B <small>{{ t('离线', 'offline') }}</small></div>
              </div>
            </div>
            <div class="mq9-compare-item mq9-compare-good">
              <div class="mq9-compare-label">{{ t('mq9', 'mq9') }}</div>
              <div class="mq9-compare-content">
                <div class="mq9-flow-node">Agent A</div>
                <div class="mq9-flow-arrow mq9-flow-good">→</div>
                <div class="mq9-flow-mailbox">📬 {{ t('邮箱等待', 'mailbox waits') }}</div>
                <div class="mq9-flow-arrow mq9-flow-good">→</div>
                <div class="mq9-flow-node">Agent B <small>{{ t('上线后收到', 'receives on reconnect') }}</small></div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- ── PRIMITIVES ── -->
    <section class="mq9-section mq9-primitives-section">
      <div class="mq9-section-inner mq9-primitives-inner">
        <div class="mq9-section-header mq9-primitives-header">
          <div class="mq9-section-tag">{{ t('核心能力', 'Core Capabilities') }}</div>
          <h2 class="mq9-section-title">{{ t('覆盖 Agent 异步通信的所有场景', 'Covers every async Agent communication scenario') }}</h2>
        </div>
        <div class="mq9-primitives">
          <div v-for="p in primitives" :key="p.title" class="mq9-primitive" :style="{'--pc': p.color}">
            <div class="mq9-primitive-header">
              <span class="mq9-primitive-icon">{{ p.icon }}</span>
              <div>
                <h3 class="mq9-primitive-title">{{ p.title }}</h3>
                <p class="mq9-primitive-subtitle">{{ p.subtitle }}</p>
              </div>
            </div>
            <pre class="mq9-code"><code>{{ p.code }}</code></pre>
          </div>
        </div>
      </div>
    </section>

    <!-- ── SCENARIOS ── -->
    <section class="mq9-section">
      <div class="mq9-section-inner">
        <div class="mq9-section-header">
          <div class="mq9-section-tag">{{ t('真实场景', 'Real Scenarios') }}</div>
          <h2 class="mq9-section-title">{{ t('八个真实使用场景', 'Eight real-world use cases') }}</h2>
        </div>
        <div class="mq9-scenarios">
          <div v-for="s in scenarios" :key="s.num" class="mq9-scenario">
            <div class="mq9-scenario-num">{{ s.num }}</div>
            <div>
              <h3 class="mq9-scenario-title">{{ s.title }}</h3>
              <p class="mq9-scenario-desc">{{ s.desc }}</p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- ── SDK ── -->
    <section class="mq9-section mq9-sdk-section">
      <div class="mq9-section-inner">
        <div class="mq9-section-header">
          <div class="mq9-section-tag">{{ t('接入方式', 'Integration') }}</div>
          <h2 class="mq9-section-title">{{ t('三种接入方式，按需选择', 'Three ways to connect — pick what fits') }}</h2>
        </div>
        <div class="mq9-sdk-cards">
          <!-- Card 1: Native NATS -->
          <div class="mq9-sdk-card">
            <div class="mq9-sdk-card-icon">🔌</div>
            <h3 class="mq9-sdk-card-title">{{ t('原生 NATS 客户端', 'Native NATS Client') }}</h3>
            <p class="mq9-sdk-card-desc">{{ t('mq9 基于 NATS 协议。任何语言的 NATS 客户端直接就是 mq9 的客户端，零依赖，零学习成本。', 'mq9 is built on NATS. Any NATS client in any language works out of the box — zero extra dependencies.') }}</p>
            <div class="mq9-langs">
              <span v-for="l in ['Go', 'Python', 'Rust', 'Java', 'JavaScript', 'C#', 'Ruby', 'Elixir']" :key="l" class="mq9-lang">{{ l }}</span>
            </div>
          </div>
          <!-- Card 2: RobustMQ SDK -->
          <div class="mq9-sdk-card mq9-sdk-card-featured">
            <div class="mq9-sdk-card-icon">📦</div>
            <h3 class="mq9-sdk-card-title">{{ t('RobustMQ SDK', 'RobustMQ SDK') }}</h3>
            <p class="mq9-sdk-card-desc">{{ t('官方 SDK，六种语言统一 API，类型安全，异步优先，开箱即用。', 'Official SDK — six languages, unified API, type-safe, async-first.') }}</p>
            <div class="mq9-sdk-installs">
              <code>pip install robustmq</code>
              <code>go get robustmq-sdk/go</code>
              <code>npm install @robustmq/sdk</code>
              <code>cargo add robustmq</code>
            </div>
          </div>
          <!-- Card 3: LangChain / LangGraph -->
          <div class="mq9-sdk-card">
            <div class="mq9-sdk-card-icon">🤖</div>
            <h3 class="mq9-sdk-card-title">{{ t('AI 框架集成', 'AI Framework Integration') }}</h3>
            <p class="mq9-sdk-card-desc">{{ t('官方 LangChain 工具包，6 个工具覆盖全部 mq9 操作，直接接入 LangChain Agent 和 LangGraph 工作流。', 'Official LangChain toolkit — 6 tools covering all mq9 operations, plug directly into LangChain Agents and LangGraph workflows.') }}</p>
            <div class="mq9-sdk-installs">
              <code>pip install langchain-mq9</code>
            </div>
            <div class="mq9-sdk-badges">
              <span class="mq9-sdk-badge">LangChain</span>
              <span class="mq9-sdk-badge">LangGraph</span>
              <span class="mq9-sdk-badge">MCP Server</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- ── RELATIONSHIP ── -->
    <section class="mq9-section">
      <div class="mq9-section-inner">
        <div class="mq9-section-header">
          <div class="mq9-section-tag">{{ t('与 RobustMQ 的关系', 'Relationship with RobustMQ') }}</div>
          <h2 class="mq9-section-title">{{ t('第五个原生协议，同一套存储', 'Fifth native protocol, same unified storage') }}</h2>
          <p class="mq9-section-desc">{{ t('mq9 与 MQTT、Kafka、NATS、AMQP 并列，共享同一套统一存储架构。部署一个 RobustMQ，五个协议全部就位。', 'mq9 sits alongside MQTT, Kafka, NATS, and AMQP, sharing the same unified storage layer. Deploy one RobustMQ — all five protocols are ready.') }}</p>
        </div>
        <div class="mq9-protocols">
          <div class="mq9-protocol-item" v-for="p in [
            { name: 'MQTT', desc: t('IoT 设备', 'IoT Devices'), color: '#10b981' },
            { name: 'Kafka', desc: t('数据流管道', 'Streaming Pipelines'), color: '#f59e0b' },
            { name: 'NATS', desc: t('轻量 Pub/Sub', 'Lightweight Pub/Sub'), color: '#3b82f6' },
            { name: 'AMQP', desc: t('企业消息', 'Enterprise Messaging'), color: '#ec4899' },
            { name: 'mq9', desc: t('AI Agent 通信', 'AI Agent Communication'), color: '#a855f7', highlight: true },
          ]" :key="p.name" :class="{ 'mq9-protocol-highlight': p.highlight }" :style="{'--pc': p.color}">
            <span class="mq9-protocol-name">{{ p.name }}</span>
            <span class="mq9-protocol-desc">{{ p.desc }}</span>
          </div>
          <div class="mq9-protocol-storage">
            <span>{{ t('统一存储层', 'Unified Storage Layer') }}</span>
          </div>
        </div>
      </div>
    </section>

    <!-- ── CTA ── -->
    <section class="mq9-section mq9-cta-section">
      <div class="mq9-section-inner">
        <div class="mq9-cta">
          <h2 class="mq9-cta-title">{{ t('开始构建', 'Start Building') }}</h2>
          <p class="mq9-cta-desc">{{ t('单机部署，一行命令，Agent 邮箱就绪。', 'Single-node deployment, one command, Agent mailbox ready.') }}</p>
          <pre class="mq9-code mq9-cta-code"><code>curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
broker-server start

# Create a mailbox — returns mail_address
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":3600}'

# Send (default priority, no suffix — works even if recipient is offline)
nats pub '$mq9.AI.MAILBOX.{mail_address}' '{"msg_id":"msg-1","from":"...","type":"task","ts":1234567890}'

# Send critical (highest priority, persisted)
nats pub '$mq9.AI.MAILBOX.{mail_address}.critical' '{"msg_id":"msg-2","type":"abort"}'</code></pre>
          <div class="mq9-cta-links">
            <a class="mq9-btn-primary" :href="isZh ? '/zh/OverView/What-is-RobustMQ' : '/en/OverView/What-is-RobustMQ'">{{ t('查看文档', 'Read the Docs') }}</a>
            <a class="mq9-btn-ghost" href="https://github.com/robustmq/robustmq" target="_blank" rel="noopener">GitHub</a>
          </div>
        </div>
      </div>
    </section>

  </div>
</template>

<style scoped>
.mq9-page {
  min-height: 100vh;
  background: #07070d;
  color: #e2e8f0;
  font-family: inherit;
}

/* ── Hero ── */
.mq9-hero {
  position: relative;
  padding: 100px 24px 80px;
  text-align: center;
  overflow: hidden;
}
.mq9-hero-bg {
  position: absolute;
  inset: 0;
  pointer-events: none;
}
.mq9-orb {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.18;
}
.mq9-orb-1 {
  width: 500px; height: 500px;
  background: radial-gradient(circle, #a855f7, transparent);
  top: -100px; left: 50%;
  transform: translateX(-50%);
}
.mq9-orb-2 {
  width: 300px; height: 300px;
  background: radial-gradient(circle, #7c3aed, transparent);
  bottom: 0; right: 10%;
}
.mq9-hero-inner {
  position: relative;
  max-width: 760px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  align-items: center;
}
.mq9-badge {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 16px;
  border-radius: 20px;
  border: 1px solid rgba(168,85,247,0.3);
  background: rgba(168,85,247,0.08);
  color: #c084fc;
  font-size: 12px;
  margin-bottom: 28px;
}
.mq9-badge-dot {
  width: 6px; height: 6px;
  border-radius: 50%;
  background: #a855f7;
  animation: pulse 2s infinite;
}
@keyframes pulse {
  0%,100% { opacity:1; transform:scale(1); }
  50% { opacity:0.5; transform:scale(1.4); }
}
.mq9-title {
  margin: 0 0 12px;
  line-height: 1;
}
.mq9-title-name {
  font-size: clamp(72px, 14vw, 120px);
  font-weight: 900;
  letter-spacing: -0.03em;
  background: linear-gradient(135deg, #e879f9 0%, #a855f7 40%, #7c3aed 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
.mq9-title-sub {
  font-size: 18px;
  color: #94a3b8;
  margin: 0 0 20px;
}
.mq9-hero-desc {
  font-size: 16px;
  line-height: 1.7;
  color: #cbd5e1;
  max-width: 600px;
  margin: 0 0 32px;
}
.mq9-hero-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  justify-content: center;
  margin-bottom: 20px;
}
.mq9-btn-primary {
  padding: 11px 28px;
  border-radius: 8px;
  background: linear-gradient(135deg, #a855f7, #7c3aed);
  color: #fff;
  font-weight: 600;
  font-size: 14px;
  text-decoration: none;
  transition: opacity 0.2s, transform 0.15s;
}
.mq9-btn-primary:hover { opacity: 0.88; transform: translateY(-1px); }
.mq9-btn-ghost {
  padding: 11px 28px;
  border-radius: 8px;
  border: 1px solid rgba(168,85,247,0.4);
  color: #c084fc;
  font-weight: 600;
  font-size: 14px;
  text-decoration: none;
  transition: border-color 0.2s, background 0.2s;
}
.mq9-btn-ghost:hover { border-color: rgba(168,85,247,0.8); background: rgba(168,85,247,0.08); }
.mq9-hero-note {
  font-size: 12px;
  color: #64748b;
}

/* ── Section ── */
.mq9-section { padding: 72px 24px; }
.mq9-section-inner { max-width: 1000px; margin: 0 auto; }
.mq9-section-header { text-align: center; margin-bottom: 48px; }
.mq9-section-tag {
  display: inline-block;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: #a855f7;
  padding: 4px 12px;
  border-radius: 20px;
  border: 1px solid rgba(168,85,247,0.3);
  background: rgba(168,85,247,0.08);
  margin-bottom: 14px;
}
.mq9-section-title {
  font-size: clamp(22px, 4vw, 32px);
  font-weight: 700;
  color: #f1f5f9;
  margin: 0 0 12px;
}
.mq9-section-desc {
  font-size: 15px;
  color: #94a3b8;
  max-width: 600px;
  margin: 0 auto;
  line-height: 1.6;
}

/* ── Problem ── */
.mq9-problem {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 48px;
  align-items: center;
}
.mq9-problem-text p { color: #94a3b8; line-height: 1.7; margin: 0 0 12px; font-size: 15px; }
.mq9-problem-highlight { color: #cbd5e1 !important; }
.mq9-solution-line { color: #c084fc !important; font-weight: 600; }
.mq9-problem-compare { display: flex; flex-direction: column; gap: 16px; }
.mq9-compare-item {
  padding: 16px 20px;
  border-radius: 12px;
  border: 1px solid rgba(255,255,255,0.06);
}
.mq9-compare-bad { background: rgba(239,68,68,0.06); border-color: rgba(239,68,68,0.2); }
.mq9-compare-good { background: rgba(168,85,247,0.06); border-color: rgba(168,85,247,0.2); }
.mq9-compare-label { font-size: 11px; font-weight: 700; color: #64748b; margin-bottom: 10px; text-transform: uppercase; letter-spacing: 0.08em; }
.mq9-compare-content { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; font-size: 13px; }
.mq9-flow-node { padding: 4px 10px; border-radius: 6px; background: rgba(255,255,255,0.06); color: #e2e8f0; }
.mq9-offline small { color: #ef4444; display: block; font-size: 10px; }
.mq9-flow-bad { color: #ef4444; }
.mq9-flow-bad span { font-size: 11px; }
.mq9-flow-good { color: #a855f7; }
.mq9-flow-mailbox { padding: 4px 10px; border-radius: 6px; background: rgba(168,85,247,0.15); color: #c084fc; font-size: 13px; }

/* ── Primitives ── */
.mq9-primitives-section { background: rgba(255,255,255,0.015); }
.mq9-primitives-header { text-align: center; }
.mq9-primitives { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin-left: -150px; }
.mq9-primitive {
  padding: 20px;
  border-radius: 14px;
  border: 1px solid rgba(255,255,255,0.07);
  background: rgba(255,255,255,0.025);
  display: flex;
  flex-direction: column;
  gap: 12px;
  transition: border-color 0.2s;
}
.mq9-primitive:hover { border-color: var(--pc, #a855f7); }
.mq9-primitive-header { display: flex; align-items: flex-start; gap: 12px; }
.mq9-primitive-icon { font-size: 28px; flex-shrink: 0; }
.mq9-primitive-title { font-size: 17px; font-weight: 700; color: #f1f5f9; margin: 0 0 2px; }
.mq9-primitive-subtitle { font-size: 12px; color: #64748b; margin: 0; }
.mq9-primitive-desc { font-size: 13px; color: #94a3b8; line-height: 1.6; margin: 0; }
.mq9-code {
  background: rgba(0,0,0,0.4);
  border: 1px solid rgba(255,255,255,0.07);
  border-radius: 8px;
  padding: 14px 16px;
  font-size: 12px;
  line-height: 1.6;
  color: #94a3b8;
  overflow-x: auto;
  margin: 0;
  white-space: pre;
}
.mq9-code code { font-family: 'JetBrains Mono', 'Fira Code', monospace; background: none; }

/* ── Scenarios ── */
.mq9-scenarios { display: grid; grid-template-columns: repeat(2, 1fr); gap: 20px; }
.mq9-scenario {
  display: flex;
  gap: 16px;
  padding: 20px;
  border-radius: 12px;
  border: 1px solid rgba(255,255,255,0.06);
  background: rgba(255,255,255,0.02);
}
.mq9-scenario-num {
  font-size: 28px;
  font-weight: 900;
  color: rgba(168,85,247,0.25);
  line-height: 1;
  flex-shrink: 0;
  font-variant-numeric: tabular-nums;
}
.mq9-scenario-title { font-size: 15px; font-weight: 600; color: #e2e8f0; margin: 0 0 6px; }
.mq9-scenario-desc { font-size: 13px; color: #64748b; line-height: 1.6; margin: 0; }

/* ── SDK ── */
.mq9-sdk-section { background: rgba(168,85,247,0.04); }
.mq9-sdk-cards {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
  margin-top: 12px;
}
@media (max-width: 900px) {
  .mq9-sdk-cards { grid-template-columns: 1fr; }
}
.mq9-sdk-card {
  background: rgba(255,255,255,0.03);
  border: 1px solid rgba(255,255,255,0.08);
  border-radius: 16px;
  padding: 28px 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.mq9-sdk-card-featured {
  border-color: rgba(168,85,247,0.4);
  background: rgba(168,85,247,0.06);
}
.mq9-sdk-card-icon { font-size: 28px; }
.mq9-sdk-card-title { font-size: 17px; font-weight: 700; color: #e2e8f0; margin: 0; }
.mq9-sdk-card-desc { font-size: 14px; color: #94a3b8; line-height: 1.6; margin: 0; flex: 1; }
.mq9-langs { display: flex; flex-wrap: wrap; gap: 8px; }
.mq9-lang {
  padding: 4px 12px;
  border-radius: 20px;
  border: 1px solid rgba(168,85,247,0.25);
  background: rgba(168,85,247,0.07);
  color: #c084fc;
  font-size: 12px;
  font-weight: 600;
}
.mq9-sdk-installs {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.mq9-sdk-installs code {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  background: rgba(0,0,0,0.3);
  border: 1px solid rgba(255,255,255,0.07);
  border-radius: 6px;
  padding: 5px 10px;
  color: #a5f3fc;
  display: block;
}
.mq9-sdk-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.mq9-sdk-badge {
  padding: 3px 10px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 700;
  background: rgba(16,185,129,0.12);
  border: 1px solid rgba(16,185,129,0.3);
  color: #6ee7b7;
}

/* ── Protocols ── */
.mq9-protocols {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  justify-content: center;
  align-items: stretch;
  margin-top: 8px;
}
.mq9-protocol-item {
  flex: 1;
  min-width: 130px;
  padding: 20px 16px;
  border-radius: 12px;
  border: 1px solid rgba(255,255,255,0.07);
  background: rgba(255,255,255,0.025);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  text-align: center;
  transition: border-color 0.2s;
}
.mq9-protocol-highlight {
  border-color: rgba(168,85,247,0.5) !important;
  background: rgba(168,85,247,0.08) !important;
}
.mq9-protocol-name { font-size: 16px; font-weight: 700; color: var(--pc, #e2e8f0); }
.mq9-protocol-desc { font-size: 12px; color: #64748b; }
.mq9-protocol-storage {
  width: 100%;
  padding: 14px;
  border-radius: 10px;
  background: rgba(168,85,247,0.06);
  border: 1px dashed rgba(168,85,247,0.3);
  text-align: center;
  font-size: 13px;
  color: #94a3b8;
  margin-top: 4px;
}

/* ── CTA ── */
.mq9-cta-section { background: rgba(168,85,247,0.04); }
.mq9-cta { text-align: center; max-width: 640px; margin: 0 auto; }
.mq9-cta-title { font-size: 28px; font-weight: 700; color: #f1f5f9; margin: 0 0 12px; }
.mq9-cta-desc { font-size: 15px; color: #94a3b8; margin: 0 0 24px; }
.mq9-cta-code { text-align: left; margin-bottom: 28px; }
.mq9-cta-links { display: flex; gap: 12px; justify-content: center; flex-wrap: wrap; }

/* ── Responsive ── */
@media (max-width: 768px) {
  .mq9-hero { padding: 72px 20px 60px; }
  .mq9-problem { grid-template-columns: 1fr; }
  .mq9-primitives { grid-template-columns: 1fr; }
  .mq9-scenarios { grid-template-columns: 1fr; }
  .mq9-protocols { flex-direction: column; }
  .mq9-section { padding: 48px 20px; }
}
</style>
