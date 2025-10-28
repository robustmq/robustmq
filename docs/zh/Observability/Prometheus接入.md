# Prometheus 接入

RobustMQ 内置 Prometheus 指标导出功能，只需简单配置即可接入 Prometheus 监控系统。

## 配置 RobustMQ

在 RobustMQ 配置文件中启用 Prometheus 指标导出：

```toml
# config/server.toml
[prometheus]
enable = true
port = 9091
```

重启 RobustMQ 服务使配置生效。

## 验证指标导出

```bash
# 检查指标端点
curl `http://localhost:9091/metrics`

# 验证指标数据
curl `http://localhost:9091/metrics` | grep mqtt_
```

## 配置 Prometheus

在 Prometheus 配置文件中添加 RobustMQ 作为抓取目标：

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'robustmq'
    static_configs:
      - targets: ['localhost:9091']
    scrape_interval: 15s
    metrics_path: /metrics
```

## 集群配置

对于多节点部署：

```yaml
scrape_configs:
  - job_name: 'robustmq-cluster'
    static_configs:
      - targets:
        - 'robustmq-node1:9091'
        - 'robustmq-node2:9091'
        - 'robustmq-node3:9091'
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
# 检查端口监听
netstat -tlnp | grep 9091

# 检查配置
grep -A 3 "\[prometheus\]" config/server.toml
```

### Prometheus 无法抓取
```bash
# 检查网络连通性
telnet robustmq-host 9091

# 查看 Prometheus 目标状态
curl http://prometheus:9090/api/v1/targets
```

通过以上配置，RobustMQ 的监控指标将自动被 Prometheus 收集，可用于告警、可视化和性能分析。
