# MQTT 专用指标

RobustMQ 通过 Prometheus `/metrics` 端点暴露 MQTT 协议、连接、消息、延迟队列和 Connector 指标。下表使用 Prometheus 实际暴露的指标名。Rust 中注册 Counter 时不写 `_total`，`prometheus-client` 会在暴露时自动追加该后缀。

## 协议包指标

### 接收数据包

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_packets_received_total` | Counter | `network` | MQTT 接收包总数 |
| `mqtt_packets_connect_received_total` | Counter | `network` | CONNECT 包接收数 |
| `mqtt_packets_publish_received_total` | Counter | `network` | PUBLISH 包接收数 |
| `mqtt_packets_connack_received_total` | Counter | `network` | CONNACK 包接收数 |
| `mqtt_packets_puback_received_total` | Counter | `network` | PUBACK 包接收数 |
| `mqtt_packets_pubrec_received_total` | Counter | `network` | PUBREC 包接收数 |
| `mqtt_packets_pubrel_received_total` | Counter | `network` | PUBREL 包接收数 |
| `mqtt_packets_pubcomp_received_total` | Counter | `network` | PUBCOMP 包接收数 |
| `mqtt_packets_subscribe_received_total` | Counter | `network` | SUBSCRIBE 包接收数 |
| `mqtt_packets_unsubscribe_received_total` | Counter | `network` | UNSUBSCRIBE 包接收数 |
| `mqtt_packets_pingreq_received_total` | Counter | `network` | PINGREQ 包接收数 |
| `mqtt_packets_disconnect_received_total` | Counter | `network` | DISCONNECT 包接收数 |
| `mqtt_packets_auth_received_total` | Counter | `network` | AUTH 包接收数 |
| `mqtt_packets_received_error_total` | Counter | `network` | 接收或解码时被拒绝的数据包数 |
| `mqtt_packets_connack_auth_error_total` | Counter | `network` | 授权失败的 CONNACK 包数 |
| `mqtt_packets_connack_error_total` | Counter | `network` | 返回错误结果的 CONNACK 包数 |
| `mqtt_bytes_received_total` | Counter | `network` | MQTT 接收字节总数 |

### 发送数据包

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_packets_sent_total` | Counter | `network`, `qos` | MQTT 发送包总数 |
| `mqtt_packets_connack_sent_total` | Counter | `network`, `qos` | CONNACK 包发送数 |
| `mqtt_packets_publish_sent_total` | Counter | `network`, `qos` | PUBLISH 包发送数 |
| `mqtt_packets_puback_sent_total` | Counter | `network`, `qos` | PUBACK 包发送数 |
| `mqtt_packets_pubrec_sent_total` | Counter | `network`, `qos` | PUBREC 包发送数 |
| `mqtt_packets_pubrel_sent_total` | Counter | `network`, `qos` | PUBREL 包发送数 |
| `mqtt_packets_pubcomp_sent_total` | Counter | `network`, `qos` | PUBCOMP 包发送数 |
| `mqtt_packets_suback_sent_total` | Counter | `network`, `qos` | SUBACK 包发送数 |
| `mqtt_packets_unsuback_sent_total` | Counter | `network`, `qos` | UNSUBACK 包发送数 |
| `mqtt_packets_pingresp_sent_total` | Counter | `network`, `qos` | PINGRESP 包发送数 |
| `mqtt_packets_disconnect_sent_total` | Counter | `network`, `qos` | DISCONNECT 包发送数 |
| `mqtt_packets_auth_sent_total` | Counter | `network`, `qos` | AUTH 包发送数；AUTH 不是 PUBLISH，因此 `qos` 为 `-1` |
| `mqtt_bytes_sent_total` | Counter | `network`, `qos` | MQTT 发送字节总数 |
| `mqtt_retain_packets_received_total` | Counter | `qos` | 保留消息接收数 |
| `mqtt_retain_packets_sent_total` | Counter | `qos` | 保留消息发送数 |

### 跨网络汇总

以下无标签 Counter 汇总所有网络类型的协议包和字节数。

| 指标名称 | 类型 | 描述 |
|---------|------|------|
| `mqtt_total_bytes_received_total` | Counter | 所有网络类型接收的 MQTT 字节数 |
| `mqtt_total_bytes_sent_total` | Counter | 所有网络类型发送的 MQTT 字节数 |
| `mqtt_total_packets_received_total` | Counter | 所有网络类型接收的 MQTT 包数 |
| `mqtt_total_packets_sent_total` | Counter | 所有网络类型发送的 MQTT 包数 |
| `mqtt_total_packets_connect_total` | Counter | 接收的 CONNECT 包数 |
| `mqtt_total_packets_connack_total` | Counter | 发送的 CONNACK 包数 |
| `mqtt_total_packets_publish_received_total` | Counter | 接收的 PUBLISH 包数 |
| `mqtt_total_packets_publish_sent_total` | Counter | 发送的 PUBLISH 包数 |
| `mqtt_total_packets_puback_received_total` | Counter | 接收的 PUBACK 包数 |
| `mqtt_total_packets_puback_sent_total` | Counter | 发送的 PUBACK 包数 |
| `mqtt_total_packets_pubrec_received_total` | Counter | 接收的 PUBREC 包数 |
| `mqtt_total_packets_pubrec_sent_total` | Counter | 发送的 PUBREC 包数 |
| `mqtt_total_packets_pubrel_received_total` | Counter | 接收的 PUBREL 包数 |
| `mqtt_total_packets_pubrel_sent_total` | Counter | 发送的 PUBREL 包数 |
| `mqtt_total_packets_pubcomp_received_total` | Counter | 接收的 PUBCOMP 包数 |
| `mqtt_total_packets_pubcomp_sent_total` | Counter | 发送的 PUBCOMP 包数 |
| `mqtt_total_packets_subscribe_total` | Counter | 接收的 SUBSCRIBE 包数 |
| `mqtt_total_packets_suback_total` | Counter | 发送的 SUBACK 包数 |
| `mqtt_total_packets_unsubscribe_total` | Counter | 接收的 UNSUBSCRIBE 包数 |
| `mqtt_total_packets_unsuback_total` | Counter | 发送的 UNSUBACK 包数 |
| `mqtt_total_packets_pingreq_total` | Counter | 接收的 PINGREQ 包数 |
| `mqtt_total_packets_pingresp_total` | Counter | 发送的 PINGRESP 包数 |
| `mqtt_total_packets_disconnect_received_total` | Counter | 接收的 DISCONNECT 包数 |
| `mqtt_total_packets_disconnect_sent_total` | Counter | 发送的 DISCONNECT 包数 |
| `mqtt_total_packets_auth_total` | Counter | 接收的 AUTH 包数 |
| `mqtt_total_packets_auth_sent_total` | Counter | 发送的 AUTH 包数 |

### 协议包耗时

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_packet_process_duration_ms` | Histogram | `network`, `packet` | MQTT 协议包处理耗时（毫秒） |
| `mqtt_packet_send_duration_ms` | Histogram | `network`, `packet` | MQTT 协议包发送耗时（毫秒） |

`network` 的取值为 `Tcp`、`Tls`、`Websocket`、`Websockets`、`Quic`；无法获取连接类型时为 `Unknown`。PUBLISH 包的 `qos` 为 `0`、`1` 或 `2`，非 PUBLISH 包为 `-1`。`packet` 使用小写值，例如 `connect`、`publish` 和 `subscribe`。

## 连接与认证指标

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `client_connections_total` | Counter | `tenant`, `client_id` | 客户端连接尝试次数，不论结果如何 |
| `mqtt_connection_success_total` | Counter | — | MQTT 连接成功次数 |
| `mqtt_connection_failed_total` | Counter | — | MQTT 连接失败次数 |
| `mqtt_disconnect_success_total` | Counter | — | MQTT 断开成功次数 |
| `mqtt_connection_expired_total` | Counter | — | MQTT 连接过期次数 |
| `mqtt_auth_success_total` | Counter | — | MQTT 认证成功次数 |
| `mqtt_auth_failed_total` | Counter | — | MQTT 认证失败次数 |
| `mqtt_acl_success_total` | Counter | — | MQTT ACL 检查通过次数 |
| `mqtt_acl_failed_total` | Counter | — | MQTT ACL 检查失败次数 |
| `mqtt_blacklist_blocked_total` | Counter | — | MQTT 黑名单拦截次数 |
| `mqtt_subscribe_success_total` | Counter | — | 订阅成功次数 |
| `mqtt_subscribe_failed_total` | Counter | — | 订阅失败次数 |
| `mqtt_unsubscribe_success_total` | Counter | — | 取消订阅成功次数 |

## 会话与消息指标

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_session_created_total` | Counter | — | MQTT 会话创建数 |
| `mqtt_session_deleted_total` | Counter | — | MQTT 会话删除数 |
| `session_messages_in_total` | Counter | `tenant`, `client_id` | 按会话统计的接收消息数 |
| `session_messages_out_total` | Counter | `tenant`, `client_id` | 按会话统计的发送消息数 |
| `connection_messages_in_total` | Counter | `connection_id` | 按连接统计的接收消息数 |
| `connection_messages_out_total` | Counter | `connection_id` | 按连接统计的发送消息数 |
| `mqtt_messages_received_total` | Counter | — | 从客户端接收的消息数 |
| `mqtt_messages_sent_total` | Counter | — | 向客户端发送的消息数 |
| `mqtt_message_bytes_received_total` | Counter | — | 从客户端接收的消息字节数 |
| `mqtt_message_bytes_sent_total` | Counter | — | 向客户端发送的消息字节数 |
| `mqtt_messages_delayed_total` | Counter | — | 延迟发布消息数 |
| `mqtt_messages_dropped_no_subscribers_total` | Counter | — | 因无订阅者丢弃的消息数 |

## 资源统计

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_connections_count` | Gauge | — | 当前 MQTT 连接数 |
| `mqtt_sessions_count` | Gauge | — | 当前 MQTT 会话数 |
| `mqtt_topics_count` | Gauge | — | 当前 MQTT 主题数 |
| `mqtt_subscribers_count` | Gauge | — | 当前 MQTT 订阅者数 |
| `mqtt_subscriptions_exclusive_count` | Gauge | — | 当前独占订阅数 |
| `mqtt_subscriptions_shared_count` | Gauge | — | 当前共享订阅数 |
| `mqtt_subscriptions_shared_group_count` | Gauge | — | 当前共享订阅组数 |
| `mqtt_retained_count` | Gauge | — | 当前保留消息数 |
| `mqtt_delay_queue_total_capacity` | Gauge | `shard_no` | 延迟队列总容量 |
| `mqtt_delay_queue_used_capacity` | Gauge | `shard_no` | 延迟队列已用容量 |
| `mqtt_delay_queue_remaining_capacity` | Gauge | `shard_no` | 延迟队列剩余容量 |

## Topic 与订阅指标

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `topic_messages_written_total` | Counter | `tenant`, `topic` | 写入 Topic 的消息数 |
| `topic_bytes_written_total` | Counter | `tenant`, `topic` | 写入 Topic 的字节数 |
| `topic_messages_sent_total` | Counter | `tenant`, `topic` | 从 Topic 发送的消息数 |
| `topic_bytes_sent_total` | Counter | `tenant`, `topic` | 从 Topic 发送的字节数 |
| `subscribe_messages_sent_total` | Counter | `tenant`, `client_id`, `path`, `status` | 向订阅路径发送的消息数 |
| `subscribe_topic_messages_sent_total` | Counter | `tenant`, `client_id`, `path`, `topic_name`, `status` | 向订阅路径和主题发送的消息数 |
| `subscribe_bytes_sent_total` | Counter | `tenant`, `client_id`, `path`, `status` | 向订阅路径发送的字节数 |
| `subscribe_topic_bytes_sent_total` | Counter | `tenant`, `client_id`, `path`, `topic_name`, `status` | 向订阅路径和主题发送的字节数 |

订阅 `status` 的取值为 `success` 和 `failed`。

## 延迟消息与 Connector 指标

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `delay_msg_enqueue_total` | Counter | — | 延迟消息入队数 |
| `delay_msg_deliver_total` | Counter | — | 延迟消息投递数 |
| `delay_msg_deliver_fail_total` | Counter | — | 延迟消息投递失败数 |
| `delay_msg_recover_total` | Counter | — | 启动时恢复的延迟消息数 |
| `delay_msg_recover_expired_total` | Counter | — | 恢复时发现的过期消息数 |
| `delay_message_retry_total` | Counter | `status` | 延迟消息重试次数，`status` 为 `success` 或 `failed` |
| `delay_message_retry_count_total` | Counter | `attempt` | 按数字重试次数统计 |
| `delay_message_reenqueued_total` | Counter | — | 投递失败后重新入队的消息数 |
| `delay_message_dead_letter_total` | Counter | — | 发送到死信队列的消息数 |
| `delay_msg_enqueue_duration_ms` | Histogram | — | 延迟消息入队耗时 |
| `delay_msg_deliver_duration_ms` | Histogram | — | 延迟消息投递耗时 |
| `delay_task_created_total` | Counter | — | 创建的延迟任务数 |
| `delay_task_executed_total` | Counter | `task_type` | 成功执行的延迟任务数 |
| `delay_task_execute_failed_total` | Counter | `task_type` | 延迟任务执行失败数 |
| `delay_task_schedule_latency_seconds` | Histogram | `task_type` | 实际执行时间与目标时间的差值 |
| `mqtt_connector_messages_sent_success_total` | Counter | `tenant`, `connector_type`, `connector_name` | Connector 成功发送的消息数 |
| `mqtt_connector_messages_sent_failure_total` | Counter | `tenant`, `connector_type`, `connector_name` | Connector 发送失败的消息数 |
| `mqtt_connector_send_duration_ms` | Histogram | `tenant`, `connector_type`, `connector_name` | Connector 发送耗时 |
| `mqtt_connector_retry_total` | Counter | `tenant`, `connector_type`, `connector_name`, `strategy` | Connector 重试次数 |
| `mqtt_connector_messages_discarded_total` | Counter | `tenant`, `connector_type`, `connector_name`, `strategy` | 按终态策略丢弃的消息数 |
| `mqtt_connector_dlq_messages_total` | Counter | `tenant`, `connector_type`, `connector_name`, `result` | 按结果统计的死信写入尝试数 |
| `mqtt_connector_offset_commit_failure_total` | Counter | `tenant`, `connector_type`, `connector_name` | Connector offset 提交失败数 |
| `mqtt_connector_source_read_failure_total` | Counter | `tenant`, `connector_type`, `connector_name` | Connector 源读取失败数 |
| `mqtt_connector_up` | Gauge | `tenant`, `connector_type`, `connector_name` | Connector 状态：1 运行，0 停止 |
| `mqtt_connector_messages_sent_success_agg_total` | Counter | — | 所有 Connector 成功发送总数 |
| `mqtt_connector_messages_sent_failure_agg_total` | Counter | — | 所有 Connector 发送失败总数 |
| `mqtt_connector_send_duration_ms_agg` | Histogram | — | 所有 Connector 的聚合发送耗时 |

## 使用示例

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

PromQL 示例：

```promql
rate(mqtt_packets_publish_received_total[5m])
rate(mqtt_packets_publish_sent_total[5m])
rate(mqtt_auth_failed_total[5m]) /
  (rate(mqtt_auth_success_total[5m]) + rate(mqtt_auth_failed_total[5m]))
histogram_quantile(0.95, sum by (le) (rate(mqtt_packet_process_duration_ms_bucket[5m])))
mqtt_delay_queue_used_capacity / mqtt_delay_queue_total_capacity
```

## Grafana 仪表板配置

仓库提供了可直接导入的仪表板文件 [`grafana/robustmq-broker.json`](../../../grafana/robustmq-broker.json)。配置和导入步骤请参阅 [Grafana 配置指南](Grafana配置指南.md)。下面的示例是面板查询片段，不是完整的仪表板 JSON 文件。

### 连接概览面板

```json
{
  "title": "MQTT 连接概览",
  "targets": [
    {
      "expr": "mqtt_connections_count",
      "legendFormat": "当前连接数"
    },
    {
      "expr": "rate(mqtt_connection_success_total[5m])",
      "legendFormat": "连接成功率/秒"
    },
    {
      "expr": "rate(mqtt_connection_failed_total[5m])",
      "legendFormat": "连接失败率/秒"
    }
  ]
}
```

### 消息吞吐量面板

协议包和字节 Counter 都带有 `network` 标签。`sum by (network)` 会合并发送方向的 QoS 序列，同时在图例中保留网络类型。

```json
{
  "title": "MQTT 消息吞吐量",
  "targets": [
    {
      "expr": "sum by (network) (rate(mqtt_packets_publish_received_total[5m]))",
      "legendFormat": "接收 PUBLISH/秒 ({{network}})"
    },
    {
      "expr": "sum by (network) (rate(mqtt_packets_publish_sent_total[5m]))",
      "legendFormat": "发送 PUBLISH/秒 ({{network}})"
    },
    {
      "expr": "sum by (network) (rate(mqtt_bytes_received_total[5m]))",
      "legendFormat": "接收字节/秒 ({{network}})"
    },
    {
      "expr": "sum by (network) (rate(mqtt_bytes_sent_total[5m]))",
      "legendFormat": "发送字节/秒 ({{network}})"
    }
  ]
}
```

### 性能监控面板

计算分位数前必须按 `le` 聚合 Histogram bucket。

```json
{
  "title": "MQTT 性能监控",
  "targets": [
    {
      "expr": "histogram_quantile(0.95, sum by (le) (rate(mqtt_packet_process_duration_ms_bucket[5m])))",
      "legendFormat": "P95 处理耗时（毫秒）"
    },
    {
      "expr": "histogram_quantile(0.99, sum by (le) (rate(mqtt_packet_process_duration_ms_bucket[5m])))",
      "legendFormat": "P99 处理耗时（毫秒）"
    }
  ]
}
```

## MQTT 告警规则

以下 Prometheus 规则示例使用当前实现实际暴露的指标名。仓库还提供了更完整的告警规则文件 [`grafana/robustmq-alerts.yml`](../../../grafana/robustmq-alerts.yml)；配置方法请参阅 [Grafana 配置指南](Grafana配置指南.md)。这些片段用于说明 MQTT 专用告警条件，并不替代完整规则文件。

### 连接告警

```yaml
- alert: RobustMQHighConnectionFailureRate
  expr: (sum(rate(mqtt_connection_failed_total[5m])) / clamp_min(sum(rate(mqtt_connection_success_total[5m])) + sum(rate(mqtt_connection_failed_total[5m])), 1e-9)) > 0.1
  for: 2m
  labels:
    severity: warning
  annotations:
    summary: "MQTT 连接失败率过高"
    description: "MQTT 连接失败占比为 {{ $value | humanizePercentage }}。"

- alert: RobustMQHighConnectionCount
  expr: mqtt_connections_count > 5000
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "MQTT 连接数过高"
    description: "当前 MQTT 连接数：{{ $value }}"
```

### 性能与错误告警

Histogram bucket 按 `le` 聚合，并保留可用的 `network` 和 `packet` 标签，以便告警指出受影响的传输类型和数据包类型。

```yaml
- alert: RobustMQHighMqttPacketProcessingLatency
  expr: histogram_quantile(0.95, sum by (le, network, packet) (rate(mqtt_packet_process_duration_ms_bucket[5m]))) > 100
  for: 3m
  labels:
    severity: warning
  annotations:
    summary: "MQTT 数据包处理延迟过高"
    description: "{{ $labels.network }} 上 {{ $labels.packet }} 的 P95 延迟为 {{ $value }}ms。"

- alert: RobustMQPacketErrors
  expr: sum by (network) (rate(mqtt_packets_received_error_total[5m])) > 1
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "MQTT 数据包错误率过高"
    description: "{{ $labels.network }} 每秒发生 {{ $value }} 次数据包错误。"
```

### 认证与容量告警

```yaml
- alert: RobustMQHighMqttAuthenticationFailureRatio
  expr: (sum(rate(mqtt_auth_failed_total[5m])) / clamp_min(sum(rate(mqtt_auth_success_total[5m])) + sum(rate(mqtt_auth_failed_total[5m])), 1e-9)) > 0.2
  for: 1m
  labels:
    severity: warning
  annotations:
    summary: "MQTT 认证失败占比过高"
    description: "认证失败占比为 {{ $value | humanizePercentage }}。"

- alert: RobustMQDelayQueueNearFull
  expr: (mqtt_delay_queue_total_capacity > 0) and ((mqtt_delay_queue_used_capacity / mqtt_delay_queue_total_capacity) > 0.9)
  for: 2m
  labels:
    severity: critical
  annotations:
    summary: "MQTT 延迟队列接近满载"
    description: "分片 {{ $labels.shard_no }} 的容量使用率为 {{ $value | humanizePercentage }}。"

- alert: RobustMQTooManyRetainedMessages
  expr: mqtt_retained_count > 100000
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "MQTT 保留消息过多"
    description: "当前保留消息数：{{ $value }}"
```
