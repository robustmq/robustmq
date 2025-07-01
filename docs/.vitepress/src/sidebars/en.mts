export const sidebar = [
    {
        text: "Introduction",
        collapsed: true,
        items: [
            { text: "What is RobustMQ", link: "/OverView/What-is-RobustMQ" },
            { text: "Why RobustMQ", link: "/OverView/Why-RobustMQ" },
            { text: "IGGY Comparison", link: "/OverView/Diff-iggy" },
            { text: "MQ Comparison", link: "/OverView/Diff-MQ" },
            {
                text: "Version planning",
                collapsed: true,
                items: [
                    { text: "2025 RoadMamp", link: "/OverView/RoadMap-2025" },
                    { text: "MQTT Release Planning", link: "/OverView/MQTT-Release" },
                    { text: "Good First Issue", link: "/OverView/Good-First-Issue" },
                ],
            },
        ],
    },
    {
        text: "QuickGuide",
        collapsed: true,
        items: [
            { text: "Overview", link: "/QuickGuide/Overview" },
            { text: "Build", link: "/QuickGuide/mqtt/Build" },
            { text: "First Task", link: "/QuickGuide/mqtt/First-Task" },
        ],
    },
    {
        text: "Architect",
        collapsed: true,
        items: [
            { text: "Overview", link: "/Architect/Overview" },
            { text: "Placement Center", link: "/Architect/Placement-Center" },
            { text: "Broker Server", link: "/Architect/Broker-Server" },
            { text: "Storage Adapter", link: "/Architect/Storage-Adapter" },
            { text: "Journal Server", link: "/Architect/Journal-Server" },
            { text: "Detailed design document", link: "/Architect/Design-Documentation" },
            { text: "Ig Test", link: "/Architect/Test-Case" },
            {
                text: "Configuration",
                collapsed: true,
                items: [
                    { text: "Placement Center", link: "/Architect/Configuration/Placement-Center" },
                    { text: "MQTT Broker", link: "/Architect/Configuration/Mqtt-Server" },
                ],
            },
        ],
    },
    {
        text: "RobustMQ MQTT",
        collapsed: true,
        items: [
            { text: "Overview", link: "/RobustMQ-MQTT/Overview" },
            {
                text: "Install and Deployment",
                collapsed: true,
                items: [
                    { text: "StandaloneMode", link: "/QuickGuide/mqtt/Run-Standalone-Mode" },
                    { text: "ClusterMode", link: "/QuickGuide/mqtt/Run-Cluster-Mode" },
                    { text: "DockerMode", link: "/QuickGuide/mqtt/Run-Docker-Mode" },
                    { text: "K8SMode", link: "/QuickGuide/mqtt/Run-K8S-Mode" },
                ],
            },

            { text: "Public MQTT Server", link: "/RobustMQ-MQTT/PublicMqttServer" },
            { text: "Architecture", link: "/RobustMQ-MQTT/SystemArchitecture" },
            {
                text: "Features",
                collapsed: true,
                items: [
                    { text: "Retain Message", link: "/RobustMQ-MQTT/RetainMessage.md" },
                    { text: "LastWill Message", link: "" },
                    { text: "Exclusive Subscription", link: "" },
                    { text: "Delayed Publish", link: "/RobustMQ-MQTT/DelayMessage.md" },
                    { text: "Automatic subscription", link: "" },
                    { text: "Topic Rewriting", link: "" },
                    { text: "Wildcard Subscription", link: "" },
                    { text: "Session Persistence", link: "" },
                    { text: "Shared Subscription", link: "" },
                    { text: "MQTT Over Quic", link: "" },
                ],
            },
            {
                text: "Security",
                collapsed: true,
                items: [
                    { text: "Authentication", link: "" },
                    { text: "Authorization", link: "" },
                    { text: "Blacklist", link: "" },
                    { text: "Flapping Detect", link: "" },
                ]
            },
            {
                text: "Data integration",
                collapsed: true,
                items: [
                    { text: "Local File", link: "" },
                    { text: "Kafka", link: "" },
                ]
            },

            {
                text: "Observability",
                collapsed: true,
                items: [
                    { text: "System Alarm", link: "/RobustMQ-MQTT/SystemAlarm.mdi" },
                    { text: "Metrics", link: "" },
                    { text: "Trace", link: "" },
                    { text: "Integrate promethrus", link: "" },
                    { text: "Integrate OpenTelemetry", link: "" },
                ]
            },
            { text: "MQTT Dashboard", link: "/zh/RobustMQ-MQTT/Dashboard.md" },
            { text: "Performance Bench", link: "" },
            {
                text: "Client SDK",
                collapsed: true,
                items: [
                    { text: "C SDK", link: "" },
                    { text: "Java SDK", link: "" },
                    { text: "Go SDK", link: "" },
                    { text: "Python SDK", link: "" },
                    { text: "JavaScript SDK", link: "" },
                ]
            },
        ],
    },
    {
        text: "RobustMQ Kafka",
        collapsed: true,
        items: [
            { text: "Overview", link: "/RobustMQ-Kafka/Overview" },
        ],
    },
    {
        text: "RobustMQ Command",
        collapsed: true,
        items: [
            { text: "Overview", link: "/RobustMQ-Command/Overview" },
            { text: "MQTT Command", link: "/RobustMQ-Command/Mqtt-Broker" },
            { text: "Placement Command", link: "/RobustMQ-Command/Placement-Center" },
            { text: "Journal Command", link: "/RobustMQ-Command/Journal Server" },
        ],
    },
    {
        text: "RobustMQ Dashboard",
        collapsed: true,
        items: [
            { text: "Overview", link: "" },
        ],
    },
    {
        text: "ContributionGuide",
        collapsed: true,
        items: [
            { text: "Contribution Guide", link: "/ContributionGuide/GitHub-Contribution-Guide" },
            { text: "PR Example", link: "/ContributionGuide/Pull-Request-Example" },
            {
                text: "ContributingCode",
                collapsed: true,
                items: [
                    { text: "Build Develop Env", link: "/ContributionGuide/ContributingCode/Build-Develop-Env" },
                    { text: "Cargo Running", link: "/ContributionGuide/ContributingCode/Cargo-Running" },
                    { text: "Troubleshooting", link: "/ContributionGuide/ContributingCode/Troubleshooting" },
                    { text: "VSCode Running", link: "/ContributionGuide/ContributingCode/VsCode-Running" },
                    { text: "Code Structure", link: "/ContributionGuide/ContributingCode/Code-Structure" },
                ],
            },
            {
                text: "ContributingDoc",
                collapsed: true,
                items: [
                    { text: "Build Doc Env", link: "/ContributionGuide/ContributingDoc/Build-Doc-Env" },
                    { text: "Doc Contribution Guide", link: "/ContributionGuide/ContributingDoc/Doc-Contribution-Guide" },
                ],
            }
        ],
    }
];
