# Experience RobustMQ MQTT

## Prerequisite: Start the Broker

Follow the [Quick Install](Quick-Install.md) guide to install RobustMQ, then start the service:

```bash
robust-server start
```

Verify the broker is running:

```bash
robust-ctl status
```

---

## Choose an MQTT Client

Use either of the following to test MQTT publish and subscribe.

### Option 1: MQTTX CLI

MQTTX CLI is an open-source MQTT command-line client by EMQ. See the installation guide at: [https://mqttx.app/zh/docs/cli](https://mqttx.app/zh/docs/cli)

Once installed, the `mqttx` command is available.

### Option 2: robust-bench (built-in)

`robust-bench` is RobustMQ's built-in benchmarking tool — no extra installation needed. See the full usage guide: [Bench CLI Documentation](../Bench/Bench-CLI.md)

---

## Publish and Subscribe

### Using MQTTX

Open two terminals and run:

```bash
# Terminal 1: Subscribe
mqttx sub -h localhost -p 1883 -t "test/topic"

# Terminal 2: Publish
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"
```

If the subscriber receives the message, MQTT is working correctly.

Common options:

```bash
# Specify QoS
mqttx pub -h localhost -p 1883 -t "test/qos1" -m "msg" -q 1

# Publish a retained message
mqttx pub -h localhost -p 1883 -t "test/retained" -m "retained msg" -r

# Wildcard subscription
mqttx sub -h localhost -p 1883 -t "test/#"
```

### Using robust-bench

```bash
# Connection benchmark (establish 100 connections)
robust-bench mqtt conn --count 100

# Publish benchmark (100 clients, 30 seconds)
robust-bench mqtt pub --count 100 --duration-secs 30

# Subscribe benchmark (100 clients)
robust-bench mqtt sub --count 100 --duration-secs 30
```

For more options see the [Bench CLI Documentation](../Bench/Bench-CLI.md).

---

## SDK Integration

RobustMQ is fully compatible with MQTT 3.x / 5.0. Any standard community MQTT SDK works out of the box — no custom adapters needed.

### Connection Details

| Parameter | Value |
|-----------|-------|
| Host | `localhost` (local) or public server `117.72.92.117` |
| Port | `1883` (TCP) / `8083` (WebSocket) |
| Username | `admin` |
| Password | `robustmq` |

### SDKs by Language

| Language | SDK | Install |
|----------|-----|---------|
| **Go** | [eclipse/paho.mqtt.golang](https://github.com/eclipse/paho.mqtt.golang) | `go get github.com/eclipse/paho.mqtt.golang` |
| **Java** | [eclipse/paho.mqtt.java](https://github.com/eclipse/paho.mqtt.java) | Maven / Gradle |
| **Python** | [eclipse/paho-mqtt-python](https://github.com/eclipse/paho.mqtt.python) | `pip install paho-mqtt` |
| **JavaScript** | [MQTT.js](https://github.com/mqttjs/MQTT.js) | `npm install mqtt` |
| **C** | [eclipse/paho.mqtt.c](https://github.com/eclipse/paho.mqtt.c) | Build from source |

### Quick Example

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

### Full Documentation

For complete examples covering QoS, SSL/TLS, will messages, and more:

- [Go SDK](../RobustMQ-MQTT/SDK/go-sdk.md)
- [Java SDK](../RobustMQ-MQTT/SDK/java-sdk.md)
- [Python SDK](../RobustMQ-MQTT/SDK/python-sdk.md)
- [JavaScript SDK](../RobustMQ-MQTT/SDK/javascript-sdk.md)
- [C SDK](../RobustMQ-MQTT/SDK/c-sdk.md)
