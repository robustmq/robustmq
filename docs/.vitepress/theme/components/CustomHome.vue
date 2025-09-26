<template>
  <div class="VPHome">
    <div class="VPHero">
      <div class="container">
        <div class="main">
          <div class="text">
            <h1 class="name">
              <span class="clip">{{ frontmatter.hero?.text || '' }}</span>
            </h1>
            <p class="tagline">{{ frontmatter.hero?.tagline || '' }}</p>
            
            <!-- 徽章放在 tagline 下面，actions 上面 -->
            <BadgeSection />
            
            <div class="actions" v-if="frontmatter.hero?.actions">
              <div class="action" v-for="action in frontmatter.hero.actions" :key="action.text">
                <VPButton
                  tag="a"
                  size="medium"
                  :theme="action.theme"
                  :text="action.text"
                  :href="action.link"
                />
              </div>
            </div>
          </div>
          <div class="image" v-if="frontmatter.hero?.image">
            <VPImage
              class="VPImage"
              :image="frontmatter.hero.image"
              :alt="frontmatter.hero.image.alt"
            />
          </div>
        </div>
      </div>
    </div>
    
    <div class="VPFeatures" v-if="frontmatter.features">
      <div class="container">
        <div class="items">
          <div class="item" v-for="feature in frontmatter.features" :key="feature.title">
            <VPFeature
              :icon="feature.icon"
              :title="feature.title"
              :details="feature.details"
              :link="feature.link"
              :link-text="feature.linkText"
              :rel="feature.rel"
              :target="feature.target"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { useData } from 'vitepress'
import VPButton from 'vitepress/dist/client/theme-default/components/VPButton.vue'
import VPImage from 'vitepress/dist/client/theme-default/components/VPImage.vue'
import VPFeature from 'vitepress/dist/client/theme-default/components/VPFeature.vue'
import BadgeSection from './BadgeSection.vue'

const { frontmatter } = useData()
</script>

<style scoped>
.VPHome {
  margin-bottom: 96px;
}

.VPHero {
  padding: 48px 24px;
}

.container {
  margin: 0 auto;
  max-width: 1152px;
}

.main {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  justify-content: space-between;
}

.text {
  flex: 1;
  min-width: 320px;
  order: 2;
}

.name {
  line-height: 1.1;
  font-size: 48px;
  font-weight: 700;
  white-space: pre-wrap;
  margin: 0 0 16px;
}

.clip {
  background: var(--vp-c-brand-1);
  background: linear-gradient(120deg, var(--vp-c-brand-1) 30%, #41d1ff);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.tagline {
  padding-top: 8px;
  line-height: 28px;
  font-size: 18px;
  font-weight: 500;
  white-space: pre-wrap;
  color: var(--vp-c-text-2);
  margin-bottom: 0;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  padding-top: 32px;
}

.image {
  flex: 1;
  order: 1;
  margin: 0 auto 24px;
  min-width: 320px;
  max-width: 576px;
}

.VPFeatures {
  position: relative;
  padding: 0 24px;
}

.items {
  display: flex;
  flex-wrap: wrap;
  margin: -8px;
}

.item {
  padding: 8px;
  width: 100%;
}

@media (min-width: 640px) {
  .item {
    width: 50%;
  }
}

@media (min-width: 960px) {
  .item {
    width: 33.333333%;
  }
  
  .text {
    order: 1;
    width: 392px;
  }
  
  .image {
    order: 2;
    margin: 0 0 0 64px;
    min-width: 576px;
  }
}

@media (max-width: 768px) {
  .name {
    font-size: 32px;
  }
  
  .tagline {
    font-size: 16px;
    line-height: 24px;
  }
}
</style>
