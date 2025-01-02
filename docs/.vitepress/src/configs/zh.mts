import type {DefaultTheme, LocaleSpecificConfig} from 'vitepress'

//引入以上配置 是英文界面需要修改zh为en

import getNavs from "../nav/zh.mts";

import {sidebar} from '../sidebars/zh.mjs'

export const zhConfig: LocaleSpecificConfig<DefaultTheme.Config> = {

    themeConfig: {

        logo: "/logo.png",

        nav: getNavs(),

        sidebar,

        outline: {

            level: "deep", // 右侧大纲标题层级

            label: "目录", // 右侧大纲标题文本配置

        },

    },

}
