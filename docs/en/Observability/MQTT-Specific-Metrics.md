# MQTT Specific Metrics

RobustMQ provides comprehensive MQTT protocol-specific monitoring metrics covering packet processing, connection management, authentication and authorization, message publishing, session management, and other aspects. These metrics help operations teams gain deep insights into MQTT service operational status and performance.

## Packet Metrics

### Received Packet Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_packets_received` | Gauge | `network` | Total number of MQTT packets received |
| `mqtt_packets_connect_received` | Gauge | `network` | Number of CONNECT packets received |
| `mqtt_packets_publish_received` | Gauge | `network` | Number of PUBLISH packets received |
| `mqtt_packets_connack_received` | Gauge | `network` | Number of CONNACK packets received |
| `mqtt_packets_puback_received` | Gauge | `network` | Number of PUBACK packets received |
| `mqtt_packets_pubrec_received` | Gauge | `network` | Number of PUBREC packets received |
| `mqtt_packets_pubrel_received` | Gauge | `network` | Number of PUBREL packets received |
| `mqtt_packets_pubcomp_received` | Gauge | `network` | Number of PUBCOMP packets received |
| `mqtt_packets_subscribe_received` | Gauge | `network` | Number of SUBSCRIBE packets received |
| `mqtt_packets_unsubscribe_received` | Gauge | `network` | Number of UNSUBSCRIBE packets received |
| `mqtt_packets_pingreq_received` | Gauge | `network` | Number of PINGREQ packets received |
| `mqtt_packets_disconnect_received` | Gauge | `network` | Number of DISCONNECT packets received |
| `mqtt_packets_auth_received` | Gauge | `network` | Number of AUTH packets received |

### Sent Packet Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_packets_sent` | Gauge | `network`, `qos` | Total number of MQTT packets sent |
| `mqtt_packets_connack_sent` | Gauge | `network`, `qos` | Number of CONNACK packets sent |
| `mqtt_packets_publish_sent` | Gauge | `network`, `qos` | Number of PUBLISH packets sent |
| `mqtt_packets_puback_sent` | Gauge | `network`, `qos` | Number of PUBACK packets sent |
| `mqtt_packets_pubrec_sent` | Gauge | `network`, `qos` | Number of PUBREC packets sent |
| `mqtt_packets_pubrel_sent` | Gauge | `network`, `qos` | Number of PUBREL packets sent |
| `mqtt_packets_pubcomp_sent` | Gauge | `network`, `qos` | Number of PUBCOMP packets sent |
| `mqtt_packets_suback_sent` | Gauge | `network`, `qos` | Number of SUBACK packets sent |
| `mqtt_packets_unsuback_sent` | Gauge | `network`, `qos` | Number of UNSUBACK packets sent |
| `mqtt_packets_pingresp_sent` | Gauge | `network`, `qos` | Number of PINGRESP packets sent |
| `mqtt_packets_disconnect_sent` | Gauge | `network`, `qos` | Number of DISCONNECT packets sent |

### Byte Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_bytes_received` | Gauge | `network` | Total bytes received for MQTT |
| `mqtt_bytes_sent` | Gauge | `network`, `qos` | Total bytes sent for MQTT |

### Error Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_packets_received_error` | Gauge | `network` | Number of MQTT error packets received |
| `mqtt_packets_connack_auth_error` | Gauge | `network` | Number of CONNACK auth error packets |
| `mqtt_packets_connack_error` | Gauge | `network` | Number of CONNACK error packets |

### Retained Messages and Dropped Messages

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_retain_packets_received` | Gauge | `qos` | Number of retained messages received |
| `mqtt_retain_packets_sent` | Gauge | `qos` | Number of retained messages sent |
| `mqtt_messages_dropped_no_subscribers` | Gauge | `qos` | Number of messages dropped due to no subscribers |

**Label Descriptions:**
- `network`: Network type (tcp, websocket, quic)
- `qos`: QoS level (0, 1, 2, -1 for non-PUBLISH packets)

## Connection Event Metrics

### Connection Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_connection_success` | Counter | - | Number of successful MQTT connections |
| `mqtt_connection_failed` | Counter | - | Number of failed MQTT connections |
| `mqtt_disconnect_success` | Counter | - | Number of successful MQTT disconnections |
| `mqtt_connection_expired` | Counter | - | Number of expired MQTT connections |

### Subscription Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_subscribe_success` | Counter | - | Number of successful MQTT topic subscriptions |
| `mqtt_subscribe_failed` | Counter | - | Number of failed MQTT subscriptions |
| `mqtt_unsubscribe_success` | Counter | - | Number of successful MQTT topic unsubscriptions |

### Client Connection Count

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `client_connections` | Counter | `client_id` | Number of connections for specific client |

## Authentication and Authorization Metrics

### Authentication Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_auth_success` | Counter | - | Number of successful MQTT login authentications |
| `mqtt_auth_failed` | Counter | - | Number of failed MQTT login authentications |

### Authorization Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_acl_success` | Counter | - | Number of successful MQTT ACL checks |
| `mqtt_acl_failed` | Counter | - | Number of failed MQTT ACL checks |

### Security Protection

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_blacklist_blocked` | Counter | - | Number of MQTT blacklist blocks |

## Session Management Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_session_created` | Counter | - | Number of MQTT sessions created |
| `mqtt_session_deleted` | Counter | - | Number of MQTT sessions deleted |

## Real-time Statistics Metrics

### Current Status Statistics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_connections_count` | Gauge | - | Current number of MQTT connections |
| `mqtt_sessions_count` | Gauge | - | Current number of MQTT sessions |
| `mqtt_topics_count` | Gauge | - | Current number of topics |
| `mqtt_subscribers_count` | Gauge | - | Current number of subscribers |
| `mqtt_subscriptions_shared_count` | Gauge | - | Current number of shared subscriptions |
| `mqtt_retained_count` | Gauge | - | Current number of retained messages |

### Delay Queue Monitoring

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_delay_queue_total_capacity` | Gauge | `shard_no` | Total capacity of delay queue |
| `mqtt_delay_queue_used_capacity` | Gauge | `shard_no` | Used capacity of delay queue |
| `mqtt_delay_queue_remaining_capacity` | Gauge | `shard_no` | Remaining capacity of delay queue |

**Label Descriptions:**
- `shard_no`: Shard number

## Message Publishing Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_messages_delayed` | Gauge | - | Number of stored delayed publish messages |
| `mqtt_messages_received` | Gauge | - | Number of messages received from clients |
| `mqtt_messages_sent` | Gauge | - | Number of messages sent to clients |

## Performance Metrics (Time)

### Processing Duration

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `mqtt_packet_process_duration_ms` | Histogram | `network`, `packet` | MQTT packet processing duration (milliseconds) |
| `mqtt_packet_send_duration_ms` | Histogram | `network`, `packet` | MQTT packet sending duration (milliseconds) |

**Label Descriptions:**
- `network`: Network type (tcp, websocket, quic)
- `packet`: Packet type (CONNECT, PUBLISH, SUBSCRIBE, etc.)

## Usage Examples

### Recording Metrics

```rust
use common_metrics::mqtt::*;

// Record connection success
event::record_mqtt_connection_success();

// Record authentication failure
auth::record_mqtt_auth_failed();

// Record packet processing duration
time::record_mqtt_packet_process_duration(
    NetworkConnectionType::Tcp, 
    "PUBLISH".to_string(), 
    15.5
);

// Set current connection count
statistics::record_mqtt_connections_set(100);

// Increment retained message count
statistics::record_mqtt_retained_inc();
```

### Prometheus Query Examples

```promql
# MQTT connection success rate
mqtt_connection_success / (mqtt_connection_success + mqtt_connection_failed) * 100

# MQTT packet processing average duration
rate(mqtt_packet_process_duration_ms_sum[5m]) / rate(mqtt_packet_process_duration_ms_count[5m])

# Current active connections
mqtt_connections_count

# PUBLISH packet QPS
rate(mqtt_packets_publish_received[5m])

# Authentication failure rate
rate(mqtt_auth_failed[5m]) / rate(mqtt_auth_success[5m] + mqtt_auth_failed[5m]) * 100

# Delay queue usage rate
mqtt_delay_queue_used_capacity / mqtt_delay_queue_total_capacity * 100
```

### Grafana Dashboard Configuration

#### Connection Overview Panel
```json
{
  "title": "MQTT Connection Overview",
  "targets": [
    {
      "expr": "mqtt_connections_count",
      "legendFormat": "Current Connections"
    },
    {
      "expr": "rate(mqtt_connection_success[5m])",
      "legendFormat": "Connection Success Rate/sec"
    },
    {
      "expr": "rate(mqtt_connection_failed[5m])",
      "legendFormat": "Connection Failure Rate/sec"
    }
  ]
}
```

#### Message Throughput Panel
```json
{
  "title": "MQTT Message Throughput",
  "targets": [
    {
      "expr": "rate(mqtt_packets_publish_received[5m])",
      "legendFormat": "PUBLISH Received/sec"
    },
    {
      "expr": "rate(mqtt_packets_publish_sent[5m])",
      "legendFormat": "PUBLISH Sent/sec"
    },
    {
      "expr": "rate(mqtt_bytes_received[5m])",
      "legendFormat": "Bytes Received/sec"
    },
    {
      "expr": "rate(mqtt_bytes_sent[5m])",
      "legendFormat": "Bytes Sent/sec"
    }
  ]
}
```

#### Performance Monitoring Panel
```json
{
  "title": "MQTT Performance Monitoring",
  "targets": [
    {
      "expr": "histogram_quantile(0.95, rate(mqtt_packet_process_duration_ms_bucket[5m]))",
      "legendFormat": "P95 Processing Duration"
    },
    {
      "expr": "histogram_quantile(0.99, rate(mqtt_packet_process_duration_ms_bucket[5m]))",
      "legendFormat": "P99 Processing Duration"
    }
  ]
}
```

## Alert Rules

### Connection Alerts
```yaml
- alert: MQTTHighConnectionFailureRate
  expr: rate(mqtt_connection_failed[5m]) / rate(mqtt_connection_success[5m] + mqtt_connection_failed[5m]) > 0.1
  for: 2m
  labels:
    severity: warning
  annotations:
    summary: "High MQTT connection failure rate"
    description: "MQTT connection failure rate exceeds 10%"

- alert: MQTTTooManyConnections
  expr: mqtt_connections_count > 10000
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "Too many MQTT connections"
    description: "Current MQTT connections: {{ $value }}"
```

### Performance Alerts
```yaml
- alert: MQTTHighLatency
  expr: histogram_quantile(0.95, rate(mqtt_packet_process_duration_ms_bucket[5m])) > 100
  for: 3m
  labels:
    severity: warning
  annotations:
    summary: "High MQTT processing latency"
    description: "95% of MQTT packet processing takes over 100ms"

- alert: MQTTHighErrorRate
  expr: rate(mqtt_packets_received_error[5m]) / rate(mqtt_packets_received[5m]) > 0.05
  for: 2m
  labels:
    severity: critical
  annotations:
    summary: "High MQTT error rate"
    description: "MQTT packet error rate exceeds 5%"
```

### Authentication Alerts
```yaml
- alert: MQTTHighAuthFailureRate
  expr: rate(mqtt_auth_failed[5m]) / rate(mqtt_auth_success[5m] + mqtt_auth_failed[5m]) > 0.2
  for: 1m
  labels:
    severity: warning
  annotations:
    summary: "High MQTT authentication failure rate"
    description: "MQTT authentication failure rate exceeds 20%"

- alert: MQTTFrequentBlacklistBlocks
  expr: rate(mqtt_blacklist_blocked[5m]) > 10
  for: 1m
  labels:
    severity: warning
  annotations:
    summary: "Frequent MQTT blacklist blocks"
    description: "Blacklist block rate: {{ $value }}/sec"
```

### Capacity Alerts
```yaml
- alert: MQTTDelayQueueFull
  expr: mqtt_delay_queue_used_capacity / mqtt_delay_queue_total_capacity > 0.9
  for: 2m
  labels:
    severity: critical
  annotations:
    summary: "MQTT delay queue near full"
    description: "Shard {{ $labels.shard_no }} delay queue usage: {{ $value | humanizePercentage }}"

- alert: MQTTTooManyRetainedMessages
  expr: mqtt_retained_count > 100000
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "Too many MQTT retained messages"
    description: "Current retained message count: {{ $value }}"
```

Through these MQTT-specific metrics, operations teams can gain deep insights into all aspects of MQTT services, ensuring high availability and excellent performance.
