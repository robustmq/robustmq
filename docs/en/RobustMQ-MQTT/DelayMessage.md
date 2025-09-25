# MQTT Delayed Publishing

## What is MQTT Delayed Publishing?

Delayed publishing is an extended feature of MQTT supported by RobustMQ MQTT. When a client publishes a message using the special topic prefix `$delayed/{DelayInterval}`, it triggers the delayed publishing functionality, allowing messages to be published at a user-configured time interval.

The core feature of delayed publishing is **time control**: messages are not immediately published to the target topic, but are published after a specified delay time, implementing scheduled message sending functionality.

## When to use MQTT Delayed Publishing?

Delayed publishing is suitable for the following scenarios:

- **Scheduled tasks**: Tasks that need to be executed at specific times, such as scheduled reminders and scheduled reports
- **Retry mechanisms**: Delayed retries after failures to avoid system pressure from immediate retries
- **Batch processing**: Delaying multiple messages to the same time point for batch processing
- **Traffic control**: Controlling message sending rates through delayed publishing to avoid system overload
- **Business logic**: Operations that need to wait for a certain time before execution, such as cooldown periods and waiting periods

## Syntax Rules for Delayed Publishing

The specific format for delayed publishing topics is as follows:

```text
$delayed/{DelayInterval}/{TopicName}
```

**Parameter Description**:

- `$delayed`: Messages using `$delayed` as the topic prefix will be considered as messages that need delayed publishing
- `{DelayInterval}`: Specifies the time interval for delayed publishing of the MQTT message, in seconds, with a maximum allowed interval of 4294967 seconds
- `{TopicName}`: The target topic name for the MQTT message

**Important Notes**:

- If `{DelayInterval}` cannot be parsed as an integer, the server will discard the message
- Delay time is in seconds, supporting a maximum delay time of 4294967 seconds (approximately 49.7 days)
- Delayed messages are stored on the server until the delay time is reached

## Characteristics of Delayed Publishing

- **Precise time control**: Can precisely control the publishing time of messages
- **Persistent storage**: Delayed messages are persistently stored, ensuring normal execution after server restart
- **High reliability**: Supports QoS levels to ensure reliable message delivery
- **Flexible configuration**: Supports delayed publishing to any topic

## Using Delayed Publishing via MQTTX

### Using MQTTX CLI

1. **Publish delayed message**

   ```bash
   mqttx pub -t "$delayed/10/sensor/temperature" -m "25.5°C" -h '117.72.92.117' -p 1883
   ```

   This message will be published to the `sensor/temperature` topic after 10 seconds.

2. **Subscribe to target topic**

   ```bash
   mqttx sub -t "sensor/temperature" -h '117.72.92.117' -p 1883 -v
   ```

   After waiting 10 seconds, subscribers will receive the delayed published message.

3. **Publish messages with different delay times**

   ```bash
   # Publish after 15 seconds
   mqttx pub -t "$delayed/15/device/status" -m "online" -h '117.72.92.117' -p 1883
   
   # Publish after 60 seconds
   mqttx pub -t "$delayed/60/system/backup" -m "backup completed" -h '117.72.92.117' -p 1883
   
   # Publish after 3600 seconds (1 hour)
   mqttx pub -t "$delayed/3600/report/daily" -m "daily report ready" -h '117.72.92.117' -p 1883
   ```

### Practical Application Examples

#### Scheduled Reminder System

```bash
# Set reminder for 30 seconds later
mqttx pub -t "$delayed/30/reminder/meeting" -m '{"title":"Meeting Reminder","time":"14:00","location":"Conference Room A"}' -h '117.72.92.117' -p 1883

# Set reminder for 1 hour later
mqttx pub -t "$delayed/3600/reminder/break" -m '{"title":"Break Reminder","message":"Time for a break"}' -h '117.72.92.117' -p 1883

# Subscribe to reminder topics
mqttx sub -t "reminder/+" -h '117.72.92.117' -p 1883 -v
```

#### Retry Mechanism

```bash
# Retry after 5 seconds delay on failure
mqttx pub -t "$delayed/5/retry/upload" -m '{"file_id":"123","retry_count":1}' -h '117.72.92.117' -p 1883

# Retry after 30 seconds delay on failure
mqttx pub -t "$delayed/30/retry/sync" -m '{"data_id":"456","retry_count":2}' -h '117.72.92.117' -p 1883

# Subscribe to retry topics
mqttx sub -t "retry/+" -h '117.72.92.117' -p 1883 -v
```

#### Batch Processing

```bash
# Delay multiple messages to the same time point (10 minutes later)
mqttx pub -t "$delayed/600/batch/process" -m '{"batch_id":"001","type":"data_analysis"}' -h '117.72.92.117' -p 1883
mqttx pub -t "$delayed/600/batch/process" -m '{"batch_id":"002","type":"data_analysis"}' -h '117.72.92.117' -p 1883
mqttx pub -t "$delayed/600/batch/process" -m '{"batch_id":"003","type":"data_analysis"}' -h '117.72.92.117' -p 1883

# Subscribe to batch processing topic
mqttx sub -t "batch/process" -h '117.72.92.117' -p 1883 -v
```

#### Traffic Control

```bash
# Control message sending rate, send one message every 2 seconds
mqttx pub -t "$delayed/2/rate/limited" -m "message 1" -h '117.72.92.117' -p 1883
mqttx pub -t "$delayed/4/rate/limited" -m "message 2" -h '117.72.92.117' -p 1883
mqttx pub -t "$delayed/6/rate/limited" -m "message 3" -h '117.72.92.117' -p 1883

# Subscribe to rate-limited topic
mqttx sub -t "rate/limited" -h '117.72.92.117' -p 1883 -v
```

#### Business Logic Delay

```bash
# Device cooldown period, can restart after 30 minutes
mqttx pub -t "$delayed/1800/device/cooldown" -m '{"device_id":"D001","action":"restart_allowed"}' -h '117.72.92.117' -p 1883

# User operation waiting period, can perform sensitive operations after 5 minutes
mqttx pub -t "$delayed/300/user/security" -m '{"user_id":"U001","operation":"sensitive_allowed"}' -h '117.72.92.117' -p 1883

# Subscribe to business logic topics
mqttx sub -t "device/+" -h '117.72.92.117' -p 1883 -v
mqttx sub -t "user/+" -h '117.72.92.117' -p 1883 -v
```

## Delay Time Examples

| Delay Time | Example Topic | Description |
|------------|---------------|-------------|
| 15 seconds | `$delayed/15/sensor/data` | Publish to `sensor/data` after 15 seconds |
| 60 seconds | `$delayed/60/system/status` | Publish to `system/status` after 1 minute |
| 3600 seconds | `$delayed/3600/report/hourly` | Publish to `report/hourly` after 1 hour |
| 86400 seconds | `$delayed/86400/report/daily` | Publish to `report/daily` after 1 day |

## Important Notes

1. **Time format**: Delay time must be an integer in seconds
2. **Maximum delay**: Maximum supported delay time is 4294967 seconds (approximately 49.7 days)
3. **Topic format**: Must use the `$delayed/{DelayInterval}/{TopicName}` format
4. **Message persistence**: Delayed messages are persistently stored and can execute normally after server restart
5. **QoS support**: Delayed publishing supports MQTT QoS levels
6. **Error handling**: Messages will be discarded if the delay time format is incorrect
