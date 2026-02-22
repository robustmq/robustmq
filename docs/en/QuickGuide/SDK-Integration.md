# SDK Integration

RobustMQ is fully compatible with MQTT 3.x / 5.0. Any standard community MQTT SDK works out of the box — no custom adapters needed.

## Connection Details

| Parameter | Value |
|-----------|-------|
| Host | `localhost` (local) or public server `117.72.92.117` |
| Port | `1883` (TCP) / `8083` (WebSocket) |
| Username | `admin` |
| Password | `robustmq` |

## SDKs by Language

| Language | SDK | Install |
|----------|-----|---------|
| **Go** | [eclipse/paho.mqtt.golang](https://github.com/eclipse/paho.mqtt.golang) | `go get github.com/eclipse/paho.mqtt.golang` |
| **Java** | [eclipse/paho.mqtt.java](https://github.com/eclipse/paho.mqtt.java) | Maven / Gradle |
| **Python** | [eclipse/paho-mqtt-python](https://github.com/eclipse/paho.mqtt.python) | `pip install paho-mqtt` |
| **JavaScript** | [MQTT.js](https://github.com/mqttjs/MQTT.js) | `npm install mqtt` |
| **C** | [eclipse/paho.mqtt.c](https://github.com/eclipse/paho.mqtt.c) | Build from source |

## Quick Example

The following Go snippet shows the minimal connect / publish / subscribe flow:

```go
import mqtt "github.com/eclipse/paho.mqtt.golang"

opts := mqtt.NewClientOptions().
    AddBroker("tcp://localhost:1883").
    SetClientID("my-client").
    SetUsername("admin").
    SetPassword("robustmq")

client := mqtt.NewClient(opts)
client.Connect().Wait()

// Publish
client.Publish("test/topic", 0, false, "Hello RobustMQ!")

// Subscribe
client.Subscribe("test/topic", 0, func(_ mqtt.Client, msg mqtt.Message) {
    fmt.Println(string(msg.Payload()))
})
```

## Full Documentation

For complete examples covering QoS, SSL/TLS, will messages, and more:

- [Go SDK](../RobustMQ-MQTT/SDK/go-sdk.md)
- [Java SDK](../RobustMQ-MQTT/SDK/java-sdk.md)
- [Python SDK](../RobustMQ-MQTT/SDK/python-sdk.md)
- [JavaScript SDK](../RobustMQ-MQTT/SDK/javascript-sdk.md)
- [C SDK](../RobustMQ-MQTT/SDK/c-sdk.md)
