# MQTT 保留消息

## 什么是 MQTT 保留消息？

发布者发布消息时，如果 Retained 标记被设置为 true，则该消息即是 MQTT 中的保留消息（Retained Message）。MQTT 服务器会为每个主题存储最新一条保留消息，以方便消息发布后才上线的客户端在订阅主题时仍可以接收到该消息。当客户端订阅主题时，如果服务端存在该主题匹配的保留消息，则该保留消息将被立即发送给该客户端。

## 何时使用 MQTT 保留消息？

发布订阅模式虽然能让消息的发布者与订阅者充分解耦，但也存在一个缺点，即订阅者无法主动向发布者请求消息。订阅者何时收到消息完全依赖于发布者何时发布消息，这在某些场景中就产生了不便。

借助保留消息，新的订阅者能够立即获取最近的状态，而不需要等待无法预期的时间，例如：

- 智能家居设备的状态只有在变更时才会上报，但是控制端需要在上线后就能获取到设备的状态；
- 传感器上报数据的间隔太长，但是订阅者需要在订阅后立即获取到最新的数据；
- 传感器的版本号、序列号等不会经常变更的属性，可在上线后发布一条保留消息告知后续的所有订阅者。

## 保留消息的特性

- **唯一性**：每个主题只能存储一条保留消息，新发布的保留消息会覆盖之前的保留消息
- **持久性**：保留消息会一直保存在服务器中，直到被新的保留消息覆盖或手动删除
- **即时性**：新订阅者订阅主题时会立即收到该主题的保留消息
- **可清除**：可以通过发布空消息（payload为空）来清除保留消息

## 通过 MQTTX 发送保留消息给 RobustMQ

### 使用 MQTTX CLI

1. **发布保留消息**

   ```bash
   mqttx pub -t 'sensor/temperature' -m '25.5°C' --retain true -h '117.72.92.117' -p 1883
   ```

2. **订阅主题验证**

   ```bash
   mqttx sub -t 'sensor/temperature' -h '117.72.92.117' -p 1883 -v
   ```

   输出示例：

   ```text
   topic:  sensor/temperature
   payload:  25.5°C
   retain: true
   ```

3. **清除保留消息**

   ```bash
   mqttx pub -t 'sensor/temperature' -m '' --retain true -h '117.72.92.117' -p 1883
   ```

### 实际应用示例

#### 智能家居设备状态

```bash
# 发布设备状态保留消息
mqttx pub -t 'home/livingroom/light' -m '{"status":"on","brightness":80}' --retain true -h '117.72.92.117' -p 1883

# 新客户端订阅时会立即收到设备状态
mqttx sub -t 'home/+/light' -h '117.72.92.117' -p 1883 -v
```

#### 传感器数据

```bash
# 发布传感器数据保留消息
mqttx pub -t 'sensor/weather/temperature' -m '22.3' --retain true -h '117.72.92.117' -p 1883
mqttx pub -t 'sensor/weather/humidity' -m '65' --retain true -h '117.72.92.117' -p 1883

# 订阅所有传感器数据
mqttx sub -t 'sensor/weather/+' -h '117.72.92.117' -p 1883 -v
```

#### 设备信息

```bash
# 发布设备版本信息
mqttx pub -t 'device/camera/version' -m 'v2.1.0' --retain true -h '117.72.92.117' -p 1883
mqttx pub -t 'device/camera/serial' -m 'CAM001234' --retain true -h '117.72.92.117' -p 1883
```
