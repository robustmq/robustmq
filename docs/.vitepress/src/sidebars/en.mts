export const sidebar = [
    {
        text: "Introduction",
        collapsed: true,
        items: [
            { text: "What is RobustMQ", link: "/en/OverView/What-is-RobustMQ" },
            { text: "Why RobustMQ", link: "/en/OverView/Why-RobustMQ" },
            { text: "IGGY Comparison", link: "/en/OverView/Diff-iggy" },
            { text: "MQ Comparison", link: "/en/OverView/Diff-MQ" },
            {
                text: "Version planning",
                collapsed: true,
                items: [
                    { text: "2025 RoadMap", link: "/en/OverView/RoadMap-2025" },
                    { text: "MQTT Release Planning", link: "/en/OverView/MQTT-Release" },
                    { text: "Good First Issue", link: "/en/OverView/Good-First-Issue" },
                ],
            },
            { text: "Sign Your Name", link: "/en/OverView/SignYourName" },
        ],
    },
    {
        text: "QuickGuide",
        collapsed: true,
        items: [
            { text: "Public Server", link: "/en/QuickGuide/PublicMqttServer" },
            { text: "Experience RobustMQ MQTT", link: "/en/QuickGuide/Experience-MQTT" },
        ],
    },
    {
        text: "Install and Deployment",
        collapsed: true,
        items: [
            { text: "Build && Package", link: "/en/QuickGuide/Build-and-Package" },
            { text: "StandaloneMode", link: "/en/InstallationDeployment/Single-Machine-Cluster" },
            { text: "ClusterMode", link: "/en/InstallationDeployment/Multi-Node-Cluster" },
            { text: "DockerMode", link: "/en/InstallationDeployment/Docker-Deployment" },
            { text: "K8SMode", link: "/en/InstallationDeployment/Kubernetes-Operator" },
        ],
    },
    {
        text: "Architect",
        collapsed: true,
        items: [
            { text: "Overview", link: "/en/Architect/Overall-Architecture" },
            { text: "Meta Service", link: "/en/Architect/MetaService-Architecture" },
        ],
    },
    {
        text: "RobustMQ MQTT",
        collapsed: true,
        items: [
            { text: "Overview", link: "/en/RobustMQ-MQTT/Overview" },
            { text: "MQTT Core Concepts", link: "/en/RobustMQ-MQTT/MQTTCoreConcepts" },
            { text: "Architecture", link: "/en/RobustMQ-MQTT/SystemArchitecture" },
            {
                text: "Features",
                collapsed: true,
                items: [
                    { text: "Shared Subscription", link: "/en/RobustMQ-MQTT/SharedSubscription" },
                    { text: "Retain Message", link: "/en/RobustMQ-MQTT/RetainMessage" },
                    { text: "Will Message", link: "/en/RobustMQ-MQTT/WillMessage" },
                    { text: "Exclusive Subscription", link: "/en/RobustMQ-MQTT/ExclusiveSubscription" },
                    { text: "Delayed Publishing", link: "/en/RobustMQ-MQTT/DelayMessage" },
                    { text: "Auto Subscription", link: "/en/RobustMQ-MQTT/AutoSubscription" },
                    { text: "Topic Rewrite", link: "/en/RobustMQ-MQTT/TopicRewrite" },
                    { text: "Wildcard Subscription", link: "/en/RobustMQ-MQTT/WildcardSubscription" },
                    { text: "Session Persistence", link: "/en/RobustMQ-MQTT/SessionPersistence" },
                    { text: "System Alarm", link: "/en/RobustMQ-MQTT/SystemAlarm" },
                ],
            },
            {
                text: "Security",
                collapsed: true,
                items: [
                    { text: "Authentication", link: "/en/RobustMQ-MQTT/Security/Authentication" },
                    { text: "Authorization", link: "/en/RobustMQ-MQTT/Security/Authorization" },
                    { text: "Blacklist", link: "/en/RobustMQ-MQTT/Security/Blacklist" },
                    { text: "Flapping Detect", link: "/en/RobustMQ-MQTT/FlappingDetect" },
                ]
            },
            {
                text: "Data integration",
                collapsed: true,
                items: [
                    { text: "Overview", link: "/en/RobustMQ-MQTT/Bridge/Overview" },
                    { text: "Local File", link: "/en/RobustMQ-MQTT/Bridge/LocalFile" },
                    { text: "Kafka", link: "/en/RobustMQ-MQTT/Bridge/Kafka" },
                    { text: "Pulsar", link: "/en/RobustMQ-MQTT/Bridge/Pulsar" },
                    { text: "GreptimeDB", link: "/en/RobustMQ-MQTT/Bridge/GreptimeDB" },
                ]
            },
      { text: "MQTTX Testing Guide", link: "/en/RobustMQ-MQTT/MQTTX-Guide" },
      { text: "Performance", link: "" },
      {
        text: "Client SDK",
        collapsed: true,
        items: [
          { text: "C SDK", link: "/en/RobustMQ-MQTT/SDK/c-sdk" },
          { text: "Java SDK", link: "/en/RobustMQ-MQTT/SDK/java-sdk" },
          { text: "Go SDK", link: "/en/RobustMQ-MQTT/SDK/go-sdk" },
          { text: "Python SDK", link: "/en/RobustMQ-MQTT/SDK/python-sdk" },
          { text: "JavaScript SDK", link: "/en/RobustMQ-MQTT/SDK/javascript-sdk" },
        ],
      },
      {
        text: "Reference Guide",
        collapsed: true,
        items: [
          { text: "MQTT Tutorial", link: "https://www.emqx.com/zh/mqtt-guide" },
          { text: "MQTT 5.0 Protocol", link: "https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html" },
          { text: "MQTT 3.1.1 Protocol", link: "https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/mqtt-v3.1.1.html" },
          { text: "MQTT Terminology", link: "https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901003" },
          { text: "MQTT 5.0 Features", link: "https://www.emqx.com/zh/blog/introduction-to-mqtt-5" },
          { text: "MQTT Reason Codes", link: "https://www.emqx.com/en/blog/mqtt5-new-features-reason-code-and-ack" },
        ],
      },
    ],
  },
  {
    text: "RobustMQ Kafka",
    collapsed: true,
    items: [{ text: "Overview", link: "/en/RobustMQ-Kafka/Overview" }],
  },
  {
    text: "RobustMQ Dashboard",
    collapsed: true,
    items: [{ text: "Overview", link: "" }],
  },
  {
    text: "RobustMQ Command",
    collapsed: true,
    items: [
      { text: "Overview", link: "/en/RobustMQ-Command/CLI_COMMON" },
      { text: "Cluster Manager", link: "/en/RobustMQ-Command/CLI_CLUSTER" },
      { text: "MQTT Manager", link: "/en/RobustMQ-Command/CLI_MQTT" },
      { text: "Journal Manager", link: "/en/RobustMQ-Command/CLI_JOURNAL" },
    ],
  },
  {
    text: "HTTP Rest API",
    collapsed: true,
    items: [
      { text: "Overview", link: "/en/Api/COMMON" },
      { text: "Cluster API", link: "/en/Api/CLUSTER" },
      { text: "MQTT API", link: "/en/Api/MQTT" },
    ],
  },
  {
    text: "Observability",
    collapsed: true,
    items: [
      { text: "Prometheus Integration", link: "/en/Observability/Prometheus-Integration" },
      { text: "Infrastructure Metrics", link: "/en/Observability/Infrastructure-Metrics" },
      { text: "MQTT Specific Metrics", link: "/en/Observability/MQTT-Specific-Metrics" },
      { text: "Grafana Configuration Guide", link: "/en/Observability/Grafana-Configuration-Guide" },
    ],
  },
  {
    text: "Configuration",
    collapsed: true,
    items: [
      { text: "Common Configuration", link: "/en/Configuration/COMMON" },
      { text: "MQTT Configuration", link: "/en/Configuration/MQTT" },
      { text: "Meta Configuration", link: "/en/Configuration/META" },
      { text: "Journal Configuration", link: "/en/Configuration/JOURNAL" },
      { text: "Logging Configuration", link: "/en/Configuration/Logging" },
    ],
  },
  {
    text: "ContributionGuide",
    collapsed: true,
    items: [
      {
        text: "Contribution Guide",
        link: "/en/ContributionGuide/GitHub-Contribution-Guide",
      },
      { text: "PR Example", link: "/en/ContributionGuide/Pull-Request-Example" },
      {
        text: "ContributingCode",
        collapsed: true,
        items: [
          {
            text: "Build Develop Env",
            link: "/en/ContributionGuide/ContributingCode/Build-Develop-Env",
          },
          {
            text: "Cargo Running",
            link: "/en/ContributionGuide/ContributingCode/Cargo-Running",
          },
          {
            text: "Troubleshooting",
            link: "/en/ContributionGuide/ContributingCode/Troubleshooting",
          },
          {
            text: "VSCode Running",
            link: "/en/ContributionGuide/ContributingCode/VsCode-Running",
          },
          {
            text: "Code Structure",
            link: "/en/ContributionGuide/ContributingCode/Code-Structure",
          },
          {
            text: "Pprof Usage",
            link: "/en/ContributionGuide/ContributingCode/Pprof-Usage",
          },
        ],
      },
      {
        text: "ContributingDoc",
        collapsed: true,
        items: [
          {
            text: "Build Doc Env",
            link: "/en/ContributionGuide/ContributingDoc/Build-Doc-Env",
          },
          {
            text: "Doc Contribution Guide",
            link: "/en/ContributionGuide/ContributingDoc/Doc-Contribution-Guide",
          },
        ],
      },
    ],
  },
  {
    text: "Blog Articles",
    collapsed: true,
    items: [
      {
        text: "Redefining Cloud-Native Message Queues with Rust",
        link: "/en/Blogs/01",
      },
      {
        text: "RobustMQ: Overview of Technical Design Philosophy",
        link: "/en/Blogs/02",
      },
      { text: "Introduction to RobustMQ Roles", link: "/en/Blogs/03" },
    ],
  },
];
