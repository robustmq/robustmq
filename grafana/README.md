# RobustMQ Grafana Monitoring Dashboard

This directory contains Grafana dashboard templates for monitoring RobustMQ MQTT Broker.

## 📊 Dashboard Overview

The `robustmq-mqtt-broker-dashboard.json` provides comprehensive monitoring for RobustMQ MQTT Broker with the following sections:

### 🏗️ Server Overview
- **Active Connections**: Current number of active client connections
- **Max Connections Reached**: Maximum connections reached since startup
- **Active Threads**: Thread pool status by network type and thread type

### 🚀 Performance Metrics
- **Request Total Latency**: End-to-end request processing time (95th/50th percentiles)
- **Request Handler Latency**: Handler processing time (95th/50th percentiles)
- **Network Queue Sizes**: Request and response queue depths

### 📦 MQTT Packets
- **Packets Received Rate**: Incoming packet rates by type and network
- **Packets Sent Rate**: Outgoing packet rates by type, network, and QoS
- **Network Traffic**: Bytes received/sent rates

### 🔌 Client Connections
- **Client Connection Rate**: Rate of new client connections
- **Error Rates**: Packet errors, authentication errors, and connection errors

### 📝 Message Handling
- **Retained Messages**: Retained message handling by QoS level
- **Dropped Messages**: Messages dropped due to no subscribers or discard policy

## 🛠️ Setup Instructions

### Prerequisites
- Grafana 8.0 or later
- Prometheus data source configured
- RobustMQ MQTT Broker with metrics enabled

### 1. Configure RobustMQ Metrics Export

Ensure your RobustMQ MQTT Broker is configured to export Prometheus metrics. The broker should expose metrics on the configured metrics endpoint (typically `:9090/metrics`).

### 2. Configure Prometheus

Add RobustMQ MQTT Broker as a target in your Prometheus configuration:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'robustmq-mqtt-broker'
    static_configs:
      - targets: ['localhost:9090']  # Adjust to your broker's metrics endpoint
    scrape_interval: 15s
    metrics_path: /metrics
```

### 3. Import Dashboard to Grafana

#### Option 1: Import via Grafana UI
1. Open Grafana web interface
2. Navigate to **Dashboards** → **Import**
3. Upload `robustmq-mqtt-broker-dashboard.json` file
4. Select your Prometheus data source
5. Click **Import**

#### Option 2: Import via Grafana API
```bash
curl -X POST \
  http://your-grafana-host:3000/api/dashboards/db \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d @robustmq-mqtt-broker-dashboard.json
```

### 4. Configure Data Source Variable

The dashboard uses a template variable `${DS_PROMETHEUS}` for the Prometheus data source. When importing:

1. Select your Prometheus data source from the dropdown
2. The dashboard will automatically use this data source for all queries

## 📈 Key Metrics Explained

### Server Performance Metrics

| Metric | Description | Alert Threshold |
|--------|-------------|-----------------|
| `mqtt_connections_count` | Current active MQTT connections | > 5000 (warning) |
| `handler_total_ms` | End-to-end handler latency | P95 > 100ms |
| `handler_apply_ms` | Handler command.apply() latency | Track latency trends |
| `handler_queue_size` | Handler queue depth | > 1000 (warning) |

### MQTT Packet Metrics

| Metric | Description | Labels |
|--------|-------------|--------|
| `mqtt_packets_received_total` | Total packets received | `network` |
| `mqtt_packets_sent_total` | Total packets sent | `network`, `qos` |
| `mqtt_packets_*_received_total` | Specific packet type received | `network` |
| `mqtt_packets_*_sent_total` | Specific packet type sent | `network`, `qos` |
| `mqtt_bytes_received_total` | Bytes received | `network` |
| `mqtt_bytes_sent_total` | Bytes sent | `network`, `qos` |

### Error Metrics

| Metric | Description | Labels |
|--------|-------------|--------|
| `mqtt_packets_received_error_total` | Malformed packets | `network` |
| `mqtt_packets_connack_auth_error_total` | Authentication failures | `network` |
| `mqtt_packets_connack_error_total` | Connection errors | `network` |

### Message Handling Metrics

| Metric | Description | Labels |
|--------|-------------|--------|
| `mqtt_retain_packets_received_total` | Retained messages received | `qos` |
| `mqtt_retain_packets_sent_total` | Retained messages sent | `qos` |
| `mqtt_messages_dropped_no_subscribers_total` | Messages dropped (no subscribers) | — |
| `mqtt_connector_messages_discarded_total` | Connector messages dropped (discard policy) | `tenant`, `connector_type`, `connector_name`, `strategy` |

## 🚨 Recommended Alerts

### High Priority Alerts
```yaml
# High connection count
- alert: RobustMQHighConnectionCount
  expr: mqtt_connections_count > 5000
  labels:
    severity: warning
  annotations:
    summary: "High number of active connections"

# High latency
- alert: RobustMQHighRequestLatency
  expr: histogram_quantile(0.95, sum by (le, network) (rate(handler_total_ms_bucket[5m]))) > 100
  labels:
    severity: warning
  annotations:
    summary: "High request latency detected"

# Authentication failures
- alert: RobustMQAuthenticationFailures
  expr: sum by (network) (rate(mqtt_packets_connack_auth_error_total[5m])) > 10
  labels:
    severity: critical
  annotations:
    summary: "High authentication failure rate"
```

### Medium Priority Alerts
```yaml
# Queue depth
- alert: RobustMQHighQueueDepth
  expr: handler_queue_size{label="handler"} > 1000
  labels:
    severity: warning
  annotations:
    summary: "High network queue depth"

# Message drops
- alert: RobustMQHighMessageDrops
  expr: rate(mqtt_messages_dropped_no_subscribers_total[5m]) > 100
  labels:
    severity: warning
  annotations:
    summary: "High message drop rate"
```

## 🔧 Customization

### Adding Custom Panels
To add custom metrics panels:

1. Edit the dashboard in Grafana
2. Add new panel with your desired metric query
3. Export the updated dashboard
4. Replace the JSON file

### Modifying Time Ranges
The dashboard defaults to:
- **Time Range**: Last 1 hour
- **Refresh Rate**: 5 seconds

Adjust these in the dashboard settings as needed.

### Network Labels
The dashboard assumes these network types based on the code analysis:
- `tcp` - TCP connections
- `websocket` - WebSocket connections
- `ssl` - SSL/TLS connections
- `wss` - WebSocket Secure connections

## 🐛 Troubleshooting

### No Data Showing
1. Verify Prometheus is scraping RobustMQ metrics endpoint
2. Check that RobustMQ is exposing metrics on the configured port
3. Ensure data source is correctly configured in Grafana

### Missing Metrics
1. Confirm RobustMQ version supports all metrics
2. Check that the specific features (e.g., retained messages) are enabled
3. Verify metric names match your RobustMQ version

### Performance Issues
1. Reduce query frequency for high-cardinality metrics
2. Adjust time ranges for heavy queries
3. Consider using recording rules in Prometheus for complex queries

## 📚 Additional Resources

- [RobustMQ Documentation](https://robustmq.com/)
- [Prometheus Monitoring](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)
- [MQTT Protocol Specifications](https://mqtt.org/mqtt-specification/)

## 🤝 Contributing

To contribute improvements to this dashboard:

1. Make changes in Grafana UI
2. Export the updated dashboard JSON
3. Update this README if needed
4. Submit a pull request

## 📝 Version History

- **v1.0**: Initial dashboard with comprehensive MQTT broker monitoring
  - Server performance metrics
  - MQTT packet statistics
  - Client connection monitoring
  - Error tracking
  - Message handling metrics
