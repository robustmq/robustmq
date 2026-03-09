# Health Check

RobustMQ exposes standard health check endpoints via the HTTP Admin Server, compatible with Kubernetes Liveness and Readiness Probes out of the box.

## Endpoints

All endpoints are served by the Admin Server on the configured `http_port` (default `9981`).

| Endpoint | K8s Probe | Description |
|----------|-----------|-------------|
| `GET /health/ready` | Liveness / Readiness Probe | Checks whether all configured ports are ready. Returns `200 OK` when ready, `503 Service Unavailable` when not |
| `GET /health/node` | Deep node check | Performs a deep status check on the current node. Not recommended for K8s probes |
| `GET /health/cluster` | Deep cluster check | Performs a deep status check on the cluster. Not recommended for K8s probes |

## Kubernetes Configuration Example

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

## Response Format

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

When not ready, the Readiness Probe returns:

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
