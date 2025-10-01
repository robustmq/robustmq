// Default theme
import DefaultTheme from 'vitepress/theme'
import './custom.css'
import './layout-fix.css'

// 导入GitHub Stars脚本
import { initGitHubStars } from './github-stars.js'

export default {
  extends: DefaultTheme,
  enhanceApp({ app, router }) {
    // 初始化GitHub Stars功能
    if (typeof window !== 'undefined') {
      initGitHubStars()
      
      // 监听路由变化
      router.onAfterRouteChanged = () => {
        setTimeout(() => {
          initGitHubStars()
        }, 100)
      }
    }
  }
}
