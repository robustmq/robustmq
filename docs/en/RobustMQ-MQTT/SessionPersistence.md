# MQTT Session Persistence

## What is MQTT Session Persistence?

MQTT session persistence allows a client session to outlive one network connection. RobustMQ stores session records in the Meta Service while processing CONNECT and DISCONNECT events, and can restore an eligible session when the client reconnects with the same client ID.

Session persistence is controlled by the MQTT protocol flags and expiry properties. The `durable_sessions_enable` configuration value does not override those protocol rules; it is copied into session metadata used by connection events. Offline-message storage is a separate setting controlled by `[mqtt_runtime.offline_message]`.

## When to Use MQTT Session Persistence?

Session persistence is particularly useful in scenarios where:

- **Reliable Message Delivery**: You need to ensure that messages are not lost when clients are temporarily offline
- **Intermittent Connectivity**: Clients operate in environments with unstable network connections
- **Mobile Applications**: Mobile devices frequently switch between networks or enter sleep mode
- **IoT Devices**: IoT devices may have limited power and intermittent connectivity
- **Critical Business Applications**: Applications where message loss is unacceptable

## Features of Session Persistence

### How RobustMQ Retains Sessions

RobustMQ writes session state to the Meta Service during session processing. Whether that state is retained after disconnect depends on the protocol version:

- **MQTT 3.x**: `Clean Session = 1` deletes the session on disconnect. `Clean Session = 0` keeps it for a later reconnect.
- **MQTT 5.0**: `Clean Start = 1` starts a new session. At disconnect, a Session Expiry Interval of `0` deletes the session; a non-zero interval keeps it until it expires. If the DISCONNECT packet omits the property, the stored session expiry interval is used.
- **Offline messages**: Messages for an offline client are controlled independently by `[mqtt_runtime.offline_message]`. They are not implied by the session flag alone.

The Meta Service provides the broker's shared session storage. Replication and availability therefore follow the Meta Service deployment rather than a local MQTT-only switch.

### Key Characteristics

- **Clean Start / Clean Session**: Clients must use `Clean Start = false` (MQTT 5.0) or `Clean Session = false` (MQTT 3.x/3.1.1) when they want an existing session to be resumed
- **Client ID Requirement**: Persistent sessions require a unique client ID
- **Session Expiry**: MQTT 5.0 sessions remain available only for their configured expiry interval
- **Automatic Recovery**: An eligible stored session is loaded when the client reconnects with the same tenant and client ID

## Using Session Persistence with MQTTX

### 1. Connect with Persistent Session

Use MQTTX CLI to connect with a persistent session by setting `Clean Start = false`:

```bash
mqttx sub -t 'sensor/temperature' -i 'robustmq_client' --no-clean -h '117.72.92.117' -p 1883
```

### 2. Disconnect and Verify Session Persistence

Disconnect the client. Because the example uses `Clean Session = false`, the session remains eligible for recovery according to its expiry interval. You can verify the session state through the RobustMQ Dashboard at [http://demo.robustmq.com/](http://demo.robustmq.com/).

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

If offline-message storage is enabled and the messages are eligible for offline delivery, the client will receive messages published while it was offline:

```
[2024-12-19] [10:30:15] › …  Connecting...
[2024-12-19] [10:30:15] › ✔  Connected
[2024-12-19] [10:30:15] › …  Subscribing to sensor/temperature...
[2024-12-19] [10:30:15] › ✔  Subscribed to sensor/temperature
[2024-12-19] [10:30:15] › payload: {"temperature": 25.5, "humidity": 60}
```
