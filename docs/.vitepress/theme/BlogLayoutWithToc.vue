<script setup>
import { computed } from 'vue'
import { useData } from 'vitepress'
import DefaultTheme from 'vitepress/theme'
import BlogToc from './components/BlogToc.vue'
import GiscusComment from './components/GiscusComment.vue'

const { frontmatter } = useData()
const isBlogPost = computed(() => frontmatter.value.pageClass === 'blog-post-page')
const showComment = computed(() => frontmatter.value.comment !== false)
</script>

<template>
  <DefaultTheme.Layout>
    <template #doc-before v-if="isBlogPost">
      <BlogToc />
    </template>
    <template #doc-after v-if="showComment">
      <GiscusComment />
    </template>
  </DefaultTheme.Layout>
</template>
