const items = [
    {
        text: "Introduction",
        collapsed: true,
        items: [
            { text: "What", link: "/en/OverView/What-is-RobustMQ" },
            { text: "Why", link: "/en/OverView/Why-RobustMQ" },
            { text: "Engineering Philosophy", link: "/en/Blogs/83" },
            { text: "Still on the Way", link: "/en/OverView/SignYourName" },
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
                text: "Version Planning",
                collapsed: true,
                items: [
                    { text: "2026 RoadMap", link: "/en/OverView/RoadMap-2026" },
                    { text: "2025 RoadMap", link: "/en/OverView/RoadMap-2025" },
                    { text: "MQTT Release Planning", link: "/en/OverView/MQTT-Release" },
                    { text: "Good First Issue", link: "/en/OverView/Good-First-Issue" },
                ],
            },
        ],
    },
    {
        text: "QuickGuide",
        collapsed: true,
        items: [
            { text: "Quick Install", link: "/en/QuickGuide/Quick-Install" },
            { text: "Experience MQTT", link: "/en/QuickGuide/Experience-MQTT" },
            { text: "Experience NATS Core", link: "/en/QuickGuide/Experience-NATS" },
            { text: "Experience mq9", link: "/en/QuickGuide/Experience-MQ9" },
            { text: "Public Server", link: "/en/QuickGuide/PublicMqttServer" },
        ],
    },
    {
        text: "Install and Deployment",
        collapsed: true,
        items: [
            { text: "Binary Deployment", link: "/en/InstallationDeployment/Binary-Deployment" },
            { text: "Docker Mode", link: "/en/InstallationDeployment/Docker-Deployment" },
            { text: "K8S Mode", link: "/en/InstallationDeployment/Kubernetes-Operator" },
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
            { text: "Connector", link: "/en/Architect/Connector-Architecture" },
            { text: "Multi-Protocol", link: "/en/Architect/MultiProtocol-Architecture" },
            { text: "Pluggable Storage", link: "/en/Architect/PluggableStorage-Architecture" },
        ],
    },
    {
        text: "RobustMQ MQTT",
        collapsed: true,
        items: [
            { text: "Overview", link: "/en/RobustMQ-MQTT/Overview" },
            { text: "MQTT Core Concepts", link: "/en/RobustMQ-MQTT/MQTTCoreConcepts" },
            { text: "Multi-Tenancy", link: "/en/RobustMQ-MQTT/MultiTenant" },
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
                    { text: "Flapping Detect", link: "/en/RobustMQ-MQTT/FlappingDetect" },
                    { text: "System Alarm", link: "/en/RobustMQ-MQTT/SystemAlarm" },
                    { text: "System Topics", link: "/en/RobustMQ-MQTT/SystemTopic" },
                ],
            },
            {
                text: "Security",
                collapsed: true,
                items: [
                    {
                        text: "Data Source",
                        collapsed: true,
                        items: [
                            { text: "Overview", link: "/en/RobustMQ-MQTT/Security/DataSource" },
                            { text: "Built-in (Meta Service)", link: "/en/RobustMQ-MQTT/Security/DataSource/BuiltIn" },
                            { text: "MySQL", link: "/en/RobustMQ-MQTT/Security/DataSource/MySQL" },
                            { text: "PostgreSQL", link: "/en/RobustMQ-MQTT/Security/DataSource/PostgreSQL" },
                            { text: "Redis", link: "/en/RobustMQ-MQTT/Security/DataSource/Redis" },
                            { text: "MongoDB", link: "/en/RobustMQ-MQTT/Security/DataSource/MongoDB" },
                            { text: "HTTP", link: "/en/RobustMQ-MQTT/Security/DataSource/HTTP" },
                        ]
                    },
                    {
                        text: "Authentication",
                        collapsed: true,
                        items: [
                            { text: "Overview", link: "/en/RobustMQ-MQTT/Security/Authentication" },
                            { text: "Password", link: "/en/RobustMQ-MQTT/Security/Authentication-Password" },
                            { text: "JWT", link: "/en/RobustMQ-MQTT/Security/Authentication-JWT" },
                        ]
                    },
                    { text: "Authorization", link: "/en/RobustMQ-MQTT/Security/Authorization" },
                    { text: "Blacklist", link: "/en/RobustMQ-MQTT/Security/Blacklist" },
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
    text: "ROBUSTMQ MQ9",
    link: "https://mq9.robustmq.com/docs/overview",
  },

  {
    text: "RobustMQ NATS",
    collapsed: true,
    items: [
      { text: "Overview", link: "/en/nats/Overview" },
      { text: "Quick Start", link: "/en/nats/QuickStart" },
      { text: "NATS Core", link: "/en/nats/NatsCore" },
      { text: "JetStream", link: "/en/nats/JetStream" },
      { text: "SDK Integration", link: "/en/nats/SDK" },
    ],
  },

  {
    text: "RobustMQ Kafka",
    collapsed: true,
    items: [
      { text: "Overview", link: "/en/RobustMQ-Kafka/Overview" },
      { text: "Core Concepts", link: "/en/RobustMQ-Kafka/KafkaCoreConcepts" },
      { text: "System Architecture", link: "/en/RobustMQ-Kafka/SystemArchitecture" },
      { text: "Protocol Support", link: "/en/RobustMQ-Kafka/Protocol" },
      { text: "Quick Start", link: "/en/RobustMQ-Kafka/QuickStart" },
      { text: "CLI Guide", link: "/en/RobustMQ-Kafka/CLI-Guide" },
      {
        text: "Messaging",
        collapsed: true,
        items: [
          { text: "Producer", link: "/en/RobustMQ-Kafka/Producer" },
          { text: "Idempotent Producer", link: "/en/RobustMQ-Kafka/Idempotence" },
          { text: "Consumer", link: "/en/RobustMQ-Kafka/Consumer" },
          { text: "Consumer Group", link: "/en/RobustMQ-Kafka/ConsumerGroup" },
          { text: "Next-Gen Consumer Group (KIP-848)", link: "/en/RobustMQ-Kafka/ConsumerGroupNext" },
          { text: "Offset Management", link: "/en/RobustMQ-Kafka/OffsetManagement" },
        ],
      },
      {
        text: "Storage",
        collapsed: true,
        items: [
          { text: "Storage Model", link: "/en/RobustMQ-Kafka/Storage" },
          { text: "Delete Records & Retention", link: "/en/RobustMQ-Kafka/DeleteRecords" },
        ],
      },
      {
        text: "Security",
        collapsed: true,
        items: [
          { text: "Overview", link: "/en/RobustMQ-Kafka/Security/Overview" },
          { text: "SASL/SCRAM Authentication", link: "/en/RobustMQ-Kafka/Security/Authentication-SASL-SCRAM" },
          { text: "Authorization (ACL)", link: "/en/RobustMQ-Kafka/Security/Authorization-ACL" },
          { text: "Client Quota", link: "/en/RobustMQ-Kafka/Security/Quota" },
          { text: "Delegation Token", link: "/en/RobustMQ-Kafka/Security/DelegationToken" },
        ],
      },
      {
        text: "Configuration",
        collapsed: true,
        items: [
          { text: "Broker Config", link: "/en/RobustMQ-Kafka/Configuration/BrokerConfig" },
          { text: "Topic Config", link: "/en/RobustMQ-Kafka/Configuration/TopicConfig" },
          { text: "Cluster Dynamic Config", link: "/en/RobustMQ-Kafka/Configuration/DynamicConfig" },
        ],
      },
      {
        text: "Operations",
        collapsed: true,
        items: [
          { text: "Cluster & Controller", link: "/en/RobustMQ-Kafka/Operations/ClusterAndController" },
          { text: "Advertised Listeners", link: "/en/RobustMQ-Kafka/Operations/AdvertisedListeners" },
          { text: "Deployment", link: "/en/RobustMQ-Kafka/Operations/Deployment" },
        ],
      },
      {
        text: "Client SDK",
        collapsed: true,
        items: [
          { text: "Java Client", link: "/en/RobustMQ-Kafka/SDK/java-client" },
        ],
      },
      { text: "Compatibility & Limitations", link: "/en/RobustMQ-Kafka/Compatibility-and-Limitations" },
      { text: "Roadmap", link: "/en/RobustMQ-Kafka/Roadmap" },
    ],
  },
  {
    text: "RobustMQ AMQP",
    collapsed: true,
    items: [
      { text: "Overview", link: "/en/RobustMQ-AMQP/Overview" },
      { text: "Protocol Support", link: "/en/RobustMQ-AMQP/Protocol" },
    ],
  },
  {
    text: "Data Processing",
    collapsed: true,
    items: [
      {
        text: "Connector",
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
          { text: "Redis", link: "/en/RobustMQ-MQTT/Bridge/Redis" },
          { text: "Webhook", link: "/en/RobustMQ-MQTT/Bridge/Webhook" },
          { text: "OpenTSDB", link: "/en/RobustMQ-MQTT/Bridge/OpenTSDB" },
          { text: "MQTT Bridge", link: "/en/RobustMQ-MQTT/Bridge/MQTT" },
          { text: "ClickHouse", link: "/en/RobustMQ-MQTT/Bridge/ClickHouse" },
          { text: "InfluxDB", link: "/en/RobustMQ-MQTT/Bridge/InfluxDB" },
          { text: "Cassandra", link: "/en/RobustMQ-MQTT/Bridge/Cassandra" },
        ],
      },
      {
        text: "Rule Engine",
        collapsed: true,
        items: [
          { text: "Introduction", link: "/en/RuleEngine/Introduction" },
          { text: "Operator List", link: "/en/RuleEngine/overview" },
          { text: "Processing Demo", link: "/en/RuleEngine/Demo" },
        ],
      },
    ],
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
    text: "Administrator Guide",
    collapsed: true,
    items: [
      { text: "Dashboard", link: "/en/Operations/Dashboard" },
      { text: "Health Check", link: "/en/Operations/HealthCheck" },
      {
        text: "HTTP Rest API",
        collapsed: true,
        items: [
          { text: "Overview", link: "/en/Api/COMMON" },
          { text: "Cluster API", link: "/en/Api/CLUSTER" },
          { text: "MQTT API", link: "/en/Api/MQTT" },
          { text: "Connector API", link: "/en/Api/Connector" },
          { text: "MQ9 API", link: "/en/Api/MQ9" },
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
          { text: "Performance Tuning Guide", link: "/en/Configuration/Tuning" },
        ],
      },
    ],
  },
  {
    text: "Contribution Guide",
    collapsed: true,
    items: [
      {
        text: "Contribution Guide",
        link: "/en/ContributionGuide/GitHub-Contribution-Guide",
      },
      { text: "PR Example", link: "/en/ContributionGuide/Pull-Request-Example" },
      { text: "AI Skills Guide", link: "/en/ContributionGuide/AI-Skills" },
      {
        text: "Contributing Code",
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
          {
            text: "Build and Package",
            link: "/en/ContributionGuide/ContributingCode/Build-and-Package",
          },
        ],
      },
      {
        text: "Contributing Doc",
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
]

export const sidebar = {
  '/en/': items,
};
