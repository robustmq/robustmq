export const sidebar = [
    {
        text: "简介",
        collapsed: true,
        items: [
            { text: "什么是 RobustMQ", link: "/zh/OverView/What-is-RobustMQ" },
            { text: "为什么有 RobustMQ", link: "/zh/OverView/Why-RobustMQ" },
            { text: "和IGGY对比", link: "/zh/OverView/Diff-iggy" },
            { text: "和主流消息队列对比", link: "/zh/OverView/Diff-MQ" },
            {
                text: "版本计划",
                collapsed: true,
                items: [
                    { text: "2025 年 RoadMamp", link: "/zh/OverView/RoadMap-2025" },
                    { text: "MQTT Release 计划", link: "/zh/OverView/MQTT-Release" },
                    { text: "Good First Issue", link: "/zh/OverView/Good-First-Issue" },
                ],
            },
        ],
    },
    {
        text: "快速启动",
        collapsed: true,
        items: [
            { text: "概览", link: "/zh/QuickGuide/Overview" },
            { text: "编译打包", link: "/zh/QuickGuide/mqtt/Build" },
            { text: "第一个任务", link: "/zh/QuickGuide/mqtt/First-Task" },
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
            { text: "详细设计文档", link: "/zh/Architect/Design-Documentation" },
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
            {
                text: "安装部署",
                collapsed: true,
                items: [
                    { text: "二进制运行[单机]", link: "/zh/QuickGuide/mqtt/Run-Standalone-Mode" },
                    { text: "二进制运行[集群]", link: "/zh/QuickGuide/mqtt/Run-Cluster-Mode" },
                    { text: "Docker 模式", link: "/zh/QuickGuide/mqtt/Run-Docker-Mode" },
                    { text: "K8S 模式", link: "/zh/QuickGuide/mqtt/Run-K8S-Mode" },
                ],
            },
            { text: "公共 MQTT Server", link: "/zh/RobustMQ-MQTT/PublicMqttServer" },
            { text: "系统架构", link: "/zh/RobustMQ-MQTT/SystemArchitecture.md" },
            {
                text: "核心功能",
                collapsed: true,
                items: [
                    { text: "保留消息", link: "/zh/RobustMQ-MQTT/RetainMessage.md" },
                    { text: "遗嘱消息", link: "" },
                    { text: "排他订阅", link: "" },
                    { text: "延迟发布", link: "/zh/RobustMQ-MQTT/DelayMessage.md" },
                    { text: "自动订阅", link: "" },
                    { text: "主题重写", link: "" },
                    { text: "通配符订阅", link: "" },
                    { text: "Session 持久化", link: "" },
                    { text: "共享订阅", link: "" },
                    { text: "MQTT Over Quic", link: "" },
                ],
            },
            {
                text: "安全",
                collapsed: true,
                items: [
                    { text: "认证", link: "" },
                    { text: "授权", link: "" },
                    { text: "黑名单", link: "" },
                    { text: "连接抖动", link: "" },
                ]
            },
            {
                text: "数据集成",
                collapsed: true,
                items: [
                    { text: " Local File", link: "" },
                    { text: "Kafka", link: "" },
                ]
            },

            {
                text: "可观测性",
                collapsed: true,
                items: [
                    { text: "系统告警", link: "/zh/RobustMQ-MQTT/SystemAlarm.md" },
                    { text: "指标", link: "" },
                    { text: "Trace", link: "" },
                    { text: "集成 Prometheus", link: "" },
                    { text: "集成 OpenTelemetry", link: "" },
                ]
            },
            { text: "MQTT Dashboard", link: "/zh/RobustMQ-MQTT/Dashboard.md" },
            { text: "Bench 性能压测", link: "" },
            {
                text: "客户端 SDK",
                collapsed: true,
                items: [
                    { text: "使用 C SDK 连接", link: "" },
                    { text: "使用 Java SDK 连接", link: "" },
                    { text: "使用 Go SDK 连接", link: "" },
                    { text: "使用 Python SDK 连接", link: "" },
                    { text: "使用 JavaScript SDK 连接", link: "" },
                ]
            },
        ],
    },

    {
        text: "RobustMQ Kafka",
        collapsed: true,
        items: [
            { text: "概览", link: "/zh/RobustMQ-Kafka/Overview" },
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
        text: "RobustMQ Dashboard",
        collapsed: true,
        items: [
            { text: "概览", link: "" }
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
                    { text: "故障排查", link: "/zh/ContributionGuide/ContributingCode/Troubleshooting" },
                    { text: "VSCode 运行", link: "/zh/ContributionGuide/ContributingCode/VsCode-Running" },
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
    }
];
