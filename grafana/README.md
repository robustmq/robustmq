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
| `broker_connections_num` | Current active connections | > 5000 (warning) |
| `broker_connections_max` | Peak connections reached | Track capacity planning |
| `request_total_ms` | End-to-end latency | 95th percentile > 100ms |
| `request_handler_ms` | Handler processing time | 95th percentile > 50ms |
| `network_queue_num` | Queue depth | > 1000 (warning) |

### MQTT Packet Metrics

| Metric | Description | Labels |
|--------|-------------|--------|
| `packets_received` | Total packets received | `network` |
| `packets_sent` | Total packets sent | `network`, `qos` |
| `packets_*_received` | Specific packet type received | `network` |
| `packets_*_sent` | Specific packet type sent | `network`, `qos` |
| `bytes_received` | Bytes received | `network` |
| `bytes_sent` | Bytes sent | `network`, `qos` |

### Error Metrics

| Metric | Description | Labels |
|--------|-------------|--------|
| `packets_received_error` | Malformed packets | `network` |
| `packets_connack_auth_error` | Authentication failures | `network` |
| `packets_connack_error` | Connection errors | `network` |

### Message Handling Metrics

| Metric | Description | Labels |
|--------|-------------|--------|
| `retain_packets_received` | Retained messages received | `qos` |
| `retain_packets_sent` | Retained messages sent | `qos` |
| `messages_dropped_no_subscribers` | Messages dropped (no subscribers) | `qos` |
| `messages_dropped_discard` | Messages dropped (discard policy) | `qos` |

## 🚨 Recommended Alerts

### High Priority Alerts
```yaml
# High connection count
- alert: HighConnectionCount
  expr: broker_connections_num > 5000
  labels:
    severity: warning
  annotations:
    summary: "High number of active connections"

# High latency
- alert: HighLatency
  expr: histogram_quantile(0.95, rate(request_total_ms_bucket[5m])) > 100
  labels:
    severity: warning
  annotations:
    summary: "High request latency detected"

# Authentication failures
- alert: AuthenticationFailures
  expr: rate(packets_connack_auth_error[5m]) > 10
  labels:
    severity: critical
  annotations:
    summary: "High authentication failure rate"
```

### Medium Priority Alerts
```yaml
# Queue depth
- alert: HighQueueDepth
  expr: network_queue_num > 1000
  labels:
    severity: warning
  annotations:
    summary: "High network queue depth"

# Message drops
- alert: MessageDrops
  expr: rate(messages_dropped_no_subscribers[5m]) > 100
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