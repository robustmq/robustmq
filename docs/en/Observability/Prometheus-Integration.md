# Prometheus Integration

RobustMQ has built-in Prometheus metrics export. Metrics are always exposed via the Admin HTTP API's `GET /metrics`, sharing `http_port` (default `58080`) — there's no separate enable switch or port to configure.

## Verify Metrics Export

Replace `58080` below with the `http_port` configured in your `config/server.toml`:

```bash
# Check metrics endpoint
curl http://localhost:58080/metrics

# Verify metrics data
curl http://localhost:58080/metrics | grep mqtt_
```

## Configure Prometheus

Add RobustMQ as a scrape target in Prometheus configuration:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'robustmq'
    static_configs:
      - targets: ['localhost:58080']
    scrape_interval: 15s
    metrics_path: /metrics
```

## Cluster Configuration

For multi-node deployment (replace with each node's actual `http_port`):

```yaml
scrape_configs:
  - job_name: 'robustmq-cluster'
    static_configs:
      - targets:
        - 'robustmq-node1:58080'
        - 'robustmq-node2:58080'
        - 'robustmq-node3:58080'
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
# Check port listening (http_port defaults to 58080)
netstat -tlnp | grep 58080

# Check the configured http_port
grep "http_port" config/server.toml
```

### Prometheus Cannot Scrape
```bash
# Check network connectivity
telnet robustmq-host 58080

# View Prometheus targets status
curl http://prometheus:9090/api/v1/targets
```

With the above configuration, RobustMQ monitoring metrics will be automatically collected by Prometheus for alerting, visualization, and performance analysis.
