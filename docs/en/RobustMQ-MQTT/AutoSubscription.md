# MQTT Auto Subscription

## What is MQTT Auto Subscription?

Auto subscription is an extended MQTT feature supported by RobustMQ. Auto subscription allows you to configure multiple rules for RobustMQ, which will automatically subscribe to specified topics for devices after they successfully connect, without requiring additional subscription requests.

Auto subscription enables automatic topic subscription for clients upon connection, eliminating the need for manual subscription requests and simplifying client implementation.

## When to Use MQTT Auto Subscription?

Auto subscription is particularly useful in the following scenarios:

- **Simplified Client Implementation**: Clients don't need to handle subscription logic, reducing code complexity
- **Consistent Topic Access**: Ensure all connected clients automatically have access to specific topics
- **Device Management**: Automatically subscribe devices to status or control topics upon connection
- **System Integration**: Seamlessly integrate new devices into existing topic structures
- **Reduced Network Traffic**: Eliminate the need for separate subscription requests

## Features of Auto Subscription

- **Automatic Subscription**: Clients are automatically subscribed to configured topics upon connection
- **Dynamic Topic Construction**: Support for placeholders to create dynamic topics based on client information
- **Configurable Parameters**: Set QoS levels, retain flags, and other subscription parameters
- **Multiple Rules**: Configure multiple auto subscription rules for different scenarios
- **Client-Specific Topics**: Create unique topics for each client using placeholders

## Auto Subscription Configuration

Auto subscription supports the following configuration parameters:

- **Topic**: The topic pattern to automatically subscribe to (supports placeholders)
- **QoS**: Quality of Service level (0, 1, or 2)
- **No Local**: Whether to exclude messages published by the same client
- **Retain As Published**: Whether to retain messages published with this subscription
- **Retain Handling**: How to handle retained messages (0, 1, or 2)

## Placeholders

Auto subscription supports placeholders to dynamically construct topics. Placeholder format is `${}`. Supported variables include:

- `${clientid}`: Client ID
- `${username}`: Client username
- `${host}`: IP address of the client connection

For example, when client ID is `device001` and the configured topic is `sensor/${clientid}/data`, the client will automatically subscribe to topic `sensor/device001/data` upon connection.

## Using Auto Subscription with MQTTX

### Using MQTTX CLI

1. **Connect with Auto Subscription**

   ```bash
   # Connect with client ID (auto subscription will be applied based on configured rules)
   mqttx conn -i device001 -h '117.72.92.117' -p 1883
   ```

2. **Publish to Auto-Subscribed Topics**

   ```bash
   # Publish to a topic that clients are auto-subscribed to
   mqttx pub -t 'sensor/device001/data' -m '{"temperature":25.5,"humidity":60}' -h '117.72.92.117' -p 1883
   ```

3. **Verify Auto Subscription**

   ```bash
   # Connect another client to verify auto subscription
   mqttx conn -i device002 -h '117.72.92.117' -p 1883
   # This client will automatically receive messages published to its auto-subscribed topics
   ```

### Practical Application Examples

#### Device Status Monitoring

```bash
# Device connects and is auto-subscribed to status topic
mqttx conn -i sensor001 -h '117.72.92.117' -p 1883

# System publishes status request to device-specific topic
mqttx pub -t 'device/sensor001/status/request' -m '{"action":"get_status"}' -h '117.72.92.117' -p 1883

# Device automatically receives the request and can respond
mqttx pub -t 'device/sensor001/status/response' -m '{"status":"online","battery":85}' -h '117.72.92.117' -p 1883
```

#### Smart Home Device Control

```bash
# Smart light connects and is auto-subscribed to control topic
mqttx conn -i light001 -h '117.72.92.117' -p 1883

# Home automation system sends control commands
mqttx pub -t 'home/light001/control' -m '{"action":"turn_on","brightness":80,"color":"white"}' -h '117.72.92.117' -p 1883

# Light automatically receives and processes the command
```

#### Industrial Equipment Management

```bash
# Machine connects and is auto-subscribed to command topic
mqttx conn -i machine001 -h '117.72.92.117' -p 1883

# Control system sends operational commands
mqttx pub -t 'factory/machine001/command' -m '{"operation":"start","speed":1000,"mode":"production"}' -h '117.72.92.117' -p 1883

# Machine automatically receives and executes the command
```

#### Fleet Management System

```bash
# Vehicle connects and is auto-subscribed to dispatch topic
mqttx conn -i vehicle001 -h '117.72.92.117' -p 1883

# Dispatch center sends route assignments
mqttx pub -t 'fleet/vehicle001/dispatch' -m '{"route":"Route_A","pickup":"Location_1","delivery":"Location_2"}' -h '117.72.92.117' -p 1883

# Vehicle automatically receives the dispatch information
```

#### IoT Sensor Network

```bash
# Multiple sensors connect with different client IDs
mqttx conn -i temp_sensor_001 -h '117.72.92.117' -p 1883
mqttx conn -i humidity_sensor_001 -h '117.72.92.117' -p 1883
mqttx conn -i pressure_sensor_001 -h '117.72.92.117' -p 1883

# Data collection system sends configuration to each sensor
mqttx pub -t 'sensor/temp_sensor_001/config' -m '{"interval":30,"threshold":25}' -h '117.72.92.117' -p 1883
mqttx pub -t 'sensor/humidity_sensor_001/config' -m '{"interval":60,"threshold":70}' -h '117.72.92.117' -p 1883
mqttx pub -t 'sensor/pressure_sensor_001/config' -m '{"interval":45,"threshold":1013}' -h '117.72.92.117' -p 1883
```

## Auto Subscription vs Manual Subscription

| Feature | Auto Subscription | Manual Subscription |
|---------|------------------|-------------------|
| Setup | Configured on server side | Client must send SUBSCRIBE packet |
| Complexity | Simple client implementation | Requires subscription logic in client |
| Flexibility | Server-controlled | Client-controlled |
| Network Traffic | Reduced (no SUBSCRIBE packets) | Additional SUBSCRIBE packets required |
| Dynamic Topics | Supported via placeholders | Client must construct topics |
| Error Handling | Server manages subscription | Client must handle subscription errors |

## Best Practices

1. **Topic Design**: Use meaningful topic patterns with placeholders for dynamic topics
2. **QoS Selection**: Choose appropriate QoS levels based on message importance
3. **Client ID Strategy**: Use consistent client ID naming conventions for predictable topic construction
4. **Resource Management**: Monitor auto subscription usage to avoid resource exhaustion
5. **Security**: Ensure proper authentication and authorization for auto-subscribed topics
6. **Testing**: Test auto subscription rules with various client scenarios

## Configuration Examples

### Basic Auto Subscription

```text
Topic: sensor/data
QoS: 1
Description: All clients automatically subscribe to sensor data topic
```

### Client-Specific Auto Subscription

```text
Topic: device/${clientid}/commands
QoS: 2
Description: Each client automatically subscribes to its own command topic
```

### User-Based Auto Subscription

```text
Topic: user/${username}/notifications
QoS: 1
Description: Clients automatically subscribe to user-specific notification topics
```

### Host-Based Auto Subscription

```text
Topic: location/${host}/status
QoS: 0
Description: Clients automatically subscribe to location-specific status topics
```

## Important Notes

1. **Topic Format**: Use correct placeholder syntax `${variable}` for dynamic topics
2. **Client ID**: Ensure client IDs are unique and follow naming conventions
3. **QoS Levels**: Choose appropriate QoS levels based on message delivery requirements
4. **Resource Limits**: Monitor the number of auto subscriptions to prevent resource exhaustion
5. **Security**: Implement proper access control for auto-subscribed topics
6. **Testing**: Thoroughly test auto subscription rules before deploying to production
