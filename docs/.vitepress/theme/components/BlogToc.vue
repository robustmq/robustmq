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
  left: max(1rem, calc((100vw - 1400px) / 2 + 1rem));
  top: 80px;
  width: 240px;
  max-height: calc(100vh - 100px);
  overflow-y: auto;
  z-index: 10;
}

.toc-container {
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.toc-title {
  margin: 0 0 1rem 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--vp-c-text-1);
  padding-bottom: 0.75rem;
  border-bottom: 2px solid rgb(147, 51, 234);
}

.toc-nav :deep(.toc-list) {
  list-style: none;
  padding: 0;
  margin: 0;
}

.toc-nav :deep(.toc-item) {
  margin: 0;
}

.toc-nav :deep(.toc-item) {
  margin: 0.25rem 0;
}

.toc-nav :deep(.toc-item-h2 a) {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0.6rem 0.75rem;
  color: var(--vp-c-text-2);
  text-decoration: none;
  border-radius: 8px;
  transition: all 0.2s ease;
  font-size: 0.9rem;
  line-height: 1.4;
  font-weight: 500;
}

.toc-nav :deep(.toc-item-h2 .toc-number) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  height: 24px;
  background: rgba(147, 51, 234, 0.15);
  color: rgb(147, 51, 234);
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 600;
  padding: 0 6px;
  flex-shrink: 0;
}

.toc-nav :deep(.toc-item-h2 .toc-text) {
  flex: 1;
}

.toc-nav :deep(.toc-item-h3 a) {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0.5rem 0.75rem 0.5rem 1rem;
  color: var(--vp-c-text-3);
  text-decoration: none;
  border-radius: 8px;
  transition: all 0.2s ease;
  font-size: 0.85rem;
  line-height: 1.4;
}

.toc-nav :deep(.toc-item-h3 .toc-number) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 40px;
  color: var(--vp-c-text-3);
  font-size: 0.75rem;
  font-weight: 500;
  flex-shrink: 0;
}

.toc-nav :deep(.toc-item-h3 .toc-text) {
  flex: 1;
}

.toc-nav :deep(.toc-item a:hover) {
  background: rgba(147, 51, 234, 0.1);
  color: rgb(147, 51, 234);
  transform: translateX(4px);
}

.toc-nav :deep(.toc-item-h2 a:hover .toc-number) {
  background: rgba(147, 51, 234, 0.25);
  color: rgb(126, 34, 206);
}

.toc-nav :deep(.toc-item-h3 a:hover .toc-number) {
  color: rgb(147, 51, 234);
}

.toc-nav :deep(.no-toc) {
  color: var(--vp-c-text-3);
  font-size: 0.9rem;
  margin: 0;
}

/* 滚动条样式 */
.blog-toc-sidebar::-webkit-scrollbar {
  width: 4px;
}

.blog-toc-sidebar::-webkit-scrollbar-track {
  background: transparent;
}

.blog-toc-sidebar::-webkit-scrollbar-thumb {
  background: var(--vp-c-divider);
  border-radius: 2px;
}

/* 响应式 */
@media (max-width: 1400px) {
  .blog-toc-sidebar {
    left: 1rem;
  }
}

@media (max-width: 1200px) {
  .blog-toc-sidebar {
    display: none;
  }
}
</style>
