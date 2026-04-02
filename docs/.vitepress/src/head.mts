import type { HeadConfig } from "vitepress";

export const head: HeadConfig[] = [
    // Favicon
    ['link', { rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' }],

    // 基础 SEO
    ['meta', { name: 'author', content: 'RobustMQ' }],
    ['meta', { name: 'keywords', content: 'RobustMQ, mq9, message queue, MQTT, NATS, Kafka, AI agent communication, IoT messaging, distributed messaging, open source MQ' }],
    ['meta', { name: 'robots', content: 'index, follow' }],

    // Open Graph
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:site_name', content: 'RobustMQ' }],
    ['meta', { property: 'og:image', content: 'https://robustmq.com/og-image.png' }],
    ['meta', { property: 'og:image:width', content: '1200' }],
    ['meta', { property: 'og:image:height', content: '630' }],

    // Twitter Card
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    ['meta', { name: 'twitter:site', content: '@robustmq' }],
    ['meta', { name: 'twitter:image', content: 'https://robustmq.com/og-image.png' }],

    // 51.la 访问统计
    ['script', { charset: 'UTF-8', id: 'LA_COLLECT', src: '//sdk.51.la/js-sdk-pro.min.js' }],
    ['script', {}, `LA.init({id:"3PUlhxY3LHemHVJk",ck:"3PUlhxY3LHemHVJk",autoTrack:true,hashMode:true})`],
];
