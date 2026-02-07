<template>
  <div class="blog-home">
    <!-- 轮播推荐区 -->
    <div class="featured-carousel">
      <div class="carousel-container">
        <div class="carousel-track" :style="{ transform: `translateX(-${currentSlide * 100}%)` }">
          <div v-for="(post, index) in currentFeaturedPosts" :key="index" class="carousel-slide">
            <div class="carousel-background" :style="{ background: post.backgroundGradient }"></div>
            <a :href="post.url || post.link" class="featured-card">
              <h2>{{ post.title }}</h2>
              <div class="featured-tags" v-if="post.tags && post.tags.length > 0">
                <span v-for="tag in post.tags" :key="tag" class="tag">{{ tag }}</span>
              </div>
              <p class="featured-excerpt">{{ post.excerpt }}</p>
              <p class="featured-date" v-if="post.date || post.modifiedTime">{{ post.date || post.modifiedTime }}</p>
            </a>
          </div>
        </div>
        
        <!-- 轮播控制按钮 -->
        <button class="carousel-btn prev" @click="handlePrevClick" v-if="currentFeaturedPosts.length > 1">‹</button>
        <button class="carousel-btn next" @click="handleNextClick" v-if="currentFeaturedPosts.length > 1">›</button>
        
        <!-- 轮播指示器 -->
        <div class="carousel-indicators" v-if="currentFeaturedPosts.length > 1">
          <span 
            v-for="(post, index) in currentFeaturedPosts" 
            :key="index"
            :class="['indicator', { active: index === currentSlide }]"
            @click="handleIndicatorClick(index)"
          ></span>
        </div>
      </div>
    </div>

    <!-- 博客文章网格 -->
    <div class="blog-grid">
      <a 
        v-for="(post, index) in currentBlogPosts" 
        :key="index"
        :href="post.url || post.link"
        class="blog-card"
      >
        <div class="card-content">
          <h3>{{ post.title }}</h3>
          <p class="card-date" v-if="post.date || post.modifiedTime">{{ post.date || post.modifiedTime }}</p>
          <div class="card-tags" v-if="post.tags && post.tags.length > 0">
            <span v-for="tag in post.tags" :key="tag" class="tag">{{ tag }}</span>
          </div>
          <p class="card-excerpt">{{ post.excerpt }}</p>
        </div>
      </a>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useData } from 'vitepress'

const { lang } = useData()

// 接收从页面传递的博客数据
const props = defineProps({
  blogData: {
    type: Object,
    default: () => ({ zh: [], en: [] })
  }
})

// 博客列表完全由数据加载器提供，从 docs/zh/Blogs 和 docs/en/Blogs 目录动态读取

// 获取真实的博客列表（从数据加载器）
const currentBlogPosts = computed(() => {
  console.log('blogData:', props.blogData)
  if (props.blogData && props.blogData.zh && props.blogData.en) {
    const posts = lang.value === 'zh' ? props.blogData.zh : props.blogData.en
    console.log('Using real data, posts count:', posts.length)
    return posts
  }
  console.log('blogData not available yet')
  return []
})

// 渐变背景色方案（更稳定，无需外部 API）
const gradientBackgrounds = [
  'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
  'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)',
  'linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)',
  'linear-gradient(135deg, #43e97b 0%, #38f9d7 100%)',
  'linear-gradient(135deg, #fa709a 0%, #fee140 100%)',
  'linear-gradient(135deg, #30cfd0 0%, #330867 100%)',
  'linear-gradient(135deg, #a8edea 0%, #fed6e3 100%)',
  'linear-gradient(135deg, #ff9a9e 0%, #fecfef 100%)'
]

// 根据语言选择内容（取前4篇最新的作为推荐）
const currentFeaturedPosts = computed(() => {
  const posts = currentBlogPosts.value
  return posts.slice(0, 4).map((post, index) => {
    // 为每个轮播项分配不同的渐变背景
    const background = gradientBackgrounds[index % gradientBackgrounds.length]
    
    return {
      ...post,
      link: post.url || post.link,
      date: post.date || '最新',
      category: post.category || '技术博客',
      excerpt: post.excerpt || post.title,
      backgroundGradient: background
    }
  })
})

// 轮播控制
const currentSlide = ref(0)
let autoPlayInterval = null

const nextSlide = () => {
  currentSlide.value = (currentSlide.value + 1) % currentFeaturedPosts.value.length
}

const prevSlide = () => {
  currentSlide.value = currentSlide.value === 0 
    ? currentFeaturedPosts.value.length - 1 
    : currentSlide.value - 1
}

const goToSlide = (index) => {
  currentSlide.value = index
}

// 自动播放
const startAutoPlay = () => {
  autoPlayInterval = setInterval(() => {
    nextSlide()
  }, 5000) // 5秒自动切换
}

const stopAutoPlay = () => {
  if (autoPlayInterval) {
    clearInterval(autoPlayInterval)
    autoPlayInterval = null
  }
}

// 点击按钮时重置自动播放
const handlePrevClick = () => {
  stopAutoPlay()
  prevSlide()
  startAutoPlay()
}

const handleNextClick = () => {
  stopAutoPlay()
  nextSlide()
  startAutoPlay()
}

const handleIndicatorClick = (index) => {
  stopAutoPlay()
  goToSlide(index)
  startAutoPlay()
}

onMounted(() => {
  startAutoPlay()
})

onUnmounted(() => {
  stopAutoPlay()
})
</script>

<style scoped>
/* 容器占满并居中内容 */
.blog-home {
  width: 100%;
  max-width: none;
  padding: 2rem 0;
}

/* 轮播推荐区 */
.featured-carousel {
  margin: 0 auto 1rem auto;
  width: 70vw;
  max-width: 1400px;
}

.carousel-container {
  position: relative;
  width: 100%;
  height: 350px;
  overflow: hidden;
  border-radius: 12px;
  background: #1a1a1a;
}

.carousel-track {
  display: flex;
  height: 100%;
  transition: transform 0.5s ease-in-out;
}

.carousel-slide {
  min-width: 100%;
  height: 100%;
  position: relative;
}

.carousel-background {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  opacity: 0.9;
  transition: opacity 0.3s ease;
}

.carousel-slide:hover .carousel-background {
  opacity: 1;
}

.featured-card {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;
  height: 100%;
  padding: 3rem;
  color: white !important;
  text-decoration: none !important;
  transition: all 0.3s ease;
}

.featured-card:hover {
  opacity: 0.9;
  text-decoration: none !important;
}

.featured-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 12px 0;
  justify-content: center;
}

.featured-tags .tag {
  display: inline-block;
  padding: 4px 12px;
  background: rgba(147, 51, 234, 0.8);
  border-radius: 12px;
  font-size: 12px;
  color: white !important;
  text-decoration: none !important;
  backdrop-filter: blur(10px);
}

.featured-card h2 {
  font-size: 2rem;
  margin: 0 0 1rem 0;
  font-weight: 600;
  color: white !important;
}

.featured-meta {
  font-size: 0.9rem;
  opacity: 0.9;
  margin: 0 0 1rem 0;
  color: white !important;
}

.featured-excerpt {
  font-size: 0.95rem;
  line-height: 1.6;
  max-width: 600px;
  opacity: 0.9;
  margin: 0 0 0.5rem 0;
  color: white !important;
}

.featured-date {
  font-size: 0.85rem;
  opacity: 0.8;
  margin: 0;
  color: white !important;
  font-style: italic;
}

/* 轮播控制按钮 */
.carousel-btn {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  background: rgba(255, 255, 255, 0.3);
  border: none;
  color: white;
  font-size: 2rem;
  width: 50px;
  height: 50px;
  border-radius: 50%;
  cursor: pointer;
  transition: background 0.3s;
  z-index: 10;
}

.carousel-btn:hover {
  background: rgba(255, 255, 255, 0.5);
}

.carousel-btn.prev {
  left: 20px;
}

.carousel-btn.next {
  right: 20px;
}

/* 轮播指示器 */
.carousel-indicators {
  position: absolute;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 10px;
  z-index: 10;
}

.indicator {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  transition: background 0.3s;
}

.indicator:hover,
.indicator.active {
  background: white;
}

/* 博客网格 */
.blog-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 2rem;
  width: 70vw;
  max-width: 1400px;
  margin: 0 auto;
  padding: 0.5rem 0 3rem 0;
}

.blog-card {
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 12px;
  padding: 1.75rem;
  text-decoration: none !important;
  color: inherit;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  flex-direction: column;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  position: relative;
  overflow: hidden;
}

.blog-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg, rgb(147, 51, 234), rgb(126, 34, 206));
  transform: scaleX(0);
  transition: transform 0.3s ease;
}

.blog-card:hover::before {
  transform: scaleX(1);
}

.blog-card:hover {
  transform: translateY(-6px);
  box-shadow: 0 12px 24px rgba(147, 51, 234, 0.15), 0 4px 8px rgba(0, 0, 0, 0.08);
  border-color: rgba(147, 51, 234, 0.3);
  text-decoration: none !important;
}

.card-content h3 {
  margin: 0 0 0.5rem 0;
  font-size: 1.15rem;
  line-height: 1.5;
  font-weight: 600;
  color: var(--vp-c-text-1);
  transition: color 0.2s ease;
}

.blog-card:hover .card-content h3 {
  color: rgb(147, 51, 234);
}

.card-date {
  font-size: 0.8rem;
  color: var(--vp-c-text-3);
  margin: 0 0 0.5rem 0;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.card-date::before {
  content: '📅';
  font-size: 0.85rem;
}

.card-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 14px;
}

.card-tags .tag {
  display: inline-block;
  padding: 4px 12px;
  background: rgba(147, 51, 234, 0.08);
  border: 1px solid rgba(147, 51, 234, 0.2);
  border-radius: 12px;
  font-size: 0.75rem;
  color: rgb(147, 51, 234);
  font-weight: 500;
  transition: all 0.2s ease;
}

.blog-card:hover .card-tags .tag {
  background: rgba(147, 51, 234, 0.15);
  border-color: rgb(147, 51, 234);
  color: rgb(126, 34, 206);
  transform: translateY(-1px);
}

.card-meta {
  font-size: 0.85rem;
  color: var(--vp-c-text-3);
  margin: 0 0 0.75rem 0;
}

.card-excerpt {
  font-size: 0.9rem;
  color: var(--vp-c-text-2);
  line-height: 1.6;
  margin: 0;
  flex: 1;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.card-category {
  display: inline-block;
  padding: 0.25rem 0.75rem;
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand);
  border-radius: 12px;
  font-size: 0.75rem;
  font-weight: 500;
}

/* 响应式 */
@media (max-width: 1200px) {
  .blog-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 960px) {
  .blog-grid {
    grid-template-columns: repeat(1, 1fr);
  }
  
  .featured-card h2 {
    font-size: 1.5rem;
  }
  
  .carousel-container {
    height: 250px;
  }
}

@media (max-width: 640px) {
  .blog-grid {
    grid-template-columns: 1fr;
  }
  
  .featured-card {
    padding: 2rem 1.5rem;
  }
  
  .featured-card h2 {
    font-size: 1.25rem;
  }
  
  .carousel-container {
    height: 220px;
  }
  
  .carousel-btn {
    width: 40px;
    height: 40px;
    font-size: 1.5rem;
  }
}
</style>
