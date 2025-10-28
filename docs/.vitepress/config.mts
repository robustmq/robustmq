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
    
    // 保持主题切换功能
    appearance: true,
    
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


});
