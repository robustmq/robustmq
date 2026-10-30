# MQTT Specific Metrics

RobustMQ exposes MQTT protocol, connection, message, delay-queue, and connector metrics through the Prometheus `/metrics` endpoint. The names below are the names emitted by Prometheus. Counter families are registered without `_total` in Rust; `prometheus-client` adds that suffix to the exposed name.

## Packet Metrics

### Received packets

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_packets_received_total` | Counter | `network` | MQTT packets received |
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
| `mqtt_packets_received_error_total` | Counter | `network` | Packets rejected during receive or decoding |
| `mqtt_packets_connack_auth_error_total` | Counter | `network` | CONNACK packets rejected for authorization |
| `mqtt_packets_connack_error_total` | Counter | `network` | CONNACK packets with an error result |
| `mqtt_bytes_received_total` | Counter | `network` | MQTT bytes received |

### Sent packets

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_packets_sent_total` | Counter | `network`, `qos` | MQTT packets sent |
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
| `mqtt_packets_auth_sent_total` | Counter | `network`, `qos` | AUTH packets sent; `qos` is `-1` because AUTH is not a PUBLISH packet |
| `mqtt_bytes_sent_total` | Counter | `network`, `qos` | MQTT bytes sent |
| `mqtt_retain_packets_received_total` | Counter | `qos` | Retained messages received |
| `mqtt_retain_packets_sent_total` | Counter | `qos` | Retained messages sent |

### Cross-network totals

These unlabeled counters aggregate packet and byte totals across all network types.

| Metric Name | Type | Description |
|-------------|------|-------------|
| `mqtt_total_bytes_received_total` | Counter | MQTT bytes received across all network types |
| `mqtt_total_bytes_sent_total` | Counter | MQTT bytes sent across all network types |
| `mqtt_total_packets_received_total` | Counter | MQTT packets received across all network types |
| `mqtt_total_packets_sent_total` | Counter | MQTT packets sent across all network types |
| `mqtt_total_packets_connect_total` | Counter | CONNECT packets received |
| `mqtt_total_packets_connack_total` | Counter | CONNACK packets sent |
| `mqtt_total_packets_publish_received_total` | Counter | PUBLISH packets received |
| `mqtt_total_packets_publish_sent_total` | Counter | PUBLISH packets sent |
| `mqtt_total_packets_puback_received_total` | Counter | PUBACK packets received |
| `mqtt_total_packets_puback_sent_total` | Counter | PUBACK packets sent |
| `mqtt_total_packets_pubrec_received_total` | Counter | PUBREC packets received |
| `mqtt_total_packets_pubrec_sent_total` | Counter | PUBREC packets sent |
| `mqtt_total_packets_pubrel_received_total` | Counter | PUBREL packets received |
| `mqtt_total_packets_pubrel_sent_total` | Counter | PUBREL packets sent |
| `mqtt_total_packets_pubcomp_received_total` | Counter | PUBCOMP packets received |
| `mqtt_total_packets_pubcomp_sent_total` | Counter | PUBCOMP packets sent |
| `mqtt_total_packets_subscribe_total` | Counter | SUBSCRIBE packets received |
| `mqtt_total_packets_suback_total` | Counter | SUBACK packets sent |
| `mqtt_total_packets_unsubscribe_total` | Counter | UNSUBSCRIBE packets received |
| `mqtt_total_packets_unsuback_total` | Counter | UNSUBACK packets sent |
| `mqtt_total_packets_pingreq_total` | Counter | PINGREQ packets received |
| `mqtt_total_packets_pingresp_total` | Counter | PINGRESP packets sent |
| `mqtt_total_packets_disconnect_received_total` | Counter | DISCONNECT packets received |
| `mqtt_total_packets_disconnect_sent_total` | Counter | DISCONNECT packets sent |
| `mqtt_total_packets_auth_total` | Counter | AUTH packets received |
| `mqtt_total_packets_auth_sent_total` | Counter | AUTH packets sent |

### Packet timing

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_packet_process_duration_ms` | Histogram | `network`, `packet` | MQTT packet processing duration in milliseconds |
| `mqtt_packet_send_duration_ms` | Histogram | `network`, `packet` | MQTT packet sending duration in milliseconds |

`network` values are `Tcp`, `Tls`, `Websocket`, `Websockets`, `Quic`, and `Unknown` when the connection type is unavailable. The `qos` value is `0`, `1`, or `2` for PUBLISH packets and `-1` for non-PUBLISH packets. Packet names are lowercase values such as `connect`, `publish`, and `subscribe`.

## Connection and Authentication Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `client_connections_total` | Counter | `tenant`, `client_id` | Client connection attempts, regardless of outcome |
| `mqtt_connection_success_total` | Counter | — | Successful MQTT connections |
| `mqtt_connection_failed_total` | Counter | — | Failed MQTT connections |
| `mqtt_disconnect_success_total` | Counter | — | Successful MQTT disconnections |
| `mqtt_connection_expired_total` | Counter | — | Expired MQTT connections |
| `mqtt_auth_success_total` | Counter | — | Successful MQTT authentication checks |
| `mqtt_auth_failed_total` | Counter | — | Failed MQTT authentication checks |
| `mqtt_acl_success_total` | Counter | — | Successful MQTT ACL checks |
| `mqtt_acl_failed_total` | Counter | — | Failed MQTT ACL checks |
| `mqtt_blacklist_blocked_total` | Counter | — | Connections blocked by the MQTT blacklist |
| `mqtt_subscribe_success_total` | Counter | — | Successful subscriptions |
| `mqtt_subscribe_failed_total` | Counter | — | Failed subscriptions |
| `mqtt_unsubscribe_success_total` | Counter | — | Successful unsubscriptions |

## Session and Message Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_session_created_total` | Counter | — | MQTT sessions created |
| `mqtt_session_deleted_total` | Counter | — | MQTT sessions deleted |
| `session_messages_in_total` | Counter | `tenant`, `client_id` | Messages received per session |
| `session_messages_out_total` | Counter | `tenant`, `client_id` | Messages sent per session |
| `connection_messages_in_total` | Counter | `connection_id` | Messages received per connection |
| `connection_messages_out_total` | Counter | `connection_id` | Messages sent per connection |
| `mqtt_messages_received_total` | Counter | — | Messages received from clients |
| `mqtt_messages_sent_total` | Counter | — | Messages sent to clients |
| `mqtt_message_bytes_received_total` | Counter | — | Message bytes received from clients |
| `mqtt_message_bytes_sent_total` | Counter | — | Message bytes sent to clients |
| `mqtt_messages_delayed_total` | Counter | — | Delayed publish messages |
| `mqtt_messages_dropped_no_subscribers_total` | Counter | — | Messages dropped because there were no subscribers |

## Resource Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_connections_count` | Gauge | — | Current MQTT connection count |
| `mqtt_sessions_count` | Gauge | — | Current MQTT session count |
| `mqtt_topics_count` | Gauge | — | Current MQTT topic count |
| `mqtt_subscribers_count` | Gauge | — | Current MQTT subscriber count |
| `mqtt_subscriptions_exclusive_count` | Gauge | — | Current exclusive subscription count |
| `mqtt_subscriptions_shared_count` | Gauge | — | Current shared subscription count |
| `mqtt_subscriptions_shared_group_count` | Gauge | — | Current shared subscription-group count |
| `mqtt_retained_count` | Gauge | — | Current retained-message count |
| `mqtt_delay_queue_total_capacity` | Gauge | `shard_no` | Total delay-queue capacity |
| `mqtt_delay_queue_used_capacity` | Gauge | `shard_no` | Used delay-queue capacity |
| `mqtt_delay_queue_remaining_capacity` | Gauge | `shard_no` | Remaining delay-queue capacity |

## Topic and Subscription Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `topic_messages_written_total` | Counter | `tenant`, `topic` | Messages written to a topic |
| `topic_bytes_written_total` | Counter | `tenant`, `topic` | Bytes written to a topic |
| `topic_messages_sent_total` | Counter | `tenant`, `topic` | Messages sent from a topic |
| `topic_bytes_sent_total` | Counter | `tenant`, `topic` | Bytes sent from a topic |
| `subscribe_messages_sent_total` | Counter | `tenant`, `client_id`, `path`, `status` | Messages sent to a subscription path |
| `subscribe_topic_messages_sent_total` | Counter | `tenant`, `client_id`, `path`, `topic_name`, `status` | Messages sent to a subscription path and topic |
| `subscribe_bytes_sent_total` | Counter | `tenant`, `client_id`, `path`, `status` | Bytes sent to a subscription path |
| `subscribe_topic_bytes_sent_total` | Counter | `tenant`, `client_id`, `path`, `topic_name`, `status` | Bytes sent to a subscription path and topic |

Subscription `status` values are `success` and `failed`.

## Delay Message and Connector Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `delay_msg_enqueue_total` | Counter | — | Delay messages enqueued |
| `delay_msg_deliver_total` | Counter | — | Delay messages delivered |
| `delay_msg_deliver_fail_total` | Counter | — | Delay-message delivery failures |
| `delay_msg_recover_total` | Counter | — | Delay messages recovered at startup |
| `delay_msg_recover_expired_total` | Counter | — | Expired messages found during recovery |
| `delay_message_retry_total` | Counter | `status` | Retry attempts; status is `success` or `failed` |
| `delay_message_retry_count_total` | Counter | `attempt` | Retry attempts grouped by numeric attempt value |
| `delay_message_reenqueued_total` | Counter | — | Messages re-enqueued after a failed delivery |
| `delay_message_dead_letter_total` | Counter | — | Messages sent to the dead-letter queue |
| `delay_msg_enqueue_duration_ms` | Histogram | — | Delay-message enqueue duration |
| `delay_msg_deliver_duration_ms` | Histogram | — | Delay-message delivery duration |
| `delay_task_created_total` | Counter | — | Delay tasks created |
| `delay_task_executed_total` | Counter | `task_type` | Delay tasks executed successfully |
| `delay_task_execute_failed_total` | Counter | `task_type` | Delay-task execution failures |
| `delay_task_schedule_latency_seconds` | Histogram | `task_type` | Difference between actual and target execution time |
| `mqtt_connector_messages_sent_success_total` | Counter | `tenant`, `connector_type`, `connector_name` | Messages sent successfully by a connector |
| `mqtt_connector_messages_sent_failure_total` | Counter | `tenant`, `connector_type`, `connector_name` | Messages that failed to send |
| `mqtt_connector_send_duration_ms` | Histogram | `tenant`, `connector_type`, `connector_name` | Connector send duration |
| `mqtt_connector_retry_total` | Counter | `tenant`, `connector_type`, `connector_name`, `strategy` | Connector retry attempts |
| `mqtt_connector_messages_discarded_total` | Counter | `tenant`, `connector_type`, `connector_name`, `strategy` | Messages discarded by terminal strategy |
| `mqtt_connector_dlq_messages_total` | Counter | `tenant`, `connector_type`, `connector_name`, `result` | DLQ write attempts by result |
| `mqtt_connector_offset_commit_failure_total` | Counter | `tenant`, `connector_type`, `connector_name` | Connector offset-commit failures |
| `mqtt_connector_source_read_failure_total` | Counter | `tenant`, `connector_type`, `connector_name` | Connector source-read failures |
| `mqtt_connector_up` | Gauge | `tenant`, `connector_type`, `connector_name` | Connector status: 1 running, 0 stopped |
| `mqtt_connector_messages_sent_success_agg_total` | Counter | — | Successful sends across all connectors |
| `mqtt_connector_messages_sent_failure_agg_total` | Counter | — | Failed sends across all connectors |
| `mqtt_connector_send_duration_ms_agg` | Histogram | — | Aggregate connector send duration |

## Usage Examples

```rust
use common_metrics::mqtt::{auth, event, time};
use metadata_struct::connection::NetworkConnectionType;

event::record_mqtt_connection_success();
auth::record_mqtt_auth_failed();
time::record_packet_process_duration(
    &NetworkConnectionType::Tcp,
    "publish",
    15.5,
);
```

Example PromQL queries:

```promql
rate(mqtt_packets_publish_received_total[5m])
rate(mqtt_packets_publish_sent_total[5m])
rate(mqtt_auth_failed_total[5m]) /
  (rate(mqtt_auth_success_total[5m]) + rate(mqtt_auth_failed_total[5m]))
histogram_quantile(0.95, sum by (le) (rate(mqtt_packet_process_duration_ms_bucket[5m])))
mqtt_delay_queue_used_capacity / mqtt_delay_queue_total_capacity
```

## Grafana Dashboard Configuration

The repository includes an importable dashboard at [`grafana/robustmq-broker.json`](../../../grafana/robustmq-broker.json). For setup and import instructions, see the [Grafana Configuration Guide](Grafana-Configuration-Guide.md). The examples below are panel query snippets, not a complete dashboard JSON document.

### Connection Overview Panel

```json
{
  "title": "MQTT Connection Overview",
  "targets": [
    {
      "expr": "mqtt_connections_count",
      "legendFormat": "Current Connections"
    },
    {
      "expr": "rate(mqtt_connection_success_total[5m])",
      "legendFormat": "Connection Success Rate/sec"
    },
    {
      "expr": "rate(mqtt_connection_failed_total[5m])",
      "legendFormat": "Connection Failure Rate/sec"
    }
  ]
}
```

### Message Throughput Panel

The packet and byte counters have network labels. The `sum by (network)` aggregation combines sent QoS series while preserving the transport type in the legend.

```json
{
  "title": "MQTT Message Throughput",
  "targets": [
    {
      "expr": "sum by (network) (rate(mqtt_packets_publish_received_total[5m]))",
      "legendFormat": "PUBLISH Received/sec ({{network}})"
    },
    {
      "expr": "sum by (network) (rate(mqtt_packets_publish_sent_total[5m]))",
      "legendFormat": "PUBLISH Sent/sec ({{network}})"
    },
    {
      "expr": "sum by (network) (rate(mqtt_bytes_received_total[5m]))",
      "legendFormat": "Bytes Received/sec ({{network}})"
    },
    {
      "expr": "sum by (network) (rate(mqtt_bytes_sent_total[5m]))",
      "legendFormat": "Bytes Sent/sec ({{network}})"
    }
  ]
}
```

### Performance Monitoring Panel

Histogram buckets must be aggregated by `le` before calculating a percentile.

```json
{
  "title": "MQTT Performance Monitoring",
  "targets": [
    {
      "expr": "histogram_quantile(0.95, sum by (le) (rate(mqtt_packet_process_duration_ms_bucket[5m])))",
      "legendFormat": "P95 Processing Duration (ms)"
    },
    {
      "expr": "histogram_quantile(0.99, sum by (le) (rate(mqtt_packet_process_duration_ms_bucket[5m])))",
      "legendFormat": "P99 Processing Duration (ms)"
    }
  ]
}
```

## MQTT Alert Rules

The Prometheus rule examples below use metric names exposed by the current implementation. The repository also provides a broader alert rules file at [`grafana/robustmq-alerts.yml`](../../../grafana/robustmq-alerts.yml); see the [Grafana Configuration Guide](Grafana-Configuration-Guide.md) for setup. These snippets show MQTT-specific alert conditions and are not a replacement for the complete file.

### Connection Alerts

```yaml
- alert: RobustMQHighConnectionFailureRate
  expr: (sum(rate(mqtt_connection_failed_total[5m])) / clamp_min(sum(rate(mqtt_connection_success_total[5m])) + sum(rate(mqtt_connection_failed_total[5m])), 1e-9)) > 0.1
  for: 2m
  labels:
    severity: warning
  annotations:
    summary: "High MQTT connection failure rate"
    description: "MQTT connection failure ratio is {{ $value | humanizePercentage }}."

- alert: RobustMQHighConnectionCount
  expr: mqtt_connections_count > 5000
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "High MQTT connection count"
    description: "Current MQTT connections: {{ $value }}"
```

### Performance and Error Alerts

Histogram buckets are aggregated by `le` and grouped by the available `network` and `packet` labels so the alert identifies the affected transport and packet type.

```yaml
- alert: RobustMQHighMqttPacketProcessingLatency
  expr: histogram_quantile(0.95, sum by (le, network, packet) (rate(mqtt_packet_process_duration_ms_bucket[5m]))) > 100
  for: 3m
  labels:
    severity: warning
  annotations:
    summary: "High MQTT packet processing latency"
    description: "P95 latency is {{ $value }}ms for {{ $labels.packet }} on {{ $labels.network }}."

- alert: RobustMQPacketErrors
  expr: sum by (network) (rate(mqtt_packets_received_error_total[5m])) > 1
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "High MQTT packet error rate"
    description: "Packet errors are {{ $value }} per second on {{ $labels.network }}."
```

### Authentication and Capacity Alerts

```yaml
- alert: RobustMQHighMqttAuthenticationFailureRatio
  expr: (sum(rate(mqtt_auth_failed_total[5m])) / clamp_min(sum(rate(mqtt_auth_success_total[5m])) + sum(rate(mqtt_auth_failed_total[5m])), 1e-9)) > 0.2
  for: 1m
  labels:
    severity: warning
  annotations:
    summary: "High MQTT authentication failure ratio"
    description: "Authentication failure ratio is {{ $value | humanizePercentage }}."

- alert: RobustMQDelayQueueNearFull
  expr: (mqtt_delay_queue_total_capacity > 0) and ((mqtt_delay_queue_used_capacity / mqtt_delay_queue_total_capacity) > 0.9)
  for: 2m
  labels:
    severity: critical
  annotations:
    summary: "MQTT delay queue near capacity"
    description: "Shard {{ $labels.shard_no }} is using {{ $value | humanizePercentage }} of capacity."

- alert: RobustMQTooManyRetainedMessages
  expr: mqtt_retained_count > 100000
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "Too many MQTT retained messages"
    description: "Current retained message count: {{ $value }}"
```
