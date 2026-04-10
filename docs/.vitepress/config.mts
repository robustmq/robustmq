import {defineConfig} from "vitepress";


import {docsConfig} from "./src/docs.mts";

import {themeConfig} from "./src/theme.mts";

import {head} from "./src/head.mts";


import {enConfig} from './src/configs/en.mts';

import {zhConfig} from './src/configs/zh.mts';

export default defineConfig({

    locales: {

        en: {label: 'English', lang: 'en', ...enConfig},

        zh: {label: '简体中文', lang: 'zh', ...zhConfig},

    },

    // 确保正确的路由配置
    cleanUrls: true,
    
    // 默认语言配置
    defaultLocale: 'en',
    
    // 修复布局和水合不匹配问题
    vite: {
        define: {
            '__VUE_PROD_HYDRATION_MISMATCH_DETAILS__': false,
            '__VUE_PROD_DEVTOOLS__': false
        },
        ssr: {
            noExternal: ['vitepress']
        },
        build: {
            rollupOptions: {
                output: {
                    manualChunks: undefined
                }
            }
        },
        css: {
            preprocessorOptions: {
                scss: {
                    charset: false
                }
            }
        },
        // 确保开发和生产环境的一致性
        optimizeDeps: {
            exclude: ['vitepress']
        }
    },
    
    // 强制暗色模式，禁用切换
    appearance: 'force-dark',

    // 显示页面最后更新时间（读取 git commit 时间）
    lastUpdated: true,
    
    // 禁用可能导致布局问题的功能
    mpa: false,
    
    // 忽略死链接检查，特别是 localhost 链接
    ignoreDeadLinks: [
        // 忽略所有 localhost 链接
        /^https?:\/\/localhost/,
        // 忽略本地 IP 地址
        /^https?:\/\/127\.0\.0\.1/,
        /^https?:\/\/0\.0\.0\.0/,
        // 忽略内网 IP 地址
        /^https?:\/\/192\.168\./,
        /^https?:\/\/10\./,
        /^https?:\/\/172\.(1[6-9]|2[0-9]|3[0-1])\./
    ],

    /* 文档配置 */

    ...docsConfig,

    /* 标头配置 */

    head,

    /* 主题配置 */

    themeConfig,

    /* 语言配置 */

    // Sitemap
    sitemap: {
        hostname: 'https://robustmq.com',
    },

    // Auto-apply blog post layout + SEO meta for all pages
    transformPageData(pageData) {
        // 博客页 layout 处理
        if (pageData.relativePath.match(/^(zh|en)\/Blogs\/(?!index)\d+\.md$/)) {
            pageData.frontmatter.layout    = pageData.frontmatter.layout    ?? 'doc'
            pageData.frontmatter.sidebar   = false
            pageData.frontmatter.aside     = false
            pageData.frontmatter.pageClass = 'blog-post-page'
        }

        // 动态注入 og:title / og:description / canonical
        const siteUrl = 'https://robustmq.com'
        const title = pageData.title
            ? `${pageData.title} | RobustMQ`
            : 'RobustMQ — Next-generation unified messaging infrastructure'
        const description = pageData.frontmatter.description
            || pageData.description
            || 'RobustMQ is a next-generation cloud-native message queue supporting MQTT, NATS, Kafka protocols with unified storage, built for AI, IoT, and big data.'

        pageData.frontmatter.head = pageData.frontmatter.head ?? []
        pageData.frontmatter.head.push(
            ['meta', { property: 'og:title', content: title }],
            ['meta', { property: 'og:description', content: description }],
            ['meta', { property: 'og:url', content: `${siteUrl}/${pageData.relativePath.replace(/\.md$/, '')}` }],
            ['meta', { name: 'twitter:title', content: title }],
            ['meta', { name: 'twitter:description', content: description }],
            ['link', { rel: 'canonical', href: `${siteUrl}/${pageData.relativePath.replace(/\.md$/, '')}` }],
        )
    },

});
