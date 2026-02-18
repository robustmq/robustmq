# Infrastructure Metrics

RobustMQ provides comprehensive infrastructure monitoring metrics to help operations teams monitor system health, performance, and resource usage. All metrics are based on Prometheus format and can be accessed through the `/metrics` endpoint.

> **Note**: Counter-type metrics are automatically suffixed with `_total` in Prometheus. For example, a Counter registered as `grpc_requests` in code will be exposed as `grpc_requests_total` in Prometheus/Grafana. The metric names in the tables below are **Prometheus-exposed names** (Counters already include `_total`).

## Network Layer Metrics

### Request Processing Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `handler_total_ms` | Histogram | `network` | End-to-end total time from request received to response written (ms) |
| `handler_queue_wait_ms` | Histogram | `network` | Time a request spent waiting in the handler queue (ms) |
| `handler_apply_ms` | Histogram | `network` | Time spent in command.apply() processing the request (ms) |
| `handler_write_ms` | Histogram | `network` | Time spent writing the response back to the client (ms) |

### Queue Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `handler_queue_size` | Gauge | `label` | Current number of pending requests in the handler queue |
| `handler_queue_remaining` | Gauge | `label` | Remaining capacity in the handler queue |

### Request Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `handler_requests_total` | Gauge | `network` | Total number of requests processed by handlers |
| `handler_slow_requests_total` | Gauge | `network` | Total number of slow requests (exceeding threshold) |

### Thread Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `broker_active_thread_num` | Gauge | `network`, `thread_type` | Number of active threads by type |

**Label Descriptions:**

| Label | Values | Description |
|-------|--------|-------------|
| `network` | `TCP`, `WebSocket`, `QUIC` | Network connection type |
| `thread_type` | `accept`, `handler`, `response` | Thread type |
| `label` | Custom string | Queue label identifying a specific queue instance |

## gRPC Service Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `grpc_requests_total` | Counter | `service`, `method` | Total number of gRPC requests |
| `grpc_errors_total` | Counter | `service`, `method`, `status_code` | Total number of gRPC errors |
| `grpc_request_duration_ms` | Histogram | `service`, `method` | gRPC request duration (ms) |

**Label Descriptions:**
- `service`: gRPC service name
- `method`: gRPC method name
- `status_code`: gRPC status code

## HTTP Service Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `http_requests_total` | Counter | `method`, `uri` | Total number of HTTP requests |
| `http_errors_total` | Counter | `method`, `uri`, `status_code` | Total number of HTTP errors |
| `http_request_duration_ms` | Histogram | `method`, `uri` | HTTP request duration (ms) |

**Label Descriptions:**
- `method`: HTTP method (GET, POST, PUT, DELETE)
- `uri`: Request path
- `status_code`: HTTP status code

## Storage Layer Metrics (RocksDB)

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `rocksdb_operation_count_total` | Counter | `source`, `operation` | Number of RocksDB operations |
| `rocksdb_operation_ms` | Histogram | `source`, `operation` | RocksDB operation duration (ms) |

**Label Descriptions:**
- `source`: Data source (e.g., metadata, session, message)
- `operation`: Operation type (save, get, delete, list)

## Raft Consensus Layer Metrics

### Write Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `raft_write_requests_total` | Counter | `machine` | Total Raft write requests |
| `raft_write_success_total` | Counter | `machine` | Total successful Raft writes |
| `raft_write_failures_total` | Counter | `machine` | Total failed Raft writes |
| `raft_write_duration_ms` | Histogram | `machine` | Raft write operation duration (ms) |

### RPC Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `raft_rpc_requests_total` | Counter | `machine`, `rpc_type` | Total Raft RPC requests |
| `raft_rpc_success_total` | Counter | `machine`, `rpc_type` | Total successful Raft RPCs |
| `raft_rpc_failures_total` | Counter | `machine`, `rpc_type` | Total failed Raft RPCs |
| `raft_rpc_duration_ms` | Histogram | `machine`, `rpc_type` | Raft RPC operation duration (ms) |

**Label Descriptions:**
- `machine`: State machine type (e.g., metadata, offset, mqtt)
- `rpc_type`: RPC type (only reported in multi-node cluster setups)

## MQTT Protocol Metrics

### Resource Statistics (Gauge)

Real-time counts of various system resources.

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_connections_count` | Gauge | — | Current MQTT connection count |
| `mqtt_sessions_count` | Gauge | — | Current MQTT session count |
| `mqtt_topics_count` | Gauge | — | Current MQTT topic count |
| `mqtt_subscribers_count` | Gauge | — | Current MQTT subscriber count (all types) |
| `mqtt_subscriptions_exclusive_count` | Gauge | — | Current exclusive subscription count |
| `mqtt_subscriptions_shared_count` | Gauge | — | Current shared subscription count |
| `mqtt_subscriptions_shared_group_count` | Gauge | — | Current shared subscription group count |
| `mqtt_retained_count` | Gauge | — | Current retained message count |

### Connection and Authentication Events

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `client_connections_total` | Counter | `client_id` | Client connection attempts (regardless of success) |
| `mqtt_connection_success_total` | Counter | — | Successful MQTT connections |
| `mqtt_connection_failed_total` | Counter | — | Failed MQTT connections |
| `mqtt_disconnect_success_total` | Counter | — | MQTT disconnections |
| `mqtt_connection_expired_total` | Counter | — | Expired MQTT connections |
| `mqtt_auth_success_total` | Counter | — | Successful MQTT authentications |
| `mqtt_auth_failed_total` | Counter | — | Failed MQTT authentications |
| `mqtt_acl_success_total` | Counter | — | Successful MQTT ACL checks |
| `mqtt_acl_failed_total` | Counter | — | Failed MQTT ACL checks |
| `mqtt_blacklist_blocked_total` | Counter | — | MQTT connections blocked by blacklist |

### Subscription Events

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_subscribe_success_total` | Counter | — | Successful MQTT subscriptions |
| `mqtt_subscribe_failed_total` | Counter | — | Failed MQTT subscriptions |
| `mqtt_unsubscribe_success_total` | Counter | — | Successful MQTT unsubscriptions |

### Session Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_session_created_total` | Counter | — | MQTT sessions created |
| `mqtt_session_deleted_total` | Counter | — | MQTT sessions deleted |
| `session_messages_in_total` | Counter | `client_id` | Messages received per session |
| `session_messages_out_total` | Counter | `client_id` | Messages sent per session |
| `connection_messages_in_total` | Counter | `connection_id` | Messages received per connection |
| `connection_messages_out_total` | Counter | `connection_id` | Messages sent per connection |

### Message Delivery Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_messages_received_total` | Counter | — | Total messages received from clients |
| `mqtt_messages_sent_total` | Counter | — | Total messages sent to clients |
| `mqtt_message_bytes_received_total` | Counter | — | Total bytes received from clients |
| `mqtt_message_bytes_sent_total` | Counter | — | Total bytes sent to clients |
| `mqtt_messages_delayed_total` | Counter | — | Total delayed publish messages |
| `mqtt_messages_dropped_no_subscribers_total` | Counter | — | Messages dropped due to no subscribers |

### Per-Topic Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `topic_messages_written_total` | Counter | `topic` | Messages written to topic |
| `topic_bytes_written_total` | Counter | `topic` | Bytes written to topic |
| `topic_messages_sent_total` | Counter | `topic` | Messages sent from topic |
| `topic_bytes_sent_total` | Counter | `topic` | Bytes sent from topic |

### Per-Subscription Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `subscribe_messages_sent_total` | Counter | `client_id`, `path`, `status` | Messages sent per subscription path |
| `subscribe_topic_messages_sent_total` | Counter | `client_id`, `path`, `topic_name`, `status` | Messages sent per subscription path + topic |
| `subscribe_bytes_sent_total` | Counter | `client_id`, `path`, `status` | Bytes sent per subscription path |
| `subscribe_topic_bytes_sent_total` | Counter | `client_id`, `path`, `topic_name`, `status` | Bytes sent per subscription path + topic |

### Packet Statistics (Received)

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_packets_received_total` | Counter | `network` | Total MQTT packets received |
| `mqtt_packets_connect_received_total` | Counter | `network` | CONNECT packets received |
| `mqtt_packets_publish_received_total` | Counter | `network` | PUBLISH packets received |
| `mqtt_packets_connack_received_total` | Counter | `network` | CONNACK packets received |
| `mqtt_packets_puback_received_total` | Counter | `network` | PUBACK packets received |
| `mqtt_packets_pubrec_received_total` | Counter | `network` | PUBREC packets received |
| `mqtt_packets_pubrel_received_total` | Counter | `network` | PUBREL packets received |
| `mqtt_packets_pubcomp_received_total` | Counter | `network` | PUBCOMP packets received |
| `mqtt_packets_subscribe_received_total` | Counter | `network` | SUBSCRIBE packets received |
| `mqtt_packets_unsubscribe_received_total` | Counter | `network` | UNSUBSCRIBE packets received |
| `mqtt_packets_pingreq_received_total` | Counter | `network` | PINGREQ packets received |
| `mqtt_packets_disconnect_received_total` | Counter | `network` | DISCONNECT packets received |
| `mqtt_packets_auth_received_total` | Counter | `network` | AUTH packets received |
| `mqtt_packets_received_error_total` | Counter | `network` | Error packets received |
| `mqtt_packets_connack_auth_error_total` | Counter | `network` | CONNACK auth error packets |
| `mqtt_packets_connack_error_total` | Counter | `network` | CONNACK error packets |
| `mqtt_bytes_received_total` | Counter | `network` | Total MQTT bytes received |

### Packet Statistics (Sent)

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_packets_sent_total` | Counter | `network`, `qos` | Total MQTT packets sent |
| `mqtt_packets_connack_sent_total` | Counter | `network`, `qos` | CONNACK packets sent |
| `mqtt_packets_publish_sent_total` | Counter | `network`, `qos` | PUBLISH packets sent |
| `mqtt_packets_puback_sent_total` | Counter | `network`, `qos` | PUBACK packets sent |
| `mqtt_packets_pubrec_sent_total` | Counter | `network`, `qos` | PUBREC packets sent |
| `mqtt_packets_pubrel_sent_total` | Counter | `network`, `qos` | PUBREL packets sent |
| `mqtt_packets_pubcomp_sent_total` | Counter | `network`, `qos` | PUBCOMP packets sent |
| `mqtt_packets_suback_sent_total` | Counter | `network`, `qos` | SUBACK packets sent |
| `mqtt_packets_unsuback_sent_total` | Counter | `network`, `qos` | UNSUBACK packets sent |
| `mqtt_packets_pingresp_sent_total` | Counter | `network`, `qos` | PINGRESP packets sent |
| `mqtt_packets_disconnect_sent_total` | Counter | `network`, `qos` | DISCONNECT packets sent |
| `mqtt_bytes_sent_total` | Counter | `network`, `qos` | Total MQTT bytes sent |
| `mqtt_retain_packets_received_total` | Counter | `qos` | Retained messages received |
| `mqtt_retain_packets_sent_total` | Counter | `qos` | Retained messages sent |

### Packet Processing Duration

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_packet_process_duration_ms` | Histogram | `network`, `packet` | MQTT packet processing duration (ms) |
| `mqtt_packet_send_duration_ms` | Histogram | `network`, `packet` | MQTT packet sending duration (ms) |

## Delay Message Queue Metrics

### Queue Capacity (Gauge)

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_delay_queue_total_capacity` | Gauge | `shard_no` | Delay queue total capacity |
| `mqtt_delay_queue_used_capacity` | Gauge | `shard_no` | Delay queue used capacity |
| `mqtt_delay_queue_remaining_capacity` | Gauge | `shard_no` | Delay queue remaining capacity |

### Message Delivery Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `delay_msg_enqueue_total` | Counter | — | Total messages enqueued |
| `delay_msg_deliver_total` | Counter | — | Total delay messages delivered |
| `delay_msg_deliver_fail_total` | Counter | — | Total delivery failures |
| `delay_msg_recover_total` | Counter | — | Total messages recovered from storage on startup |
| `delay_msg_recover_expired_total` | Counter | — | Total expired messages found during recovery |

### Latency Distribution

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `delay_msg_enqueue_duration_ms` | Histogram | — | Message enqueue duration (ms) |
| `delay_msg_deliver_duration_ms` | Histogram | — | Message delivery duration (ms) |

## Connector Metrics

### Per-Connector

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_connector_messages_sent_success_total` | Counter | `connector_name` | Messages successfully sent by connector |
| `mqtt_connector_messages_sent_failure_total` | Counter | `connector_name` | Messages failed to send by connector |
| `mqtt_connector_send_duration_ms` | Histogram | `connector_name` | Message send duration by connector (ms) |

### Aggregate

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_connector_messages_sent_success_agg_total` | Counter | — | Total messages successfully sent by all connectors |
| `mqtt_connector_messages_sent_failure_agg_total` | Counter | — | Total messages failed to send by all connectors |
| `mqtt_connector_send_duration_ms_agg` | Histogram | — | Aggregate send duration across all connectors (ms) |

## Usage Examples

### Prometheus Configuration

```yaml
scrape_configs:
  - job_name: 'robustmq'
    static_configs:
      - targets: ['localhost:9091']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### Grafana Query Examples

**Network layer queries:**

```promql
# Request processing P95 latency (by network type)
histogram_quantile(0.95, rate(handler_total_ms_bucket[5m]))

# Handler queue wait P95 latency
histogram_quantile(0.95, rate(handler_queue_wait_ms_bucket[5m]))

# Average command.apply() execution time
rate(handler_apply_ms_sum[5m]) / rate(handler_apply_ms_count[5m])

# Queue backlog
handler_queue_size

# Active threads by type
broker_active_thread_num{thread_type="handler"}
```

**gRPC/HTTP queries:**

```promql
# gRPC request rate (per second)
sum(rate(grpc_requests_total[5m]))

# gRPC error rate
rate(grpc_errors_total[5m]) / rate(grpc_requests_total[5m]) * 100

# HTTP request rate (per second)
sum(rate(http_requests_total[5m]))
```

**RocksDB queries:**

```promql
# RocksDB QPS by operation type
sum(rate(rocksdb_operation_count_total[5m])) by (operation)

# RocksDB average operation latency
rate(rocksdb_operation_ms_sum[5m]) / rate(rocksdb_operation_ms_count[5m])
```

**Raft queries:**

```promql
# Raft write QPS by state machine
sum(rate(raft_write_requests_total[5m])) by (machine)

# Raft write success rate
sum(rate(raft_write_success_total[5m])) / sum(rate(raft_write_requests_total[5m]))

# Raft write P99 latency
histogram_quantile(0.99, sum(rate(raft_write_duration_ms_bucket[5m])) by (le, machine))
```

**MQTT resource queries:**

```promql
# Current connections
mqtt_connections_count

# Current subscribers
mqtt_subscribers_count

# Connection success rate
rate(mqtt_connection_success_total[5m])

# MQTT packet processing P99 latency by type
histogram_quantile(0.99, sum(rate(mqtt_packet_process_duration_ms_bucket[5m])) by (le, packet))
```

## Alert Rules Examples

```yaml
groups:
  - name: robustmq_network_metrics
    rules:
      - alert: HighRequestLatency
        expr: histogram_quantile(0.95, rate(handler_total_ms_bucket[5m])) > 1000
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High request latency"
          description: "P95 request latency exceeds 1000ms, current: {{ $value }}ms"

      - alert: HandlerQueueBacklog
        expr: handler_queue_size > 10000
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Handler queue backlog"
          description: "Queue pending requests exceed 10000, current: {{ $value }}"

  - name: robustmq_grpc_metrics
    rules:
      - alert: HighGrpcErrorRate
        expr: rate(grpc_errors_total[5m]) / rate(grpc_requests_total[5m]) > 0.05
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "High gRPC error rate"
          description: "gRPC error rate exceeds 5%, current: {{ $value | humanizePercentage }}"

  - name: robustmq_storage_metrics
    rules:
      - alert: RocksDBSlowOperations
        expr: histogram_quantile(0.95, rate(rocksdb_operation_ms_bucket[5m])) > 100
        for: 3m
        labels:
          severity: warning
        annotations:
          summary: "Slow RocksDB operations"
          description: "P95 RocksDB operation latency exceeds 100ms, current: {{ $value }}ms"

  - name: robustmq_raft_metrics
    rules:
      - alert: RaftWriteFailureRate
        expr: rate(raft_write_failures_total[5m]) / rate(raft_write_requests_total[5m]) > 0.01
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High Raft write failure rate"
          description: "Raft write failure rate exceeds 1%, current: {{ $value | humanizePercentage }}"
```

## Related Files

- **Metric definitions**: `src/common/metrics/src/`
  - Network: `network.rs`
  - gRPC: `grpc.rs`
  - HTTP: `http.rs`
  - RocksDB: `rocksdb.rs`
  - Raft: `meta/raft.rs`
  - MQTT: `mqtt/` directory
- **Grafana dashboard**: `grafana/robustmq-broker.json`
