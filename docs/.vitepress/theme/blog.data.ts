import { createContentLoader } from 'vitepress'
import fs from 'fs'
import path from 'path'

interface BlogPost {
  title: string
  url: string
  date?: string
  excerpt?: string
  number: number
  tags?: string[]
  modifiedTime?: string
}

// 关键字提取函数（从标题和内容提取）
function extractTags(title: string, content: string): string[] {
  // 中文关键字列表
  const zhKeywords = [
    'RobustMQ', 'Rust', 'MQTT', 'Kafka', 'AMQP', 'RocketMQ', 'GRPC', 'TLS', 'SSL',
    '云原生', '消息队列', '架构', '性能', '高可用', 'Raft', 'RocksDB',
    '持久化', '会话', '集群', '部署', '监控', '安全', '认证', '授权',
    'IoT', 'AI', '存储', '计算', 'Serverless', 'K8s', 'Kubernetes',
    '测试', '优化', '设计', '开源', '社区', 'WebSocket', 'HTTP',
    '分布式', '一致性', '容错', '扩展性', '延迟', '吞吐量', '弹性',
    '协议', '多协议', '存算分离', '临时会话', '持久会话'
  ]
  
  // 英文关键字列表
  const enKeywords = [
    'RobustMQ', 'Rust', 'MQTT', 'Kafka', 'AMQP', 'RocketMQ', 'GRPC', 'TLS', 'SSL',
    'Cloud Native', 'Message Queue', 'Architecture', 'Performance', 'High Availability',
    'Raft', 'RocksDB', 'Persistence', 'Session', 'Cluster', 'Deployment',
    'Monitoring', 'Security', 'Authentication', 'Authorization', 'IoT', 'AI',
    'Storage', 'Compute', 'Serverless', 'K8s', 'Kubernetes', 'Testing',
    'Optimization', 'Design', 'Open Source', 'Community', 'WebSocket', 'HTTP',
    'Distributed', 'Consistency', 'Fault Tolerance', 'Scalability', 'Latency', 'Throughput'
  ]
  
  const keywords = [...zhKeywords, ...enKeywords]
  const tags: string[] = []
  
  // 移除数字前缀（如 "02: "）
  const cleanTitle = title.replace(/^\d+:\s*/, '')
  
  // 合并标题和内容前500字符来提取标签
  const textToSearch = cleanTitle + ' ' + content.substring(0, 500)
  
  // 查找标题和内容中包含的关键字
  for (const keyword of keywords) {
    if (textToSearch.includes(keyword) && !tags.includes(keyword)) {
      tags.push(keyword)
      if (tags.length >= 3) break // 固定提取3个标签
    }
  }
  
  // 如果不足3个，补充默认标签
  while (tags.length < 3) {
    if (!tags.includes('技术博客')) {
      tags.push('技术博客')
    } else if (!tags.includes('RobustMQ')) {
      tags.push('RobustMQ')
    } else {
      tags.push('消息队列')
      break
    }
  }
  
  // 固定返回3个标签
  return tags.slice(0, 3)
}

export default {
  async load(): Promise<{ zh: BlogPost[], en: BlogPost[] }> {
    // 读取中文博客
    const zhBlogsDir = path.resolve(__dirname, '../../zh/Blogs')
    const zhFiles = fs.readdirSync(zhBlogsDir)
      .filter(file => file.endsWith('.md'))
      .sort((a, b) => {
        // 提取数字并倒序排列
        const numA = parseInt(a.replace('.md', ''))
        const numB = parseInt(b.replace('.md', ''))
        return numB - numA
      })
    
    const zhPosts: BlogPost[] = []
    
    for (const file of zhFiles) {
      const filePath = path.join(zhBlogsDir, file)
      const content = fs.readFileSync(filePath, 'utf-8')
      const number = parseInt(file.replace('.md', ''))
      
      // 获取文件修改时间
      const stats = fs.statSync(filePath)
      const modifiedDate = new Date(stats.mtime)
      const modifiedTime = modifiedDate.toLocaleDateString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit'
      }).replace(/\//g, '-')
      
      // 提取标题（第一行的 # 标题）
      const titleMatch = content.match(/^#\s+(.+)$/m)
      const title = titleMatch ? titleMatch[1] : file
      
      // 提取标签（从标题和内容）
      const tags = extractTags(title, content)
      
      // 提取摘要（第一段非空内容，跳过 frontmatter、标题和图片）
      const lines = content.split('\n')
      let excerpt = ''
      let inFrontmatter = false
      let frontmatterCount = 0
      
      for (const line of lines) {
        const trimmed = line.trim()
        
        // 跳过 frontmatter
        if (trimmed === '---') {
          frontmatterCount++
          inFrontmatter = frontmatterCount === 1
          continue
        }
        if (inFrontmatter || frontmatterCount < 2) {
          continue
        }
        
        // 提取有效内容
        if (trimmed && 
            !trimmed.startsWith('#') && 
            !trimmed.startsWith('<') && 
            !trimmed.startsWith('>') &&
            !trimmed.startsWith('!') &&
            !trimmed.startsWith('-') &&
            trimmed.length > 10) {
          // 提取约60个字符作为摘要
          excerpt = trimmed.substring(0, 60) + '...'
          break
        }
      }
      
      zhPosts.push({
        title,
        url: `/zh/Blogs/${file.replace('.md', '')}`,
        excerpt,
        number,
        tags,
        date: modifiedTime,
        modifiedTime
      })
    }
    
    // 读取英文博客
    const enBlogsDir = path.resolve(__dirname, '../../en/Blogs')
    const enFiles = fs.existsSync(enBlogsDir) 
      ? fs.readdirSync(enBlogsDir)
          .filter(file => file.endsWith('.md'))
          .sort((a, b) => {
            const numA = parseInt(a.replace('.md', ''))
            const numB = parseInt(b.replace('.md', ''))
            return numB - numA
          })
      : []
    
    const enPosts: BlogPost[] = []
    
    for (const file of enFiles) {
      const filePath = path.join(enBlogsDir, file)
      const content = fs.readFileSync(filePath, 'utf-8')
      const number = parseInt(file.replace('.md', ''))
      
      // 获取文件修改时间
      const stats = fs.statSync(filePath)
      const modifiedDate = new Date(stats.mtime)
      const modifiedTime = modifiedDate.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric'
      })
      
      const titleMatch = content.match(/^#\s+(.+)$/m)
      const title = titleMatch ? titleMatch[1] : file
      
      // 提取标签（从标题和内容）
      const tags = extractTags(title, content)
      
      const lines = content.split('\n')
      let excerpt = ''
      let inFrontmatter = false
      let frontmatterCount = 0
      
      for (const line of lines) {
        const trimmed = line.trim()
        
        // 跳过 frontmatter
        if (trimmed === '---') {
          frontmatterCount++
          inFrontmatter = frontmatterCount === 1
          continue
        }
        if (inFrontmatter || frontmatterCount < 2) {
          continue
        }
        
        // 提取有效内容
        if (trimmed && 
            !trimmed.startsWith('#') && 
            !trimmed.startsWith('<') && 
            !trimmed.startsWith('>') &&
            !trimmed.startsWith('!') &&
            !trimmed.startsWith('-') &&
            trimmed.length > 10) {
          // 提取约60个字符作为摘要
          excerpt = trimmed.substring(0, 60) + '...'
          break
        }
      }
      
      enPosts.push({
        title,
        url: `/en/Blogs/${file.replace('.md', '')}`,
        excerpt,
        number,
        tags,
        date: modifiedTime,
        modifiedTime
      })
    }
    
    return {
      zh: zhPosts,
      en: enPosts
    }
  }
}
