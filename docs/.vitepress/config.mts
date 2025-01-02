import { defineConfig } from "vitepress";


import { docsConfig } from "./src/docs.mts";

import { themeConfig } from "./src/theme.mts";

import { head } from "./src/head.mts";


import { enConfig } from './src/configs/en.mts';

import { zhConfig } from './src/configs/zh.mts';

export default defineConfig({

  locales: {

    root: { label: '简体中文',  ...zhConfig },

    en: { label: 'English',  link: '/en/', ...enConfig },

  },

  /* 文档配置 */

  ...docsConfig,  /* 标头配置 */

  head,

  /* 主题配置 */

  themeConfig,

  /* 语言配置 */


});
