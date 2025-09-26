# MQTT Session Persistence

## What is MQTT Session Persistence?

MQTT Session Persistence is a feature that allows MQTT clients to maintain their session state and message queues even when they disconnect from the broker. This ensures that messages sent to offline clients are preserved and delivered when they reconnect.

## When to Use MQTT Session Persistence?

Session persistence is particularly useful in scenarios where:

- **Reliable Message Delivery**: You need to ensure that messages are not lost when clients are temporarily offline
- **Intermittent Connectivity**: Clients operate in environments with unstable network connections
- **Mobile Applications**: Mobile devices frequently switch between networks or enter sleep mode
- **IoT Devices**: IoT devices may have limited power and intermittent connectivity
- **Critical Business Applications**: Applications where message loss is unacceptable

## Features of Session Persistence

### Default Persistence in RobustMQ

Unlike many MQTT brokers, **RobustMQ MQTT enables session persistence by default**. Session data is automatically stored in the Meta Service, providing:

- **Automatic Session Management**: No additional configuration required
- **High Availability**: Session data is replicated across Meta Service nodes
- **Persistent Message Queues**: Messages are stored until clients reconnect
- **Session State Preservation**: Subscription information and QoS settings are maintained

### Key Characteristics

- **Clean Start = false**: Clients must set `Clean Start = false` to maintain persistent sessions
- **Client ID Requirement**: Persistent sessions require a unique client ID
- **Message Retention**: Messages are retained according to QoS levels and retention policies
- **Automatic Recovery**: Sessions are automatically restored after broker restarts

## Using Session Persistence with MQTTX

### 1. Connect with Persistent Session

Use MQTTX CLI to connect with a persistent session by setting `Clean Start = false`:

```bash
mqttx sub -t 'sensor/temperature' -i 'robustmq_client' --no-clean -h '117.72.92.117' -p 1883
```

### 2. Disconnect and Verify Session Persistence

Disconnect the client. The session will be preserved in RobustMQ's Meta Service. You can verify this through the RobustMQ Dashboard at [http://117.72.92.117:8080/](http://117.72.92.117:8080/).

### 3. Send Messages to Offline Client

While the client is offline, publish messages to the subscribed topic:

```bash
mqttx pub -t 'sensor/temperature' -m '{"temperature": 25.5, "humidity": 60}' -h '117.72.92.117' -p 1883
```

### 4. Reconnect and Receive Queued Messages

Reconnect using the same client ID and `--no-clean` option:

```bash
mqttx sub -t 'sensor/temperature' -i 'robustmq_client' --no-clean -h '117.72.92.117' -p 1883
```

The client will receive all messages that were published while it was offline:

```
[2024-12-19] [10:30:15] › …  Connecting...
[2024-12-19] [10:30:15] › ✔  Connected
[2024-12-19] [10:30:15] › …  Subscribing to sensor/temperature...
[2024-12-19] [10:30:15] › ✔  Subscribed to sensor/temperature
[2024-12-19] [10:30:15] › payload: {"temperature": 25.5, "humidity": 60}
```