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
                    { text: "2025 RoadMap", link: "/OverView/RoadMap-2025" },
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
            { text: "Public Server", link: "/QuickGuide/PublicMqttServer" },
            { text: "Build && Package", link: "/QuickGuide/Build-and-Package" },
            { text: "Experience RobustMQ MQTT", link: "/QuickGuide/Experience-MQTT" },
        ],
    },
    {
        text: "Install and Deployment",
        collapsed: true,
        items: [
            { text: "StandaloneMode", link: "/InstallationDeployment/Single-Machine-Cluster" },
            { text: "ClusterMode", link: "/InstallationDeployment/Multi-Node-Cluster" },
            { text: "DockerMode", link: "/InstallationDeployment/Docker-Deployment" },
            { text: "K8SMode", link: "/InstallationDeployment/Kubernetes-Operator" },
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
            { text: "Ig Test", link: "/Architect/Test-Case" }
        ],
    },
    {
        text: "RobustMQ MQTT",
        collapsed: true,
        items: [
            { text: "Overview", link: "/RobustMQ-MQTT/Overview" },
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
            { text: "MQTTX Testing Guide", link: "/RobustMQ-MQTT/MQTTX-Guide" },
            { text: "Performance", link: "" },
            {
                text: "Client SDK",
                collapsed: true,
                items: [
                    { text: "C SDK", link: "/RobustMQ-MQTT/SDK/c-sdk" },
                    { text: "Java SDK", link: "/RobustMQ-MQTT/SDK/java-sdk" },
                    { text: "Go SDK", link: "/RobustMQ-MQTT/SDK/go-sdk" },
                    { text: "Python SDK", link: "/RobustMQ-MQTT/SDK/python-sdk" },
                    { text: "JavaScript SDK", link: "/RobustMQ-MQTT/SDK/javascript-sdk" },
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
        text: "RobustMQ Dashboard",
        collapsed: true,
        items: [
            { text: "Overview", link: "" },
        ],
    },
    {
        text: "RobustMQ Command",
        collapsed: true,
        items: [
            { text: "Overview", link: "/RobustMQ-Command/CLI_COMMON" },
            { text: "Cluster Manager", link: "/RobustMQ-Command/CLI_CLUSTER" },
            { text: "MQTT Manager", link: "/RobustMQ-Command/CLI_MQTT" },
            { text: "Journal Manager", link: "/RobustMQ-Command/CLI_JOURNAL" },
        ],
    },
    {
        text: "HTTP Rest API",
        collapsed: true,
        items: [
            { text: "Overview", link: "/Api/COMMON" },
            { text: "Cluster API", link: "/Api/CLUSTER" },
            { text: "MQTT API", link: "/Api/MQTT" },
        ],
    },
    {
        text: "Configuration",
        collapsed: true,
        items: [
            { text: "Common Configuration", link: "/Configuration/COMMON" },
            { text: "MQTT Configuration", link: "/Configuration/MQTT" },
            { text: "Meta Configuration", link: "/Configuration/META" },
            { text: "Journal Configuration", link: "/Configuration/JOURNAL" },
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
    },
    {
        text: "Blog Articles",
        collapsed: true,
        items: [
            { text: "Redefining Cloud-Native Message Queues with Rust", link: "/Blogs/01" },
        ],
    }
];
