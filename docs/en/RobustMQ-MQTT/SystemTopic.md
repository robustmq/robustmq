# System Topics

## Overview

RobustMQ MQTT Broker publishes runtime status, aggregate statistics, client
events, and system alarms through `$SYS/` topics. These topics are ordinary
MQTT topics and are subject to the broker's system-topic ACL rules.

Periodic reports are emitted after the system-topic monitor starts and then at
the configured interval (60 seconds by default,
`mqtt_runtime.system_monitor.system_topic_interval_ms`). Client events are
queued and written to their event stream as they occur. Alarm checks run on a
60-second interval.

## Topic and Payload Format

The implementation uses fixed topic names. It does not add a node or client ID
to the topic path. The node that produced a report is included in the common
payload envelope. `node_ip` uses the configured `broker_ip` when present and
falls back to local-IP detection when it is omitted:

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

The populated payload above is an alarm report example. Other system topics use
their own report-specific object under `value`; the cluster topic `$SYS/brokers`
is the exception because its value is the cluster node list.

## Broker Information

| Topic | Value |
|-------|-------|
| `$SYS/brokers` | Cluster node list |
| `$SYS/brokers/info` | `version`, `uptime`, `datetime`, and `os_info` |

## Statistics

Each report is one JSON object under the `value` field; the broker does not
publish separate `/count` or `/max` child topics.

| Topic | Value fields |
|-------|-------------|
| `$SYS/brokers/stats/connections` | `count`, `max`, `sessions` |
| `$SYS/brokers/stats/routes` | `count`, `max` |
| `$SYS/brokers/stats/subscriptions` | `count`, `max`, `exclusive_count`, `shared_count`, `shared_group_count` |
| `$SYS/brokers/stats/topics` | `count`, `max` |

## Metrics

Each report is one aggregate JSON object under `value`; the broker does not
publish separate child topics for each field.

| Topic | Value fields |
|-------|-------------|
| `$SYS/brokers/metrics/bytes` | `received`, `sent` |
| `$SYS/brokers/metrics/messages` | `received`, `sent`, `expired`, `retained`, `dropped`, `forward`, per-QoS received/sent fields, `qos2_expired`, `qos2_dropped` |
| `$SYS/brokers/metrics/packets` | `received`, `sent`, CONNECT/CONNACK, PUBLISH, PUBACK, PUBREC, PUBREL, PUBCOMP, SUBSCRIBE/SUBACK, UNSUBSCRIBE/UNSUBACK, PINGREQ/PINGRESP, DISCONNECT, and AUTH fields |

The packet report includes `*_missed` fields for the QoS acknowledgement
families. The current implementation reports these fields as zero.

## Client Events

All connect, disconnect, subscribe, and unsubscribe events are written to the
single `$SYS/events` topic. Events are batched by the event consumer (up to 100
events per write) and each event is wrapped in the common payload envelope.
The client ID and node identity are payload fields; they are not topic-path
segments.

## System Alarms

The implemented alarm report is:

| Topic | Trigger |
|-------|---------|
| `$SYS/brokers/alarms/alert` | The reporting node's broker-process CPU or memory usage exceeds its configured high-water mark |

There is currently no `$SYS/brokers/alarms/clear` publisher. The alert topic
publishes an active report (`activated: true`) on each 60-second check where
the broker-process CPU or memory usage exceeds its configured threshold. When
usage is at or below the threshold, no alarm message is published.

The monitor is stateless and does not publish `activated: false` recovery
messages. If an internal active-report write fails, the next polling interval
retries it; subscriber delivery acknowledgements are not tracked. Active events
are still persisted for the existing alarm-history API and CLI.

Example message received on `$SYS/brokers/alarms/alert`:

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

## Subscription Examples

```bash
# Subscribe to all system topics
mqttx sub -t '$SYS/#' -h 127.0.0.1 -p 1883

# Subscribe to aggregate broker statistics
mqttx sub -t '$SYS/brokers/stats/#' -h 127.0.0.1 -p 1883

# Subscribe to all client events
mqttx sub -t '$SYS/events' -h 127.0.0.1 -p 1883
```
