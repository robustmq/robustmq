# Grafana 配置指南

本文档介绍如何为 RobustMQ 配置 Grafana 监控系统，包括快速部署、数据源配置和仪表板导入。

## 环境要求

- Grafana 8.0+、Prometheus 2.30+、Docker 20.10+（可选）
- 默认端口：RobustMQ 指标端口(9091)、Prometheus(9090)、Grafana(3000)、Alertmanager(9093)

## 快速部署

### 使用 Docker Compose（推荐）

```bash
cd grafana/
docker-compose -f docker-compose.monitoring.yml up -d
```

该命令会启动以下服务：

| 服务 | 地址 | 说明 |
|------|------|------|
| Grafana | `http://localhost:3000` | 默认账号 admin/admin |
| Prometheus | `http://localhost:9090` | 指标采集与查询 |
| Alertmanager | `http://localhost:9093` | 告警管理 |
| Node Exporter | `http://localhost:9100` | 系统指标（可选） |

## RobustMQ 配置

在 `config/server.toml` 中启用 Prometheus 指标导出：

```toml
[prometheus]
enable = true
port = 9091
```

验证指标是否正常暴露：

```bash
curl http://localhost:9091/metrics
```

## Prometheus 配置

项目提供了示例配置 `grafana/prometheus-config-example.yml`，包含以下采集目标：

### 单机配置

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

### 集群配置

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

### 多服务配置

如果同时运行 Meta Service 和 Journal Server，可以分别采集：

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

## Grafana 配置

### 添加 Prometheus 数据源

**Web 界面方式：**
1. 登录 Grafana (`http://localhost:3000`)
2. **Configuration** → **Data Sources** → **Add data source**
3. 选择 **Prometheus**，URL 填写 `http://localhost:9090`（Docker 环境下填 `http://prometheus:9090`）

**配置文件方式（Provisioning）：**

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

RobustMQ 提供了预置仪表板文件 `grafana/robustmq-broker.json`。

**Web 界面导入：**
1. **Dashboards** → **Import**
2. 上传 `grafana/robustmq-broker.json`
3. 在 `DS_PROMETHEUS` 下拉框中选择你的 Prometheus 数据源
4. 点击 **Import**

**API 导入：**

```bash
curl -X POST http://localhost:3000/api/dashboards/db \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d @grafana/robustmq-broker.json
```

## 仪表板面板说明

`robustmq-broker.json` 包含以下区域：

### Resource（资源概览）

顶部 Stat 面板展示系统当前状态：

| 面板 | 指标 | 说明 |
|------|------|------|
| Connections | `mqtt_connections_count` | 当前 MQTT 连接数 |
| Sessions | `mqtt_sessions_count` | 当前会话数 |
| Topics | `mqtt_topics_count` | 当前主题数 |
| Subscribers | `mqtt_subscribers_count` | 当前订阅者总数 |
| Shared Subscriptions | `mqtt_subscriptions_shared_count` | 当前共享订阅数 |
| Retained Messages | `mqtt_retained_count` | 当前保留消息数 |

下方 Timeseries 面板展示资源变化趋势，包括连接速率、会话创建/删除速率、主题消息读写速率、订阅成功/失败速率、共享订阅分类统计、保留消息收发速率。

### 🌐 Network（网络层）

| 面板 | 说明 |
|------|------|
| Handler Total Latency | 请求端到端处理耗时分位数（P50/P95/P99） |
| Handler Queue Wait Latency | 请求在队列中等待的耗时分位数 |
| Handler Apply Latency | command.apply() 执行耗时分位数 |
| Response Write Latency | 响应写回客户端的耗时分位数 |

### 📈 MQTT Protocol（MQTT 协议）

| 面板 | 说明 |
|------|------|
| MQTT Received Packet Rate (QPS) | 各类型接收包的速率 |
| MQTT Sent Packet Rate (QPS) | 各类型发送包的速率 |
| MQTT Packet Process Latency Percentiles | 协议包处理耗时分位数 |
| MQTT Packet Process P99 Latency by Type | 按包类型区分的 P99 处理延迟 |
| MQTT Packet Process QPS by Type | 按包类型区分的处理速率 |
| MQTT Packet Process Avg Latency by Type | 按包类型区分的平均处理延迟 |

### 🔗 gRPC Server

| 面板 | 说明 |
|------|------|
| gRPC Requests Rate | gRPC 请求速率（req/s） |
| gRPC QPS by Method | 按方法区分的 gRPC 请求速率 |
| gRPC P99 Latency by Method | 按方法区分的 P99 延迟 |

### 📡 gRPC Client

| 面板 | 说明 |
|------|------|
| gRPC Client Call P99 Latency by Method | 每个 gRPC 客户端调用接口的 P99 延迟 |
| gRPC Client Call Latency Percentiles | 客户端调用整体延迟分位数（P50/P95/P99/P999） |
| gRPC Client Call QPS by Method | 每个接口的客户端调用 QPS |

> 此区域展示 Broker 作为 gRPC 客户端向 Meta Service 等发起调用的耗时统计，有助于定位连接建立等流程中的性能瓶颈。

### 🌍 HTTP Admin

| 面板 | 说明 |
|------|------|
| HTTP Requests Rate | HTTP Admin 请求速率（req/s） |
| HTTP QPS by Endpoint | 按端点区分的请求速率 |
| HTTP Admin P99 Latency by Endpoint | 按端点区分的 P99 延迟 |

### 📦 Raft Machine

| 面板 | 说明 |
|------|------|
| Raft Write Rate / Success Rate / Failure Rate | Raft 写入请求/成功/失败速率 |
| Raft RPC Rate | Raft RPC 请求速率 |
| Raft Write QPS (by Machine) | 按状态机类型区分的写入 QPS |
| Raft Write Latency (by Machine) | 按状态机类型区分的写入延迟 |
| Raft RPC QPS (by Machine / RPC Type) | 按状态机和 RPC 类型区分的 QPS |
| Raft RPC Latency (by Machine / RPC Type) | 按状态机和 RPC 类型区分的延迟 |

> Raft RPC 指标仅在多节点集群部署时才会有数据。

### 📖 RocksDB

| 面板 | 说明 |
|------|------|
| RocksDB QPS by Operation | 按操作类型（save/get/delete/list）区分的 QPS |
| RocksDB QPS by Source | 按数据源区分的 QPS |
| RocksDB Write Latency | 写入操作延迟分位数 |
| RocksDB Read (Get) Latency | 读取操作延迟分位数 |

### ⏱ Delay Message（延迟消息队列）

| 面板 | 说明 |
|------|------|
| Delay Message Enqueue / Deliver / Failure Rate | 入队/投递/失败速率 |
| Enqueue Latency Percentiles | 入队耗时分位数 |
| Deliver Latency Percentiles | 投递耗时分位数 |

> 延迟消息指标仅在实际使用延迟发布功能时才有数据。

## 告警配置

### 预置告警规则

项目提供了 `grafana/robustmq-alerts.yml`，包含以下告警规则：

| 告警 | 级别 | 条件 | 说明 |
|------|------|------|------|
| RobustMQBrokerDown | Critical | `up == 0` | Broker 实例不可达 |
| RobustMQHighRequestLatency | Warning | P95 延迟 > 100ms 持续 10m | 请求处理延迟偏高 |
| RobustMQCriticalRequestLatency | Critical | P95 延迟 > 500ms 持续 5m | 请求处理延迟严重 |
| RobustMQAuthenticationFailures | Critical | 认证失败 > 10/s 持续 2m | 认证失败频繁 |
| RobustMQConnectionErrors | Warning | 连接错误 > 5/s 持续 5m | 连接错误频繁 |
| RobustMQHighQueueDepth | Warning | 队列积压 > 1000 持续 5m | 队列积压 |
| RobustMQCriticalQueueDepth | Critical | 队列积压 > 5000 持续 2m | 队列严重积压 |
| RobustMQHighMessageDrops | Warning | 消息丢弃 > 100/s 持续 5m | 无订阅者消息丢弃频繁 |
| RobustMQHighThreadUtilization | Warning | 活跃线程 > 50 持续 10m | 线程数过高 |

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

### 自定义告警

在 `grafana/robustmq-alerts.yml` 中添加自定义规则：

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
          summary: "gRPC 错误率超过 5%"
          description: "当前错误率: {{ $value | humanizePercentage }}"

      - alert: RocksDBSlowOperations
        expr: histogram_quantile(0.95, rate(rocksdb_operation_ms_bucket[5m])) > 100
        for: 3m
        labels:
          severity: warning
        annotations:
          summary: "RocksDB P95 操作耗时超过 100ms"
```

## 性能优化

### Prometheus 优化

**存储配置：**

```bash
# 启动参数
--storage.tsdb.retention.time=30d
--storage.tsdb.retention.size=50GB
```

**Recording 规则（预计算）：**

对高频查询创建 Recording 规则以提升查询性能：

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

### Grafana 优化

- 使用 Recording 规则减少复杂的实时聚合查询
- 合理设置面板刷新间隔（建议 15s - 1m）
- 避免高基数标签（如 `client_id`、`connection_id`）的大范围聚合查询
- 对历史数据查询使用较大的 `rate()` 窗口（如 `[5m]` 而非 `[1m]`）

## 相关文件

| 文件 | 说明 |
|------|------|
| `grafana/robustmq-broker.json` | Grafana 仪表板定义 |
| `grafana/prometheus-config-example.yml` | Prometheus 采集配置示例 |
| `grafana/robustmq-alerts.yml` | 告警规则定义 |
| `grafana/docker-compose.monitoring.yml` | Docker Compose 监控栈 |
| `config/server.toml` | RobustMQ 服务配置（含 Prometheus 端口） |
| `docs/zh/Observability/基础设施指标.md` | 完整指标参考文档 |
