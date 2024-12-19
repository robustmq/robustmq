import type { DefaultTheme } from "vitepress";

export default function getNavs() {

  return [

    { text: "Home", link: "/en/" },

    // {

    //   text: "关于",

    //   items: [

    //     {

    //       text: "团队",

    //       link: "/zh/examples/about/team",

    //       activeMatch: "/about/team",

    //     },

    //     {

    //       text: "常见问题",

    //       link: "/zh/简介/什么是 RobustMQ",

    //       activeMatch: "/简介/什么是 RobustMQ",

    //     },

    //   ],

    //   activeMatch: "/zh/examples/about/", // // 当前页面处于匹配路径下时, 对应导航菜单将突出显示

    // },

  ]

};