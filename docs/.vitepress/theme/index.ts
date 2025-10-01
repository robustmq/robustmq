// Default theme
import DefaultTheme from 'vitepress/theme'
import './custom.css'
import './layout-fix.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    // 徽章组件已移除
  }
}
