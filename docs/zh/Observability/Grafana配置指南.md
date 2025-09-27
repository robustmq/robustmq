# Grafana 配置指南

本文档详细介绍如何为 RobustMQ 配置 Grafana 监控系统，包括 Prometheus 数据源配置、仪表板导入和告警设置。

## 环境准备

### 系统要求

- **Grafana**: 8.0 或更高版本
- **Prometheus**: 2.30 或更高版本
- **RobustMQ**: 启用指标导出功能
- **Docker**: 20.10 或更高版本（可选）

### 端口规划

| 服务 | 默认端口 | 用途 |
|------|----------|------|
| RobustMQ Broker | 9091 | 指标导出 |
| Prometheus | 9090 | 指标收集 |
| Grafana | 3000 | 可视化界面 |
| Alertmanager | 9093 | 告警管理 |

## 快速部署

### 使用 Docker Compose（推荐）

RobustMQ 提供了完整的监控栈 Docker Compose 配置：

```bash
# 1. 进入 grafana 目录
cd grafana/

# 2. 启动监控栈
docker-compose -f docker-compose.monitoring.yml up -d

# 3. 验证服务状态
docker-compose -f docker-compose.monitoring.yml ps
```

**服务访问地址：**
- Grafana: `localhost:3000` (admin/admin)
- Prometheus: `localhost:9090`
- Alertmanager: `localhost:9093`

### 手动安装

#### 1. 安装 Prometheus

```bash
# 下载 Prometheus
wget https://github.com/prometheus/prometheus/releases/download/v2.40.0/prometheus-2.40.0.linux-amd64.tar.gz
tar xvfz prometheus-2.40.0.linux-amd64.tar.gz
cd prometheus-2.40.0.linux-amd64

# 复制配置文件
cp /path/to/robustmq/grafana/prometheus-config-example.yml ./prometheus.yml

# 启动 Prometheus
./prometheus --config.file=prometheus.yml --storage.tsdb.path=./data
```

#### 2. 安装 Grafana

```bash
# Ubuntu/Debian
sudo apt-get install -y software-properties-common
sudo add-apt-repository "deb https://packages.grafana.com/oss/deb stable main"
wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
sudo apt-get update
sudo apt-get install grafana

# CentOS/RHEL
sudo yum install -y https://dl.grafana.com/oss/release/grafana-9.0.0-1.x86_64.rpm

# 启动 Grafana
sudo systemctl start grafana-server
sudo systemctl enable grafana-server
```

## RobustMQ 配置

### 启用指标导出

确保 RobustMQ 配置文件中启用了指标导出功能：

```toml
# config/server.toml
[prometheus]
# 启用 Prometheus 指标导出
enable = true
# 指标导出端口
port = 9091
```

### 验证指标导出

```bash
# 检查 RobustMQ Broker 指标
curl `http://localhost:9091/metrics`
```

## Prometheus 配置

### 基础配置

使用 RobustMQ 提供的配置模板：

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

# 告警规则
rule_files:
  - "robustmq-alerts.yml"

# 抓取配置
scrape_configs:
  # RobustMQ Broker
  - job_name: 'robustmq-broker'
    static_configs:
      - targets: ['localhost:9091']
    scrape_interval: 15s
    metrics_path: /metrics
```

### 集群配置

对于集群部署，配置多个目标：

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

### 服务发现配置

使用 Kubernetes 服务发现：

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

## Grafana 配置

### 数据源配置

#### 1. 添加 Prometheus 数据源

通过 Grafana Web 界面：

1. 登录 Grafana (`localhost:3000`)
2. 进入 **Configuration** → **Data Sources**
3. 点击 **Add data source**
4. 选择 **Prometheus**
5. 配置连接信息：
   - **URL**: `http://localhost:9090`
   - **Access**: Server (default)
   - **Scrape interval**: 15s

#### 2. 通过配置文件

创建数据源配置文件：

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

### 仪表板导入

#### 方法一：Web 界面导入

1. 进入 **Dashboards** → **Import**
2. 上传 `robustmq-mqtt-broker-dashboard.json` 文件
3. 选择 Prometheus 数据源
4. 点击 **Import**

#### 方法二：API 导入

```bash
# 使用 Grafana API 导入仪表板
curl -X POST \
  `http://localhost:3000/api/dashboards/db` \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d @robustmq-mqtt-broker-dashboard.json
```

#### 方法三：自动配置

创建仪表板配置文件：

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

### 仪表板功能说明

#### 📊 服务器概览面板

- **当前连接数**: 实时显示活跃连接数量
- **最大连接数**: 显示连接峰值，用于容量规划
- **活跃线程数**: 按网络类型和线程类型分组显示

#### 🚀 性能指标面板

- **请求总延迟**: 显示 P95/P50 分位数延迟
- **处理器延迟**: 显示请求处理器的执行时间
- **网络队列大小**: 显示请求和响应队列深度

#### 📦 MQTT 数据包面板

- **接收数据包速率**: 按类型和网络分组的入站数据包
- **发送数据包速率**: 按类型、网络和 QoS 分组的出站数据包
- **网络流量**: 字节级别的收发速率

#### 🔌 客户端连接面板

- **客户端连接速率**: 新连接建立速率
- **错误率**: 数据包错误、认证错误、连接错误统计

#### 📝 消息处理面板

- **保留消息**: 按 QoS 级别统计保留消息处理
- **丢弃消息**: 因无订阅者或丢弃策略导致的消息丢失

## 告警配置

### Alertmanager 配置

创建 Alertmanager 配置文件：

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

### 告警规则说明

RobustMQ 提供了预配置的告警规则，涵盖以下场景：

#### 🚨 高优先级告警

- **服务下线**: `RobustMQBrokerDown`
- **认证失败**: `RobustMQAuthenticationFailures`
- **严重延迟**: `RobustMQCriticalRequestLatency`
- **队列积压**: `RobustMQCriticalQueueDepth`

#### ⚠️ 中等优先级告警

- **高连接数**: `RobustMQHighConnectionCount`
- **连接错误**: `RobustMQConnectionErrors`
- **数据包错误**: `RobustMQPacketErrors`
- **消息丢弃**: `RobustMQHighMessageDrops`

#### ℹ️ 信息级告警

- **低吞吐量**: `RobustMQLowThroughput`
- **容量规划**: `RobustMQCapacityPlanningNeeded`

### 自定义告警规则

添加自定义告警规则：

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

## 性能优化

### Prometheus 优化

#### 存储优化

```yaml
# prometheus.yml
global:
  # 减少抓取间隔以降低存储压力
  scrape_interval: 30s
  
# 配置数据保留策略
command_args:
  - '--storage.tsdb.retention.time=30d'
  - '--storage.tsdb.retention.size=50GB'
```

#### 记录规则

使用预计算规则提升查询性能：

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

### Grafana 优化

#### 查询优化

- 使用记录规则减少复杂查询
- 设置合适的时间范围和刷新间隔
- 避免高基数标签的聚合查询

#### 缓存配置

```ini
# grafana.ini
[caching]
enabled = true
ttl = 300s
```

## 故障排查

### 常见问题

#### 1. 无数据显示

**检查步骤：**
```bash
# 1. 验证 RobustMQ Broker 指标导出
curl `http://localhost:9091/metrics`

# 2. 检查 Prometheus 目标状态
curl `http://localhost:9090/api/v1/targets`

# 3. 验证 Grafana 数据源连接
curl -H "Authorization: Bearer YOUR_API_KEY" \
     `http://localhost:3000/api/datasources/proxy/1/api/v1/query?query=up`
```

#### 2. 指标缺失

**可能原因：**
- RobustMQ 版本不支持某些指标
- 功能未启用（如保留消息、认证等）
- 指标名称不匹配

**解决方案：**
```bash
# 检查可用指标
curl `http://localhost:9090/metrics` | grep mqtt_

# 验证功能配置
grep -r "enable.*true" config/
```

#### 3. 性能问题

**优化建议：**
- 减少高频查询的刷新间隔
- 使用记录规则预计算复杂指标
- 调整时间范围避免大数据量查询
- 配置 Prometheus 数据保留策略

### 日志分析

#### Prometheus 日志

```bash
# 查看 Prometheus 日志
docker logs robustmq-prometheus

# 检查抓取错误
grep "scrape_pool" /var/log/prometheus/prometheus.log
```

#### Grafana 日志

```bash
# 查看 Grafana 日志
docker logs robustmq-grafana

# 检查数据源连接
grep "datasource" /var/log/grafana/grafana.log
```

## 安全配置

### 认证设置

#### Grafana 认证

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

#### Prometheus 认证

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

### HTTPS 配置

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

## 维护和升级

### 备份策略

#### Grafana 备份

```bash
# 备份仪表板
curl -H "Authorization: Bearer YOUR_API_KEY" \
     `http://localhost:3000/api/search?type=dash-db` | \
     jq -r '.[].uri' | \
     xargs -I {} curl -H "Authorization: Bearer YOUR_API_KEY" \
     `http://localhost:3000/api/dashboards/{}` > backup.json

# 备份数据源
curl -H "Authorization: Bearer YOUR_API_KEY" \
     `http://localhost:3000/api/datasources` > datasources-backup.json
```

#### Prometheus 备份

```bash
# 备份配置
cp prometheus.yml prometheus.yml.backup

# 备份数据（停机备份）
tar -czf prometheus-data-backup.tar.gz data/
```

### 版本升级

#### 升级 Grafana

```bash
# Docker 方式
docker-compose -f docker-compose.monitoring.yml pull grafana
docker-compose -f docker-compose.monitoring.yml up -d grafana

# 包管理器方式
sudo apt-get update && sudo apt-get upgrade grafana
```

#### 升级 Prometheus

```bash
# 下载新版本
wget https://github.com/prometheus/prometheus/releases/download/v2.41.0/prometheus-2.41.0.linux-amd64.tar.gz

# 停止服务，替换二进制文件，重启服务
systemctl stop prometheus
cp prometheus-new /usr/local/bin/prometheus
systemctl start prometheus
```

通过本指南，您可以完整地配置 RobustMQ 的 Grafana 监控系统，实现全面的可观测性和主动监控能力。
