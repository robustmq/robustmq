# Prometheus Integration

RobustMQ provides built-in Prometheus metrics export functionality. Simply enable it with basic configuration to integrate with Prometheus monitoring system.

## Configure RobustMQ

Enable Prometheus metrics export in RobustMQ configuration file:

```toml
# config/server.toml
[prometheus]
enable = true
port = 9091
```

Restart RobustMQ service to apply the configuration.

## Verify Metrics Export

```bash
# Check metrics endpoint
curl `http://localhost:9091/metrics`

# Verify metrics data
curl `http://localhost:9091/metrics` | grep mqtt_
```

## Configure Prometheus

Add RobustMQ as a scrape target in Prometheus configuration:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'robustmq'
    static_configs:
      - targets: ['localhost:9091']
    scrape_interval: 15s
    metrics_path: /metrics
```

## Cluster Configuration

For multi-node deployment:

```yaml
scrape_configs:
  - job_name: 'robustmq-cluster'
    static_configs:
      - targets:
        - 'robustmq-node1:9091'
        - 'robustmq-node2:9091'
        - 'robustmq-node3:9091'
```

## Available Metrics

RobustMQ exports the following types of metrics:

- **MQTT Protocol Metrics**: Packet send/receive, connection management, authentication statistics
- **Performance Metrics**: Request latency, processing time, queue depth
- **Business Metrics**: Session count, topic statistics, message processing
- **System Metrics**: Network connections, thread pools, error statistics

## Common Queries

```text
# Current connection count
mqtt_connections_count

# MQTT packet receive rate
rate(mqtt_packets_received[5m])

# Request processing latency P95
histogram_quantile(0.95, rate(request_total_ms_bucket[5m]))

# Authentication failure rate
rate(mqtt_auth_failed[5m])
```

## Troubleshooting

### Metrics Endpoint Inaccessible
```bash
# Check port listening
netstat -tlnp | grep 9091

# Check configuration
grep -A 3 "\[prometheus\]" config/server.toml
```

### Prometheus Cannot Scrape
```bash
# Check network connectivity
telnet robustmq-host 9091

# View Prometheus targets status
curl http://prometheus:9090/api/v1/targets
```

With the above configuration, RobustMQ monitoring metrics will be automatically collected by Prometheus for alerting, visualization, and performance analysis.
