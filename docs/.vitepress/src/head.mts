import type { HeadConfig } from "vitepress";

export const head: HeadConfig[] = [
    ['link', { rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' }], // 添加 favicon
    ['script', { charset: 'UTF-8', id: 'LA_COLLECT', src: '//sdk.51.la/js-sdk-pro.min.js' }],
    ['script', {}, `LA.init({id:"3PUlhxY3LHemHVJk",ck:"3PUlhxY3LHemHVJk",autoTrack:true,hashMode:true})`],
];
