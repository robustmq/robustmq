# MQTT Topic Rewrite

## What is MQTT Topic Rewrite?

Many IoT devices do not support reconfiguration or upgrades, making it very difficult to modify device business topics. Topic rewrite functionality can help make such business upgrades easier: by setting up a set of rules for RobustMQ, it can change the original topic to a new target topic when subscribing and publishing.

Topic rewrite is an extended MQTT feature supported by RobustMQ, which allows dynamic modification of topic names during message publishing and subscription, enabling transparent topic conversion and flexible business logic adjustment.

## When to Use MQTT Topic Rewrite?

Topic rewrite is suitable for the following scenarios:

- **Device Compatibility**: Old devices use fixed topics and need to integrate with new systems
- **Business Upgrade**: Adjust topic structure without modifying client code
- **System Migration**: Migrate messages from old topic structure to new topic structure
- **Multi-tenant Support**: Provide personalized topic namespaces for different tenants
- **Protocol Adaptation**: Adapt to topic naming conventions of different vendor devices
- **Security Isolation**: Implement message routing and access control through topic rewrite

## Features of Topic Rewrite

- **Dynamic Rewrite**: Dynamically modify topic names at runtime
- **Rule Matching**: Support wildcard and regular expression matching
- **Multi-rule Support**: Can configure multiple rewrite rules
- **Direction Control**: Can separately control publishing and subscription rewriting
- **Variable Substitution**: Support using client information for topic construction
- **Compatibility**: Compatible with retained messages and delayed publishing features

## Topic Rewrite Rule Configuration

Topic rewrite rules consist of the following parts:

- **action**: Rule scope (`publish`, `subscribe`, `all`)
- **source_topic**: Source topic filter (supports wildcards)
- **dest_topic**: Target topic template
- **re**: Regular expression (used to extract topic information)

### Rule Types

- **publish rules**: Only match topics carried by PUBLISH packets
- **subscribe rules**: Only match topics carried by SUBSCRIBE, UNSUBSCRIBE packets
- **all rules**: Effective for PUBLISH, SUBSCRIBE, and UNSUBSCRIBE packets

### Variable Support

Target expressions support the following variables:

- `$N`: The Nth element extracted by regular expression
- `${clientid}`: Client ID
- `${username}`: Client username

## Using Topic Rewrite with MQTTX

### Using MQTTX CLI

1. **Configure Topic Rewrite Rules**

   Assume the following rewrite rules are configured:

   ```text
   Rule 1: y/+/z/# → y/z/$2 (regex: ^y/(.+)/z/(.+)$)
   Rule 2: x/# → z/y/x/$1 (regex: ^x/y/(.+)$)
   Rule 3: x/y/+ → z/y/$1 (regex: ^x/y/(\d+)$)
   ```

2. **Test Topic Rewrite**

   ```bash
   # Subscribe to topics that will be rewritten
   mqttx sub -t 'y/a/z/b' -h '117.72.92.117' -p 1883 -v
   # Actually subscribes to: y/z/b
   
   mqttx sub -t 'x/y/123' -h '117.72.92.117' -p 1883 -v
   # Actually subscribes to: z/y/123
   
   mqttx sub -t 'x/1/2' -h '117.72.92.117' -p 1883 -v
   # Actually subscribes to: x/1/2 (doesn't match regex)
   ```

3. **Publish to Rewritten Topics**

   ```bash
   # Publish to original topic, will be rewritten
   mqttx pub -t 'y/device1/z/data' -m '{"temperature":25.5}' -h '117.72.92.117' -p 1883
   # Actually publishes to: y/z/data
   
   mqttx pub -t 'x/y/sensor' -m '{"humidity":60}' -h '117.72.92.117' -p 1883
   # Actually publishes to: z/y/x/sensor
   ```

### Practical Application Examples

#### Device Topic Standardization

```bash
# Old device uses non-standard topic
mqttx pub -t 'device/001/temp' -m '{"value":25.5}' -h '117.72.92.117' -p 1883

# Configure rewrite rule: device/+/temp → sensors/$1/temperature
# Actually publishes to: sensors/001/temperature

# New system subscribes to standardized topic
mqttx sub -t 'sensors/+/temperature' -h '117.72.92.117' -p 1883 -v
```

#### Multi-tenant Topic Isolation

```bash
# Tenant A's device publishes message
mqttx pub -t 'tenantA/sensor/data' -m '{"value":30}' -h '117.72.92.117' -p 1883

# Configure rewrite rule: tenantA/# → ${username}/tenantA/#
# Actually publishes to: user1/tenantA/sensor/data

# Tenant A subscribes to their own topic
mqttx sub -t 'user1/tenantA/#' -h '117.72.92.117' -p 1883 -v
```

#### Protocol Version Upgrade

```bash
# Old version device uses v1 topic
mqttx pub -t 'v1/sensor/temperature' -m '{"temp":25}' -h '117.72.92.117' -p 1883

# Configure rewrite rule: v1/# → v2/legacy/#
# Actually publishes to: v2/legacy/sensor/temperature

# New version system subscribes to v2 topic
mqttx sub -t 'v2/legacy/#' -h '117.72.92.117' -p 1883 -v
```

#### Geographic Location Topic Routing

```bash
# Device publishes by geographic location
mqttx pub -t 'beijing/sensor/air' -m '{"pm25":50}' -h '117.72.92.117' -p 1883

# Configure rewrite rule: beijing/# → region/north/beijing/#
# Actually publishes to: region/north/beijing/sensor/air

# Regional monitoring system subscribes
mqttx sub -t 'region/north/#' -h '117.72.92.117' -p 1883 -v
```

#### Device Type Classification

```bash
# Different types of devices publish messages
mqttx pub -t 'device/001/status' -m '{"online":true}' -h '117.72.92.117' -p 1883
mqttx pub -t 'device/002/status' -m '{"online":true}' -h '117.72.92.117' -p 1883

# Configure rewrite rule: device/+/status → devices/${clientid}/status
# Actually publishes to: devices/device001/status, devices/device002/status

# Device management system subscribes
mqttx sub -t 'devices/+/status' -h '117.72.92.117' -p 1883 -v
```

## Topic Rewrite and Shared Subscription

When topic rewrite acts on client subscription/unsubscription of shared subscription topics, it only affects the actual topic. That is, it only affects the part after removing the prefix `$share/<group-name>/` or `$queue` from the shared subscription topic.

For example:

- When client subscribes to `$share/group/t/1`, only tries to match and rewrite `t/1`
- When client subscribes to `$queue/t/2`, only tries to match and rewrite `t/2`

```bash
# Shared subscription topic rewrite example
mqttx sub -t '$share/group1/y/a/z/b' -h '117.72.92.117' -p 1883 -v
# Actually subscribes to: $share/group1/y/z/b (rewrite rule acts on y/a/z/b)

mqttx sub -t '$queue/x/y/123' -h '117.72.92.117' -p 1883 -v
# Actually subscribes to: $queue/z/y/123 (rewrite rule acts on x/y/123)
```

## Topic Rewrite Rule Priority

When multiple rules match the same topic, RobustMQ processes them with the following priority:

1. **Configuration Order**: Execute rules in the order they appear in the configuration file
2. **First Match**: Use the first matching rule for rewriting
3. **Regex Match**: If regular expression doesn't match, rewrite fails and won't try other rules

## Integration with Retained Messages and Delayed Publishing

Topic rewrite can be combined with retained messages and delayed publishing features:

```bash
# Delayed publishing combined with topic rewrite
mqttx pub -t '$delayed/10/y/device/data' -m '{"delayed":true}' -h '117.72.92.117' -p 1883
# Actually publishes to: $delayed/10/y/z/data (rewrite rule acts on y/device/data)

# Retained message combined with topic rewrite
mqttx pub -t 'x/y/config' -m '{"retain":true}' --retain -h '117.72.92.117' -p 1883
# Actually publishes to: z/y/x/config (rewrite rule acts on x/y/config)
```

## Important Notes

1. **ACL Check**: Publish/subscribe authorization check is executed before topic rewrite
2. **Rule Order**: Rules are executed in configuration order, first matching rule takes effect
3. **Regex Match**: When regular expression doesn't match, rewrite fails
4. **Shared Subscription**: Topic rewrite for shared subscription only affects the actual topic part
5. **Performance Impact**: Each topic needs to match all rules, affecting performance
6. **Compatibility**: Ensure rewritten topics are compatible with existing systems
