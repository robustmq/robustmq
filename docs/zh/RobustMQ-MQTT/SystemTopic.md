# 系统主题

## 概述

RobustMQ MQTT Broker 通过 `$SYS/` 系统主题对外发布 Broker 自身的运行状态和统计数据。客户端可以像订阅普通 MQTT 主题一样订阅这些系统主题，从而实时感知 Broker 的连接数、消息吞吐、告警事件等关键指标，无需额外的监控 Agent。

系统主题由 Broker 定期自动发布（默认每 **60 秒**刷新一次，可通过 `mqtt_system_topic.interval_ms` 配置项调整），也有部分主题在事件触发时（如客户端上/下线）即时发布。

> **注意**：系统主题以 `$SYS/` 开头，默认情况下非管理员客户端无法订阅，请确认 ACL 配置正确。

---

## 主题格式

所有系统主题以 `$SYS/brokers/${node}/` 为前缀，其中 `${node}` 会替换为当前节点的 IP 地址。

---

## Broker 基础信息

定期发布，描述 Broker 自身的版本、运行时间等静态信息。

| 主题 | 说明 | 示例值 |
|------|------|--------|
| `$SYS/brokers` | 当前集群的节点列表（JSON） | `[{"node_id":1,...}]` |
| `$SYS/brokers/${node}/version` | Broker 版本号 | `0.4.0` |
| `$SYS/brokers/${node}/uptime` | 运行时长 | `0 days, 2 hours, 15 minutes, 30 seconds` |
| `$SYS/brokers/${node}/datetime` | 当前系统时间 | `2026-02-23 10:00:00` |
| `$SYS/brokers/${node}/sysdescr` | 操作系统信息 | `Ubuntu 22.04` |

---

## 统计数据（Stats）

定期发布，反映当前系统中各类对象的实时数量。

### 连接

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/${node}/stats/connections/count` | 当前在线连接数 |
| `$SYS/brokers/${node}/stats/connections/max` | 历史最大连接数（以会话数近似） |

### Topic

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/${node}/stats/topics/count` | 当前 Topic 数量 |
| `$SYS/brokers/${node}/stats/topics/max` | 历史最大 Topic 数量 |

### 路由

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/${node}/stats/routes/count` | 当前路由数量（等于活跃订阅总数） |
| `$SYS/brokers/${node}/stats/routes/max` | 历史最大路由数量 |

### 订阅

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/${node}/stats/subscribers/count` | 当前独占订阅者数量 |
| `$SYS/brokers/${node}/stats/subscribers/max` | 历史最大独占订阅者数量 |
| `$SYS/brokers/${node}/stats/subscriptions/count` | 当前总订阅数（独占 + 共享） |
| `$SYS/brokers/${node}/stats/subscriptions/max` | 历史最大总订阅数 |
| `$SYS/brokers/${node}/stats/subscriptions/shared/count` | 当前共享订阅数 |
| `$SYS/brokers/${node}/stats/subscriptions/shared/max` | 历史最大共享订阅数 |
| `$SYS/brokers/${node}/stats/suboptions/count` | 当前订阅选项总数 |
| `$SYS/brokers/${node}/stats/suboptions/max` | 历史最大订阅选项数 |

---

## 指标数据（Metrics）

定期发布，反映累计的字节、消息、报文数量。

### 字节

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/${node}/metrics/bytes/received` | 累计接收字节数 |
| `$SYS/brokers/${node}/metrics/bytes/sent` | 累计发送字节数 |

### 消息

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/${node}/metrics/messages/received` | 累计接收消息数 |
| `$SYS/brokers/${node}/metrics/messages/sent` | 累计发送消息数 |
| `$SYS/brokers/${node}/metrics/messages/dropped` | 累计丢弃消息数（无订阅者） |
| `$SYS/brokers/${node}/metrics/messages/retained` | 当前保留消息数 |
| `$SYS/brokers/${node}/metrics/messages/expired` | 累计过期消息数 |
| `$SYS/brokers/${node}/metrics/messages/forward` | 累计转发消息数（集群节点间） |
| `$SYS/brokers/${node}/metrics/messages/qos0/received` | 累计 QoS 0 接收消息数 |
| `$SYS/brokers/${node}/metrics/messages/qos0/sent` | 累计 QoS 0 发送消息数 |
| `$SYS/brokers/${node}/metrics/messages/qos1/received` | 累计 QoS 1 接收消息数 |
| `$SYS/brokers/${node}/metrics/messages/qos1/sent` | 累计 QoS 1 发送消息数 |
| `$SYS/brokers/${node}/metrics/messages/qos2/received` | 累计 QoS 2 接收消息数 |
| `$SYS/brokers/${node}/metrics/messages/qos2/sent` | 累计 QoS 2 发送消息数 |
| `$SYS/brokers/${node}/metrics/messages/qos2/expired` | 累计 QoS 2 过期消息数 |
| `$SYS/brokers/${node}/metrics/messages/qos2/dropped` | 累计 QoS 2 丢弃消息数 |

### 报文（Packets）

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/${node}/metrics/packets/received` | 累计接收报文数 |
| `$SYS/brokers/${node}/metrics/packets/sent` | 累计发送报文数 |
| `$SYS/brokers/${node}/metrics/packets/connect` | 累计 CONNECT 报文数 |
| `$SYS/brokers/${node}/metrics/packets/connack` | 累计 CONNACK 报文数 |
| `$SYS/brokers/${node}/metrics/packets/publish/received` | 累计接收 PUBLISH 报文数 |
| `$SYS/brokers/${node}/metrics/packets/publish/sent` | 累计发送 PUBLISH 报文数 |
| `$SYS/brokers/${node}/metrics/packets/puback/received` | 累计接收 PUBACK 报文数 |
| `$SYS/brokers/${node}/metrics/packets/puback/sent` | 累计发送 PUBACK 报文数 |
| `$SYS/brokers/${node}/metrics/packets/puback/missed` | 累计 PUBACK 超时未确认数 |
| `$SYS/brokers/${node}/metrics/packets/pubrec/received` | 累计接收 PUBREC 报文数 |
| `$SYS/brokers/${node}/metrics/packets/pubrec/sent` | 累计发送 PUBREC 报文数 |
| `$SYS/brokers/${node}/metrics/packets/pubrec/missed` | 累计 PUBREC 超时未确认数 |
| `$SYS/brokers/${node}/metrics/packets/pubrel/received` | 累计接收 PUBREL 报文数 |
| `$SYS/brokers/${node}/metrics/packets/pubrel/sent` | 累计发送 PUBREL 报文数 |
| `$SYS/brokers/${node}/metrics/packets/pubrel/missed` | 累计 PUBREL 超时未确认数 |
| `$SYS/brokers/${node}/metrics/packets/pubcomp/received` | 累计接收 PUBCOMP 报文数 |
| `$SYS/brokers/${node}/metrics/packets/pubcomp/sent` | 累计发送 PUBCOMP 报文数 |
| `$SYS/brokers/${node}/metrics/packets/pubcomp/missed` | 累计 PUBCOMP 超时未确认数 |
| `$SYS/brokers/${node}/metrics/packets/subscribe` | 累计 SUBSCRIBE 报文数 |
| `$SYS/brokers/${node}/metrics/packets/suback` | 累计 SUBACK 报文数 |
| `$SYS/brokers/${node}/metrics/packets/unsubscribe` | 累计 UNSUBSCRIBE 报文数 |
| `$SYS/brokers/${node}/metrics/packets/unsuback` | 累计 UNSUBACK 报文数 |
| `$SYS/brokers/${node}/metrics/packets/pingreq` | 累计 PINGREQ 报文数 |
| `$SYS/brokers/${node}/metrics/packets/pingresp` | 累计 PINGRESP 报文数 |
| `$SYS/brokers/${node}/metrics/packets/disconnect/received` | 累计接收 DISCONNECT 报文数 |
| `$SYS/brokers/${node}/metrics/packets/disconnect/sent` | 累计发送 DISCONNECT 报文数 |
| `$SYS/brokers/${node}/metrics/packets/auth` | 累计 AUTH 报文数 |

---

## 客户端事件

以下主题在事件发生时**即时触发**，`${clientid}` 替换为实际的客户端 ID。

| 主题 | 触发时机 | Payload 格式 |
|------|----------|------------|
| `$SYS/brokers/${node}/clients/${clientid}/connected` | 客户端连接成功 | JSON |
| `$SYS/brokers/${node}/clients/${clientid}/disconnected` | 客户端断开连接 | JSON |
| `$SYS/brokers/${node}/clients/${clientid}/subscribed` | 客户端订阅成功 | JSON |
| `$SYS/brokers/${node}/clients/${clientid}/unsubscribed` | 客户端取消订阅 | JSON |

### connected 消息示例

```json
{
  "username": "user1",
  "ts": 1700000000000,
  "sock_port": 54321,
  "proto_ver": "V4",
  "proto_name": "MQTT",
  "keepalive": 60,
  "ip_address": "192.168.1.100",
  "expiry_interval": 0,
  "connected_at": 1700000000000,
  "connect_ack": 0,
  "client_id": "my-client-001",
  "clean_start": true
}
```

### disconnected 消息示例

```json
{
  "username": "user1",
  "ts": 1700000100000,
  "sock_port": 54321,
  "reason": "NormalDisconnection",
  "proto_ver": "V4",
  "proto_name": "MQTT",
  "ip_address": "192.168.1.100",
  "disconnected_at": 1700000100000,
  "client_id": "my-client-001"
}
```

---

## 系统告警

告警事件触发时即时发布。更多告警配置见[系统告警](./SystemAlarm.md)文档。

| 主题 | 说明 |
|------|------|
| `$SYS/brokers/${node}/alarms/alert` | 新告警触发时发布 |
| `$SYS/brokers/${node}/alarms/clear` | 告警恢复/清除时发布 |

### 告警消息示例

```json
{
  "name": "HighCpuUsage",
  "message": "HighCpuUsage is 85%, but config is 70%",
  "create_time": 1700000000
}
```

---

## 订阅示例

使用 MQTTX 或任意 MQTT 客户端订阅所有系统主题：

```bash
# 订阅所有系统主题
mqttx sub -t '$SYS/#' -h 127.0.0.1 -p 1883

# 订阅当前节点所有统计数据
mqttx sub -t '$SYS/brokers/+/stats/#' -h 127.0.0.1 -p 1883

# 订阅所有客户端连接事件
mqttx sub -t '$SYS/brokers/+/clients/+/connected' -h 127.0.0.1 -p 1883
```
