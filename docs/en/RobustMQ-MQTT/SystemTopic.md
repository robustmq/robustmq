# System Topics

## Overview

RobustMQ MQTT Broker publishes its own runtime status and statistics via `$SYS/` system topics. Clients can subscribe to these topics just like any regular MQTT topic to monitor connections, message throughput, alarm events, and other key metrics in real time — no additional monitoring agent required.

System topics are periodically published by the Broker (refreshed every **60 seconds** by default, configurable via `mqtt_system_topic.interval_ms`). Some topics are published immediately when an event occurs (e.g., client connect/disconnect).

> **Note**: System topics start with `$SYS/`. By default, non-admin clients cannot subscribe to them. Make sure your ACL configuration allows access.

---

## Topic Format

All system topics are prefixed with `$SYS/brokers/${node}/`, where `${node}` is replaced with the current node's IP address.

---

## Broker Info

Published periodically. Describes the Broker's version, uptime, and other static information.

| Topic | Description | Example Value |
|-------|-------------|---------------|
| `$SYS/brokers` | Cluster node list (JSON) | `[{"node_id":1,...}]` |
| `$SYS/brokers/${node}/version` | Broker version | `0.4.0` |
| `$SYS/brokers/${node}/uptime` | Running duration | `0 days, 2 hours, 15 minutes, 30 seconds` |
| `$SYS/brokers/${node}/datetime` | Current system time | `2026-02-23 10:00:00` |
| `$SYS/brokers/${node}/sysdescr` | OS information | `Ubuntu 22.04` |

---

## Statistics (Stats)

Published periodically. Reflects real-time counts of various objects in the system.

### Connections

| Topic | Description |
|-------|-------------|
| `$SYS/brokers/${node}/stats/connections/count` | Current online connection count |
| `$SYS/brokers/${node}/stats/connections/max` | Peak connection count (approximated by session count) |

### Topics

| Topic | Description |
|-------|-------------|
| `$SYS/brokers/${node}/stats/topics/count` | Current topic count |
| `$SYS/brokers/${node}/stats/topics/max` | Peak topic count |

### Routes

| Topic | Description |
|-------|-------------|
| `$SYS/brokers/${node}/stats/routes/count` | Current route count (equals total active subscriptions) |
| `$SYS/brokers/${node}/stats/routes/max` | Peak route count |

### Subscriptions

| Topic | Description |
|-------|-------------|
| `$SYS/brokers/${node}/stats/subscribers/count` | Current exclusive subscriber count |
| `$SYS/brokers/${node}/stats/subscribers/max` | Peak exclusive subscriber count |
| `$SYS/brokers/${node}/stats/subscriptions/count` | Total subscriptions (exclusive + shared) |
| `$SYS/brokers/${node}/stats/subscriptions/max` | Peak total subscriptions |
| `$SYS/brokers/${node}/stats/subscriptions/shared/count` | Current shared subscription count |
| `$SYS/brokers/${node}/stats/subscriptions/shared/max` | Peak shared subscription count |
| `$SYS/brokers/${node}/stats/suboptions/count` | Current subscription options count |
| `$SYS/brokers/${node}/stats/suboptions/max` | Peak subscription options count |

---

## Metrics

Published periodically. Reflects cumulative byte, message, and packet counts.

### Bytes

| Topic | Description |
|-------|-------------|
| `$SYS/brokers/${node}/metrics/bytes/received` | Total bytes received |
| `$SYS/brokers/${node}/metrics/bytes/sent` | Total bytes sent |

### Messages

| Topic | Description |
|-------|-------------|
| `$SYS/brokers/${node}/metrics/messages/received` | Total messages received |
| `$SYS/brokers/${node}/metrics/messages/sent` | Total messages sent |
| `$SYS/brokers/${node}/metrics/messages/dropped` | Total messages dropped (no subscribers) |
| `$SYS/brokers/${node}/metrics/messages/retained` | Current retained message count |
| `$SYS/brokers/${node}/metrics/messages/expired` | Total expired messages |
| `$SYS/brokers/${node}/metrics/messages/forward` | Total forwarded messages (cluster nodes) |
| `$SYS/brokers/${node}/metrics/messages/qos0/received` | Total QoS 0 messages received |
| `$SYS/brokers/${node}/metrics/messages/qos0/sent` | Total QoS 0 messages sent |
| `$SYS/brokers/${node}/metrics/messages/qos1/received` | Total QoS 1 messages received |
| `$SYS/brokers/${node}/metrics/messages/qos1/sent` | Total QoS 1 messages sent |
| `$SYS/brokers/${node}/metrics/messages/qos2/received` | Total QoS 2 messages received |
| `$SYS/brokers/${node}/metrics/messages/qos2/sent` | Total QoS 2 messages sent |
| `$SYS/brokers/${node}/metrics/messages/qos2/expired` | Total QoS 2 expired messages |
| `$SYS/brokers/${node}/metrics/messages/qos2/dropped` | Total QoS 2 dropped messages |

### Packets

| Topic | Description |
|-------|-------------|
| `$SYS/brokers/${node}/metrics/packets/received` | Total packets received |
| `$SYS/brokers/${node}/metrics/packets/sent` | Total packets sent |
| `$SYS/brokers/${node}/metrics/packets/connect` | Total CONNECT packets |
| `$SYS/brokers/${node}/metrics/packets/connack` | Total CONNACK packets |
| `$SYS/brokers/${node}/metrics/packets/publish/received` | Total PUBLISH packets received |
| `$SYS/brokers/${node}/metrics/packets/publish/sent` | Total PUBLISH packets sent |
| `$SYS/brokers/${node}/metrics/packets/puback/received` | Total PUBACK packets received |
| `$SYS/brokers/${node}/metrics/packets/puback/sent` | Total PUBACK packets sent |
| `$SYS/brokers/${node}/metrics/packets/puback/missed` | Total PUBACK timeout/unacknowledged |
| `$SYS/brokers/${node}/metrics/packets/pubrec/received` | Total PUBREC packets received |
| `$SYS/brokers/${node}/metrics/packets/pubrec/sent` | Total PUBREC packets sent |
| `$SYS/brokers/${node}/metrics/packets/pubrec/missed` | Total PUBREC timeout/unacknowledged |
| `$SYS/brokers/${node}/metrics/packets/pubrel/received` | Total PUBREL packets received |
| `$SYS/brokers/${node}/metrics/packets/pubrel/sent` | Total PUBREL packets sent |
| `$SYS/brokers/${node}/metrics/packets/pubrel/missed` | Total PUBREL timeout/unacknowledged |
| `$SYS/brokers/${node}/metrics/packets/pubcomp/received` | Total PUBCOMP packets received |
| `$SYS/brokers/${node}/metrics/packets/pubcomp/sent` | Total PUBCOMP packets sent |
| `$SYS/brokers/${node}/metrics/packets/pubcomp/missed` | Total PUBCOMP timeout/unacknowledged |
| `$SYS/brokers/${node}/metrics/packets/subscribe` | Total SUBSCRIBE packets |
| `$SYS/brokers/${node}/metrics/packets/suback` | Total SUBACK packets |
| `$SYS/brokers/${node}/metrics/packets/unsubscribe` | Total UNSUBSCRIBE packets |
| `$SYS/brokers/${node}/metrics/packets/unsuback` | Total UNSUBACK packets |
| `$SYS/brokers/${node}/metrics/packets/pingreq` | Total PINGREQ packets |
| `$SYS/brokers/${node}/metrics/packets/pingresp` | Total PINGRESP packets |
| `$SYS/brokers/${node}/metrics/packets/disconnect/received` | Total DISCONNECT packets received |
| `$SYS/brokers/${node}/metrics/packets/disconnect/sent` | Total DISCONNECT packets sent |
| `$SYS/brokers/${node}/metrics/packets/auth` | Total AUTH packets |

---

## Client Events

The following topics are published **immediately** when the event occurs. `${clientid}` is replaced with the actual client ID.

| Topic | Trigger | Payload |
|-------|---------|---------|
| `$SYS/brokers/${node}/clients/${clientid}/connected` | Client connects | JSON |
| `$SYS/brokers/${node}/clients/${clientid}/disconnected` | Client disconnects | JSON |
| `$SYS/brokers/${node}/clients/${clientid}/subscribed` | Client subscribes | JSON |
| `$SYS/brokers/${node}/clients/${clientid}/unsubscribed` | Client unsubscribes | JSON |

### connected Payload Example

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

### disconnected Payload Example

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

## System Alarms

Published immediately when an alarm event is triggered. See the [System Alarm](./SystemAlarm.md) documentation for configuration details.

| Topic | Description |
|-------|-------------|
| `$SYS/brokers/${node}/alarms/alert` | Published when a new alarm is triggered |
| `$SYS/brokers/${node}/alarms/clear` | Published when an alarm is resolved/cleared |

### Alarm Payload Example

```json
{
  "name": "HighCpuUsage",
  "message": "HighCpuUsage is 85%, but config is 70%",
  "create_time": 1700000000
}
```

---

## Subscription Examples

Subscribe to system topics using MQTTX or any MQTT client:

```bash
# Subscribe to all system topics
mqttx sub -t '$SYS/#' -h 127.0.0.1 -p 1883

# Subscribe to all stats for the current node
mqttx sub -t '$SYS/brokers/+/stats/#' -h 127.0.0.1 -p 1883

# Subscribe to all client connection events
mqttx sub -t '$SYS/brokers/+/clients/+/connected' -h 127.0.0.1 -p 1883
```
