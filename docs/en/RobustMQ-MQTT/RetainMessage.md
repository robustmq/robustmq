# MQTT Retained Messages

## What are MQTT Retained Messages?

When a publisher publishes a message, if the Retained flag is set to true, then the message is a Retained Message in MQTT. The MQTT server stores the latest retained message for each topic, so that clients that come online after the message is published can still receive the message when they subscribe to the topic. When a client subscribes to a topic, if there exists a retained message matching the topic on the server, the retained message will be sent to the client immediately.

## When to use MQTT Retained Messages?

Although the publish-subscribe model can decouple the publisher from the subscriber, it also has the disadvantage that the subscriber cannot actively request messages from the publisher. When the subscriber receives the message is completely dependent on when the publisher publishes the message, which is inconvenient in some scenarios.

With retained messages, new subscribers are able to get the most recent status immediately, without having to wait an unpredictable amount of time, such as:

- The status of smart home devices will only be reported when they change, but the controller needs to be able to access the device status after coming online;
- The interval between sensors reporting data is too long, but the subscriber needs to get the latest data immediately after subscribing;
- Sensor version numbers, serial numbers, and other attributes that do not change frequently can be published as a retained message to inform all subsequent subscribers after coming online.

## Characteristics of Retained Messages

- **Uniqueness**: Each topic can only store one retained message, new retained messages will overwrite previous ones
- **Persistence**: Retained messages are stored in the server until overwritten by new retained messages or manually deleted
- **Immediacy**: New subscribers will immediately receive retained messages when subscribing to a topic
- **Clearable**: Can be cleared by publishing empty messages (empty payload)

## Sending Retained Messages to RobustMQ via MQTTX

### Using MQTTX CLI

1. **Publish Retained Message**

   ```bash
   mqttx pub -t 'sensor/temperature' -m '25.5°C' --retain true -h '117.72.92.117' -p 1883
   ```

2. **Subscribe to Topic for Verification**

   ```bash
   mqttx sub -t 'sensor/temperature' -h '117.72.92.117' -p 1883 -v
   ```

   Output example:

   ```text
   topic:  sensor/temperature
   payload:  25.5°C
   retain: true
   ```

3. **Clear Retained Message**

   ```bash
   mqttx pub -t 'sensor/temperature' -m '' --retain true -h '117.72.92.117' -p 1883
   ```

### Practical Application Examples

#### Smart Home Device Status

```bash
# Publish device status retained message
mqttx pub -t 'home/livingroom/light' -m '{"status":"on","brightness":80}' --retain true -h '117.72.92.117' -p 1883

# New clients will immediately receive device status when subscribing
mqttx sub -t 'home/+/light' -h '117.72.92.117' -p 1883 -v
```

#### Sensor Data

```bash
# Publish sensor data retained messages
mqttx pub -t 'sensor/weather/temperature' -m '22.3' --retain true -h '117.72.92.117' -p 1883
mqttx pub -t 'sensor/weather/humidity' -m '65' --retain true -h '117.72.92.117' -p 1883

# Subscribe to all sensor data
mqttx sub -t 'sensor/weather/+' -h '117.72.92.117' -p 1883 -v
```

#### Device Information

```bash
# Publish device version information
mqttx pub -t 'device/camera/version' -m 'v2.1.0' --retain true -h '117.72.92.117' -p 1883
mqttx pub -t 'device/camera/serial' -m 'CAM001234' --retain true -h '117.72.92.117' -p 1883
```
