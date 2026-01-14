export const sidebar = [
    {
        text: "简介",
        collapsed: true,
        items: [
            { text: "什么是 RobustMQ", link: "/zh/OverView/What-is-RobustMQ" },
            { text: "为什么有 RobustMQ", link: "/zh/OverView/Why-RobustMQ" },
            {
                text: "和主流MQ的对比",
                collapsed: true,
                items: [
                    { text: "和 Kafka 对比", link: "/zh/OverView/Diff-kafka" },
                    { text: "和 Pulsar 对比", link: "/zh/OverView/Diff-pulsar" },
                    { text: "和 NATS 对比", link: "/zh/OverView/Diff-nats" },
                    { text: "和 Redpanda 对比", link: "/zh/OverView/Diff-redpanda" },
                    { text: "和 Iggy 对比", link: "/zh/OverView/Diff-iggy" },
                    { text: "综合对比", link: "/zh/OverView/Diff-MQ" },
                    { text: "对比总结与技术评估", link: "/zh/OverView/Summary" },
                ],
            },
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
            { text: "快速安装", link: "/zh/QuickGuide/Quick-Install" },
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
                    { text: "RabbitMQ", link: "/zh/RobustMQ-MQTT/Bridge/RabbitMQ" },
                    { text: "GreptimeDB", link: "/zh/RobustMQ-MQTT/Bridge/GreptimeDB" },
                    { text: "PostgreSQL", link: "/zh/RobustMQ-MQTT/Bridge/PostgreSQL" },
                    { text: "MySQL", link: "/zh/RobustMQ-MQTT/Bridge/MySQL" },
                    { text: "MongoDB", link: "/zh/RobustMQ-MQTT/Bridge/MongoDB" },
                    { text: "Elasticsearch", link: "/zh/RobustMQ-MQTT/Bridge/Elasticsearch" },
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
            text: "VSCode 运行",
            link: "/zh/ContributionGuide/ContributingCode/VsCode-Running",
          },
          {
            text: "代码结构",
            link: "/zh/ContributionGuide/ContributingCode/Code-Structure",
          },
          {
            text: "Tokio Console",
            link: "/zh/ContributionGuide/ContributingCode/Troubleshooting",
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
      { text: "01: 基于 Rust 的新一代云原生消息队列", link: "/zh/Blogs/01" },
      { text: "02: 技术设计理念综述", link: "/zh/Blogs/02" },
      { text: "03: 介绍 RobustMQ 的 Roles", link: "/zh/Blogs/03" },
      { text: "04: 产品层面的思考与定位", link: "/zh/Blogs/04" },
      { text: "05: 我们真的能成为下一代消息基础设施吗？", link: "/zh/Blogs/05" },
      { text: "06: 整体架构概述", link: "/zh/Blogs/06" },
      { text: "07: 很高兴有机会让你看到不一样的作品", link: "/zh/Blogs/07" },
      { text: "08: RobustMQ 0.2.0 RELEASE 正式发布", link: "/zh/Blogs/08" },
      { text: "09: 单二进制架构设计", link: "/zh/Blogs/09" },
      { text: "10: Meta Service 系统架构", link: "/zh/Blogs/10" },
      { text: "11: MQTT Broker 系统架构", link: "/zh/Blogs/11" },
      { text: "12: 存储层 Storage Adapter 架构", link: "/zh/Blogs/12" },
      { text: "13: 关于消息队列存储层的一些想法", link: "/zh/Blogs/13" },
      { text: "14: MQTT 存储模型", link: "/zh/Blogs/14" },
      { text: "15: 存储层设计", link: "/zh/Blogs/15" },
      { text: "16: Segment Engine 技术架构", link: "/zh/Blogs/16" },
      { text: "17: 构建下一代统一消息平台的战略思考", link: "/zh/Blogs/17" },
      { text: "18: 在 AI 场景的思考和探索", link: "/zh/Blogs/18" },
      { text: "19: 关于 AI Coding 的一些思考", link: "/zh/Blogs/19" },
      { text: "20: IBM 收购 Confluent，聊聊 RobustMQ", link: "/zh/Blogs/20" },
      { text: "21: 我们要做定义者，而不是追随者", link: "/zh/Blogs/21" },
      { text: "22: RobustMQ 是一个探索的过程", link: "/zh/Blogs/22" },
      { text: "23: I/O Pool + 零拷贝 + Partitioned Log", link: "/zh/Blogs/23" },
      { text: "24: 存储层设计：核心特性", link: "/zh/Blogs/24" },
      { text: "25: 存储层设计思考（下）：从理念到方案", link: "/zh/Blogs/25" },
      { text: "26: 对标NATS：内存模式", link: "/zh/Blogs/26" },
      { text: "27: 在边缘场景的一些想法", link: "/zh/Blogs/27" },
      { text: "28: RobustMQ 的设计思考：为什么选择组合而非创新", link: "/zh/Blogs/28" },
      { text: '29: 基础软件的"不性感"与真实价值', link: "/zh/Blogs/29" },
      { text: "30: 边思考边实现：RobustMQ 写作手记", link: "/zh/Blogs/30" },
      { text: "31: 消息队列巨头的 AI 战略：StreamNative、Confluent、Redpanda 都在做什么", link: "/zh/Blogs/31" },
      { text: "32: StreamNative Orca Agent Engine 是什么", link: "/zh/Blogs/32" },
      { text: "33: StreamNative Ursa：重新定义数据流引擎", link: "/zh/Blogs/33" },
      { text: "34: Apache Iggy：用 Rust 重写消息流平台", link: "/zh/Blogs/34" },
      { text: "35: 消息队列与 MCP Server：有用但不核心", link: "/zh/Blogs/35" },
      { text: "36: 在 RobustMQ 中我和 AI 是如何协作的", link: "/zh/Blogs/36" },
      { text: "37: 消息队列这个领域，很难有什么革命性的创新", link: "/zh/Blogs/37" },
      { text: "38: NATS：技术优雅，天花板明显", link: "/zh/Blogs/38" },
    ],
  },
];
