export const sidebar = [
    {
        text: "简介",
        collapsed: true,
        items: [
            { text: "什么是 RobustMQ", link: "/zh/OverView/What-is-RobustMQ" },
            { text: "为什么有 RobustMQ", link: "/zh/OverView/Why-RobustMQ" },
            { text: "非常欢迎你", link: "/zh/OverView/Welcome" },
            { text: "RoadMamp", link: "/zh/OverView/RoadMap" },
        ],
    },
    {
        text: "快速启动",
        collapsed: true,
        items: [
            { text: "概览", link: "/zh/QuickGuide/Overview" },
            { text: "编译打包", link: "/zh/QuickGuide/Build" },
            {
                text: "RobustMQ MQTT",
                collapsed: true,
                items: [
                    { text: "单机模式", link: "/zh/QuickGuide/Run-Standalone-Mode" },
                    { text: "集群模式", link: "/zh/QuickGuide/Run-Cluster-Mode" },
                    { text: "Docker 模式", link: "/zh/QuickGuide/Run-Docker-Mode" },
                    { text: "K8S 模式", link: "/zh/QuickGuide/Run-K8S-Mode" },
                ],
            },

        ],
    },
    {
        text: "系统架构",
        collapsed: true,
        items: [
            { text: "概览", link: "/zh/Architect/Overview" },
            { text: "Placement Center", link: "/zh/Architect/Placement-Center" },
            { text: "Broker Server", link: "/zh/Architect/Broker-Server" },
            { text: "Storage Adapter", link: "/zh/Architect/Storage-Adapter" },
            { text: "Journal Server", link: "/zh/Architect/Journal-Server" },
            { text: "集成测试", link: "/zh/Architect/Test-Case" },
            {
                text: "配置说明",
                collapsed: true,
                items: [
                    { text: "Placement Center", link: "/zh/Architect/Configuration/Placement-Center" },
                    { text: "MQTT Broker", link: "/zh/Architect/Configuration/Mqtt-Server" },
                ],
            },
        ],
    },
    {
        text: "RobustMQ MQTT",
        collapsed: true,
        items: [
            { text: "概览", link: "/zh/RobustMQ-MQTT/Overview" },
        ],
    },
    {
        text: "RobustMQ 命令行",
        collapsed: true,
        items: [
            { text: "概览", link: "/zh/RobustMQ-Command/Overview" },
            { text: "MQTT Command", link: "/zh/RobustMQ-Command/Mqtt-Broker" },
            { text: "Placement Command", link: "/zh/RobustMQ-Command/Placement-Center" },
            { text: "Journal Command", link: "/zh/RobustMQ-Command/Journal Server" },
        ],
    },
    {
        text: "贡献指南",
        collapsed: true,
        items: [
            { text: "GitHub 贡献指南", link: "/zh/ContributionGuide/GitHub-Contribution-Guide" },
            { text: "PR 提交示例", link: "/zh/ContributionGuide/Pull-Request-Example" },
            {
                text: "代码贡献",
                collapsed: true,
                items: [
                    { text: "环境搭建", link: "/zh/ContributionGuide/ContributingCode/Build-Develop-Env" },
                    { text: "Cargo运行", link: "/zh/ContributionGuide/ContributingCode/Cargo-Running" },
                    { text: "VsCode 运行", link: "/zh/ContributionGuide/ContributingCode/VsCode-Running" },
                    { text: "代码结构", link: "/zh/ContributionGuide/ContributingCode/Code-Structure" },
                ]
            },
            {
                text: "文档贡献",
                collapsed: true,
                items: [
                    { text: "环境搭建", link: "/zh/ContributionGuide/ContributingDoc/Build-Doc-Env" },
                    { text: "文档贡献指导", link: "/zh/ContributionGuide/ContributingDoc/Doc-Contribution-Guide" },
                ]
            }
        ],
    },
    {
        text: "相关资料",
        collapsed: true,
        items: [

        ],
    },
];
