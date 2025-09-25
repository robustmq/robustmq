# MQTT Shared Subscription

## What is MQTT Shared Subscription?

Shared subscription is an extended feature of MQTT that allows multiple subscribers to share a subscription to the same topic. In shared subscription, messages are load-balanced and distributed to different clients in the subscription group, rather than being broadcast to all subscribers.

The core feature of shared subscription is **load balancing**: when a message is published to a shared subscription topic, the message is distributed to one client in the subscription group, not all clients, thereby achieving load balancing for message processing.

## When to Use MQTT Shared Subscription?

Shared subscription is suitable for the following scenarios:

- **Load Balancing**: Multiple clients need to process the same type of messages, but each message only needs to be processed once
- **Task Distribution**: Distribute task messages to multiple worker nodes to improve processing efficiency
- **Message Queuing**: Implement message queue-like functionality to ensure messages are processed in order
- **High Availability**: When a client goes offline, other clients can continue processing messages
- **Horizontal Scaling**: Scale message processing capability by adding subscribers

## Shared Subscription Syntax Rules

Shared subscription has two formats:

### Shared Subscription with Groups

```text
$share/{group}/{topic}
```

**Parameter Description**:

- `$share`: Prefix identifier for shared subscription
- `{group}`: Subscription group name, can be any string
- `{topic}`: Original topic name

### Shared Subscription without Groups

```text
$queue/{topic}
```

**Parameter Description**:

- `$queue`: Prefix identifier for queue subscription
- `{topic}`: Original topic name

**Important Notes**:

- Shared subscription with groups allows creating multiple subscription groups, each group independently performs load balancing
- Shared subscription without groups (`$queue/`) is a special case where all subscribers are in one group
- Messages are published to the original topic, and subscribers receive messages through shared subscription topics

## Features of Shared Subscription

- **Load Balancing**: Messages are load-balanced and distributed within subscription groups
- **Group Isolation**: Different subscription groups are independent of each other
- **High Availability**: Supports dynamic client joining and leaving
- **Session Management**: Supports persistent and temporary sessions
- **QoS Support**: Supports MQTT QoS levels

## Shared Subscription and Sessions

When a client has a persistent session and subscribes to a shared subscription, the session will continue to receive messages published to the shared subscription topic when the client disconnects. If the client is disconnected for a long time and the message publishing rate is high, the internal message queue in the session state may overflow.

**Recommendations**:

- Use `clean_session=true` sessions for shared subscriptions
- When using MQTT v5, it is recommended to set short session expiration times
- When a session expires, unprocessed messages will be redistributed to other sessions in the same group

## Using Shared Subscription with MQTTX

### Using MQTTX CLI

1. **Create Shared Subscription with Groups**

   ```bash
   # Subscribers in group 1
   mqttx sub -t '$share/group1/sensor/data' -h '117.72.92.117' -p 1883 -v
   mqttx sub -t '$share/group1/sensor/data' -h '117.72.92.117' -p 1883 -v
   
   # Subscribers in group 2
   mqttx sub -t '$share/group2/sensor/data' -h '117.72.92.117' -p 1883 -v
   mqttx sub -t '$share/group2/sensor/data' -h '117.72.92.117' -p 1883 -v
   ```

2. **Create Shared Subscription without Groups**

   ```bash
   # Queue subscription
   mqttx sub -t '$queue/task/processing' -h '117.72.92.117' -p 1883 -v
   mqttx sub -t '$queue/task/processing' -h '117.72.92.117' -p 1883 -v
   mqttx sub -t '$queue/task/processing' -h '117.72.92.117' -p 1883 -v
   ```

3. **Publish Messages to Original Topic**

   ```bash
   # Publish to original topic
   mqttx pub -t 'sensor/data' -m '{"temperature":25.5,"humidity":60}' -h '117.72.92.117' -p 1883
   mqttx pub -t 'task/processing' -m '{"task_id":"T001","type":"analysis"}' -h '117.72.92.117' -p 1883
   ```

### Practical Application Examples

#### Sensor Data Processing

```bash
# Data processing group 1
mqttx sub -t '$share/processor1/sensor/temperature' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$share/processor1/sensor/temperature' -h '117.72.92.117' -p 1883 -v

# Data processing group 2
mqttx sub -t '$share/processor2/sensor/temperature' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$share/processor2/sensor/temperature' -h '117.72.92.117' -p 1883 -v

# Publish sensor data
mqttx pub -t 'sensor/temperature' -m '{"value":25.5,"timestamp":"2024-01-01T12:00:00Z"}' -h '117.72.92.117' -p 1883
```

#### Task Queue Processing

```bash
# Worker nodes subscribe to task queue
mqttx sub -t '$queue/job/queue' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$queue/job/queue' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$queue/job/queue' -h '117.72.92.117' -p 1883 -v

# Publish tasks to queue
mqttx pub -t 'job/queue' -m '{"job_id":"J001","type":"image_processing","data":"base64..."}' -h '117.72.92.117' -p 1883
mqttx pub -t 'job/queue' -m '{"job_id":"J002","type":"data_analysis","data":"csv_data"}' -h '117.72.92.117' -p 1883
```

#### Message Notification System

```bash
# Notification processing group A (high priority)
mqttx sub -t '$share/notify_high/notification/alert' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$share/notify_high/notification/alert' -h '117.72.92.117' -p 1883 -v

# Notification processing group B (normal priority)
mqttx sub -t '$share/notify_normal/notification/info' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$share/notify_normal/notification/info' -h '117.72.92.117' -p 1883 -v

# Publish different types of notifications
mqttx pub -t 'notification/alert' -m '{"level":"critical","message":"System overload"}' -h '117.72.92.117' -p 1883
mqttx pub -t 'notification/info' -m '{"level":"info","message":"Daily backup completed"}' -h '117.72.92.117' -p 1883
```

#### Log Processing System

```bash
# Log processing group
mqttx sub -t '$share/log_processor/application/logs' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$share/log_processor/application/logs' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$share/log_processor/application/logs' -h '117.72.92.117' -p 1883 -v

# Publish log messages
mqttx pub -t 'application/logs' -m '{"level":"INFO","message":"User login successful","user_id":"123"}' -h '117.72.92.117' -p 1883
mqttx pub -t 'application/logs' -m '{"level":"ERROR","message":"Database connection failed","error":"timeout"}' -h '117.72.92.117' -p 1883
```

## Differences Between Shared Subscription and Regular Subscription

| Feature | Regular Subscription | Shared Subscription |
|---------|---------------------|-------------------|
| Message Distribution | Broadcast to all subscribers | Load-balanced distribution to subscription groups |
| Topic Format | Original topic | $share/{group}/{topic} or $queue/{topic} |
| Processing Method | Each subscriber processes | Each message processed by one subscriber |
| Load Balancing | None | Yes |
| High Availability | Depends on client | Supports dynamic failover |
| Use Cases | Notifications, status sync | Task processing, message queuing |

## Load Balancing Strategies

Shared subscription supports multiple load balancing strategies:

- **Round Robin**: Distribute to subscribers in order
- **Random**: Randomly select subscribers
- **Least Connections**: Distribute to subscribers with the least connections
- **Hash**: Distribute based on message content hash value

## Important Notes

1. **Topic Format**: Must use correct shared subscription topic format
2. **Group Management**: Reasonably design subscription groups, avoid too few or too many subscribers in a group
3. **Session Management**: Recommend using `clean_session=true` to avoid message accumulation
4. **QoS Level**: Choose appropriate QoS level based on business requirements
5. **Error Handling**: Clients should properly handle message processing failures
6. **Monitoring and Alerting**: Monitor subscription group status and message processing
