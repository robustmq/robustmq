import {defineConfig} from "vitepress";


import {docsConfig} from "./src/docs.mts";

import {themeConfig} from "./src/theme.mts";

import {head} from "./src/head.mts";


import {enConfig} from './src/configs/en.mts';

import {zhConfig} from './src/configs/zh.mts';

export default defineConfig({

    locales: {

        root: {label: 'English', lang: 'en',dir: 'en',  ...enConfig},

        zh: {label: '简体中文', lang: 'zh', dir: 'zh', ...zhConfig},

    },

    // 确保正确的路由配置
    cleanUrls: true,
    
    // 默认语言配置
    defaultLocale: 'en',
    
    // 修复布局和水合不匹配问题
    vite: {
        define: {
            '__VUE_PROD_HYDRATION_MISMATCH_DETAILS__': false
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
        }
    },
    
    // 保持主题切换功能
    appearance: true,
    
    // 禁用可能导致布局问题的功能
    mpa: false,

    /* 文档配置 */

    ...docsConfig,

    /* 标头配置 */

    head,

    /* 主题配置 */

    themeConfig,

    /* 语言配置 */


});
