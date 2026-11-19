# MQTT 会话持久化

## 什么是 MQTT 会话持久化？

MQTT 会话持久化允许客户端会话跨越一次网络连接继续存在。RobustMQ 在处理 CONNECT 和 DISCONNECT 事件时将会话记录写入 Meta Service；客户端使用相同的客户端 ID 重新连接时，可以恢复仍然有效的会话。

会话是否保留由 MQTT 协议标志和过期属性控制。`durable_sessions_enable` 配置不会覆盖这些协议规则，它会被写入会话元数据并用于连接事件；离线消息存储则由 `[mqtt_runtime.offline_message]` 独立控制。

## 何时使用 MQTT 会话持久化？

会话持久化在以下场景中特别有用：

- **可靠消息投递**：需要确保客户端临时离线时消息不会丢失
- **间歇性连接**：客户端在不稳定网络环境中运行
- **移动应用程序**：移动设备经常在网络间切换或进入睡眠模式
- **物联网设备**：物联网设备可能功率有限且连接不稳定
- **关键业务应用**：消息丢失不可接受的应用

## 会话持久化的特性

### RobustMQ 如何保留会话

RobustMQ 在会话处理过程中将会话状态写入 Meta Service。断开连接后是否保留，取决于协议版本：

- **MQTT 3.x**：`Clean Session = 1` 时，断开连接会删除会话；`Clean Session = 0` 时，会话会保留以便后续重新连接。
- **MQTT 5.0**：`Clean Start = 1` 时创建新会话；断开时 Session Expiry Interval 为 `0` 会删除会话，非 `0` 则会在过期前保留。如果 DISCONNECT 未携带该属性，则使用会话中保存的过期时间。
- **离线消息**：离线客户端的消息由 `[mqtt_runtime.offline_message]` 独立控制，不能仅根据会话配置推断消息一定会保留。

Meta Service 是 Broker 共享会话存储的位置，因此可用性和复制行为取决于 Meta Service 的部署，而不是某个本地 MQTT 开关。

### 关键特性

- **Clean Start / Clean Session**：希望恢复已有会话时，MQTT 5.0 应使用 `Clean Start = false`，MQTT 3.x/3.1.1 应使用 `Clean Session = false`
- **客户端 ID 要求**：持久会话需要唯一的客户端 ID
- **会话过期**：MQTT 5.0 会话只在配置的过期时间内有效
- **自动恢复**：客户端使用相同租户和客户端 ID 重新连接时，仍有效的会话会被加载

## 通过 MQTTX 使用会话持久化

### 1. 使用持久会话连接

使用 MQTTX CLI 通过设置 `Clean Start = false` 连接持久会话：

```bash
mqttx sub -t 'sensor/temperature' -i 'robustmq_client' --no-clean -h '117.72.92.117' -p 1883
```

### 2. 断开连接并验证会话持久化

断开客户端连接。由于示例使用了 `Clean Session = false`，会话会根据过期时间保持可恢复状态。您可以通过 [http://demo.robustmq.com/](http://demo.robustmq.com/) 的 RobustMQ Dashboard 验证会话状态。

### 3. 向离线客户端发送消息

在客户端离线时，向订阅的主题发布消息：

```bash
mqttx pub -t 'sensor/temperature' -m '{"temperature": 25.5, "humidity": 60}' -h '117.72.92.117' -p 1883
```

### 4. 重新连接并接收队列消息

使用相同的客户端 ID 和 `--no-clean` 选项重新连接：

```bash
mqttx sub -t 'sensor/temperature' -i 'robustmq_client' --no-clean -h '117.72.92.117' -p 1883
```

如果启用了离线消息存储，且消息符合离线投递条件，客户端将接收离线期间发布的消息：

```
[2024-12-19] [10:30:15] › …  正在连接...
[2024-12-19] [10:30:15] › ✔  已连接
[2024-12-19] [10:30:15] › …  正在订阅 sensor/temperature...
[2024-12-19] [10:30:15] › ✔  已订阅 sensor/temperature
[2024-12-19] [10:30:15] › payload: {"temperature": 25.5, "humidity": 60}
```
