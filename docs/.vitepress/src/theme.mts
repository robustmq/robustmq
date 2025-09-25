import type {DefaultTheme} from "vitepress";

export const themeConfig: DefaultTheme.Config = {

    logo: "/logo.png",

    // i18n路由
    i18nRouting: true,

    // 导航配置
    nav: [
        { text: "Doc", link: "/OverView/What-is-RobustMQ" }
    ],

};
