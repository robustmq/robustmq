// Default theme
import DefaultTheme from 'vitepress/theme'
import './custom.css'
import './layout-fix.css'
import BadgeSection from './components/BadgeSection.vue'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('BadgeSection', BadgeSection)
  }
}
