# Infrastructure Metrics

RobustMQ provides comprehensive infrastructure monitoring metrics to help operations teams monitor system health, performance, and resource usage. All metrics are based on Prometheus format and can be accessed through the `/metrics` endpoint.

## Network Layer Metrics

### Request Processing Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `request_total_ms` | Histogram | `request_type`, `network` | Total request processing time (milliseconds) |
| `request_queue_ms` | Histogram | `request_type`, `network` | Request queue waiting time (milliseconds) |
| `request_handler_ms` | Histogram | `request_type`, `network` | Request handler execution time (milliseconds) |
| `request_response_ms` | Histogram | `request_type`, `network` | Response processing time (milliseconds) |
| `request_response_queue_ms` | Histogram | `request_type`, `network` | Response queue waiting time (milliseconds) |

### Connection and Queue Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `broker_connections_max` | Gauge | - | Maximum connection limit for broker |
| `broker_network_queue_num` | Gauge | `queue_type` | Number of messages in network queue |
| `broker_active_thread_num` | Gauge | `thread_type` | Number of active threads |

**Label Descriptions:**
- `request_type`: Request type (e.g., mqtt, grpc, http)
- `network`: Network type (tcp, websocket, quic)
- `queue_type`: Queue type (accept, handler, response)
- `thread_type`: Thread type (accept, handler, response)

## gRPC Service Metrics

### Request Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `grpc_requests_total` | Counter | `method`, `service` | Total number of gRPC requests |
| `grpc_request_errors_total` | Counter | `method`, `service`, `error_code` | Total number of gRPC request errors |

### Performance Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `grpc_request_duration_milliseconds` | Histogram | `method`, `service` | gRPC request duration (milliseconds) |
| `grpc_request_size_bytes` | Histogram | `method`, `service` | gRPC request size (bytes) |
| `grpc_response_size_bytes` | Histogram | `method`, `service` | gRPC response size (bytes) |

**Label Descriptions:**
- `method`: gRPC method name
- `service`: gRPC service name
- `error_code`: Error code

## HTTP Service Metrics

### Request Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `http_requests_total` | Counter | `method`, `path`, `status_code` | Total number of HTTP requests |
| `http_request_errors_total` | Counter | `method`, `path`, `error_type` | Total number of HTTP request errors |

### Performance Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `http_request_duration_milliseconds` | Histogram | `method`, `path` | HTTP request duration (milliseconds) |
| `http_request_size_bytes` | Histogram | `method`, `path` | HTTP request size (bytes) |
| `http_response_size_bytes` | Histogram | `method`, `path` | HTTP response size (bytes) |

**Label Descriptions:**
- `method`: HTTP method (GET, POST, PUT, DELETE)
- `path`: Request path
- `status_code`: HTTP status code
- `error_type`: Error type

## Storage Layer Metrics (RocksDB)

### Operation Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `rocksdb_operation_count` | Counter | `source`, `operation` | Number of RocksDB operations |
| `rocksdb_operation_ms` | Histogram | `source`, `operation` | RocksDB operation duration (milliseconds) |

**Label Descriptions:**
- `source`: Data source (e.g., metadata, session, message)
- `operation`: Operation type (save, get, delete, list)

### Common Operation Types

- **save**: Data write operations
- **get**: Data read operations
- **delete**: Data deletion operations
- **list**: Data list query operations

## Broker Core Metrics

### System Status

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `broker_status` | Gauge | `node_id` | Broker node status (1=running, 0=stopped) |
| `broker_uptime_seconds` | Counter | `node_id` | Broker uptime (seconds) |

### Resource Usage

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `broker_memory_usage_bytes` | Gauge | `node_id`, `type` | Memory usage (bytes) |
| `broker_cpu_usage_percent` | Gauge | `node_id` | CPU usage (percentage) |
| `broker_disk_usage_bytes` | Gauge | `node_id`, `path` | Disk usage (bytes) |

**Label Descriptions:**
- `node_id`: Node identifier
- `type`: Memory type (heap, non_heap, total)
- `path`: Disk path

## Meta Service Metrics

### Cluster Status

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `meta_cluster_nodes_total` | Gauge | `cluster_id` | Total number of cluster nodes |
| `meta_cluster_nodes_active` | Gauge | `cluster_id` | Number of active cluster nodes |
| `meta_leader_elections_total` | Counter | `cluster_id` | Number of leader elections |

### Data Synchronization

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `meta_sync_operations_total` | Counter | `operation_type` | Number of metadata sync operations |
| `meta_sync_latency_ms` | Histogram | `operation_type` | Metadata sync latency (milliseconds) |

## Usage Examples

### Prometheus Configuration

```yaml
scrape_configs:
  - job_name: 'robustmq'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### Grafana Query Examples

```promql
# Average request processing time
rate(request_total_ms_sum[5m]) / rate(request_total_ms_count[5m])

# gRPC error rate
rate(grpc_request_errors_total[5m]) / rate(grpc_requests_total[5m]) * 100

# RocksDB operation QPS
rate(rocksdb_operation_count[5m])

# Network queue backlog
broker_network_queue_num
```

## Alert Rules Examples

```yaml
groups:
  - name: robustmq_infrastructure
    rules:
      - alert: HighRequestLatency
        expr: histogram_quantile(0.95, rate(request_total_ms_bucket[5m])) > 1000
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High request latency"
          description: "95% of requests have latency over 1 second"

      - alert: HighErrorRate
        expr: rate(grpc_request_errors_total[5m]) / rate(grpc_requests_total[5m]) > 0.05
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "High gRPC error rate"
          description: "gRPC error rate exceeds 5%"

      - alert: RocksDBSlowOperations
        expr: histogram_quantile(0.95, rate(rocksdb_operation_ms_bucket[5m])) > 100
        for: 3m
        labels:
          severity: warning
        annotations:
          summary: "Slow RocksDB operations"
          description: "95% of RocksDB operations take over 100ms"
```

Through these infrastructure metrics, operations teams can comprehensively understand the operational status of RobustMQ systems, identify and resolve performance issues in a timely manner, and ensure stable system operation.
