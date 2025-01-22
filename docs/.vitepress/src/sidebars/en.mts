export const sidebar = [
    {
        text: "Introduction",
        collapsed: true,
        items: [
            { text: "What is RobustMQ", link: "/OverView/What-is-RobustMQ" },
            { text: "Why RobustMQ", link: "/OverView/Why-RobustMQ" },
            { text: "RoadMamp", link: "/OverView/RoadMap" },
        ],
    },
    {
        text: "QuickGuide",
        collapsed: true,
        items: [
            { text: "Overview", link: "/QuickGuide/Overview" },
            { text: "Build", link: "/QuickGuide/Build" },
            {
                text: "RobustMQ MQTT",
                collapsed: true,
                items: [
                    { text: "Stand-loneMode", link: "/QuickGuide/Run-Standalone-Mode" },
                    { text: "ClusterMode", link: "/QuickGuide/Run-Cluster-Mode" },
                    { text: "DockerMode", link: "/QuickGuide/Run-Docker-Mode" },
                    { text: "K8SMode", link: "/QuickGuide/Run-K8S-Mode" },
                ],
            },

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
                    { text: "VsCode Running", link: "/ContributionGuide/ContributingCode/VsCode-Running" },
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
        text: "Other Data",
        collapsed: true,
        items: [],
    },
];
