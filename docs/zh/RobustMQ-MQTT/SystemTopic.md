# 系统主题

## 概述

RobustMQ MQTT Broker 通过 `$SYS/` 主题发布运行状态、聚合统计、客户端
事件和系统告警。这些主题和普通 MQTT 主题一样，受 Broker 系统主题 ACL
规则控制。

周期性报告在系统主题监控线程启动后发送，并按配置的间隔重复发送（默认
为 60 秒，可通过 `mqtt_runtime.system_monitor.system_topic_interval_ms`
调整）。客户端事件发生后进入事件流并写入；告警检查按 60 秒间隔执行。

## 主题和消息格式

当前实现使用固定的主题名称，不会把节点 ID 或客户端 ID 拼接到主题路径
中。报告来源节点通过统一的消息封装提供。`node_ip` 优先使用配置中的
`broker_ip`，未配置时回退到本机 IP 检测结果：

```json
{
  "node_id": 1,
  "node_ip": "127.0.0.1",
  "ts": 1749774914000,
  "value": {
    "name": "HighMemoryUsage",
    "message": "HighMemoryUsage is 85.2%, but config is 80%",
    "activate_at": 1749774914,
    "activated": true
  }
}
```

上面的完整消息是系统告警报告示例。其他系统主题会在 `value` 中放置各自的
报告对象；`$SYS/brokers` 是例外，其 `value` 是集群节点列表。

## Broker 基础信息

| 主题 | value 内容 |
|------|------------|
| `$SYS/brokers` | 集群节点列表 |
| `$SYS/brokers/info` | `version`、`uptime`、`datetime` 和 `os_info` |

## 统计数据

每个报告都在 `value` 中携带一个 JSON 对象，当前实现不会再发布独立的
`/count` 或 `/max` 子主题。

| 主题 | value 字段 |
|------|------------|
| `$SYS/brokers/stats/connections` | `count`、`max`、`sessions` |
| `$SYS/brokers/stats/routes` | `count`、`max` |
| `$SYS/brokers/stats/subscriptions` | `count`、`max`、`exclusive_count`、`shared_count`、`shared_group_count` |
| `$SYS/brokers/stats/topics` | `count`、`max` |

## 指标数据

每个报告都在 `value` 中携带一个聚合 JSON 对象，当前实现不会为每个字段
发布独立的子主题。

| 主题 | value 字段 |
|------|------------|
| `$SYS/brokers/metrics/bytes` | `received`、`sent` |
| `$SYS/brokers/metrics/messages` | `received`、`sent`、`expired`、`retained`、`dropped`、`forward`、各 QoS 的接收/发送字段、`qos2_expired`、`qos2_dropped` |
| `$SYS/brokers/metrics/packets` | `received`、`sent`、CONNECT/CONNACK、PUBLISH、PUBACK、PUBREC、PUBREL、PUBCOMP、SUBSCRIBE/SUBACK、UNSUBSCRIBE/UNSUBACK、PINGREQ/PINGRESP、DISCONNECT 和 AUTH 字段 |

报文报告包含 QoS 确认报文族的 `*_missed` 字段。当前实现会将这些字段
报告为 0。

## 客户端事件

客户端连接、断开、订阅和取消订阅事件统一写入 `$SYS/events` 主题。事件
消费者会批量处理（每次最多写入 100 个事件），每个事件都使用统一的消息
封装。客户端 ID 和节点信息位于消息内容中，不位于主题路径中。

## 系统告警

当前实现的告警主题为：

| 主题 | 触发条件 |
|------|----------|
| `$SYS/brokers/alarms/alert` | 上报节点的 Broker 进程 CPU 或内存使用率超过配置的高水位线 |

当前没有 `$SYS/brokers/alarms/clear` 发布逻辑。告警主题在每次 60 秒检查中，
只有 Broker 进程 CPU 或内存使用率超过配置阈值时才发布活动告警
（`activated: true`）。达到或低于阈值时不发布告警消息。

监控逻辑是无状态的，不会发布 `activated: false` 恢复消息。如果活动告警的内部
写入失败，下一次轮询会重试；系统不会跟踪 MQTT 客户端的投递确认。活动告警事件
仍会为现有告警历史 API 和 CLI 持久化。

在 `$SYS/brokers/alarms/alert` 收到的示例消息：

```json
{
  "node_id": 1,
  "node_ip": "127.0.0.1",
  "ts": 1749774914000,
  "value": {
    "name": "HighMemoryUsage",
    "message": "HighMemoryUsage is 85.2%, but config is 80%",
    "activate_at": 1749774914,
    "activated": true
  }
}
```

## 订阅示例

```bash
# 订阅所有系统主题
mqttx sub -t '$SYS/#' -h 127.0.0.1 -p 1883

# 订阅聚合统计数据
mqttx sub -t '$SYS/brokers/stats/#' -h 127.0.0.1 -p 1883

# 订阅所有客户端事件
mqttx sub -t '$SYS/events' -h 127.0.0.1 -p 1883
```
