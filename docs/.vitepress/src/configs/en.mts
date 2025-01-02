import type { DefaultTheme, LocaleSpecificConfig } from 'vitepress'

//引入以上配置 是英文界面需要修改zh为en

import getNavs  from "../nav/en.mts";

import {sidebar} from '../sidebars/en.mts'

export const enConfig: LocaleSpecificConfig<DefaultTheme.Config> = {


    themeConfig: {

      lastUpdatedText: 'Last Updated',

      returnToTopLabel: 'Return To Top',

        // 文档页脚文本配置

      docFooter: {

        prev: 'Previous Page',

        next: 'Next Page'

      },

    //   editLink: {

    //     pattern: '路径地址',

    //     text: '对本页提出修改建议',

    //   },

    logo: "./images/logo.png",

     nav: getNavs(),

     sidebar,

     outline: {

       level: "deep", // 右侧大纲标题层级

       label: "content", // 右侧大纲标题文本配置

     },

       },

  }
