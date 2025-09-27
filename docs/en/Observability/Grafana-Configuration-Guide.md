# Grafana Configuration Guide

This document provides detailed instructions on how to configure Grafana monitoring system for RobustMQ, including Prometheus data source configuration, dashboard import, and alerting setup.

## Environment Preparation

### System Requirements

- **Grafana**: 8.0 or higher
- **Prometheus**: 2.30 or higher
- **RobustMQ**: With metrics export enabled
- **Docker**: 20.10 or higher (optional)

### Port Planning

| Service | Default Port | Purpose |
|---------|--------------|---------|
| RobustMQ Broker | 9091 | Metrics export |
| Prometheus | 9090 | Metrics collection |
| Grafana | 3000 | Visualization interface |
| Alertmanager | 9093 | Alert management |

## Quick Deployment

### Using Docker Compose (Recommended)

RobustMQ provides a complete monitoring stack Docker Compose configuration:

```bash
# 1. Navigate to grafana directory
cd grafana/

# 2. Start monitoring stack
docker-compose -f docker-compose.monitoring.yml up -d

# 3. Verify service status
docker-compose -f docker-compose.monitoring.yml ps
```

**Service Access URLs:**
- Grafana: `localhost:3000` (admin/admin)
- Prometheus: `localhost:9090`
- Alertmanager: `localhost:9093`

### Manual Installation

#### 1. Install Prometheus

```bash
# Download Prometheus
wget https://github.com/prometheus/prometheus/releases/download/v2.40.0/prometheus-2.40.0.linux-amd64.tar.gz
tar xvfz prometheus-2.40.0.linux-amd64.tar.gz
cd prometheus-2.40.0.linux-amd64

# Copy configuration file
cp /path/to/robustmq/grafana/prometheus-config-example.yml ./prometheus.yml

# Start Prometheus
./prometheus --config.file=prometheus.yml --storage.tsdb.path=./data
```

#### 2. Install Grafana

```bash
# Ubuntu/Debian
sudo apt-get install -y software-properties-common
sudo add-apt-repository "deb https://packages.grafana.com/oss/deb stable main"
wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
sudo apt-get update
sudo apt-get install grafana

# CentOS/RHEL
sudo yum install -y https://dl.grafana.com/oss/release/grafana-9.0.0-1.x86_64.rpm

# Start Grafana
sudo systemctl start grafana-server
sudo systemctl enable grafana-server
```

## RobustMQ Configuration

### Enable Metrics Export

Ensure metrics export is enabled in RobustMQ configuration file:

```toml
# config/server.toml
[prometheus]
# Enable Prometheus metrics export
enable = true
# Metrics export port
port = 9091
```

### Verify Metrics Export

```bash
# Check RobustMQ Broker metrics
curl `http://localhost:9091/metrics`
```

## Prometheus Configuration

### Basic Configuration

Use the configuration template provided by RobustMQ:

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

# Alert rules
rule_files:
  - "robustmq-alerts.yml"

# Scrape configurations
scrape_configs:
  # RobustMQ Broker
  - job_name: 'robustmq-broker'
    static_configs:
      - targets: ['localhost:9091']
    scrape_interval: 15s
    metrics_path: /metrics
```

### Cluster Configuration

For cluster deployment, configure multiple targets:

```yaml
scrape_configs:
  - job_name: 'robustmq-broker-cluster'
    static_configs:
      - targets:
        - 'robustmq-node1:9091'
        - 'robustmq-node2:9091'
        - 'robustmq-node3:9091'
    relabel_configs:
      - target_label: cluster
        replacement: 'production'
```

### Service Discovery Configuration

Using Kubernetes service discovery:

```yaml
scrape_configs:
  - job_name: 'robustmq-k8s'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        action: keep
        regex: robustmq
```

## Grafana Configuration

### Data Source Configuration

#### 1. Add Prometheus Data Source

Through Grafana Web Interface:

1. Login to Grafana (`localhost:3000`)
2. Go to **Configuration** → **Data Sources**
3. Click **Add data source**
4. Select **Prometheus**
5. Configure connection information:
   - **URL**: `http://localhost:9090`
   - **Access**: Server (default)
   - **Scrape interval**: 15s

#### 2. Via Configuration File

Create data source configuration file:

```yaml
# /etc/grafana/provisioning/datasources/prometheus.yml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: true
```

### Dashboard Import

#### Method 1: Web Interface Import

1. Go to **Dashboards** → **Import**
2. Upload `robustmq-mqtt-broker-dashboard.json` file
3. Select Prometheus data source
4. Click **Import**

#### Method 2: API Import

```bash
# Import dashboard using Grafana API
curl -X POST \
  `http://localhost:3000/api/dashboards/db` \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d @robustmq-mqtt-broker-dashboard.json
```

#### Method 3: Automatic Configuration

Create dashboard configuration file:

```yaml
# /etc/grafana/provisioning/dashboards/robustmq.yml
apiVersion: 1

providers:
  - name: 'robustmq'
    orgId: 1
    folder: 'RobustMQ'
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    options:
      path: /var/lib/grafana/dashboards
```

### Dashboard Features

#### 📊 Server Overview Panel

- **Current Connections**: Real-time display of active connection count
- **Max Connections**: Shows connection peaks for capacity planning
- **Active Threads**: Grouped by network type and thread type

#### 🚀 Performance Metrics Panel

- **Request Total Latency**: Shows P95/P50 percentile latency
- **Handler Latency**: Shows request handler execution time
- **Network Queue Size**: Shows request and response queue depth

#### 📦 MQTT Packets Panel

- **Received Packet Rate**: Inbound packets grouped by type and network
- **Sent Packet Rate**: Outbound packets grouped by type, network, and QoS
- **Network Traffic**: Byte-level send/receive rates

#### 🔌 Client Connections Panel

- **Client Connection Rate**: New connection establishment rate
- **Error Rates**: Packet errors, authentication errors, connection error statistics

#### 📝 Message Handling Panel

- **Retained Messages**: Retained message handling by QoS level
- **Dropped Messages**: Message loss due to no subscribers or discard policy

## Alert Configuration

### Alertmanager Configuration

Create Alertmanager configuration file:

```yaml
# alertmanager.yml
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'alertmanager@robustmq.com'

route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'web.hook'

receivers:
  - name: 'web.hook'
    email_configs:
      - to: 'admin@robustmq.com'
        subject: 'RobustMQ Alert: {{ .GroupLabels.alertname }}'
        body: |
          {{ range .Alerts }}
          Alert: {{ .Annotations.summary }}
          Description: {{ .Annotations.description }}
          {{ end }}

  - name: 'slack'
    slack_configs:
      - api_url: 'YOUR_SLACK_WEBHOOK_URL'
        channel: '#alerts'
        title: 'RobustMQ Alert'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'
```

### Alert Rules Description

RobustMQ provides pre-configured alert rules covering the following scenarios:

#### 🚨 High Priority Alerts

- **Service Down**: `RobustMQBrokerDown`
- **Authentication Failures**: `RobustMQAuthenticationFailures`
- **Critical Latency**: `RobustMQCriticalRequestLatency`
- **Queue Backlog**: `RobustMQCriticalQueueDepth`

#### ⚠️ Medium Priority Alerts

- **High Connection Count**: `RobustMQHighConnectionCount`
- **Connection Errors**: `RobustMQConnectionErrors`
- **Packet Errors**: `RobustMQPacketErrors`
- **Message Drops**: `RobustMQHighMessageDrops`

#### ℹ️ Info Level Alerts

- **Low Throughput**: `RobustMQLowThroughput`
- **Capacity Planning**: `RobustMQCapacityPlanningNeeded`

### Custom Alert Rules

Add custom alert rules:

```yaml
# custom-alerts.yml
groups:
  - name: robustmq.custom
    rules:
      - alert: CustomHighCPU
        expr: cpu_usage_percent > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "RobustMQ CPU usage high"
          description: "CPU usage is {{ $value }}%"
```

## Performance Optimization

### Prometheus Optimization

#### Storage Optimization

```yaml
# prometheus.yml
global:
  # Reduce scrape interval to lower storage pressure
  scrape_interval: 30s
  
# Configure data retention policy
command_args:
  - '--storage.tsdb.retention.time=30d'
  - '--storage.tsdb.retention.size=50GB'
```

#### Recording Rules

Use pre-computed rules to improve query performance:

```yaml
# recording-rules.yml
groups:
  - name: robustmq.performance
    rules:
      - record: robustmq:request_latency_95th
        expr: histogram_quantile(0.95, rate(request_total_ms_bucket[5m]))
      
      - record: robustmq:error_rate_total
        expr: rate(mqtt_packets_received_error[5m])
```

### Grafana Optimization

#### Query Optimization

- Use recording rules to reduce complex queries
- Set appropriate time ranges and refresh intervals
- Avoid aggregation queries with high cardinality labels

#### Cache Configuration

```ini
# grafana.ini
[caching]
enabled = true
ttl = 300s
```

## Troubleshooting

### Common Issues

#### 1. No Data Displayed

**Check Steps:**
```bash
# 1. Verify RobustMQ Broker metrics export
curl `http://localhost:9091/metrics`

# 2. Check Prometheus target status
curl `http://localhost:9090/api/v1/targets`

# 3. Verify Grafana data source connection
curl -H "Authorization: Bearer YOUR_API_KEY" \
     `http://localhost:3000/api/datasources/proxy/1/api/v1/query?query=up`
```

#### 2. Missing Metrics

**Possible Causes:**
- RobustMQ version doesn't support certain metrics
- Features not enabled (e.g., retained messages, authentication)
- Metric names don't match

**Solutions:**
```bash
# Check available metrics
curl `http://localhost:9091/metrics` | grep mqtt_

# Verify feature configuration
grep -r "enable.*true" config/
```

#### 3. Performance Issues

**Optimization Suggestions:**
- Reduce refresh intervals for high-frequency queries
- Use recording rules to pre-compute complex metrics
- Adjust time ranges to avoid large data volume queries
- Configure Prometheus data retention policies

### Log Analysis

#### Prometheus Logs

```bash
# View Prometheus logs
docker logs robustmq-prometheus

# Check scrape errors
grep "scrape_pool" /var/log/prometheus/prometheus.log
```

#### Grafana Logs

```bash
# View Grafana logs
docker logs robustmq-grafana

# Check data source connections
grep "datasource" /var/log/grafana/grafana.log
```

## Security Configuration

### Authentication Setup

#### Grafana Authentication

```ini
# grafana.ini
[auth]
disable_login_form = false

[auth.basic]
enabled = true

[auth.ldap]
enabled = true
config_file = /etc/grafana/ldap.toml
```

#### Prometheus Authentication

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'robustmq-broker-secure'
    basic_auth:
      username: 'prometheus'
      password: 'secure_password'
    static_configs:
      - targets: ['localhost:9091']
```

### HTTPS Configuration

#### Grafana HTTPS

```ini
# grafana.ini
[server]
protocol = https
cert_file = /etc/grafana/grafana.crt
cert_key = /etc/grafana/grafana.key
```

#### Prometheus HTTPS

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'robustmq-broker-https'
    scheme: https
    tls_config:
      ca_file: /etc/prometheus/ca.crt
      cert_file: /etc/prometheus/client.crt
      key_file: /etc/prometheus/client.key
```

## Maintenance and Upgrades

### Backup Strategy

#### Grafana Backup

```bash
# Backup dashboards
curl -H "Authorization: Bearer YOUR_API_KEY" \
     `http://localhost:3000/api/search?type=dash-db` | \
     jq -r '.[].uri' | \
     xargs -I {} curl -H "Authorization: Bearer YOUR_API_KEY" \
     `http://localhost:3000/api/dashboards/{}` > backup.json

# Backup data sources
curl -H "Authorization: Bearer YOUR_API_KEY" \
     `http://localhost:3000/api/datasources` > datasources-backup.json
```

#### Prometheus Backup

```bash
# Backup configuration
cp prometheus.yml prometheus.yml.backup

# Backup data (offline backup)
tar -czf prometheus-data-backup.tar.gz data/
```

### Version Upgrades

#### Upgrade Grafana

```bash
# Docker method
docker-compose -f docker-compose.monitoring.yml pull grafana
docker-compose -f docker-compose.monitoring.yml up -d grafana

# Package manager method
sudo apt-get update && sudo apt-get upgrade grafana
```

#### Upgrade Prometheus

```bash
# Download new version
wget https://github.com/prometheus/prometheus/releases/download/v2.41.0/prometheus-2.41.0.linux-amd64.tar.gz

# Stop service, replace binary, restart service
systemctl stop prometheus
cp prometheus-new /usr/local/bin/prometheus
systemctl start prometheus
```

Through this guide, you can completely configure RobustMQ's Grafana monitoring system to achieve comprehensive observability and proactive monitoring capabilities.
