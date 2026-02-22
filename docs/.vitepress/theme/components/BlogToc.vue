<template>
  <aside class="blog-toc-sidebar">
    <div class="toc-container">
      <h3 class="toc-title">目录</h3>
      <nav class="toc-nav" v-html="tocHtml"></nav>
    </div>
  </aside>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import { useRoute } from 'vitepress'

const route = useRoute()
const tocHtml = ref('')

const generateToc = () => {
  setTimeout(() => {
    const headers = document.querySelectorAll('.vp-doc h2, .vp-doc h3')
    
    if (headers.length === 0) {
      tocHtml.value = '<p class="no-toc">暂无目录</p>'
      return
    }

    let html = '<ul class="toc-list">'
    let h2Index = 0
    let h3Index = 0
    
    headers.forEach((header, index) => {
      const level = header.tagName.toLowerCase()
      const text = header.textContent
      const id = header.id || `heading-${index}`
      
      if (!header.id) {
        header.id = id
      }

      if (level === 'h2') {
        h2Index++
        h3Index = 0
        const className = 'toc-item toc-item-h2'
        html += `<li class="${className}"><a href="#${id}"><span class="toc-number">${h2Index}</span><span class="toc-text">${text}</span></a></li>`
      } else {
        h3Index++
        const className = 'toc-item toc-item-h3'
        html += `<li class="${className}"><a href="#${id}"><span class="toc-number">${h2Index}.${h3Index}</span><span class="toc-text">${text}</span></a></li>`
      }
    })
    html += '</ul>'
    
    tocHtml.value = html
  }, 200)
}

onMounted(() => {
  generateToc()
})

watch(() => route.path, () => {
  generateToc()
})
</script>

<style scoped>
.blog-toc-sidebar {
  position: fixed;
  /* Align right edge of TOC to just left of the centered 760px content area */
  left: calc(50vw - 380px - 260px);
  top: 120px;
  width: 196px;
  max-height: calc(100vh - 88px);
  overflow-y: auto;
  z-index: 10;
}

.toc-container {
  background: rgba(18, 10, 35, 0.85);
  border: 1px solid rgba(168, 85, 247, 0.18);
  border-radius: 14px;
  padding: 18px 14px;
  backdrop-filter: blur(12px);
}

.toc-title {
  margin: 0 0 12px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: rgba(168, 85, 247, 0.6);
  padding-bottom: 10px;
  border-bottom: 1px solid rgba(168, 85, 247, 0.15);
}

.toc-nav :deep(.toc-list) {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.toc-nav :deep(.toc-item-h2 a) {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  color: var(--vp-c-text-2);
  text-decoration: none;
  border-radius: 7px;
  transition: all 0.18s ease;
  font-size: 12px;
  line-height: 1.45;
  font-weight: 500;
}

.toc-nav :deep(.toc-item-h2 .toc-number) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  height: 20px;
  background: rgba(168, 85, 247, 0.12);
  color: #a855f7;
  border-radius: 5px;
  font-size: 10px;
  font-weight: 700;
  padding: 0 5px;
  flex-shrink: 0;
}

.toc-nav :deep(.toc-item-h2 .toc-text) {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.toc-nav :deep(.toc-item-h3 a) {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px 5px 14px;
  color: var(--vp-c-text-3);
  text-decoration: none;
  border-radius: 7px;
  transition: all 0.18s ease;
  font-size: 11.5px;
  line-height: 1.4;
}

.toc-nav :deep(.toc-item-h3 .toc-number) {
  color: var(--vp-c-text-3);
  font-size: 10px;
  font-weight: 500;
  flex-shrink: 0;
  min-width: 28px;
}

.toc-nav :deep(.toc-item-h3 .toc-text) {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.toc-nav :deep(.toc-item a:hover) {
  background: rgba(168, 85, 247, 0.1);
  color: #c084fc;
  transform: translateX(3px);
}

.toc-nav :deep(.toc-item-h2 a:hover .toc-number) {
  background: rgba(168, 85, 247, 0.22);
  color: #c084fc;
}

.toc-nav :deep(.no-toc) {
  color: var(--vp-c-text-3);
  font-size: 11px;
  margin: 0;
}

/* 滚动条 */
.blog-toc-sidebar::-webkit-scrollbar { width: 3px; }
.blog-toc-sidebar::-webkit-scrollbar-track { background: transparent; }
.blog-toc-sidebar::-webkit-scrollbar-thumb {
  background: rgba(168, 85, 247, 0.3);
  border-radius: 2px;
}

/* 响应式 */
@media (max-width: 1280px) {
  .blog-toc-sidebar { left: 0.5rem; }
}

@media (max-width: 1100px) {
  .blog-toc-sidebar { display: none; }
}
</style>
