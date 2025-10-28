# MQTT 专用指标

RobustMQ 提供了全面的 MQTT 协议专用监控指标，涵盖数据包处理、连接管理、认证授权、消息发布、会话管理等各个方面。这些指标帮助运维人员深入了解 MQTT 服务的运行状态和性能表现。

## 数据包指标 (Packets)

### 接收数据包统计

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_packets_received` | Gauge | `network` | MQTT 数据包接收总数 |
| `mqtt_packets_connect_received` | Gauge | `network` | CONNECT 数据包接收数 |
| `mqtt_packets_publish_received` | Gauge | `network` | PUBLISH 数据包接收数 |
| `mqtt_packets_connack_received` | Gauge | `network` | CONNACK 数据包接收数 |
| `mqtt_packets_puback_received` | Gauge | `network` | PUBACK 数据包接收数 |
| `mqtt_packets_pubrec_received` | Gauge | `network` | PUBREC 数据包接收数 |
| `mqtt_packets_pubrel_received` | Gauge | `network` | PUBREL 数据包接收数 |
| `mqtt_packets_pubcomp_received` | Gauge | `network` | PUBCOMP 数据包接收数 |
| `mqtt_packets_subscribe_received` | Gauge | `network` | SUBSCRIBE 数据包接收数 |
| `mqtt_packets_unsubscribe_received` | Gauge | `network` | UNSUBSCRIBE 数据包接收数 |
| `mqtt_packets_pingreq_received` | Gauge | `network` | PINGREQ 数据包接收数 |
| `mqtt_packets_disconnect_received` | Gauge | `network` | DISCONNECT 数据包接收数 |
| `mqtt_packets_auth_received` | Gauge | `network` | AUTH 数据包接收数 |

### 发送数据包统计

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_packets_sent` | Gauge | `network`, `qos` | MQTT 数据包发送总数 |
| `mqtt_packets_connack_sent` | Gauge | `network`, `qos` | CONNACK 数据包发送数 |
| `mqtt_packets_publish_sent` | Gauge | `network`, `qos` | PUBLISH 数据包发送数 |
| `mqtt_packets_puback_sent` | Gauge | `network`, `qos` | PUBACK 数据包发送数 |
| `mqtt_packets_pubrec_sent` | Gauge | `network`, `qos` | PUBREC 数据包发送数 |
| `mqtt_packets_pubrel_sent` | Gauge | `network`, `qos` | PUBREL 数据包发送数 |
| `mqtt_packets_pubcomp_sent` | Gauge | `network`, `qos` | PUBCOMP 数据包发送数 |
| `mqtt_packets_suback_sent` | Gauge | `network`, `qos` | SUBACK 数据包发送数 |
| `mqtt_packets_unsuback_sent` | Gauge | `network`, `qos` | UNSUBACK 数据包发送数 |
| `mqtt_packets_pingresp_sent` | Gauge | `network`, `qos` | PINGRESP 数据包发送数 |
| `mqtt_packets_disconnect_sent` | Gauge | `network`, `qos` | DISCONNECT 数据包发送数 |

### 字节统计

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_bytes_received` | Gauge | `network` | MQTT 接收字节总数 |
| `mqtt_bytes_sent` | Gauge | `network`, `qos` | MQTT 发送字节总数 |

### 错误统计

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_packets_received_error` | Gauge | `network` | MQTT 错误数据包接收数 |
| `mqtt_packets_connack_auth_error` | Gauge | `network` | CONNACK 认证错误数据包数 |
| `mqtt_packets_connack_error` | Gauge | `network` | CONNACK 错误数据包数 |

### 保留消息和丢弃消息

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_retain_packets_received` | Gauge | `qos` | 保留消息接收数 |
| `mqtt_retain_packets_sent` | Gauge | `qos` | 保留消息发送数 |
| `mqtt_messages_dropped_no_subscribers` | Gauge | `qos` | 因无订阅者丢弃的消息数 |

**标签说明：**
- `network`: 网络类型（tcp, websocket, quic）
- `qos`: QoS 级别（0, 1, 2, -1 表示非 PUBLISH 包）

## 连接事件指标 (Events)

### 连接统计

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_connection_success` | Counter | - | MQTT 连接成功次数 |
| `mqtt_connection_failed` | Counter | - | MQTT 连接失败次数 |
| `mqtt_disconnect_success` | Counter | - | MQTT 主动断开成功次数 |
| `mqtt_connection_expired` | Counter | - | MQTT 连接过期断开次数 |

### 订阅统计

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_subscribe_success` | Counter | - | MQTT 成功订阅主题次数 |
| `mqtt_subscribe_failed` | Counter | - | MQTT 订阅失败次数 |
| `mqtt_unsubscribe_success` | Counter | - | MQTT 成功取消订阅主题次数 |

### 客户端连接计数

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `client_connections` | Counter | `client_id` | 特定客户端的连接次数 |

## 认证授权指标 (Auth)

### 认证统计

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_auth_success` | Counter | - | MQTT 登录认证成功次数 |
| `mqtt_auth_failed` | Counter | - | MQTT 登录认证失败次数 |

### 授权统计

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_acl_success` | Counter | - | MQTT ACL 权限检查通过次数 |
| `mqtt_acl_failed` | Counter | - | MQTT ACL 权限检查失败次数 |

### 安全防护

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_blacklist_blocked` | Counter | - | MQTT 黑名单拦截次数 |

## 会话管理指标 (Session)

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_session_created` | Counter | - | MQTT 会话创建次数 |
| `mqtt_session_deleted` | Counter | - | MQTT 会话删除次数 |

## 实时统计指标 (Statistics)

### 当前状态统计

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_connections_count` | Gauge | - | 当前 MQTT 连接数量 |
| `mqtt_sessions_count` | Gauge | - | 当前 MQTT 会话数量 |
| `mqtt_topics_count` | Gauge | - | 当前主题数量 |
| `mqtt_subscribers_count` | Gauge | - | 当前订阅者数量 |
| `mqtt_subscriptions_shared_count` | Gauge | - | 当前共享订阅数量 |
| `mqtt_retained_count` | Gauge | - | 当前保留消息数量 |

### 延时队列监控

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_delay_queue_total_capacity` | Gauge | `shard_no` | 延时队列总容量 |
| `mqtt_delay_queue_used_capacity` | Gauge | `shard_no` | 延时队列已使用容量 |
| `mqtt_delay_queue_remaining_capacity` | Gauge | `shard_no` | 延时队列剩余容量 |

**标签说明：**
- `shard_no`: 分片编号

## 消息发布指标 (Publish)

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_messages_delayed` | Gauge | - | 存储的延迟发布消息数量 |
| `mqtt_messages_received` | Gauge | - | 接收来自客户端的消息数量 |
| `mqtt_messages_sent` | Gauge | - | 发送给客户端的消息数量 |

## 性能指标 (Time)

### 处理耗时

| 指标名称 | 类型 | 标签 | 描述 |
|---------|------|------|------|
| `mqtt_packet_process_duration_ms` | Histogram | `network`, `packet` | MQTT 数据包处理耗时（毫秒） |
| `mqtt_packet_send_duration_ms` | Histogram | `network`, `packet` | MQTT 数据包发送耗时（毫秒） |

**标签说明：**
- `network`: 网络类型（tcp, websocket, quic）
- `packet`: 数据包类型（CONNECT, PUBLISH, SUBSCRIBE 等）

## 使用示例

### 记录指标

```rust
use common_metrics::mqtt::*;

// 记录连接成功
event::record_mqtt_connection_success();

// 记录认证失败
auth::record_mqtt_auth_failed();

// 记录数据包处理耗时
time::record_mqtt_packet_process_duration(
    NetworkConnectionType::Tcp, 
    "PUBLISH".to_string(), 
    15.5
);

// 设置当前连接数
statistics::record_mqtt_connections_set(100);

// 增加保留消息计数
statistics::record_mqtt_retained_inc();
```

### Prometheus 查询示例

```text
# MQTT 连接成功率
mqtt_connection_success / (mqtt_connection_success + mqtt_connection_failed) * 100

# MQTT 数据包处理平均耗时
rate(mqtt_packet_process_duration_ms_sum[5m]) / rate(mqtt_packet_process_duration_ms_count[5m])

# 当前活跃连接数
mqtt_connections_count

# PUBLISH 数据包 QPS
rate(mqtt_packets_publish_received[5m])

# 认证失败率
rate(mqtt_auth_failed[5m]) / rate(mqtt_auth_success[5m] + mqtt_auth_failed[5m]) * 100

# 延时队列使用率
mqtt_delay_queue_used_capacity / mqtt_delay_queue_total_capacity * 100
```

### Grafana 仪表板配置

#### 连接概览面板
```json
{
  "title": "MQTT 连接概览",
  "targets": [
    {
      "expr": "mqtt_connections_count",
      "legendFormat": "当前连接数"
    },
    {
      "expr": "rate(mqtt_connection_success[5m])",
      "legendFormat": "连接成功率/秒"
    },
    {
      "expr": "rate(mqtt_connection_failed[5m])",
      "legendFormat": "连接失败率/秒"
    }
  ]
}
```

#### 消息吞吐量面板
```json
{
  "title": "MQTT 消息吞吐量",
  "targets": [
    {
      "expr": "rate(mqtt_packets_publish_received[5m])",
      "legendFormat": "接收 PUBLISH/秒"
    },
    {
      "expr": "rate(mqtt_packets_publish_sent[5m])",
      "legendFormat": "发送 PUBLISH/秒"
    },
    {
      "expr": "rate(mqtt_bytes_received[5m])",
      "legendFormat": "接收字节/秒"
    },
    {
      "expr": "rate(mqtt_bytes_sent[5m])",
      "legendFormat": "发送字节/秒"
    }
  ]
}
```

#### 性能监控面板
```json
{
  "title": "MQTT 性能监控",
  "targets": [
    {
      "expr": "histogram_quantile(0.95, rate(mqtt_packet_process_duration_ms_bucket[5m]))",
      "legendFormat": "P95 处理耗时"
    },
    {
      "expr": "histogram_quantile(0.99, rate(mqtt_packet_process_duration_ms_bucket[5m]))",
      "legendFormat": "P99 处理耗时"
    }
  ]
}
```

## 告警规则

### 连接告警
```yaml
- alert: MQTTHighConnectionFailureRate
  expr: rate(mqtt_connection_failed[5m]) / rate(mqtt_connection_success[5m] + mqtt_connection_failed[5m]) > 0.1
  for: 2m
  labels:
    severity: warning
  annotations:
    summary: "MQTT 连接失败率过高"
    description: "MQTT 连接失败率超过 10%"

- alert: MQTTTooManyConnections
  expr: mqtt_connections_count > 10000
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "MQTT 连接数过多"
    description: "当前 MQTT 连接数: {{ $value }}"
```

### 性能告警
```yaml
- alert: MQTTHighLatency
  expr: histogram_quantile(0.95, rate(mqtt_packet_process_duration_ms_bucket[5m])) > 100
  for: 3m
  labels:
    severity: warning
  annotations:
    summary: "MQTT 处理延迟过高"
    description: "95% 的 MQTT 数据包处理耗时超过 100ms"

- alert: MQTTHighErrorRate
  expr: rate(mqtt_packets_received_error[5m]) / rate(mqtt_packets_received[5m]) > 0.05
  for: 2m
  labels:
    severity: critical
  annotations:
    summary: "MQTT 错误率过高"
    description: "MQTT 数据包错误率超过 5%"
```

### 认证告警
```yaml
- alert: MQTTHighAuthFailureRate
  expr: rate(mqtt_auth_failed[5m]) / rate(mqtt_auth_success[5m] + mqtt_auth_failed[5m]) > 0.2
  for: 1m
  labels:
    severity: warning
  annotations:
    summary: "MQTT 认证失败率过高"
    description: "MQTT 认证失败率超过 20%"

- alert: MQTTFrequentBlacklistBlocks
  expr: rate(mqtt_blacklist_blocked[5m]) > 10
  for: 1m
  labels:
    severity: warning
  annotations:
    summary: "MQTT 黑名单拦截频繁"
    description: "黑名单拦截频率: {{ $value }}/秒"
```

### 容量告警
```yaml
- alert: MQTTDelayQueueFull
  expr: mqtt_delay_queue_used_capacity / mqtt_delay_queue_total_capacity > 0.9
  for: 2m
  labels:
    severity: critical
  annotations:
    summary: "MQTT 延时队列接近满载"
    description: "分片 {{ $labels.shard_no }} 延时队列使用率: {{ $value | humanizePercentage }}"

- alert: MQTTTooManyRetainedMessages
  expr: mqtt_retained_count > 100000
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "MQTT 保留消息过多"
    description: "当前保留消息数量: {{ $value }}"
```

通过这些 MQTT 专用指标，运维团队可以深入了解 MQTT 服务的各个方面，确保服务的高可用性和优异性能。
