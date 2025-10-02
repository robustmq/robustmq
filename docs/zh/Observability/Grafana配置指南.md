# Grafana 配置指南

本文档介绍如何为 RobustMQ 配置 Grafana 监控系统，包括快速部署、数据源配置和仪表板导入。

## 环境要求

- Grafana 8.0+, Prometheus 2.30+, Docker 20.10+（可选）
- 默认端口：RobustMQ(9091)、Prometheus(9090)、Grafana(3000)、Alertmanager(9093)

## 快速部署

### 使用 Docker Compose（推荐）

```bash
cd grafana/
docker-compose -f docker-compose.monitoring.yml up -d
```

**访问地址：**
- Grafana: `localhost:3000` (admin/admin)
- Prometheus: `localhost:9090`
- Alertmanager: `localhost:9093`

## RobustMQ 配置

在 `config/server.toml` 中启用指标导出：

```toml
[prometheus]
enable = true
port = 9091
```

验证指标：`curl http://localhost:9091/metrics`

## Prometheus 配置

### 单机配置

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

rule_files:
  - "robustmq-alerts.yml"

scrape_configs:
  - job_name: 'robustmq-broker'
    static_configs:
      - targets: ['localhost:9091']
```

### 集群配置

```yaml
scrape_configs:
  - job_name: 'robustmq-cluster'
    static_configs:
      - targets:
        - 'node1:9091'
        - 'node2:9091'
        - 'node3:9091'
```

## Grafana 配置

### 添加 Prometheus 数据源

**Web 界面方式：**
1. 登录 Grafana (`localhost:3000`)
2. **Configuration** → **Data Sources** → **Add data source**
3. 选择 **Prometheus**，URL: `http://localhost:9090`

**配置文件方式：**

```yaml
# /etc/grafana/provisioning/datasources/prometheus.yml
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    url: http://prometheus:9090
    isDefault: true
```

### 导入仪表板

**Web 界面：**
1. **Dashboards** → **Import**
2. 上传 `robustmq-mqtt-broker-dashboard.json`
3. 选择 Prometheus 数据源并导入

**API 导入：**

```bash
curl -X POST http://localhost:3000/api/dashboards/db \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d @robustmq-mqtt-broker-dashboard.json
```

### 仪表板功能说明

**📊 服务器概览**
- 当前/最大连接数、活跃线程数

**🚀 性能指标**
- 请求延迟 P95/P50、网络队列大小

**📦 MQTT 数据包**
- 收发数据包速率、网络流量统计

**🔌 客户端连接**
- 连接速率、错误率统计

**📝 消息处理**
- 保留消息、丢弃消息统计

## 告警配置

### Alertmanager 配置

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

### 预定义告警规则

RobustMQ 提供的告警规则包括：

- 🚨 **高优先级**：服务下线、认证失败、严重延迟、队列积压
- ⚠️ **中优先级**：高连接数、连接错误、数据包错误、消息丢弃
- ℹ️ **信息级**：低吞吐量、容量规划提醒

### 自定义告警

```yaml
# custom-alerts.yml
groups:
  - name: robustmq.custom
    rules:
      - alert: HighCPU
        expr: cpu_usage_percent > 80
        for: 5m
        annotations:
          summary: "CPU usage high: {{ $value }}%"
```

## 性能优化

### Prometheus 优化

**存储配置：**
```bash
# 启动参数
--storage.tsdb.retention.time=30d
--storage.tsdb.retention.size=50GB
```

**记录规则：**
```yaml
# recording-rules.yml
groups:
  - name: robustmq.performance
    rules:
      - record: robustmq:request_latency_95th
        expr: histogram_quantile(0.95, rate(request_total_ms_bucket[5m]))
```

### Grafana 优化

- 使用记录规则减少复杂查询
- 合理设置刷新间隔（建议 30s-1m）
- 避免高基数标签聚合

---

通过本指南，您可以快速配置 RobustMQ 的 Grafana 监控系统，实现全面的可观测性。更多详细信息请参考 [Prometheus 文档](https://prometheus.io/docs/) 和 [Grafana 文档](https://grafana.com/docs/)。
