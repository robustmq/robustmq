---
layout: home
---

<script setup>
import { onMounted } from 'vue'

onMounted(() => {
  // 重定向到英文版本
  if (typeof window !== 'undefined') {
    window.location.href = '/en/'
  }
})
</script>

# Redirecting to English version...

If you are not redirected automatically, [click here to go to the English version](/en/).

如果没有自动跳转，[点击这里进入英文版本](/en/)。