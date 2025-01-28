import {defineConfig} from "vitepress";


import {docsConfig} from "./src/docs.mts";

import {themeConfig} from "./src/theme.mts";

import {head} from "./src/head.mts";


import {enConfig} from './src/configs/en.mts';

import {zhConfig} from './src/configs/zh.mts';

export default defineConfig({

    rewrites: {
        'en/:rest*': ':rest*'
    },

    locales: {

        root: {label: 'English', lang: 'en', dir: 'en', ...enConfig},

        zh: {label: '简体中文', lang: 'zh', dir: 'zh', ...zhConfig},

    },

    /* 文档配置 */

    ...docsConfig,

    /* 标头配置 */

    head: [
        ['link', { rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' }], // 添加 favicon
    ],

    /* 主题配置 */

    themeConfig,

    /* 语言配置 */


});
