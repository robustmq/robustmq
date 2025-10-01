<template>
  <div class="hero-badges">
    <img alt="Latest Release" src="https://img.shields.io/github/v/release/robustmq/robustmq?style=flat" loading="lazy">
    <img alt="License" src="https://img.shields.io/github/license/robustmq/robustmq?style=flat" loading="lazy">
    <img alt="GitHub issues" src="https://img.shields.io/github/issues/robustmq/robustmq?style=flat" loading="lazy">
    <img alt="GitHub stars" src="https://img.shields.io/github/stars/robustmq/robustmq?style=flat" loading="lazy">
  </div>
</template>

<script setup>
// 移除条件渲染逻辑，确保 SSR 和客户端渲染一致
</script>

<style scoped>
.hero-badges {
  display: flex;
  gap: 8px;
  margin: 0 auto 16px;
  flex-wrap: wrap;
  justify-content: center;
  max-width: 600px;
  padding: 0 24px;
}

.hero-badges img {
  height: 20px;
  transition: transform 0.2s ease;
}

.hero-badges img:hover {
  transform: scale(1.05);
}

@media (max-width: 768px) {
  .hero-badges {
    margin: 0 auto 12px;
    padding: 0 16px;
  }
}
</style>
