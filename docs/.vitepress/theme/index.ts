// Default theme
import DefaultTheme from 'vitepress/theme'
import './custom.css'
import './layout-fix.css'
import './blog-post.css'

// 导入自定义组件
import BlogHome from './components/BlogHome.vue'
import BlogLayoutWithToc from './BlogLayoutWithToc.vue'

import { initGitHubStars } from './github-stars.js'

export default {
  extends: DefaultTheme,
  Layout: BlogLayoutWithToc,
  enhanceApp({ app, router }) {
    app.component('BlogHome', BlogHome)

    if (typeof window !== 'undefined') {
      initGitHubStars()

      router.onAfterRouteChanged = () => {
        setTimeout(() => {
          initGitHubStars()
        }, 200)
      }
    }
  }
}
