<template>
  <div class="github-stars-wrapper">
    <a 
      class="github-stars-button" 
      href="https://github.com/robustmq/robustmq" 
      target="_blank" 
      rel="noreferrer"
    >
      ⭐ GitHub Stars{{ starCount ? ` (${starCount})` : '' }}
    </a>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'

const starCount = ref('')

// 格式化数字为K格式
const formatStarCount = (count) => {
  if (count >= 1000) {
    return (count / 1000).toFixed(1) + 'K'
  }
  return count.toString()
}

// 获取GitHub star数量
const fetchStarCount = async () => {
  try {
    const response = await fetch('https://api.github.com/repos/robustmq/robustmq')
    if (response.ok) {
      const data = await response.json()
      starCount.value = formatStarCount(data.stargazers_count)
    }
  } catch (error) {
    console.log('Failed to fetch GitHub stars:', error)
    // 如果获取失败，不显示数量
  }
}

onMounted(() => {
  fetchStarCount()
})
</script>

<style scoped>
.github-stars-wrapper {
  display: inline-block;
  margin: 10px 0;
}

.github-stars-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 20px;
  padding: 0 20px;
  line-height: 38px;
  font-size: 14px;
  font-weight: 500;
  text-decoration: none;
  cursor: pointer;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: white;
  transition: all 0.3s ease;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

.github-stars-button:hover {
  background: linear-gradient(135deg, #5a67d8 0%, #6b46c1 100%);
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
  text-decoration: none;
  color: white;
}
</style>
