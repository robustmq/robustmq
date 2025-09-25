import {defineConfig} from "vitepress";


import {docsConfig} from "./src/docs.mts";

import {themeConfig} from "./src/theme.mts";

import {head} from "./src/head.mts";


import {enConfig} from './src/configs/en.mts';

import {zhConfig} from './src/configs/zh.mts';

export default defineConfig({

    locales: {

        root: {label: 'English', lang: 'en', ...enConfig},

        zh: {label: '简体中文', lang: 'zh', dir: 'zh', ...zhConfig},

    },

    // 确保正确的路由配置
    cleanUrls: true,
    
    // 默认语言配置
    defaultLocale: 'en',
    
    // 重定向配置 - 将没有语言前缀的路径重定向到英文版本
    redirects: {
        '/': '/en/',
        '/OverView/What-is-RobustMQ': '/en/OverView/What-is-RobustMQ',
        '/OverView/': '/en/OverView/',
        '/QuickGuide/': '/en/QuickGuide/',
        '/Getting-Started/': '/en/Getting-Started/',
        '/Api/': '/en/Api/',
        '/Architect/': '/en/Architect/',
        '/Configuration/': '/en/Configuration/',
        '/InstallationDeployment/': '/en/InstallationDeployment/',
        '/RobustMQ-Command/': '/en/RobustMQ-Command/',
        '/RobustMQ-Kafka/': '/en/RobustMQ-Kafka/',
        '/RobustMQ-MQTT/': '/en/RobustMQ-MQTT/',
        '/VersionRecord/': '/en/VersionRecord/',
        '/Blogs/': '/en/Blogs/',
        '/ContributionGuide/': '/en/ContributionGuide/',
    },

    /* 文档配置 */

    ...docsConfig,

    /* 标头配置 */

    head,

    /* 主题配置 */

    themeConfig,

    /* 语言配置 */


});
