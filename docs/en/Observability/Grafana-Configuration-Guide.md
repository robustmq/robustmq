# Grafana Configuration Guide

This document describes how to configure the Grafana monitoring system for RobustMQ, including quick deployment, data source configuration, and dashboard import.

## Requirements

- Grafana 8.0+, Prometheus 2.30+, Docker 20.10+ (optional)
- Default ports: RobustMQ metrics (9091), Prometheus (9090), Grafana (3000), Alertmanager (9093)

## Quick Deployment

### Using Docker Compose (Recommended)

```bash
cd grafana/
docker-compose -f docker-compose.monitoring.yml up -d
```

This starts the following services:

| Service | Address | Description |
|---------|---------|-------------|
| Grafana | `http://localhost:3000` | Default login: admin/admin |
| Prometheus | `http://localhost:9090` | Metrics collection & query |
| Alertmanager | `http://localhost:9093` | Alert management |
| Node Exporter | `http://localhost:9100` | System metrics (optional) |

## RobustMQ Configuration

Enable Prometheus metrics export in `config/server.toml`:

```toml
[prometheus]
enable = true
port = 9091
```

Verify metrics are exposed:

```bash
curl http://localhost:9091/metrics
```

## Prometheus Configuration

The project provides an example configuration at `grafana/prometheus-config-example.yml` with the following scrape targets:

### Single Node

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "robustmq-alerts.yml"

scrape_configs:
  - job_name: 'robustmq-mqtt-broker'
    static_configs:
      - targets: ['localhost:9091']
    metrics_path: /metrics
```

### Cluster

```yaml
scrape_configs:
  - job_name: 'robustmq-mqtt-broker-cluster'
    static_configs:
      - targets:
        - 'node1:9091'
        - 'node2:9091'
        - 'node3:9091'
    metrics_path: /metrics
```

### Multiple Services

If running Meta Service and Journal Server alongside the broker:

```yaml
scrape_configs:
  - job_name: 'robustmq-meta-service'
    static_configs:
      - targets: ['localhost:9092']
    metrics_path: /metrics

  - job_name: 'robustmq-journal-server'
    static_configs:
      - targets: ['localhost:9093']
    metrics_path: /metrics
```

## Grafana Configuration

### Adding Prometheus Data Source

**Via Web UI:**
1. Log in to Grafana (`http://localhost:3000`)
2. **Configuration** → **Data Sources** → **Add data source**
3. Select **Prometheus**, set URL to `http://localhost:9090` (or `http://prometheus:9090` in Docker)

**Via Provisioning File:**

```yaml
# /etc/grafana/provisioning/datasources/prometheus.yml
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    url: http://prometheus:9090
    isDefault: true
```

### Importing the Dashboard

RobustMQ provides a pre-built dashboard at `grafana/robustmq-broker.json`.

**Web UI Import:**
1. **Dashboards** → **Import**
2. Upload `grafana/robustmq-broker.json`
3. Select your Prometheus data source in the `DS_PROMETHEUS` dropdown
4. Click **Import**

**API Import:**

```bash
curl -X POST http://localhost:3000/api/dashboards/db \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d @grafana/robustmq-broker.json
```

## Dashboard Panel Guide

`robustmq-broker.json` contains the following sections:

### Resource (Overview)

Top stat panels showing current system state:

| Panel | Metric | Description |
|-------|--------|-------------|
| Connections | `mqtt_connections_count` | Current MQTT connections |
| Sessions | `mqtt_sessions_count` | Current sessions |
| Topics | `mqtt_topics_count` | Current topics |
| Subscribers | `mqtt_subscribers_count` | Current total subscribers |
| Shared Subscriptions | `mqtt_subscriptions_shared_count` | Current shared subscriptions |
| Retained Messages | `mqtt_retained_count` | Current retained messages |

Below the stat panels, timeseries panels show resource trends including connection rates, session create/delete rates, topic message read/write rates, subscription success/failure rates, shared subscription type breakdown, and retained message send/receive rates.

### 🌐 Network

| Panel | Description |
|-------|-------------|
| Handler Total Latency | End-to-end request latency percentiles (P50/P95/P99) |
| Handler Queue Wait Latency | Queue wait time percentiles |
| Handler Apply Latency | command.apply() execution time percentiles |
| Response Write Latency | Response write-back latency percentiles |

### 📈 MQTT Protocol

| Panel | Description |
|-------|-------------|
| MQTT Received Packet Rate (QPS) | Received packet rate by type |
| MQTT Sent Packet Rate (QPS) | Sent packet rate by type |
| MQTT Packet Process Latency Percentiles | Packet processing latency percentiles |
| MQTT Packet Process P99 Latency by Type | P99 processing latency by packet type |
| MQTT Packet Process QPS by Type | Processing rate by packet type |
| MQTT Packet Process Avg Latency by Type | Average processing latency by packet type |

### 🔗 gRPC Server

| Panel | Description |
|-------|-------------|
| gRPC Requests Rate | gRPC request rate (req/s) |
| gRPC QPS by Method | Per-method gRPC request rate |
| gRPC P99 Latency by Method | Per-method P99 latency |

### 📡 gRPC Client

| Panel | Description |
|-------|-------------|
| gRPC Client Call P99 Latency by Method | P99 latency of each outgoing gRPC client call |
| gRPC Client Call Latency Percentiles | Overall client call latency percentiles (P50/P95/P99/P999) |
| gRPC Client Call QPS by Method | QPS of each outgoing gRPC client call by method |

> This section shows the latency of outgoing gRPC calls from the Broker to Meta Service etc., helping identify performance bottlenecks in flows like connection establishment.

### 🌍 HTTP Admin

| Panel | Description |
|-------|-------------|
| HTTP Requests Rate | HTTP Admin request rate (req/s) |
| HTTP QPS by Endpoint | Per-endpoint request rate |
| HTTP Admin P99 Latency by Endpoint | Per-endpoint P99 latency |

### 📦 Raft Machine

| Panel | Description |
|-------|-------------|
| Raft Write Rate / Success Rate / Failure Rate | Raft write request/success/failure rates |
| Raft RPC Rate | Raft RPC request rate |
| Raft Write QPS (by Machine) | Write QPS by state machine type |
| Raft Write Latency (by Machine) | Write latency by state machine type |
| Raft RPC QPS (by Machine / RPC Type) | RPC QPS by state machine and RPC type |
| Raft RPC Latency (by Machine / RPC Type) | RPC latency by state machine and RPC type |

> Raft RPC metrics only show data in multi-node cluster deployments.

### 📖 RocksDB

| Panel | Description |
|-------|-------------|
| RocksDB QPS by Operation | QPS by operation type (save/get/delete/list) |
| RocksDB QPS by Source | QPS by data source |
| RocksDB Write Latency | Write operation latency percentiles |
| RocksDB Read (Get) Latency | Read operation latency percentiles |

### ⏱ Delay Message

| Panel | Description |
|-------|-------------|
| Delay Message Enqueue / Deliver / Failure Rate | Enqueue/deliver/failure rates |
| Enqueue Latency Percentiles | Enqueue latency percentiles |
| Deliver Latency Percentiles | Delivery latency percentiles |

> Delay message metrics only show data when the delay publish feature is actively used.

## Alert Configuration

### Pre-built Alert Rules

The project provides `grafana/robustmq-alerts.yml` with the following rules:

| Alert | Severity | Condition | Description |
|-------|----------|-----------|-------------|
| RobustMQBrokerDown | Critical | `up == 0` | Broker instance unreachable |
| RobustMQHighRequestLatency | Warning | P95 latency > 100ms for 10m | Elevated request latency |
| RobustMQCriticalRequestLatency | Critical | P95 latency > 500ms for 5m | Severe request latency |
| RobustMQAuthenticationFailures | Critical | Auth failures > 10/s for 2m | Frequent authentication failures |
| RobustMQConnectionErrors | Warning | Connection errors > 5/s for 5m | Frequent connection errors |
| RobustMQHighQueueDepth | Warning | Queue depth > 1000 for 5m | Queue backlog |
| RobustMQCriticalQueueDepth | Critical | Queue depth > 5000 for 2m | Severe queue backlog |
| RobustMQHighMessageDrops | Warning | Message drops > 100/s for 5m | Frequent no-subscriber drops |
| RobustMQHighThreadUtilization | Warning | Active threads > 50 for 10m | High thread count |

### Alertmanager Configuration

```yaml
# alertmanager.yml
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'alertmanager@robustmq.com'

route:
  group_by: ['alertname']
  repeat_interval: 1h
  receiver: 'default'

receivers:
  - name: 'default'
    email_configs:
      - to: 'admin@robustmq.com'
```

### Custom Alert Rules

Add custom rules to `grafana/robustmq-alerts.yml`:

```yaml
groups:
  - name: robustmq.custom
    rules:
      - alert: HighGrpcErrorRate
        expr: rate(grpc_errors_total[5m]) / rate(grpc_requests_total[5m]) > 0.05
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "gRPC error rate exceeds 5%"
          description: "Current error rate: {{ $value | humanizePercentage }}"

      - alert: RocksDBSlowOperations
        expr: histogram_quantile(0.95, rate(rocksdb_operation_ms_bucket[5m])) > 100
        for: 3m
        labels:
          severity: warning
        annotations:
          summary: "RocksDB P95 operation latency exceeds 100ms"
```

## Performance Optimization

### Prometheus Optimization

**Storage configuration:**

```bash
# Startup arguments
--storage.tsdb.retention.time=30d
--storage.tsdb.retention.size=50GB
```

**Recording rules (pre-computation):**

Create recording rules for frequently queried expressions:

```yaml
groups:
  - name: robustmq.recording
    rules:
      - record: robustmq:request_latency_95th
        expr: histogram_quantile(0.95, rate(handler_total_ms_bucket[5m]))

      - record: robustmq:packet_rate_received
        expr: rate(mqtt_packets_received_total[5m])

      - record: robustmq:packet_rate_sent
        expr: rate(mqtt_packets_sent_total[5m])

      - record: robustmq:error_rate_total
        expr: >
          rate(mqtt_packets_received_error_total[5m])
          + rate(mqtt_packets_connack_auth_error_total[5m])
          + rate(mqtt_packets_connack_error_total[5m])
```

### Grafana Optimization

- Use recording rules to reduce complex real-time aggregation queries
- Set appropriate panel refresh intervals (recommended 15s - 1m)
- Avoid wide-range aggregation on high-cardinality labels (e.g., `client_id`, `connection_id`)
- Use larger `rate()` windows (e.g., `[5m]` instead of `[1m]`) for historical data queries

## Related Files

| File | Description |
|------|-------------|
| `grafana/robustmq-broker.json` | Grafana dashboard definition |
| `grafana/prometheus-config-example.yml` | Prometheus scrape configuration example |
| `grafana/robustmq-alerts.yml` | Alert rules definition |
| `grafana/docker-compose.monitoring.yml` | Docker Compose monitoring stack |
| `config/server.toml` | RobustMQ server configuration (includes Prometheus port) |
| `docs/en/Observability/Infrastructure-Metrics.md` | Complete metrics reference |
