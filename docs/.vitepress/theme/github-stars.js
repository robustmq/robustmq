// GitHub Stats 动态更新脚本
// 带本地缓存（sessionStorage），避免重复请求触发 API 限流

const CACHE_KEY = 'robustmq_github_stats'
const CACHE_TTL = 5 * 60 * 1000 // 5 分钟

function formatCount(count) {
  if (count >= 1000) {
    return (count / 1000).toFixed(1) + 'k'
  }
  return count.toString()
}

async function fetchStats() {
  // 优先读缓存
  try {
    const cached = sessionStorage.getItem(CACHE_KEY)
    if (cached) {
      const { data, ts } = JSON.parse(cached)
      if (Date.now() - ts < CACHE_TTL) return data
    }
  } catch (_) {}

  const [repoRes, contribRes, releaseRes] = await Promise.allSettled([
    fetch('https://api.github.com/repos/robustmq/robustmq'),
    fetch('https://api.github.com/repos/robustmq/robustmq/contributors?per_page=1'),
    fetch('https://api.github.com/repos/robustmq/robustmq/releases/latest'),
  ])

  let stars = null
  let contributors = null
  let version = null

  if (repoRes.status === 'fulfilled' && repoRes.value.ok) {
    const d = await repoRes.value.json()
    stars = formatCount(d.stargazers_count)
  }

  if (contribRes.status === 'fulfilled' && contribRes.value.ok) {
    const link = contribRes.value.headers.get('Link')
    if (link) {
      const m = link.match(/page=(\d+)>; rel="last"/)
      if (m) contributors = formatCount(parseInt(m[1]))
    }
    if (!contributors) {
      const arr = await contribRes.value.json()
      contributors = formatCount(arr.length)
    }
  }

  if (releaseRes.status === 'fulfilled' && releaseRes.value.ok) {
    const d = await releaseRes.value.json()
    version = d.tag_name || d.name || null
  }

  const data = { stars, contributors, version }

  try {
    sessionStorage.setItem(CACHE_KEY, JSON.stringify({ data, ts: Date.now() }))
  } catch (_) {}

  return data
}

function applyStats({ stars, contributors, version }) {
  if (stars) {
    document.querySelectorAll('a[href="https://github.com/robustmq/robustmq"]').forEach(el => {
      if ((el.textContent || '').includes('Stars')) {
        el.textContent = `⭐ ${stars} Stars`
      }
    })
  }

  if (contributors) {
    document.querySelectorAll('a[href="https://github.com/robustmq/robustmq/graphs/contributors"]').forEach(el => {
      if ((el.textContent || '').includes('Contributors')) {
        el.textContent = `👥 ${contributors} Contributors`
      }
    })
  }

  if (version) {
    document.querySelectorAll('a[href="https://github.com/robustmq/robustmq/releases"]').forEach(el => {
      if ((el.textContent || '').includes('Version')) {
        el.textContent = `📦 ${version}`
      }
    })
  }
}

export async function initGitHubStars() {
  try {
    const stats = await fetchStats()
    applyStats(stats)
  } catch (err) {
    console.log('GitHub stats fetch failed:', err)
  }
}

export { fetchStats }
