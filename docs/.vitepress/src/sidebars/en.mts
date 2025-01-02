export const sidebar = [
    {
        text: "Introduction",
        collapsed: true,
        items: [
            {text: "What is RobustMQ", link: "/OverView/What-is-RobustMQ"},
            {text: "Why RobustMQ", link: "/OverView/Why-RobustMQ"},
            {text: "RoadMamp", link: "/OverView/RoadMap"},
        ],
    },
    {
        text: "QuickGuide",
        collapsed: true,
        items: [
            {text: "Overview", link: "/QuickGuide/Overview"},
            {text: "Stand-loneMode", link: "/QuickGuide/Run-Standalone-Mode"},
            {text: "ClusterMode", link: "/QuickGuide/Run-Cluster-Mode"},
        ],
    },
    {
        text: "Architect",
        collapsed: true,
        items: [
            {text: "Overview", link: "/Architect/Overview"},
            {text: "Placement Center", link: "/Architect/Placement-Center"},
            {text: "Broker Server", link: "/Architect/Broker-Server"},
            {text: "Storage Adapter", link: "/Architect/Storage-Adapter"},
            {text: "Journal Server", link: "/Architect/Journal-Server"},
            {text: "Ig Test", link: "/Architect/Test-Case"},
            {
                text: "Configuration",
                collapsed: true,
                items: [
                    {text: "Placement Center", link: "/Architect/Configuration/Placement-Center"},
                    {text: "MQTT Broker", link: "/Architect/Configuration/Mqtt-Server"},
                ],
            },
        ],
    },
    {
        text: "RobustMQ MQTT",
        collapsed: true,
        items: [
            {text: "Overview", link: "/RobustMQ-MQTT/Overview"},
        ],
    },
    {
        text: "ContributionGuide",
        collapsed: true,
        items: [
            {text: "Build Env", link: "/ContributionGuide/Build-Develop-Env"},
            {text: "Cargo", link: "/ContributionGuide/Cargo-Running"},
            {text: "VsCode", link: "/ContributionGuide/VsCode-Running"},
            {text: "Code Structure", link: "/ContributionGuide/Code-Structure"},
            {text: "Contribution Guide", link: "/ContributionGuide/GitHub-Contribution-Guide"},
            {text: "PR Example", link: "/ContributionGuide/Pull-Request-Example"},
        ],
    },
    {
        text: "VersionRecord",
        collapsed: true,
        items: [
            {text: "0.1.0-beta", link: "/VersionRecord/0.1.0-beta"},
        ],
    },
    {
        text: "Other Data",
        collapsed: true,
        items: [
            {text: "RobustMQ Rust China For 2024", link: "/OtherData/RobustMQ-Rust-China-For-2024"},
        ],
    },
];
