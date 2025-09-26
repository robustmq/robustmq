import type {DefaultTheme, LocaleSpecificConfig} from 'vitepress'

//引入以上配置 是英文界面需要修改zh为en

import getNavs from "../nav/en.mts";

import {sidebar} from '../sidebars/en.mts'

export const enConfig: LocaleSpecificConfig<DefaultTheme.Config> = {

    base: '/en/',
    
    // 设置页面标题（浏览器标签页）
    title: 'RobustMQ：Glad to have the opportunity to show you something different',

    themeConfig: {
        // 设置 Logo 旁边的文字
        siteTitle: 'RobustMQ',

        logo: "/logo.png",

        nav: getNavs(),

        sidebar,

        outline: {

            level: "deep", // 右侧大纲标题层级

            label: "content", // 右侧大纲标题文本配置

        },

        socialLinks: [
            {icon: 'github', link: 'https://github.com/robustmq/robustmq'}
        ]

    },

}
