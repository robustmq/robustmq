// Default theme
import DefaultTheme from 'vitepress/theme'
import './custom.css'
import './layout-fix.css'
import './blog-post.css'

// 导入自定义组件
import BlogHome from './components/BlogHome.vue'
import BlogLayoutWithToc from './BlogLayoutWithToc.vue'

// 导入GitHub Stars脚本（暂时禁用以避免 API 限制）
// import { initGitHubStars } from './github-stars.js'

export default {
  extends: DefaultTheme,
  Layout: BlogLayoutWithToc,
  enhanceApp({ app, router }) {
    // 注册全局组件
    app.component('BlogHome', BlogHome)
    
    // 初始化GitHub Stars功能（暂时禁用）
    // if (typeof window !== 'undefined') {
    //   initGitHubStars()
    //   
    //   // 监听路由变化
    //   router.onAfterRouteChanged = () => {
    //     setTimeout(() => {
    //       initGitHubStars()
    //     }, 100)
    //   }
    // }
  }
}
