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
    subtitle: t('点对点异步投递', 'Async point-to-point'),
    desc: t(
      '每个 Agent 有自己的邮箱。发件方不需要知道对方在不在线，直接发到邮箱，消息在那里等着。对方上线后自动收到。',
      'Every Agent has its own inbox. Send without knowing if the recipient is online — the message waits. The Agent receives it when it comes back.'
    ),
    color: '#a855f7',
    code: `nats req '$mq9.AI.MAILBOX.CREATE' '{}'
# → {"agent_id": "agt-001", "token": "tok-xxx"}

nats pub '$mq9.AI.INBOX.agt-001.normal' \\
  '{"from":"agt-002","payload":"task done"}'

nats sub '$mq9.AI.INBOX.agt-001.*'`,
  },
  {
    icon: '📡',
    title: t('广播', 'Broadcast'),
    subtitle: t('一发多收', 'Publish once, many receive'),
    desc: t(
      'Agent 广播一个事件，不需要知道谁在监听。关心的 Agent 自己来订阅，新 Agent 加入自动感知，拓扑变化零成本。',
      'An Agent broadcasts an event without knowing who is listening. Interested Agents subscribe themselves. Topology changes are zero-cost.'
    ),
    color: '#7c3aed',
    code: `# Broadcast an event
nats pub '$mq9.AI.BROADCAST.task.available' \\
  '{"task_id":"t-001","type":"analysis"}'

# Subscribe to all task events
nats sub '$mq9.AI.BROADCAST.task.*'

# Subscribe to anomalies across all domains
nats sub '$mq9.AI.BROADCAST.*.anomaly'`,
  },
  {
    icon: '⚡',
    title: t('优先级', 'Priority'),
    subtitle: t('紧急消息先处理', 'Critical messages first'),
    desc: t(
      '邮箱支持三个优先级：urgent、normal、notify。紧急指令永远不会被普通任务淹没。同一优先级保证 FIFO。',
      'Three priority levels: urgent, normal, notify. Critical instructions are never buried under routine tasks. FIFO within each level.'
    ),
    color: '#6d28d9',
    code: `# Urgent — processed first
nats pub '$mq9.AI.INBOX.agt-001.urgent' \\
  '{"type":"stop","reason":"anomaly"}'

# Normal — routine tasks
nats pub '$mq9.AI.INBOX.agt-001.normal' \\
  '{"type":"task","payload":"..."}'

# Notify — background info, no persistence
nats pub '$mq9.AI.INBOX.agt-001.notify' \\
  '{"type":"heartbeat"}'`,
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
    desc: t('一行通配符订阅，主 Agent 自动感知所有子 Agent 的上线、运行、消亡，不需要注册或注销。', 'One wildcard subscription — the orchestrator automatically tracks every sub-Agent coming online, running, or dying. No registration needed.'),
  },
  {
    num: '03',
    title: t('任务广播竞争消费', 'Task broadcast with competing consumers'),
    desc: t('主 Agent 广播任务，有能力的 Worker 自己来抢。queue group 保证每个任务只被一个 Worker 处理。', 'Orchestrator broadcasts a task, capable Workers compete to claim it. Queue group ensures each task is handled by exactly one Worker.'),
  },
  {
    num: '04',
    title: t('边缘 Agent 离线积压', 'Edge Agent offline buffering'),
    desc: t('云端给边缘 Agent 发指令，边缘不在线消息等着，联网后按优先级处理。', 'Cloud sends instructions to an edge Agent that\'s offline. Messages wait, then are processed by priority when the edge reconnects.'),
  },
  {
    num: '05',
    title: t('人机混合工作流', 'Human-in-the-loop workflows'),
    desc: t('Agent 遇到需要人工判断的节点，发审批请求到人类邮箱，审批结果发回 Agent 邮箱，流程继续。', 'Agent hits a decision point requiring human judgment, sends to a human\'s inbox, and resumes when the approval comes back.'),
  },
  {
    num: '06',
    title: t('异步 Request-Reply', 'Async Request-Reply'),
    desc: t('Agent A 发请求到 Agent B 邮箱。B 不在线请求等着，上线处理完发响应到 A 邮箱，通过 correlation_id 匹配。', 'Agent A sends a request to Agent B\'s inbox. B processes when online and replies to A\'s inbox, matched by correlation_id.'),
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
            'Agent 需要邮箱。发出去，对方上线自然收到。mq9 是 RobustMQ 为 AI Agent 设计的通信层，核心是 Agent 邮箱——点对点异步投递、广播、优先级队列，任何 NATS 客户端直接接入。',
            'Agents need a mailbox. Send a message, the recipient gets it when they come online. mq9 is RobustMQ\'s communication layer for AI Agents — async mailbox, broadcast, priority queue. Any NATS client connects directly.'
          ) }}
        </p>

        <div class="mq9-hero-actions">
          <a class="mq9-btn-primary" :href="isZh ? '/zh/OverView/What-is-RobustMQ' : '/en/OverView/What-is-RobustMQ'">
            {{ t('快速开始', 'Get Started') }} →
          </a>
          <a class="mq9-btn-ghost" href="https://github.com/robustmq/robustmq" target="_blank" rel="noopener">
            GitHub
          </a>
        </div>

        <div class="mq9-hero-note">
          {{ t('基于 NATS 协议 · 无需额外 SDK · 部署一个 RobustMQ 即可使用', 'Built on NATS · No extra SDK · Ships with every RobustMQ instance') }}
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
      <div class="mq9-section-inner">
        <div class="mq9-section-header">
          <div class="mq9-section-tag">{{ t('三个核心原语', 'Three Core Primitives') }}</div>
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
            <p class="mq9-primitive-desc">{{ p.desc }}</p>
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
          <h2 class="mq9-section-title">{{ t('六个真实使用场景', 'Six real-world use cases') }}</h2>
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

    <!-- ── NO NEW SDK ── -->
    <section class="mq9-section mq9-sdk-section">
      <div class="mq9-section-inner">
        <div class="mq9-sdk-box">
          <h2 class="mq9-section-title">{{ t('不需要新 SDK', 'No New SDK Required') }}</h2>
          <p class="mq9-sdk-desc">{{ t('mq9 基于 NATS 协议。所有语言的 NATS 客户端直接就是 mq9 的客户端。NATS 生态有多大，mq9 的接入生态就有多大。', 'mq9 is built on the NATS protocol. Any NATS client in any language is already an mq9 client. The entire NATS ecosystem works out of the box.') }}</p>
          <div class="mq9-langs">
            <span v-for="lang in ['Go', 'Python', 'Rust', 'Java', 'JavaScript', 'C#', 'Ruby', 'Elixir']" :key="lang" class="mq9-lang">{{ lang }}</span>
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

# Agent gets a mailbox
nats req '$mq9.AI.MAILBOX.CREATE' '{}'</code></pre>
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
.mq9-primitives { display: grid; grid-template-columns: repeat(3, 1fr); gap: 24px; margin-left: -160px; }
.mq9-primitive {
  padding: 28px;
  border-radius: 16px;
  border: 1px solid rgba(255,255,255,0.07);
  background: rgba(255,255,255,0.025);
  display: flex;
  flex-direction: column;
  gap: 14px;
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
.mq9-sdk-box { text-align: center; }
.mq9-sdk-desc { font-size: 15px; color: #94a3b8; max-width: 560px; margin: 12px auto 28px; line-height: 1.6; }
.mq9-langs { display: flex; flex-wrap: wrap; gap: 10px; justify-content: center; }
.mq9-lang {
  padding: 6px 16px;
  border-radius: 20px;
  border: 1px solid rgba(168,85,247,0.25);
  background: rgba(168,85,247,0.07);
  color: #c084fc;
  font-size: 13px;
  font-weight: 600;
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
