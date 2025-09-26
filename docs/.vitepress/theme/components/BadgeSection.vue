<template>
  <div class="hero-badges" v-if="mounted">
    <img alt="Latest Release" src="https://img.shields.io/github/v/release/robustmq/robustmq?style=flat">
    <img alt="License" src="https://img.shields.io/github/license/robustmq/robustmq?style=flat">
    <img alt="GitHub issues" src="https://img.shields.io/github/issues/robustmq/robustmq?style=flat">
    <img alt="GitHub stars" src="https://img.shields.io/github/stars/robustmq/robustmq?style=flat">
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'

const mounted = ref(false)

onMounted(() => {
  mounted.value = true
})
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
