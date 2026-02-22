<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import DefaultTheme from 'vitepress/theme'
import BlogToc from './components/BlogToc.vue'

const { frontmatter } = useData()
const isBlogPost = computed(() => frontmatter.value.pageClass === 'blog-post-page')
</script>

<template>
  <DefaultTheme.Layout>
    <template #doc-before v-if="isBlogPost">
      <BlogToc />
    </template>
  </DefaultTheme.Layout>
</template>
