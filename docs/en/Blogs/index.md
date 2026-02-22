---
layout: page
sidebar: false
aside: false
---

<script setup>
import { data as blogData } from '../../.vitepress/theme/blog.data.ts'
</script>

<BlogHome :blogData="blogData" />
