export const sidebar = [
    {
        text: "Introduction",
        collapsed: true,
        items: [
            {text: "What is RobustMQ", link: "/en/OverView/What-is-RobustMQ"},
            {text: "Why RobustMQ", link: "/en/OverView/Why-RobustMQ"},
            {text: "RoadMamp", link: "/en/OverView/RoadMap"},
        ],
    },
    {
        text: "QuickGuide",
        collapsed: true,
        items: [
            {text: "Overview", link: "/en/QuickGuide/Overview"},
            {text: "Stand-loneMode", link: "/en/QuickGuide/Run-Standalone-Mode"},
            {text: "ClusterMode", link: "/en/QuickGuide/Run-Cluster-Mode"},
        ],
    },
    {
        text: "Architect",
        collapsed: true,
        items: [
            {text: "Overview", link: "/en/Architect/Overview"},
            {text: "Placement Center", link: "/en/Architect/Placement-Center"},
            {text: "Broker Server", link: "/en/Architect/Broker-Server"},
            {text: "Storage Adapter", link: "/en/Architect/Storage-Adapter"},
            {text: "Journal Server", link: "/en/Architect/Journal-Server"},
            {text: "Ig Test", link: "/en/Architect/Test-Case"},
            {
                text: "Configuration",
                collapsed: true,
                items: [
                    {text: "Placement Center", link: "/en/Architect/Configuration/Placement-Center"},
                    {text: "MQTT Broker", link: "/en/Architect/Configuration/Mqtt-Server"},
                ],
            },
        ],
    },
    {
        text: "RobustMQ MQTT",
        collapsed: true,
        items: [
            {text: "Overview", link: "/en/RobustMQ-MQTT/Overview"},
        ],
    },
    {
        text: "ContributionGuide",
        collapsed: true,
        items: [
            {text: "Build Env", link: "/en/ContributionGuide/Build-Develop-Env"},
            {text: "Cargo", link: "/en/ContributionGuide/Cargo-Running"},
            {text: "VsCode", link: "/en/ContributionGuide/VsCode-Running"},
            {text: "Code Structure", link: "/en/ContributionGuide/Code-Structure"},
            {text: "Contribution Guide", link: "/en/ContributionGuide/GitHub-Contribution-Guide"},
            {text: "PR Example", link: "/en/ContributionGuide/GitHub-Pull-Request-Example"},
        ],
    },
    {
        text: "VersionRecord",
        collapsed: true,
        items: [
            {text: "0.1.0-beta", link: "/en/VersionRecord/0.1.0-beta"},
        ],
    },
    {
        text: "Other Data",
        collapsed: true,
        items: [
            {text: "RobustMQ Rust China For 2024", link: "/en/OtherData/RobustMQ-Rust-China-For-2024"},
        ],
    },
];
