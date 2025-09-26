# MQTT Wildcard Subscription

## What is MQTT Wildcard Subscription?

MQTT topic names are UTF-8 encoded strings used for message routing. To provide greater flexibility, MQTT supports hierarchical topic namespaces. Topics are typically organized hierarchically and separated by slashes `/` between levels, for example `chat/room/1`.

Wildcard subscription allows clients to include one or more wildcard characters in topic names, matching multiple topics through topic filters, thereby enabling subscription to multiple topics with a single subscription. RobustMQ supports two types of wildcards: single-level wildcard `+` and multi-level wildcard `#`.

## When to Use MQTT Wildcard Subscription?

Wildcard subscription is suitable for the following scenarios:

- **Batch Subscription**: When you need to subscribe to multiple related topics, wildcards can simplify subscription operations
- **Dynamic Topics**: When topic names contain dynamic parts (such as device IDs, room numbers, etc.)
- **Hierarchical Subscription**: When you need to subscribe to all sub-topics under a certain level
- **System Monitoring**: Monitor the status of all devices or specific types of devices in the system
- **Message Aggregation**: Collect messages from multiple related topics for unified processing
- **Flexible Routing**: Flexibly match and route messages based on business requirements

## Wildcard Types

### Single-Level Wildcard `+`

`+` (U+002B) is a wildcard character that matches only one topic level. Single-level wildcards can be used at any level of the topic filter, including the first and last levels.

**Usage Rules**:

- Must occupy the entire filter level
- Can be used at multiple levels of the topic filter
- Can be combined with multi-level wildcards

**Valid Examples**:

```text
"+" valid
"sensor/+" valid
"sensor/+/temperature" valid
"sensor/+/+/data" valid
```

**Invalid Examples**:

```text
"sensor+" invalid (doesn't occupy entire level)
"sensor/+/" invalid (empty after slash)
```

### Multi-Level Wildcard `#`

`#` (U+0023) is a wildcard character that matches any number of levels in a topic. Multi-level wildcards represent their parent level and any number of child levels.

**Usage Rules**:

- Must occupy the entire level
- Must be the last character of the topic
- Can match zero or more levels

**Valid Examples**:

```text
"#" valid, matches all topics
"sensor/#" valid
"home/floor1/#" valid
```

**Invalid Examples**:

```text
"sensor/bedroom#" invalid (doesn't occupy entire level)
"sensor/#/temperature" invalid (not the last character of topic)
```

## Wildcard Matching Examples

### Single-Level Wildcard Matching

If a client subscribes to topic `sensor/+/temperature`, it will receive messages from the following topics:

```text
sensor/1/temperature
sensor/2/temperature
sensor/room1/temperature
sensor/device001/temperature
```

But will not match the following topics:

```text
sensor/temperature          (missing middle level)
sensor/bedroom/1/temperature (too many middle levels)
```

### Multi-Level Wildcard Matching

If a client subscribes to topic `sensor/#`, it will receive messages from the following topics:

```text
sensor
sensor/temperature
sensor/1/temperature
sensor/bedroom/1/temperature
sensor/floor1/room1/device1/data
```

## Using Wildcard Subscription with MQTTX

### Using MQTTX CLI

1. **Single-Level Wildcard Subscription**

   ```bash
   # Subscribe to temperature data from all rooms
   mqttx sub -t 'sensor/+/temperature' -h '117.72.92.117' -p 1883 -v
   
   # Subscribe to all data from specific device types
   mqttx sub -t 'device/+/status' -h '117.72.92.117' -p 1883 -v
   ```

2. **Multi-Level Wildcard Subscription**

   ```bash
   # Subscribe to all sensor data
   mqttx sub -t 'sensor/#' -h '117.72.92.117' -p 1883 -v
   
   # Subscribe to all topics
   mqttx sub -t '#' -h '117.72.92.117' -p 1883 -v
   ```

3. **Publish Messages to Test Matching**

   ```bash
   # Publish to topics matching single-level wildcard
   mqttx pub -t 'sensor/room1/temperature' -m '{"value":25.5}' -h '117.72.92.117' -p 1883
   mqttx pub -t 'sensor/room2/temperature' -m '{"value":26.0}' -h '117.72.92.117' -p 1883
   
   # Publish to topics matching multi-level wildcard
   mqttx pub -t 'sensor/humidity' -m '{"value":60}' -h '117.72.92.117' -p 1883
   mqttx pub -t 'sensor/floor1/room1/device1/data' -m '{"data":"test"}' -h '117.72.92.117' -p 1883
   ```

### Practical Application Examples

#### Smart Home Monitoring

```bash
# Subscribe to sensor data from all rooms
mqttx sub -t 'home/+/sensor/#' -h '117.72.92.117' -p 1883 -v

# Publish sensor data from different rooms
mqttx pub -t 'home/livingroom/sensor/temperature' -m '{"value":22.5}' -h '117.72.92.117' -p 1883
mqttx pub -t 'home/bedroom/sensor/humidity' -m '{"value":55}' -h '117.72.92.117' -p 1883
mqttx pub -t 'home/kitchen/sensor/light' -m '{"value":300}' -h '117.72.92.117' -p 1883
```

#### Industrial Device Monitoring

```bash
# Subscribe to device status from all production lines
mqttx sub -t 'factory/+/line/+/device/status' -h '117.72.92.117' -p 1883 -v

# Publish device status from different production lines
mqttx pub -t 'factory/plant1/line/1/device/machine1/status' -m '{"status":"running"}' -h '117.72.92.117' -p 1883
mqttx pub -t 'factory/plant1/line/2/device/conveyor/status' -m '{"status":"idle"}' -h '117.72.92.117' -p 1883
mqttx pub -t 'factory/plant2/line/1/device/robot/status' -m '{"status":"maintenance"}' -h '117.72.92.117' -p 1883
```

#### Vehicle Tracking System

```bash
# Subscribe to location information from all vehicles
mqttx sub -t 'fleet/+/location' -h '117.72.92.117' -p 1883 -v

# Publish location information from different vehicles
mqttx pub -t 'fleet/truck001/location' -m '{"lat":39.9042,"lng":116.4074}' -h '117.72.92.117' -p 1883
mqttx pub -t 'fleet/van002/location' -m '{"lat":39.9052,"lng":116.4084}' -h '117.72.92.117' -p 1883
mqttx pub -t 'fleet/car003/location' -m '{"lat":39.9062,"lng":116.4094}' -h '117.72.92.117' -p 1883
```

#### Multi-tenant System

```bash
# Subscribe to all data from specific tenant
mqttx sub -t 'tenant/user1/#' -h '117.72.92.117' -p 1883 -v

# Publish data from different tenants
mqttx pub -t 'tenant/user1/sensor/data' -m '{"value":100}' -h '117.72.92.117' -p 1883
mqttx pub -t 'tenant/user1/device/status' -m '{"online":true}' -h '117.72.92.117' -p 1883
mqttx pub -t 'tenant/user1/notification/alert' -m '{"message":"System alert"}' -h '117.72.92.117' -p 1883
```

#### Log Collection System

```bash
# Subscribe to logs from all applications
mqttx sub -t 'logs/+/#' -h '117.72.92.117' -p 1883 -v

# Publish logs from different applications
mqttx pub -t 'logs/webapp/error' -m '{"level":"ERROR","message":"Database connection failed"}' -h '117.72.92.117' -p 1883
mqttx pub -t 'logs/mobileapp/info' -m '{"level":"INFO","message":"User login successful"}' -h '117.72.92.117' -p 1883
mqttx pub -t 'logs/backend/debug' -m '{"level":"DEBUG","message":"Processing request"}' -h '117.72.92.117' -p 1883
```

## Differences Between Wildcard Subscription and Regular Subscription

| Feature | Regular Subscription | Wildcard Subscription |
|---------|---------------------|----------------------|
| Topic Matching | Exact matching | Pattern matching |
| Subscription Count | One topic per subscription | One subscription matches multiple topics |
| Flexibility | Low | High |
| Performance Impact | Low | Medium |
| Use Cases | Specific topics | Batch topics |
| Maintenance Cost | High | Low |

## Limitations of Wildcard Subscription

1. **Subscription Only**: Wildcards can only be used for subscription, not for publishing
2. **Performance Impact**: Large numbers of clients using wildcard subscriptions may impact performance
3. **Matching Rules**: Must follow strict matching rules, otherwise subscription is invalid
4. **Level Limitations**: Single-level wildcards can only match one level
5. **Position Limitations**: Multi-level wildcards must be the last character of the topic

## Important Notes

1. **Performance Considerations**: Avoid using wildcard subscriptions on large numbers of clients
2. **Matching Precision**: Ensure wildcard patterns accurately match required topics
3. **Hierarchy Design**: Reasonably design topic hierarchy structure for easy wildcard usage
4. **Testing and Validation**: Thoroughly test the matching effects of wildcard subscriptions
5. **Monitoring and Alerting**: Monitor the usage and performance impact of wildcard subscriptions
6. **Security Control**: Ensure wildcard subscriptions don't accidentally subscribe to sensitive topics
