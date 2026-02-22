export const sidebar = [
    {
        text: "Introduction",
        collapsed: true,
        items: [
            { text: "What is RobustMQ", link: "/en/OverView/What-is-RobustMQ" },
            { text: "Why RobustMQ", link: "/en/OverView/Why-RobustMQ" },
            {
                text: "MQ Comparison",
                collapsed: true,
                items: [
                    { text: "vs Kafka", link: "/en/OverView/Diff-kafka" },
                    { text: "vs Pulsar", link: "/en/OverView/Diff-pulsar" },
                    { text: "vs NATS", link: "/en/OverView/Diff-nats" },
                    { text: "vs Redpanda", link: "/en/OverView/Diff-redpanda" },
                    { text: "vs Iggy", link: "/en/OverView/Diff-iggy" },
                    { text: "RobustMQ vs Existing MQs", link: "/en/OverView/Summary" },
                ],
            },
            {
                text: "Version planning",
                collapsed: true,
                items: [
                    { text: "2026 RoadMap", link: "/en/OverView/RoadMap-2026" },
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
            { text: "Quick Install", link: "/en/QuickGuide/Quick-Install" },
            { text: "Public Server", link: "/en/QuickGuide/PublicMqttServer" },
            { text: "Experience RobustMQ MQTT", link: "/en/QuickGuide/Experience-MQTT" },
            { text: "SDK Integration", link: "/en/QuickGuide/SDK-Integration" },
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
            { text: "Storage Adapter", link: "/en/Architect/StorageAdapter-Architecture" },
            { text: "Storage Engine", link: "/en/Architect/StorageEngine-Architecture" },
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
                    { text: "RabbitMQ", link: "/en/RobustMQ-MQTT/Bridge/RabbitMQ" },
                    { text: "GreptimeDB", link: "/en/RobustMQ-MQTT/Bridge/GreptimeDB" },
                    { text: "PostgreSQL", link: "/en/RobustMQ-MQTT/Bridge/PostgreSQL" },
                    { text: "MySQL", link: "/en/RobustMQ-MQTT/Bridge/MySQL" },
                    { text: "MongoDB", link: "/en/RobustMQ-MQTT/Bridge/MongoDB" },
                    { text: "Elasticsearch", link: "/en/RobustMQ-MQTT/Bridge/Elasticsearch" },
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
      { text: "Engine Manager", link: "/en/RobustMQ-Command/CLI_ENGINE" },
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
      { text: "Broker Configuration", link: "/en/Configuration/BROKER" },
      { text: "Logging Configuration", link: "/en/Configuration/Logging" },
    ],
  },
  {
    text: "Performance Metrics",
    collapsed: true,
    items: [
      { text: "Bench CLI Guide", link: "/en/Bench/Bench-CLI" },
      { text: "MQTT Bench Guide", link: "/en/Bench/MQTT-Bench" },
      { text: "Meta Bench Guide", link: "/en/Bench/Meta-Bench" },
      { text: "Benchmark Report", link: "/en/Bench/Bench-Report" },
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
            text: "VSCode Running",
            link: "/en/ContributionGuide/ContributingCode/VsCode-Running",
          },
          {
            text: "Code Structure",
            link: "/en/ContributionGuide/ContributingCode/Code-Structure",
          },
          {
            text: "Tokio Console",
            link: "/en/ContributionGuide/ContributingCode/Troubleshooting",
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
    text: "Release Notes",
    collapsed: true,
    items: [
      { text: "0.3.0 RELEASE", link: "/en/VersionRecord/RobustMQ-0.3.0-RELEASE" },
      { text: "0.2.0 RELEASE", link: "/en/VersionRecord/RobustMQ-0.2.0-RELEASE" },
    ],
  },
];
