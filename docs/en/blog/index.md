---
layout: page
title: Blog
sidebar: false
aside: false
---

<script setup>
import { data } from '../../.vitepress/theme/blog.data'
</script>

<BlogHome :blog-data="data" />
