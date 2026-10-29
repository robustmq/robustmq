### Overview

RobustMQ provides a system alarm function to monitor and manage the operation status of the MQTT server. When an
internal system status anomaly occurs, RobustMQ will automatically generate an alarm message and publish it to a
specific alarm topic via the MQTT protocol. Users can subscribe to these topics to get timely information about system
status changes.

The current features of RobustMQ include:

- Monitoring the broker process CPU and memory usage on each node
- Retrieving and querying detailed alarm information
- Configuring alarm information

Through system alarms, users can promptly understand the operation status of the MQTT server, quickly respond to
potential issues, and ensure the stability and reliability of the system.

### Current Supported Alarms

| Alarm               | Description              |
|---------------------|--------------------------|
| HighMemoryUsage     | This node's broker process memory usage exceeds `os_memory_high_watermark` |
| HighCpuUsage        | This node's broker process CPU usage exceeds `os_cpu_high_watermark` |

### Retrieving Alarm Information

RobustMQ publishes an alarm report to the fixed `$SYS/brokers/alarms/alert` topic
when that node's broker process CPU or memory usage exceeds its configured
high-water mark. The older `${node}` topic-path form is not emitted by the
current implementation. Node attribution is retained in the common envelope,
so consumers can identify the affected machine from `node_id` and `node_ip`.
The topic reports the result of each check that exceeds the configured
threshold. Every active report has `activated: true`. When usage is at or below
the threshold, no alarm message is published; there is no `activated: false`
recovery message and no separate deactivate or clear topic.

#### Current Design Decision

Alarm activation is computed directly from each CPU/memory sample as
`current_usage > configured_watermark`. There is no in-memory alarm state. When
the value remains above the threshold, an active report is emitted on every
60-second check; when it is at or below the threshold, that check emits nothing.
This keeps the monitor stateless and makes a broker restart equivalent to the
next normal sample.

If the broker cannot write an active report to the internal system topic, the
same 60-second polling behavior naturally retries it. This confirms the broker's
internal write, not delivery acknowledgement from each individual MQTT
subscriber.

Each active report is currently also persisted so the existing
`/api/mqtt/system-alarm/list` endpoint and `robust-ctl mqtt system-alarm list`
command can show alarm history. Making alarms fully transient would
remove or redesign the existing history API and CLI, so that is a separate
follow-up decision.

```json
{
  "node_id": 1,
  "node_ip": "127.0.0.1",
  "ts": 1700000000000,
  "value": {
    "name": "HighMemoryUsage",
    "message": "HighMemoryUsage is 85%, but config is 80%",
    "activate_at": 1700000000,
    "activated": true
  }
}
```

The MQTT envelope includes the fields identifying the machine that raised the
alarm. The persisted alarm/API record exposes the same fields at the record's
top level for historical diagnosis:

| Field | Meaning |
|-------|---------|
| `node_id` | RobustMQ broker ID for the node that evaluated and emitted the alarm |
| `node_ip` | Configured `broker_ip` for that broker node, or the local-IP fallback when `broker_ip` is omitted |
| `ts` | Timestamp when the system-topic envelope was created |
| `value` | Alarm details, including the alarm name, message, activation timestamp, and activation state |

Use `node_id` as the stable node identity and `node_ip` as the network address
when determining which machine requires attention. These fields are generated
by the broker that publishes the alert; they are not supplied by the MQTT
client.

### Alarm Configuration

RobustMQ allows users to configure alarm thresholds and statuses. Users can modify the configuration file or use the
`Cli` to set the parameters related to alarm items.

#### Configuration via Configuration File

The system monitor is disabled by default. When enabled, the alarm loop checks
CPU and memory usage every 60 seconds. Configure it as follows:

```toml
[mqtt_runtime.system_monitor]
enable = true
os_cpu_high_watermark = 70.0
os_memory_high_watermark = 80.0
```

#### Configuration via `Cli`

Users can inspect persisted alarm records with the CLI:

```bash
robust-ctl mqtt system-alarm list
```

The final display effect may be similar to the following (actual results may vary depending on the command parameters
used):

```text
// This is just a display example, and the actual result depends on the command parameters used
system alarm list result:
+---------+-------------+-----------------+------------------------------------------------+-------------+-----------+
| node_id | node_ip     | name            | message                                        | create_time | activated |
+=========+=============+=================+================================================+=============+===========+
| 1       | 127.0.0.1   | HighMemoryUsage | HighMemoryUsage is 85.2%, but config is 80%    | 1749774914  | true      |
+---------+-------------+-----------------+------------------------------------------------+-------------+-----------+
```
