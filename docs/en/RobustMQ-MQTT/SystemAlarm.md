### Overview

RobustMQ provides a system alarm function to monitor and manage the operation status of the MQTT server. When an
internal system status anomaly occurs, RobustMQ will automatically generate an alarm message and publish it to a
specific alarm topic via the MQTT protocol. Users can subscribe to these topics to get timely information about system
status changes.

The current features of RobustMQ include:

- Monitoring system CPU and memory usage
- Retrieving and querying detailed alarm information
- Configuring alarm information

Through system alarms, users can promptly understand the operation status of the MQTT server, quickly respond to
potential issues, and ensure the stability and reliability of the system.

### Current Supported Alarms

| Alarm               | Description              |
|---------------------|--------------------------|
| system_memory_usage | System memory usage rate |
| cpu_high_usage      | High CPU usage rate      |
| cpu_low_usage       | Low CPU usage rate       |

### Retrieving Alarm Information

RobustMQ currently supports retrieving system alarm information via the MQTT protocol. Users can subscribe to the
following topics to receive alarm messages:
`$SYS/brokers/${node}/alarms/activate` and `$SYS/brokers/${node}/alarms/deactivate`.
Here, `${node}` is the name of the node, `activate` refers to active alarms, and `deactivate` refers to inactive alarms.
The format of the alarm message is as follows:

```json
{
  "name": "system_memory_usage",
  "message": "system_memory_usage is 80%, but config is 70%",
  "activate_at": 1700000000,
  "activated": false
}
```

### Alarm Configuration

RobustMQ allows users to configure alarm thresholds and statuses. Users can modify the configuration file or use the
`Cli` to set the parameters related to alarm items.

#### Configuration via Configuration File

::: tip
⚠️ Note: Currently, the function to set the interval for checking CPU utilization and memory usage is not supported. The
default interval is now `60 seconds`.
:::

Users can set the alarm thresholds and statuses in the RobustMQ configuration file. Here is an example configuration:

```toml
[system_monitor]
enable = true
os_cpu_check_interval_ms = 60000
os_cpu_high_watermark = 70.0
os_cpu_low_watermark = 50.0
os_memory_check_interval_ms = 60
os_memory_high_watermark = 80.0
```

#### Configuration via `Cli`

Users can use the command-line interface (CLI) of RobustMQ to configure alarm items. Here are some common command
examples:

##### Setting Current Configuration

```bash
# Enable system alarms and set the CPU high usage alarm threshold
./bin/robustmq-cli mqtt set --enable=true --cpu-high-watermark 80.0
# Set the CPU low usage alarm threshold
./bin/robustmq-cli mqtt set --cpu-low-watermark 60.0
```

The final display effect may be similar to the following (actual results may vary depending on the command parameters
used):

```text
// This is just a display example, and the actual result depends on the command parameters used
Set system alarm config successfully! Current Config:
+-----------------------+-------+
| Config Options        | Value |
+=======================+=======+
| enable                | true  |
+-----------------------+-------+
| memory-high-watermark | 80    |
+-----------------------+-------+
| cpu-high-watermark    | 81.2  |
+-----------------------+-------+
| cpu-low-watermark     | 55    |
+-----------------------+-------+
| cpu-check-interval-ms | 60000 |
+-----------------------+-------+
```

##### Retrieving Current Alarms

```bash
./bin/robustmq-cli mqtt system-alarm list
```

The final display effect may be similar to the following (actual results may vary depending on the command parameters
used):

```text
// This is just a display example, and the actual result depends on the command parameters used
system alarm list result:
+--------------+------------------------------------------------+-------------+-----------+
| name         | message                                        | activate_at | activated |
+==============+================================================+=============+===========+
| MemoryUsage  | MemoryUsage is 0.6325722%, but config is 80%   | 1749774914  | false     |
+--------------+------------------------------------------------+-------------+-----------+
| LowCpuUsage  | LowCpuUsage is 0.39186627%, but config is 50%  | 1749774914  | true      |
+--------------+------------------------------------------------+-------------+-----------+
| HighCpuUsage | HighCpuUsage is 0.39186627%, but config is 70% | 1749774914  | false     |
+--------------+------------------------------------------------+-------------+-----------+
```
