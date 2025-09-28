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
                    { text: "2025 年 RoadMap", link: "/zh/OverView/RoadMap-2025" },
                    { text: "MQTT Release 计划", link: "/zh/OverView/MQTT-Release" },
                    { text: "Good First Issue", link: "/zh/OverView/Good-First-Issue" },
                ],
            },
            { text: "给我们签个名吧", link: "/zh/OverView/SignYourName" },
        ],
    },
    {
        text: "快速启动",
        collapsed: true,
        items: [
            { text: "公共Server", link: "/zh/QuickGuide/PublicMqttServer" },
            { text: "体验 RobustMQ MQTT", link: "/zh/QuickGuide/Experience-MQTT" },
        ],
    },
    {
        text: "安装部署",
        collapsed: true,
        items: [
            { text: "编译打包", link: "/zh/QuickGuide/Build-and-Package" },
            { text: "二进制运行[单机]", link: "/zh/InstallationDeployment/Docker-Deployment" },
            { text: "二进制运行[集群]", link: "/zh/InstallationDeployment/Kubernetes-Operator" },
            { text: "Docker 模式", link: "/zh/InstallationDeployment/Docker-Deployment" },
            { text: "K8S 模式", link: "/zh/InstallationDeployment/Kubernetes-Operator" },
        ],
    },
    {
        text: "系统架构",
        collapsed: true,
        items: [
            { text: "架构概览", link: "/zh/Architect/Overall-Architecture" },
            { text: "Meta Service", link: "/zh/Architect/MetaService-Architecture" },
        ],
    },
    {
        text: "RobustMQ MQTT",
        collapsed: true,
        items: [
            { text: "概览", link: "/zh/RobustMQ-MQTT/Overview" },
            { text: "MQTT 核心概念", link: "/zh/RobustMQ-MQTT/MQTTCoreConcepts" },
            { text: "MQTT 系统架构", link: "/zh/RobustMQ-MQTT/SystemArchitecture" },
            {
                text: "核心功能",
                collapsed: true,
                items: [
                    { text: "共享订阅", link: "/zh/RobustMQ-MQTT/SharedSubscription" },
                    { text: "保留消息", link: "/zh/RobustMQ-MQTT/RetainMessage" },
                    { text: "遗嘱消息", link: "/zh/RobustMQ-MQTT/WillMessage" },
                    { text: "排他订阅", link: "/zh/RobustMQ-MQTT/ExclusiveSubscription" },
                    { text: "延迟发布", link: "/zh/RobustMQ-MQTT/DelayMessage" },
                    { text: "自动订阅", link: "/zh/RobustMQ-MQTT/AutoSubscription" },
                    { text: "主题重写", link: "/zh/RobustMQ-MQTT/TopicRewrite" },
                    { text: "通配符订阅", link: "/zh/RobustMQ-MQTT/WildcardSubscription" },
                    { text: "会话持久化", link: "/zh/RobustMQ-MQTT/SessionPersistence" },
                    { text: "系统告警", link: "/zh/RobustMQ-MQTT/SystemAlarm.md" },
                ],
            },
            {
                text: "安全",
                collapsed: true,
                items: [
                    { text: "认证", link: "/zh/RobustMQ-MQTT/Security/Authentication" },
                    { text: "授权", link: "/zh/RobustMQ-MQTT/Security/Authorization" },
                    { text: "黑名单", link: "/zh/RobustMQ-MQTT/Security/Blacklist" },
                    { text: "连接抖动", link: "/zh/RobustMQ-MQTT/FlappingDetect" },
                ]
            },
            {
                text: "数据集成",
                collapsed: true,
                items: [
                    { text: "概述", link: "/zh/RobustMQ-MQTT/Bridge/Overview" },
                    { text: "本地文件", link: "/zh/RobustMQ-MQTT/Bridge/LocalFile" },
                    { text: "Kafka", link: "/zh/RobustMQ-MQTT/Bridge/Kafka" },
                    { text: "Pulsar", link: "/zh/RobustMQ-MQTT/Bridge/Pulsar" },
                    { text: "GreptimeDB", link: "/zh/RobustMQ-MQTT/Bridge/GreptimeDB" },
                ]
            },

      { text: "MQTTX 测试指南", link: "/zh/RobustMQ-MQTT/MQTTX-Guide" },
      {
        text: "客户端 SDK",
        collapsed: true,
        items: [
          { text: "使用 C SDK 连接", link: "/zh/RobustMQ-MQTT/SDK/c-sdk" },
          {
            text: "使用 Java SDK 连接",
            link: "/zh/RobustMQ-MQTT/SDK/java-sdk",
          },
          { text: "使用 Go SDK 连接", link: "/zh/RobustMQ-MQTT/SDK/go-sdk" },
          {
            text: "使用 Python SDK 连接",
            link: "/zh/RobustMQ-MQTT/SDK/python-sdk",
          },
          {
            text: "使用 JavaScript SDK 连接",
            link: "/zh/RobustMQ-MQTT/SDK/javascript-sdk",
          },
        ],
      },
      {
        text: "参考指南",
        collapsed: true,
        items: [
          { text: "MQTT 教程", link: "https://www.emqx.com/zh/mqtt-guide" },
          { text: "MQTT 5.0 协议", link: "https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html" },
          { text: "MQTT 3.1.1 协议", link: "https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/mqtt-v3.1.1.html" },
          { text: "MQTT 术语", link: "https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901003" },
          { text: "MQTT 5.0 特性", link: "https://www.emqx.com/zh/blog/introduction-to-mqtt-5" },
          { text: "MQTT 原因码", link: "https://www.emqx.com/en/blog/mqtt5-new-features-reason-code-and-ack" },
        ],
      },
    ],
  },

  {
    text: "RobustMQ Kafka",
    collapsed: true,
    items: [{ text: "概览", link: "/zh/RobustMQ-Kafka/Overview" }],
  },

  {
    text: "RobustMQ Dashboard",
    collapsed: true,
    items: [{ text: "概览", link: "" }],
  },
  {
    text: "RobustMQ 命令行",
    collapsed: true,
    items: [
      { text: "概览", link: "/zh/RobustMQ-Command/CLI_COMMON" },
      { text: "集群管理", link: "/zh/RobustMQ-Command/CLI_CLUSTER" },
      { text: "MQTT 管理", link: "/zh/RobustMQ-Command/CLI_MQTT" },
      { text: "Journal 管理", link: "/zh/RobustMQ-Command/CLI_JOURNAL" },
    ],
  },
  {
    text: "HTTP 接口文档",
    collapsed: true,
    items: [
      { text: "概览", link: "/zh/Api/COMMON" },
      { text: "Cluster API", link: "/zh/Api/CLUSTER" },
      { text: "MQTT API", link: "/zh/Api/MQTT" },
    ],
  },
  {
    text: "可观测性",
    collapsed: true,
    items: [
      { text: "Prometheus 接入", link: "/zh/Observability/Prometheus接入" },
      { text: "基础设施指标", link: "/zh/Observability/基础设施指标" },
      { text: "MQTT 专用指标", link: "/zh/Observability/MQTT专用指标" },
      { text: "Grafana 配置指南", link: "/zh/Observability/Grafana配置指南" },
    ],
  },
  {
    text: "配置说明",
    collapsed: true,
    items: [
      { text: "通用配置", link: "/zh/Configuration/COMMON" },
      { text: "MQTT 配置", link: "/zh/Configuration/MQTT" },
      { text: "Meta 配置", link: "/zh/Configuration/META" },
      { text: "Journal 配置", link: "/zh/Configuration/JOURNAL" },
      { text: "日志配置", link: "/zh/Configuration/Logging" },
    ],
  },
  {
    text: "性能指标",
    collapsed: true,
    items: [
      { text: "RobustMQ MQTT", link: "" },
      { text: "RobustMQ Kafka", link: "" },
    ],
  },
  {
    text: "贡献指南",
    collapsed: true,
    items: [
      {
        text: "GitHub 贡献指南",
        link: "/zh/ContributionGuide/GitHub-Contribution-Guide",
      },
      {
        text: "PR 提交示例",
        link: "/zh/ContributionGuide/Pull-Request-Example",
      },
      {
        text: "代码贡献",
        collapsed: true,
        items: [
          {
            text: "环境搭建",
            link: "/zh/ContributionGuide/ContributingCode/Build-Develop-Env",
          },
          {
            text: "Cargo运行",
            link: "/zh/ContributionGuide/ContributingCode/Cargo-Running",
          },
          {
            text: "故障排查",
            link: "/zh/ContributionGuide/ContributingCode/Troubleshooting",
          },
          {
            text: "VSCode 运行",
            link: "/zh/ContributionGuide/ContributingCode/VsCode-Running",
          },
          {
            text: "代码结构",
            link: "/zh/ContributionGuide/ContributingCode/Code-Structure",
          },
          {
            text: "Pprof 使用指南",
            link: "/zh/ContributionGuide/ContributingCode/Pprof-Usage",
          },
        ],
      },
      {
        text: "文档贡献",
        collapsed: true,
        items: [
          {
            text: "环境搭建",
            link: "/zh/ContributionGuide/ContributingDoc/Build-Doc-Env",
          },
          {
            text: "文档贡献指导",
            link: "/zh/ContributionGuide/ContributingDoc/Doc-Contribution-Guide",
          },
        ],
      },
    ],
  },
  {
    text: "博客文章",
    collapsed: true,
    items: [
      { text: "01: 用 Rust 重新定义消息队列", link: "/zh/Blogs/01" },
      { text: "02: RobustMQ: 技术设计理念综述", link: "/zh/Blogs/02" },
      { text: "03: 介绍 RobustMQ 的 Roles", link: "/zh/Blogs/03" },
    ],
  },
];
