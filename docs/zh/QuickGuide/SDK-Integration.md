# SDK 接入

RobustMQ 完整兼容 MQTT 3.x / 5.0 协议，使用任意社区标准 MQTT SDK 即可直接接入，无需额外适配。

## 连接信息

| 参数 | 值 |
|------|----|
| Host | `localhost`（本地）或公共服务器 `117.72.92.117` |
| Port | `1883`（TCP）/ `8083`（WebSocket）|
| 用户名 | `admin` |
| 密码 | `robustmq` |

## 各语言 SDK

| 语言 | SDK | 安装 |
|------|-----|------|
| **Go** | [eclipse/paho.mqtt.golang](https://github.com/eclipse/paho.mqtt.golang) | `go get github.com/eclipse/paho.mqtt.golang` |
| **Java** | [eclipse/paho.mqtt.java](https://github.com/eclipse/paho.mqtt.java) | Maven / Gradle 引入 |
| **Python** | [eclipse/paho-mqtt-python](https://github.com/eclipse/paho.mqtt.python) | `pip install paho-mqtt` |
| **JavaScript** | [MQTT.js](https://github.com/mqttjs/MQTT.js) | `npm install mqtt` |
| **C** | [eclipse/paho.mqtt.c](https://github.com/eclipse/paho.mqtt.c) | 源码编译 |

## 快速示例

以下以 Go 为例，展示最简连接、发布、订阅流程：

```go
import mqtt "github.com/eclipse/paho.mqtt.golang"

opts := mqtt.NewClientOptions().
    AddBroker("tcp://localhost:1883").
    SetClientID("my-client").
    SetUsername("admin").
    SetPassword("robustmq")

client := mqtt.NewClient(opts)
client.Connect().Wait()

// 发布
client.Publish("test/topic", 0, false, "Hello RobustMQ!")

// 订阅
client.Subscribe("test/topic", 0, func(_ mqtt.Client, msg mqtt.Message) {
    fmt.Println(string(msg.Payload()))
})
```

## 详细文档

各语言完整示例（连接、发布、订阅、QoS、SSL、遗嘱消息等）请参考：

- [Go SDK](../RobustMQ-MQTT/SDK/go-sdk.md)
- [Java SDK](../RobustMQ-MQTT/SDK/java-sdk.md)
- [Python SDK](../RobustMQ-MQTT/SDK/python-sdk.md)
- [JavaScript SDK](../RobustMQ-MQTT/SDK/javascript-sdk.md)
- [C SDK](../RobustMQ-MQTT/SDK/c-sdk.md)
