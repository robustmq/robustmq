<script setup>
import { onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useData } from 'vitepress'

const route = useRoute()
const { isDark } = useData()

function loadGiscus() {
  const container = document.getElementById('giscus-container')
  if (!container) return

  // 清空旧实例
  container.innerHTML = ''

  const script = document.createElement('script')
  script.src = 'https://giscus.app/client.js'
  script.setAttribute('data-repo', 'robustmq/robustmq')
  script.setAttribute('data-repo-id', 'R_kgDOKC3GIQ')
  script.setAttribute('data-category', 'Announcements')
  script.setAttribute('data-category-id', 'DIC_kwDOKC3GIc4C51sI')
  script.setAttribute('data-mapping', 'pathname')
  script.setAttribute('data-strict', '0')
  script.setAttribute('data-reactions-enabled', '1')
  script.setAttribute('data-emit-metadata', '0')
  script.setAttribute('data-input-position', 'bottom')
  script.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
  script.setAttribute('data-lang', 'zh-CN')
  script.setAttribute('crossorigin', 'anonymous')
  script.async = true
  container.appendChild(script)
}

function updateTheme() {
  const iframe = document.querySelector('iframe.giscus-frame')
  if (!iframe) return
  iframe.contentWindow.postMessage(
    { giscus: { setConfig: { theme: isDark.value ? 'dark' : 'light' } } },
    'https://giscus.app'
  )
}

onMounted(() => loadGiscus())

watch(() => route.path, () => {
  setTimeout(loadGiscus, 300)
})

watch(isDark, updateTheme)
</script>

<template>
  <div style="margin-top: 48px; padding-top: 24px; border-top: 1px solid var(--vp-c-divider);">
    <div id="giscus-container" />
  </div>
</template>
