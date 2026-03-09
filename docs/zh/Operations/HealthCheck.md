# 健康检查

RobustMQ 通过 HTTP Admin 接口提供标准健康检查端点，可直接对接 Kubernetes 的 Liveness Probe 和 Readiness Probe。

## 接口说明

所有端点均由 Admin Server 提供，默认端口为 `http_port`（默认 `9981`）。

| 端点 | 适用场景 | 说明 |
|------|----------|------|
| `GET /health/ready` | Liveness / Readiness Probe | 检查所有配置的端口是否就绪。就绪返回 `200 OK`，未就绪返回 `503 Service Unavailable` |
| `GET /health/node` | 节点深度检查 | 对当前节点进行深度状态检查，不建议用于 K8s 探针 |
| `GET /health/cluster` | 集群深度检查 | 对集群进行深度状态检查，不建议用于 K8s 探针 |

## Kubernetes 配置示例

```yaml
livenessProbe:
  httpGet:
    path: /health/ready
    port: 9981
  initialDelaySeconds: 15
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /health/ready
    port: 9981
  initialDelaySeconds: 10
  periodSeconds: 5
  failureThreshold: 3
```

## 响应格式

```json
{
  "code": 0,
  "data": {
    "status": "ok",
    "check_type": "ready",
    "message": "all configured ports are ready"
  }
}
```

Readiness Probe 未就绪时返回：

```json
{
  "code": 0,
  "data": {
    "status": "not_ready",
    "check_type": "ready",
    "message": "one or more configured ports are not ready"
  }
}
```
