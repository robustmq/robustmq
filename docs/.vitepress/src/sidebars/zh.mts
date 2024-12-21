export const sidebar = [
    {
      text: "简介",
      collapsed:true,
      items: [
        { text: "什么是 RobustMQ", link: "/zh/OverView/What-is-RobustMQ" },
        { text: "为什么有 RobustMQ", link: "/zh/OverView/Why-RobustMQ"},
        { text: "2024 年开发计划", link: "/zh/OverView/RoadMap-For-2024" },
      ],
    },
    {
      text: "快速启动",
      collapsed:true,
      items: [
        { text: "概览", link: "/zh/QuickGuide/Overview" },
        { text: "单机模式", link: "/zh/QuickGuide/Run-Standalone-Mode" },
        { text: "集群模式", link: "/zh/QuickGuide/Run-Cluster-Mode" },
      ],
    },
    {
      text: "系统架构",
      collapsed:true,
      items: [
        { text: "概览", link: "/zh/Architect/Overview" },
        { text: "Placement Center", link: "/zh/Architect/Placement-Center" },
        { text: "Broker Server", link: "/zh/Architect/Broker-Server" },
        { text: "Storage Adapter", link: "/zh/Architect/Storage-Adapter" },
        { text: "Journal Server", link: "/zh/Architect/Journal-Server" },
        { text: "集成测试", link: "/zh/Architect/Test-Case" },
        {
          text: "配置说明",
          collapsed:true,
          items: [
            { text: "Placement Center", link: "/zh/Architect/Configuration/Placement-Center" },
            { text: "MQTT Broker", link: "/zh/Architect/Configuration/Mqtt-Server" },
          ],
        },
      ],
    },
    {
      text: "RobustMQ MQTT",
      collapsed:true,
      items: [
        { text: "概览", link: "/zh/RobustMQ-MQTT/Overview" },
      ],
    },
    {
      text: "贡献指南",
      collapsed:true,
      items: [
        { text: "环境搭建", link: "/zh/ContributionGuide/Build-Develop-Env" },
        { text: "Cargo运行", link: "/zh/ContributionGuide/Cargo-Running" },
        { text: "VsCode 运行", link: "/zh/ContributionGuide/VsCode-Running" },
        { text: "代码结构", link: "/zh/ContributionGuide/Code-Structure" },
        { text: "GitHub 贡献指南", link: "/zh/ContributionGuide/GitHub-Contribution-Guide" },
      ],
    },
    {
      text: "版本记录",
      collapsed:true,
      items: [
        { text: "0.1.0-beta", link: "/zh/VersionRecord/0.1.0-beta" },
      ],
    },
    {
      text: "相关资料",
      collapsed:true,
      items: [
        { text: "RobustMQ Rust China For 2024", link: "/zh/OtherData/RobustMQ-Rust-China-For-2024" },
      ],
    },
  ];