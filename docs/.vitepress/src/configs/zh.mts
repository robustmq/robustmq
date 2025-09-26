import type {DefaultTheme, LocaleSpecificConfig} from 'vitepress'

//引入以上配置 是英文界面需要修改zh为en

import getNavs from "../nav/zh.mts";

import {sidebar} from '../sidebars/zh.mts'

export const zhConfig: LocaleSpecificConfig<DefaultTheme.Config> = {

    // 中文版本的base路径
    base: '/zh/',
    
    // 设置页面标题（浏览器标签页）
    title: 'RobustMQ：很高兴有机会让你看到不一样的作品',

    themeConfig: {
        // 设置 Logo 旁边的文字
        siteTitle: 'RobustMQ',

        logo: "/logo.png",

        nav: getNavs(),

        sidebar,

        outline: {

            level: "deep", // 右侧大纲标题层级

            label: "目录", // 右侧大纲标题文本配置

        },

        socialLinks: [
            {icon: 'github', link: 'https://github.com/robustmq/robustmq'}
        ]

    },

}
