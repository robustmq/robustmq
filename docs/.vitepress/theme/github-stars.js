// GitHub Stats 动态更新脚本
export function initGitHubStars() {
  // 格式化数字为K格式
  function formatCount(count) {
    if (count >= 1000) {
      return (count / 1000).toFixed(1) + 'K'
    }
    return count.toString()
  }

  // 获取GitHub仓库信息
  async function fetchRepoStats() {
    try {
      const response = await fetch('https://api.github.com/repos/robustmq/robustmq')
      if (response.ok) {
        const data = await response.json()
        return {
          stars: formatCount(data.stargazers_count),
          contributors: null // 需要单独获取
        }
      }
    } catch (error) {
      console.log('Failed to fetch GitHub repo stats:', error)
    }
    return null
  }

  // 获取Contributors数量
  async function fetchContributorsCount() {
    try {
      const response = await fetch('https://api.github.com/repos/robustmq/robustmq/contributors?per_page=1')
      if (response.ok) {
        // 从Link header获取总数
        const linkHeader = response.headers.get('Link')
        if (linkHeader) {
          const match = linkHeader.match(/page=(\d+)>; rel="last"/)
          if (match) {
            return formatCount(parseInt(match[1]))
          }
        }
        // 如果没有Link header，说明contributors少于100个，直接获取数组长度
        const contributors = await response.json()
        return formatCount(contributors.length)
      }
    } catch (error) {
      console.log('Failed to fetch contributors count:', error)
    }
    return null
  }

  // 获取最新版本号
  async function fetchLatestVersion() {
    try {
      const response = await fetch('https://api.github.com/repos/robustmq/robustmq/releases/latest')
      if (response.ok) {
        const data = await response.json()
        return data.tag_name || data.name
      }
    } catch (error) {
      console.log('Failed to fetch latest version:', error)
    }
    return null
  }

  // 更新所有GitHub按钮
  async function updateGitHubButtons() {
    const [repoStats, contributorsCount, latestVersion] = await Promise.all([
      fetchRepoStats(),
      fetchContributorsCount(),
      fetchLatestVersion()
    ])

    // 更新Stars按钮
    if (repoStats && repoStats.stars) {
      const starButtons = document.querySelectorAll('a[href="https://github.com/robustmq/robustmq"]')
      starButtons.forEach(button => {
        const text = button.textContent || button.innerText
        if (text.includes('GitHub Stars')) {
          button.textContent = `⭐ GitHub Stars (${repoStats.stars})`
        }
      })
    }

    // 更新Contributors按钮
    if (contributorsCount) {
      const contributorButtons = document.querySelectorAll('a[href="https://github.com/robustmq/robustmq/graphs/contributors"]')
      contributorButtons.forEach(button => {
        const text = button.textContent || button.innerText
        if (text.includes('Contributors')) {
          button.textContent = `👥 Contributors (${contributorsCount})`
        }
      })
    }

    // 更新Version按钮
    if (latestVersion) {
      const versionButtons = document.querySelectorAll('a[href="https://github.com/robustmq/robustmq/releases"]')
      versionButtons.forEach(button => {
        const text = button.textContent || button.innerText
        if (text.includes('Version')) {
          button.textContent = `📦 Version (${latestVersion})`
        }
      })
    }
  }

  // 页面加载完成后更新按钮
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', updateGitHubButtons)
  } else {
    updateGitHubButtons()
  }

  // 监听路由变化（VitePress SPA导航）
  if (typeof window !== 'undefined') {
    window.addEventListener('popstate', () => {
      setTimeout(updateGitHubButtons, 100)
    })
  }
}
