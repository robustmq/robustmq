# MQTT Exclusive Subscription

## What are MQTT Exclusive Subscriptions?

Exclusive subscription is an extended feature of MQTT that allows mutually exclusive subscriptions to topics. Only one subscriber is allowed to exist for a topic at the same time. Other subscribers will be unable to subscribe to the corresponding topic until the current subscriber unsubscribes.

The core feature of exclusive subscription is **mutual exclusivity**: when a client successfully subscribes to an exclusive topic, other clients cannot subscribe to the same topic until the current subscriber unsubscribes.

## When to use MQTT Exclusive Subscriptions?

Exclusive subscriptions are suitable for the following scenarios:

- **Exclusive resource access**: Ensure that a resource can only be accessed by one client at a time
- **Task assignment**: Prevent multiple clients from processing the same task simultaneously, avoiding duplicate processing
- **Device control**: Ensure that a device is managed by only one controller at a time
- **Data synchronization**: Guarantee the uniqueness of data updates, avoiding concurrency conflicts
- **Load balancing**: Select one client among multiple clients to handle specific tasks

## Syntax Rules for Exclusive Subscriptions

To perform an exclusive subscription, you need to add the `$exclusive/` prefix to the topic name:

| Example             | Prefix        | Real Topic Name |
| ------------------- | ------------- | --------------- |
| $exclusive/t/1      | $exclusive/   | t/1             |

**Important Notes**:

- Exclusive subscriptions must use the `$exclusive/` prefix
- Other clients can still perform regular subscriptions through the original topic name (e.g., `t/1`)
- Exclusive subscriptions and regular subscriptions are independent of each other

## Characteristics of Exclusive Subscriptions

- **Mutual exclusivity**: Only one client can subscribe to an exclusive topic at a time
- **Exclusivity**: Exclusive subscribers exclusively receive messages for that topic
- **Automatic release**: When an exclusive subscriber disconnects, the exclusive subscription is automatically released
- **Error feedback**: Clear error codes are returned when subscription fails

## Subscription Failure Error Codes

| Error Code | Reason                                    |
| ---------- | ----------------------------------------- |
| 0x8F       | Used $exclusive/ but exclusive subscription is not enabled |
| 0x97       | Another client has already subscribed to this topic |

## Using Exclusive Subscriptions via MQTTX

### Using MQTTX CLI

1. **First client performs exclusive subscription**

   ```bash
   mqttx sub -t "$exclusive/sensor/temperature" -h '117.72.92.117' -p 1883 -v
   ```

2. **Second client attempts to exclusively subscribe to the same topic**

   ```bash
   mqttx sub -t "$exclusive/sensor/temperature" -h '117.72.92.117' -p 1883 -v
   ```

   Output result:

   ```text
   subscription negated to $exclusive/sensor/temperature with code 151
   ```

3. **Publish message to exclusive topic**

   ```bash
   mqttx pub -t "$exclusive/sensor/temperature" -m "25.5°C" -h '117.72.92.117' -p 1883
   ```

   Only the first client will receive the message.

4. **Cancel exclusive subscription**

   Disconnect the first client, then the second client can successfully subscribe.

### Practical Application Examples

#### Device Control Exclusive Subscription

```bash
# Controller A exclusively controls device
mqttx sub -t "$exclusive/device/robot/control" -h '117.72.92.117' -p 1883 -v

# Controller B attempts to control the same device (will fail)
mqttx sub -t "$exclusive/device/robot/control" -h '117.72.92.117' -p 1883 -v

# Send control command
mqttx pub -t "$exclusive/device/robot/control" -m '{"action":"start","speed":50}' -h '117.72.92.117' -p 1883
```

#### Task Assignment Exclusive Subscription

```bash
# Worker node 1 acquires task
mqttx sub -t "$exclusive/task/processing" -h '117.72.92.117' -p 1883 -v

# Worker node 2 attempts to acquire the same task (will fail)
mqttx sub -t "$exclusive/task/processing" -h '117.72.92.117' -p 1883 -v

# Publish new task
mqttx pub -t "$exclusive/task/processing" -m '{"task_id":"T001","type":"data_analysis"}' -h '117.72.92.117' -p 1883
```

#### Data Synchronization Exclusive Subscription

```bash
# Master node exclusively owns data update permission
mqttx sub -t "$exclusive/database/update" -h '117.72.92.117' -p 1883 -v

# Backup node attempts to acquire update permission (will fail)
mqttx sub -t "$exclusive/database/update" -h '117.72.92.117' -p 1883 -v

# Send data update
mqttx pub -t "$exclusive/database/update" -m '{"table":"users","operation":"insert","data":{}}' -h '117.72.92.117' -p 1883
```

#### Resource Management Exclusive Subscription

```bash
# Resource manager 1 exclusively owns resource allocation
mqttx sub -t "$exclusive/resource/allocation" -h '117.72.92.117' -p 1883 -v

# Resource manager 2 attempts to allocate resources (will fail)
mqttx sub -t "$exclusive/resource/allocation" -h '117.72.92.117' -p 1883 -v

# Request resource allocation
mqttx pub -t "$exclusive/resource/allocation" -m '{"resource_type":"cpu","amount":4,"duration":3600}' -h '117.72.92.117' -p 1883
```

## Differences Between Exclusive and Regular Subscriptions

| Feature | Regular Subscription | Exclusive Subscription |
|---------|---------------------|------------------------|
| Number of subscribers | Multiple | One |
| Topic prefix | None | $exclusive/ |
| Message distribution | Broadcast to all subscribers | Only sent to exclusive subscriber |
| Subscription failure | Never fails | May fail (error code 0x97) |
| Use cases | General message distribution | Resource exclusivity, task assignment |

## Important Notes

1. **Prefix requirement**: Exclusive subscriptions must use the `$exclusive/` prefix
2. **Feature enablement**: Ensure that exclusive subscription functionality is enabled on the server side
3. **Connection management**: When a client disconnects, the exclusive subscription is automatically released
4. **Error handling**: Clients should properly handle subscription failure situations
5. **Topic design**: Design topic structures reasonably to avoid conflicts

