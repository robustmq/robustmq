<template>
  <div class="blog-home">

    <!-- ── Page Header ── -->
    <div class="blog-header">
      <div class="blog-header-inner">
        <div class="blog-header-tag">BLOG</div>
        <h1 class="blog-header-title">{{ lang === 'zh' ? '技术博客' : 'Technical Blog' }}</h1>
        <p class="blog-header-desc">
          {{ lang === 'zh'
            ? '记录 RobustMQ 架构设计、工程实践与技术思考'
            : 'Architecture design, engineering practice and technical insights from the RobustMQ team' }}
        </p>
      </div>
      <div class="blog-header-glow"></div>
    </div>

    <div class="blog-body">

      <!-- ── Pinned Banner ── -->
      <a v-if="pinnedPost" :href="pinnedPost.url" class="pinned-banner">
        <div class="pinned-left">
          <span class="pinned-icon">📌</span>
          <span class="pinned-label">{{ lang === 'zh' ? '置顶公告' : 'Pinned' }}</span>
        </div>
        <div class="pinned-title">{{ pinnedPost.title }}</div>
        <div class="pinned-arrow">→</div>
        <div class="pinned-glow"></div>
      </a>

      <!-- ── Featured (latest post) ── -->
      <a v-if="latestPost" :href="latestPost.url || latestPost.link" class="featured-post">
        <div class="fp-num">{{ String(latestPost.number || totalCount).padStart(2, '0') }}</div>
        <div class="fp-body">
          <div class="fp-meta">
            <span class="fp-badge">{{ lang === 'zh' ? '最新' : 'Latest' }}</span>
            <span class="fp-date">{{ latestPost.date || '' }}</span>
          </div>
          <h2 class="fp-title">{{ latestPost.title }}</h2>
          <p class="fp-excerpt">{{ latestPost.excerpt }}</p>
          <span class="fp-read">{{ lang === 'zh' ? '阅读全文' : 'Read more' }} →</span>
        </div>
        <div class="fp-glow"></div>
      </a>

      <!-- ── Post Grid ── -->
      <div class="post-grid">
        <a
          v-for="(post, i) in restPosts"
          :key="post.url || post.link"
          :href="post.url || post.link"
          class="post-card"
        >
          <div class="pc-num">{{ String(post.number || (totalCount - 1 - i)).padStart(2, '0') }}</div>
          <div class="pc-body">
            <div class="pc-meta">
              <span v-for="tag in (post.tags || []).slice(0, 2)" :key="tag" class="pc-tag">{{ tag }}</span>
              <span class="pc-date">{{ post.date || '' }}</span>
            </div>
            <h3 class="pc-title">{{ post.title }}</h3>
            <p class="pc-excerpt">{{ post.excerpt }}</p>
          </div>
          <div class="pc-arrow">→</div>
          <div class="pc-glow"></div>
        </a>
      </div>

    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'

const { lang } = useData()

const props = defineProps({
  blogData: {
    type: Object,
    default: () => ({ zh: [], en: [] })
  }
})

const allPosts = computed(() => {
  if (props.blogData?.zh && props.blogData?.en) {
    return lang.value === 'zh' ? props.blogData.zh : props.blogData.en
  }
  return []
})

const pinnedPost   = computed(() => allPosts.value.find(p => p.pinned) || null)
const unpinned     = computed(() => allPosts.value.filter(p => !p.pinned))
const totalCount   = computed(() => allPosts.value.length)
const latestPost   = computed(() => unpinned.value[0] || null)
const restPosts    = computed(() => unpinned.value.slice(1))
</script>

<style scoped>
/* ── Root ── */
.blog-home {
  width: 100%;
  min-height: 100vh;
}

/* ── Header ── */
.blog-header {
  position: relative;
  overflow: hidden;
  padding: 80px 0 64px;
  text-align: center;
  border-bottom: 1px solid rgba(168, 85, 247, 0.15);
}

.blog-header-inner {
  position: relative;
  z-index: 1;
  max-width: 640px;
  margin: 0 auto;
  padding: 0 24px;
}

.blog-header-tag {
  display: inline-block;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.18em;
  color: #a855f7;
  border: 1px solid rgba(168, 85, 247, 0.35);
  border-radius: 20px;
  padding: 4px 14px;
  margin-bottom: 20px;
  background: rgba(168, 85, 247, 0.08);
}

.blog-header-title {
  font-size: clamp(32px, 5vw, 52px);
  font-weight: 800;
  letter-spacing: -0.03em;
  line-height: 1.1;
  margin: 0 0 16px;
  background: linear-gradient(135deg, #fdf4ff 0%, #e9d5ff 50%, #a855f7 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.blog-header-desc {
  font-size: 15px;
  color: var(--vp-c-text-2);
  line-height: 1.7;
  margin: 0;
}

/* Background glow */
.blog-header-glow {
  position: absolute;
  width: 600px;
  height: 300px;
  top: -80px;
  left: 50%;
  transform: translateX(-50%);
  background: radial-gradient(ellipse, rgba(124, 58, 237, 0.18) 0%, transparent 70%);
  filter: blur(60px);
  pointer-events: none;
}

/* ── Body ── */
.blog-body {
  max-width: 1080px;
  margin: 0 auto;
  padding: 48px 24px 80px;
}

/* ── Pinned Banner ── */
.pinned-banner {
  display: flex;
  align-items: center;
  gap: 16px;
  position: relative;
  overflow: hidden;
  background: linear-gradient(90deg,
    rgba(124, 58, 237, 0.12) 0%,
    rgba(168, 85, 247, 0.08) 50%,
    rgba(124, 58, 237, 0.06) 100%);
  border: 1px solid rgba(168, 85, 247, 0.35);
  border-radius: 14px;
  padding: 16px 24px;
  text-decoration: none !important;
  color: inherit;
  margin-bottom: 28px;
  transition: all 0.25s ease;
}

.pinned-banner:hover {
  border-color: rgba(168, 85, 247, 0.65);
  box-shadow: 0 0 0 1px rgba(168, 85, 247, 0.2), 0 8px 32px rgba(124, 58, 237, 0.2);
  transform: translateY(-2px);
  text-decoration: none !important;
}

.pinned-banner:hover .pinned-glow { opacity: 1; }
.pinned-banner:hover .pinned-arrow { transform: translateX(5px); color: #c084fc; }

.pinned-glow {
  position: absolute;
  inset: 0;
  background: radial-gradient(ellipse at 0% 50%, rgba(168, 85, 247, 0.1) 0%, transparent 60%);
  opacity: 0;
  transition: opacity 0.3s;
  pointer-events: none;
}

.pinned-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.pinned-icon {
  font-size: 15px;
  line-height: 1;
}

.pinned-label {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: #a855f7;
  background: rgba(168, 85, 247, 0.12);
  border: 1px solid rgba(168, 85, 247, 0.3);
  border-radius: 20px;
  padding: 3px 10px;
  white-space: nowrap;
}

.pinned-title {
  flex: 1;
  font-size: 14px;
  font-weight: 600;
  color: var(--vp-c-text-1);
  letter-spacing: -0.01em;
  transition: color 0.2s;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pinned-banner:hover .pinned-title { color: #c084fc; }

.pinned-arrow {
  flex-shrink: 0;
  font-size: 16px;
  color: rgba(168, 85, 247, 0.5);
  transition: transform 0.2s, color 0.2s;
}

/* ── Featured Post ── */
.featured-post {
  display: flex;
  align-items: flex-start;
  gap: 32px;
  position: relative;
  overflow: hidden;
  background: rgba(168, 85, 247, 0.04);
  border: 1px solid rgba(168, 85, 247, 0.2);
  border-radius: 20px;
  padding: 40px 44px;
  text-decoration: none !important;
  color: inherit;
  margin-bottom: 48px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.featured-post:hover {
  border-color: rgba(168, 85, 247, 0.5);
  box-shadow: 0 0 0 1px rgba(168, 85, 247, 0.2), 0 20px 60px rgba(124, 58, 237, 0.15);
  transform: translateY(-3px);
  text-decoration: none !important;
}

.featured-post:hover .fp-glow { opacity: 1; }

.fp-glow {
  position: absolute;
  inset: 0;
  background: radial-gradient(ellipse at 10% 50%, rgba(168, 85, 247, 0.08) 0%, transparent 60%);
  opacity: 0;
  transition: opacity 0.4s;
  pointer-events: none;
}

.fp-num {
  flex-shrink: 0;
  font-size: 56px;
  font-weight: 900;
  letter-spacing: -0.05em;
  line-height: 1;
  color: rgba(168, 85, 247, 0.2);
  font-variant-numeric: tabular-nums;
  transition: color 0.3s;
  padding-top: 4px;
}

.featured-post:hover .fp-num {
  color: rgba(168, 85, 247, 0.45);
}

.fp-body {
  flex: 1;
  min-width: 0;
}

.fp-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 14px;
}

.fp-badge {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #a855f7;
  background: rgba(168, 85, 247, 0.12);
  border: 1px solid rgba(168, 85, 247, 0.3);
  border-radius: 20px;
  padding: 3px 10px;
}

.fp-date {
  font-size: 13px;
  color: var(--vp-c-text-3);
}

.fp-title {
  font-size: clamp(20px, 2.5vw, 28px);
  font-weight: 700;
  line-height: 1.35;
  letter-spacing: -0.02em;
  color: var(--vp-c-text-1);
  margin: 0 0 14px;
  transition: color 0.2s;
}

.featured-post:hover .fp-title { color: #c084fc; }

.fp-excerpt {
  font-size: 14px;
  color: var(--vp-c-text-2);
  line-height: 1.7;
  margin: 0 0 20px;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.fp-read {
  font-size: 13px;
  font-weight: 600;
  color: #a855f7;
  letter-spacing: 0.01em;
  transition: gap 0.2s;
}

/* ── Post Grid ── */
.post-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
}

.post-card {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 20px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--vp-c-divider);
  border-radius: 16px;
  padding: 28px 28px 28px 28px;
  text-decoration: none !important;
  color: inherit;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.post-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, #7c3aed, #a855f7);
  transform: scaleX(0);
  transform-origin: left;
  transition: transform 0.3s ease;
  border-radius: 2px 2px 0 0;
}

.post-card:hover::before { transform: scaleX(1); }

.post-card:hover {
  border-color: rgba(168, 85, 247, 0.3);
  box-shadow: 0 8px 32px rgba(124, 58, 237, 0.12);
  transform: translateY(-3px);
  text-decoration: none !important;
}

.post-card:hover .pc-glow { opacity: 1; }

.pc-glow {
  position: absolute;
  inset: 0;
  background: radial-gradient(ellipse at 0% 0%, rgba(168, 85, 247, 0.06) 0%, transparent 60%);
  opacity: 0;
  transition: opacity 0.3s;
  pointer-events: none;
}

.pc-num {
  flex-shrink: 0;
  font-size: 28px;
  font-weight: 900;
  letter-spacing: -0.04em;
  color: rgba(168, 85, 247, 0.18);
  line-height: 1;
  padding-top: 3px;
  transition: color 0.25s;
  font-variant-numeric: tabular-nums;
}

.post-card:hover .pc-num { color: rgba(168, 85, 247, 0.4); }

.pc-body {
  flex: 1;
  min-width: 0;
}

.pc-meta {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 10px;
}

.pc-tag {
  font-size: 11px;
  font-weight: 600;
  color: #a855f7;
  background: rgba(168, 85, 247, 0.1);
  border: 1px solid rgba(168, 85, 247, 0.2);
  border-radius: 10px;
  padding: 2px 9px;
  letter-spacing: 0.02em;
}

.pc-date {
  font-size: 12px;
  color: var(--vp-c-text-3);
  margin-left: auto;
}

.pc-title {
  font-size: 15px;
  font-weight: 700;
  line-height: 1.5;
  letter-spacing: -0.01em;
  color: var(--vp-c-text-1);
  margin: 0 0 8px;
  transition: color 0.2s;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.post-card:hover .pc-title { color: #c084fc; }

.pc-excerpt {
  font-size: 13px;
  color: var(--vp-c-text-2);
  line-height: 1.65;
  margin: 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.pc-arrow {
  flex-shrink: 0;
  font-size: 16px;
  color: rgba(168, 85, 247, 0.3);
  padding-top: 2px;
  transition: color 0.2s, transform 0.2s;
  align-self: center;
}

.post-card:hover .pc-arrow {
  color: #a855f7;
  transform: translateX(4px);
}

/* ── Responsive ── */
@media (max-width: 768px) {
  .featured-post {
    flex-direction: column;
    gap: 16px;
    padding: 28px 24px;
  }

  .fp-num { font-size: 36px; }

  .post-grid {
    grid-template-columns: 1fr;
  }

  .blog-header { padding: 56px 0 40px; }
}

@media (max-width: 480px) {
  .blog-body { padding: 32px 16px 60px; }
  .post-card { padding: 20px; }
  .pc-num { display: none; }
}
</style>
