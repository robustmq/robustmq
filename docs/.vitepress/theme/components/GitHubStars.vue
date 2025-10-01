<template>
  <div class="github-stars">
    <a 
      href="https://github.com/robustmq/robustmq" 
      target="_blank" 
      rel="noopener noreferrer"
      class="stars-link"
    >
      <svg class="star-icon" viewBox="0 0 16 16" width="16" height="16">
        <path fill="currentColor" d="M8 .25a.75.75 0 0 1 .673.418l1.882 3.815 4.21.612a.75.75 0 0 1 .416 1.279l-3.046 2.97.719 4.192a.75.75 0 0 1-1.088.791L8 12.347l-3.766 1.98a.75.75 0 0 1-1.088-.79l.72-4.194L.818 6.374a.75.75 0 0 1 .416-1.28l4.21-.611L7.327.668A.75.75 0 0 1 8 .25Z"/>
      </svg>
      <img 
        alt="GitHub stars" 
        src="https://img.shields.io/github/stars/robustmq/robustmq?style=social&label=Star"
        loading="lazy"
        class="stars-badge"
      >
    </a>
  </div>
</template>

<script setup>
// GitHub Stars 组件 - 显示项目的 star 数
</script>

<style scoped>
.github-stars {
  display: flex;
  justify-content: center;
  align-items: center;
  margin: 16px 0 24px;
  padding: 0 24px;
}

.stars-link {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  text-decoration: none;
  padding: 8px 16px;
  border-radius: 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  font-weight: 500;
  font-size: 14px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
}

.stars-link:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
  background: linear-gradient(135deg, #5a67d8 0%, #6b46c1 100%);
}

.star-icon {
  width: 16px;
  height: 16px;
  color: #ffd700;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.2));
}

.stars-badge {
  height: 20px;
  border-radius: 10px;
  transition: transform 0.2s ease;
}

.stars-link:hover .stars-badge {
  transform: scale(1.05);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .github-stars {
    margin: 12px 0 20px;
    padding: 0 16px;
  }
  
  .stars-link {
    padding: 6px 12px;
    font-size: 13px;
    gap: 6px;
  }
  
  .star-icon {
    width: 14px;
    height: 14px;
  }
  
  .stars-badge {
    height: 18px;
  }
}

/* 暗色主题适配 */
html.dark .stars-link {
  background: linear-gradient(135deg, #4c51bf 0%, #553c9a 100%);
  border-color: rgba(255, 255, 255, 0.2);
}

html.dark .stars-link:hover {
  background: linear-gradient(135deg, #4338ca 0%, #5b21b6 100%);
}
</style>

