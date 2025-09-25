# MQTT Will Messages

## What are MQTT Will Messages?

In the real world, a person can create a will that declares how their property should be distributed and what actions should be taken after their death. After their death, the executor of the will will make the will public and execute the instructions in the will.

In MQTT, a client can register a will message on the server when connecting. Similar to ordinary messages, we can set the topic, payload, etc. of the will message. When the client unexpectedly disconnects, the server will send this will message to other clients who have subscribed to the corresponding topic. These recipients can therefore take timely action, such as sending notifications to users, switching to backup devices, etc.

## When to use MQTT Will Messages?

Suppose we have a sensor monitoring a value that rarely changes. The ordinary implementation is to periodically publish the latest value, but a better implementation is to send it as a retained message only when the value changes. This allows any new subscriber to always get the current value immediately without waiting for the sensor to publish again. However, subscribers therefore have no way to determine whether the sensor is offline based on whether they receive messages in time. With will messages, we can immediately know when the sensor keep-alive times out, and we don't have to always get the values published by the sensor.

Typical application scenarios for will messages include:

- **Device offline detection**: Immediately notify related systems when devices unexpectedly disconnect
- **Status monitoring**: Monitor the online status of critical devices to ensure normal system operation
- **Fault handling**: Automatically trigger backup devices or alarm mechanisms when devices fail
- **Resource management**: Timely release resources occupied by offline devices

## Characteristics of Will Messages

- **Automatic triggering**: Server automatically sends will messages when clients abnormally disconnect
- **Configurability**: Can set will message topic, content, QoS level, and retain flag
- **Immediacy**: Will messages are sent immediately to relevant subscribers after client disconnection
- **Reliability**: Ensures important status information is delivered even when clients suddenly go offline

## Sending Will Messages to RobustMQ via MQTTX

### Using MQTTX CLI

1. **Connect and Set Will Message**

   ```bash
   mqttx conn -h '117.72.92.117' -p 1883 --will-topic 'device/offline' --will-message 'Device disconnected unexpectedly'
   ```

2. **Subscribe to Will Message Topic**

   ```bash
   mqttx sub -t 'device/offline' -h '117.72.92.117' -p 1883 -v
   ```

3. **Disconnect to Trigger Will Message**

   When the first client connection is disconnected, subscribers will receive the will message:

   ```text
   topic:  device/offline
   payload:  Device disconnected unexpectedly
   ```

### Practical Application Examples

#### Device Status Monitoring

```bash
# Set will message when device connects
mqttx conn -h '117.72.92.117' -p 1883 --will-topic 'sensor/status' --will-message '{"device_id":"sensor001","status":"offline","timestamp":"2024-01-01T12:00:00Z"}'

# Monitoring system subscribes to device status
mqttx sub -t 'sensor/status' -h '117.72.92.117' -p 1883 -v
```

#### Smart Home Devices

```bash
# Set will message when smart door lock connects
mqttx conn -h '117.72.92.117' -p 1883 --will-topic 'home/security/doorlock' --will-message '{"device":"doorlock","status":"offline","alert":"Device disconnected"}'

# Security system subscribes to door lock status
mqttx sub -t 'home/security/+' -h '117.72.92.117' -p 1883 -v
```

#### Industrial Equipment Monitoring

```bash
# Set will message when industrial sensor connects
mqttx conn -h '117.72.92.117' -p 1883 --will-topic 'factory/machine/status' --will-message '{"machine_id":"M001","status":"offline","location":"Production Line A"}'

# Monitoring center subscribes to equipment status
mqttx sub -t 'factory/machine/status' -h '117.72.92.117' -p 1883 -v
```

#### Vehicle Tracking System

```bash
# Set will message when vehicle device connects
mqttx conn -h '117.72.92.117' -p 1883 --will-topic 'vehicle/tracking' --will-message '{"vehicle_id":"V001","status":"offline","last_location":"GPS coordinates"}'

# Dispatch center subscribes to vehicle status
mqttx sub -t 'vehicle/tracking' -h '117.72.92.117' -p 1883 -v
```

## Will Message Configuration Parameters

- **Will Topic**: The topic where the will message is published
- **Will Message**: The content of the will message
- **Will QoS**: The quality of service level for the will message (0, 1, 2)
- **Will Retain**: Whether to set the will message as a retained message
- **Will Delay Interval**: The delay time for sending the will message (MQTT 5.0 feature)
