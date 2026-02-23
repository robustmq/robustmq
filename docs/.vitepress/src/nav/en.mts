
export default function getNavs() {

  return [

    { text: "Home", link: "/en/" },
    { text: "Blog", link: "/en/Blogs/" },
    { text: "Docs", link: "/en/OverView/What-is-RobustMQ" },
    {
      text: "Demo",
      items: [
        { text: "🖥️ Dashboard", link: "http://demo.robustmq.com:8080/" },
        { text: "📊 Grafana", link: "http://demo.robustmq.com:3000/d/robustmq-mqtt-broker/robustmq-mqtt-broker-dashboard" },
        { text: "📈 Prometheus", link: "http://demo.robustmq.com:9092/classic/graph" },
        { text: "🔌 How to Test", link: "/en/QuickGuide/PublicMqttServer" },
      ]
    },
    {
      text: "Learn MQTT",
      items: [
        { text: "📘 MQTT 5 Specification", link: "https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html" },
        { text: "📗 MQTT 3.1.1 Specification", link: "http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html" },
        { text: "📕 MQTT 3.1 Specification", link: "https://public.dhe.ibm.com/software/dw/webservices/ws-mqtt/mqtt-v3r1.html" },
        { text: "📙 MQTT-SN v1.2", link: "https://www.oasis-open.org/committees/document.php?document_id=66091&wg_abbrev=mqtt" },
      ]
    },

  ]

};
