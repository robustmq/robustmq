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
- `{DelayInterval}`: An integer delay in seconds
- `{TopicName}`: The target topic name for the MQTT message

**Important Notes**:

- If `{DelayInterval}` cannot be parsed as an integer, the server will discard the message
- The delay must not exceed `42,949,669` seconds (about 497 days)
- A delayed message is queued only when offline-message storage is enabled or the target topic has subscribers; otherwise, with offline storage disabled and no subscribers, the publish is rejected

## Characteristics of Delayed Publishing

- **Precise time control**: Can precisely control the publishing time of messages
- **Persistent storage**: Accepted delayed messages are persisted and recovered after a server restart
- **High reliability**: Supports QoS levels to ensure reliable message delivery
- **Flexible configuration**: Supports delayed publishing to any topic

Delayed delivery currently provides **at-least-once delivery**. The broker writes
the message to the target topic before deleting its delayed-message records. If
the broker stops after the target write succeeds but before those records are
deleted, recovery may deliver the same message again. Consumers should therefore
handle delayed messages idempotently. Atomic delivery cleanup and broker-side
deduplication are separate future improvements.

## Using Delayed Publishing via MQTTX

### Using MQTTX CLI

1. **Publish delayed message**

   ```bash
   mqttx pub -t '$delayed/10/sensor/temperature' -m "25.5°C" -h '117.72.92.117' -p 1883
   ```

   This message will be published to the `/sensor/temperature` topic after 10 seconds.

2. **Subscribe to target topic**

   ```bash
   mqttx sub -t "/sensor/temperature" -h '117.72.92.117' -p 1883 -v
   ```

   After waiting 10 seconds, subscribers will receive the delayed published message.

3. **Publish messages with different delay times**

   ```bash
   # Publish after 15 seconds
   mqttx pub -t '$delayed/15/device/status' -m "online" -h '117.72.92.117' -p 1883

   # Publish after 60 seconds
   mqttx pub -t '$delayed/60/system/backup' -m "backup completed" -h '117.72.92.117' -p 1883

   # Publish after 3600 seconds (1 hour)
   mqttx pub -t '$delayed/3600/report/daily' -m "daily report ready" -h '117.72.92.117' -p 1883
   ```

### Practical Application Examples

#### Scheduled Reminder System

```bash
# Set reminder for 30 seconds later
mqttx pub -t '$delayed/30/reminder/meeting' -m '{"title":"Meeting Reminder","time":"14:00","location":"Conference Room A"}' -h '117.72.92.117' -p 1883

# Set reminder for 1 hour later
mqttx pub -t '$delayed/3600/reminder/break' -m '{"title":"Break Reminder","message":"Time for a break"}' -h '117.72.92.117' -p 1883

# Subscribe to reminder topics
mqttx sub -t "/reminder/+" -h '117.72.92.117' -p 1883 -v
```

#### Broker-Side Delivery Retries

```bash
# Schedule a message for delivery to /upload after 5 seconds
mqttx pub -t '$delayed/5/upload' -m '{"file_id":"123"}' -h '117.72.92.117' -p 1883

# Subscribe to the target topic
mqttx sub -t "/upload" -h '117.72.92.117' -p 1883 -v
```

The broker retries delivery to the same target topic if its write fails. The
`[delay_message]` configuration controls the maximum number of retries and
exponential backoff. By default, there are up to 3 retries after the initial
delivery attempt, with delays of approximately 2, 4, and 8 seconds. If all
attempts fail, the broker deletes the delayed message; it does not publish it to
a dead-letter topic. This mechanism retries broker-side delivery failures, not
application-level processing failures reported by subscribers. The retry count
is persisted, so a broker restart does not reset the configured retry limit.
The remaining backoff duration is not persisted, so recovery may retry a pending
message immediately after restart.

#### Batch Processing

```bash
# Delay multiple messages to the same time point (10 minutes later)
mqttx pub -t '$delayed/600/batch/process' -m '{"batch_id":"001","type":"data_analysis"}' -h '117.72.92.117' -p 1883
mqttx pub -t '$delayed/600/batch/process' -m '{"batch_id":"002","type":"data_analysis"}' -h '117.72.92.117' -p 1883
mqttx pub -t '$delayed/600/batch/process' -m '{"batch_id":"003","type":"data_analysis"}' -h '117.72.92.117' -p 1883

# Subscribe to batch processing topic
mqttx sub -t "/batch/process" -h '117.72.92.117' -p 1883 -v
```

#### Traffic Control

```bash
# Control message sending rate, send one message every 2 seconds
mqttx pub -t '$delayed/2/rate/limited' -m "message 1" -h '117.72.92.117' -p 1883
mqttx pub -t '$delayed/4/rate/limited' -m "message 2" -h '117.72.92.117' -p 1883
mqttx pub -t '$delayed/6/rate/limited' -m "message 3" -h '117.72.92.117' -p 1883

# Subscribe to rate-limited topic
mqttx sub -t "/rate/limited" -h '117.72.92.117' -p 1883 -v
```

#### Business Logic Delay

```bash
# Device cooldown period, can restart after 30 minutes
mqttx pub -t '$delayed/1800/device/cooldown' -m '{"device_id":"D001","action":"restart_allowed"}' -h '117.72.92.117' -p 1883

# User operation waiting period, can perform sensitive operations after 5 minutes
mqttx pub -t '$delayed/300/user/security' -m '{"user_id":"U001","operation":"sensitive_allowed"}' -h '117.72.92.117' -p 1883

# Subscribe to business logic topics
mqttx sub -t "/device/+" -h '117.72.92.117' -p 1883 -v
mqttx sub -t "/user/+" -h '117.72.92.117' -p 1883 -v
```

## Delay Time Examples

| Delay Time | Example Topic | Description |
|------------|---------------|-------------|
| 15 seconds | `$delayed/15/sensor/data` | Publish to `/sensor/data` after 15 seconds |
| 60 seconds | `$delayed/60/system/status` | Publish to `/system/status` after 1 minute |
| 3600 seconds | `$delayed/3600/report/hourly` | Publish to `/report/hourly` after 1 hour |
| 86400 seconds | `$delayed/86400/report/daily` | Publish to `/report/daily` after 1 day |

## Important Notes

1. **Time format**: The delay value must be an integer in seconds and is measured from the time the broker accepts the message
2. **Topic format**: Must use the `$delayed/{DelayInterval}/{TopicName}` format
3. **Message persistence**: Delayed messages are persisted when offline-message storage is enabled or the target topic has subscribers. If offline storage is disabled and there are no subscribers, the publish is rejected and the message is not scheduled
4. **QoS support**: Delayed publishing supports MQTT QoS levels. MQTT 5 QoS 1 and QoS 2 publishers receive a negative acknowledgement when this policy rejects the message. For QoS 0, and for MQTT 3/4 where acknowledgements cannot carry a failure reason, the broker disconnects the publisher rather than sending a success acknowledgement
5. **Delivery retries**: Broker-side delivery failures are retried according to `[delay_message]`; messages are deleted after retries are exhausted, without dead-letter routing
6. **Delivery guarantee**: Delayed delivery is at-least-once. A broker restart during target-topic delivery can cause duplicate delivery, so consumers should use idempotent processing
7. **Error handling**: Messages will be rejected if the delay time format is incorrect or if delayed-message storage is disabled by the no-subscriber policy
