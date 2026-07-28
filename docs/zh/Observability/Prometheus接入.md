# Prometheus 接入

RobustMQ 内置 Prometheus 指标导出功能，指标始终通过 Admin HTTP API 的 `GET /metrics` 暴露，复用 `http_port`（默认为 `58080`），没有独立的开关或端口，无需额外配置。

## 验证指标导出

将下面的 `58080` 替换为你 `config/server.toml` 中配置的 `http_port`：

```bash
# 检查指标端点
curl http://localhost:58080/metrics

# 验证指标数据
curl http://localhost:58080/metrics | grep mqtt_
```

## 配置 Prometheus

在 Prometheus 配置文件中添加 RobustMQ 作为抓取目标：

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'robustmq'
    static_configs:
      - targets: ['localhost:58080']
    scrape_interval: 15s
    metrics_path: /metrics
```

## 集群配置

对于多节点部署（各节点 `http_port` 需替换为实际值）：

```yaml
scrape_configs:
  - job_name: 'robustmq-cluster'
    static_configs:
      - targets:
        - 'robustmq-node1:58080'
        - 'robustmq-node2:58080'
        - 'robustmq-node3:58080'
```

## 可用指标

RobustMQ 导出以下类型的指标：

- **MQTT 协议指标**: 数据包收发、连接管理、认证统计
- **性能指标**: 请求延迟、处理耗时、队列深度
- **业务指标**: 会话数量、主题统计、消息处理
- **系统指标**: 网络连接、线程池、错误统计

## 常用查询

```text
# 当前连接数
mqtt_connections_count

# MQTT 数据包接收速率
rate(mqtt_packets_received[5m])

# 请求处理延迟 P95
histogram_quantile(0.95, rate(request_total_ms_bucket[5m]))

# 认证失败率
rate(mqtt_auth_failed[5m])
```

## 故障排查

### 指标无法访问
```bash
# 检查端口监听（http_port 默认为 58080）
netstat -tlnp | grep 58080

# 检查配置中实际使用的 http_port
grep "http_port" config/server.toml
```

### Prometheus 无法抓取
```bash
# 检查网络连通性
telnet robustmq-host 58080

# 查看 Prometheus 目标状态
curl http://prometheus:9090/api/v1/targets
```

通过以上配置，RobustMQ 的监控指标将自动被 Prometheus 收集，可用于告警、可视化和性能分析。
