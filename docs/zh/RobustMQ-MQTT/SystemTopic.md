# 系统主题

## 概述

RobustMQ MQTT Broker 通过 `$SYS/` 系统主题发布运行状态、统计数据和事件。客户端可像订阅普通 MQTT 主题一样订阅这些系统主题。

系统主题由 Broker 定期发布（默认每 **60 秒**，可通过 `mqtt_system_topic.interval_ms` 调整），部分主题为事件触发即时发布（如客户端连接/断开）。

> 注意：系统主题以 `$SYS/` 开头，默认情况下非管理员客户端无法订阅，请按需配置 ACL。

---

## 主题设计（当前版本）

系统主题已改为**固定 Topic 名称**，不再按节点或客户端维度拆分路径：

- 不再使用 `${node}`
- 事件主题不再使用 `${clientid}`
- 节点信息放在 Payload 中（`node_id`、`node_ip`）

这样可以避免 Topic 基数随节点数/客户端数膨胀。

---

## Payload 统一格式

除少量历史兼容数据外，系统主题消息统一采用以下外层结构：

```json
{
  "node_id": 1,
  "node_ip": "127.0.0.1",
  "ts": 1700000000000,
  "value": {}
}
```

字段说明：

- `node_id`: Broker ID
- `node_ip`: Broker IP
- `ts`: 生成时间（毫秒）
- `value`: 业务值（统计值、事件对象、告警对象等）

---

## Broker 基础信息

| 主题 | 说明 |
|------|------|
| `$SYS/brokers` | 当前集群节点列表 |
| `$SYS/brokers/version` | Broker 版本 |
| `$SYS/brokers/uptime` | Broker 运行时长 |
| `$SYS/brokers/datetime` | Broker 当前时间 |
| `$SYS/brokers/sysdescr` | 操作系统信息 |

---

## 统计数据（Stats）

### 连接

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/stats/connections/count` | 当前连接数 |
| `$SYS/brokers/stats/connections/max` | 历史最大连接数（当前实现以会话数近似） |

### Topic

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/stats/topics/count` | 当前 Topic 数量 |
| `$SYS/brokers/stats/topics/max` | 历史最大 Topic 数量 |

### 路由

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/stats/routes/count` | 当前路由数量 |
| `$SYS/brokers/stats/routes/max` | 历史最大路由数量 |

### 订阅

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/stats/subscribers/count` | 当前独占订阅者数量 |
| `$SYS/brokers/stats/subscribers/max` | 历史最大独占订阅者数量 |
| `$SYS/brokers/stats/subscriptions/count` | 当前总订阅数（独占 + 共享） |
| `$SYS/brokers/stats/subscriptions/max` | 历史最大总订阅数 |
| `$SYS/brokers/stats/subscriptions/shared/count` | 当前共享订阅数 |
| `$SYS/brokers/stats/subscriptions/shared/max` | 历史最大共享订阅数 |
| `$SYS/brokers/stats/suboptions/count` | 当前订阅选项总数 |
| `$SYS/brokers/stats/suboptions/max` | 历史最大订阅选项数 |

---

## 指标数据（Metrics）

### 字节

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/metrics/bytes/received` | 累计接收字节数 |
| `$SYS/brokers/metrics/bytes/sent` | 累计发送字节数 |

### 消息

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/metrics/messages/received` | 累计接收消息数 |
| `$SYS/brokers/metrics/messages/sent` | 累计发送消息数 |
| `$SYS/brokers/metrics/messages/dropped` | 累计丢弃消息数 |
| `$SYS/brokers/metrics/messages/retained` | 当前保留消息数 |
| `$SYS/brokers/metrics/messages/expired` | 累计过期消息数 |
| `$SYS/brokers/metrics/messages/forward` | 累计转发消息数 |
| `$SYS/brokers/metrics/messages/qos0/received` | 累计 QoS0 接收消息数 |
| `$SYS/brokers/metrics/messages/qos0/sent` | 累计 QoS0 发送消息数 |
| `$SYS/brokers/metrics/messages/qos1/received` | 累计 QoS1 接收消息数 |
| `$SYS/brokers/metrics/messages/qos1/sent` | 累计 QoS1 发送消息数 |
| `$SYS/brokers/metrics/messages/qos2/received` | 累计 QoS2 接收消息数 |
| `$SYS/brokers/metrics/messages/qos2/sent` | 累计 QoS2 发送消息数 |
| `$SYS/brokers/metrics/messages/qos2/expired` | 累计 QoS2 过期消息数 |
| `$SYS/brokers/metrics/messages/qos2/dropped` | 累计 QoS2 丢弃消息数 |

### 报文（Packets）

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/metrics/packets/received` | 累计接收报文数 |
| `$SYS/brokers/metrics/packets/sent` | 累计发送报文数 |
| `$SYS/brokers/metrics/packets/connect` | 累计 CONNECT 报文数 |
| `$SYS/brokers/metrics/packets/connack` | 累计 CONNACK 报文数 |
| `$SYS/brokers/metrics/packets/publish/received` | 累计接收 PUBLISH 报文数 |
| `$SYS/brokers/metrics/packets/publish/sent` | 累计发送 PUBLISH 报文数 |
| `$SYS/brokers/metrics/packets/puback/received` | 累计接收 PUBACK 报文数 |
| `$SYS/brokers/metrics/packets/puback/sent` | 累计发送 PUBACK 报文数 |
| `$SYS/brokers/metrics/packets/puback/missed` | 累计 PUBACK 超时未确认数 |
| `$SYS/brokers/metrics/packets/pubrec/received` | 累计接收 PUBREC 报文数 |
| `$SYS/brokers/metrics/packets/pubrec/sent` | 累计发送 PUBREC 报文数 |
| `$SYS/brokers/metrics/packets/pubrec/missed` | 累计 PUBREC 超时未确认数 |
| `$SYS/brokers/metrics/packets/pubrel/received` | 累计接收 PUBREL 报文数 |
| `$SYS/brokers/metrics/packets/pubrel/sent` | 累计发送 PUBREL 报文数 |
| `$SYS/brokers/metrics/packets/pubrel/missed` | 累计 PUBREL 超时未确认数 |
| `$SYS/brokers/metrics/packets/pubcomp/received` | 累计接收 PUBCOMP 报文数 |
| `$SYS/brokers/metrics/packets/pubcomp/sent` | 累计发送 PUBCOMP 报文数 |
| `$SYS/brokers/metrics/packets/pubcomp/missed` | 累计 PUBCOMP 超时未确认数 |
| `$SYS/brokers/metrics/packets/subscribe` | 累计 SUBSCRIBE 报文数 |
| `$SYS/brokers/metrics/packets/suback` | 累计 SUBACK 报文数 |
| `$SYS/brokers/metrics/packets/unsubscribe` | 累计 UNSUBSCRIBE 报文数 |
| `$SYS/brokers/metrics/packets/unsuback` | 累计 UNSUBACK 报文数 |
| `$SYS/brokers/metrics/packets/pingreq` | 累计 PINGREQ 报文数 |
| `$SYS/brokers/metrics/packets/pingresp` | 累计 PINGRESP 报文数 |
| `$SYS/brokers/metrics/packets/disconnect/received` | 累计接收 DISCONNECT 报文数 |
| `$SYS/brokers/metrics/packets/disconnect/sent` | 累计发送 DISCONNECT 报文数 |
| `$SYS/brokers/metrics/packets/auth` | 累计 AUTH 报文数 |

---

## 客户端事件

以下主题在事件发生时即时触发：

| 主题 | 触发时机 |
|------|----------|
| `$SYS/brokers/clients/connected` | 客户端连接成功 |
| `$SYS/brokers/clients/disconnected` | 客户端断开连接 |
| `$SYS/brokers/clients/subscribed` | 客户端订阅成功 |
| `$SYS/brokers/clients/unsubscribed` | 客户端取消订阅 |

### connected 消息示例

```json
{
  "node_id": 1,
  "node_ip": "127.0.0.1",
  "ts": 1700000000000,
  "value": {
    "username": "user1",
    "ts": 1700000000000,
    "sockport": 54321,
    "proto_ver": 4,
    "proto_name": "MQTT",
    "keepalive": 60,
    "ipaddress": "192.168.1.100",
    "expiry_interval": 0,
    "connected_at": 1700000000000,
    "connack": 0,
    "clientid": "my-client-001",
    "clean_start": true
  }
}
```

---

## 系统告警

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/alarms/alert` | 新告警触发时发布 |
| `$SYS/brokers/alarms/clear` | 告警恢复/清除时发布 |

告警消息同样封装在 `value` 中，更多字段说明见 [系统告警](./SystemAlarm.md)。

---

## 订阅示例

```bash
# 订阅所有系统主题
mqttx sub -t '$SYS/#' -h 127.0.0.1 -p 1883

# 订阅所有统计数据
mqttx sub -t '$SYS/brokers/stats/#' -h 127.0.0.1 -p 1883

# 订阅所有客户端连接事件（跨节点汇总）
mqttx sub -t '$SYS/brokers/clients/connected' -h 127.0.0.1 -p 1883
```
