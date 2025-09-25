# MQTT 遗嘱消息

## 什么是 MQTT 遗嘱消息？

在现实世界中，一个人可以制定一份遗嘱，声明在他去世后应该如何分配他的财产以及应该采取什么行动。在他去世后，遗嘱执行人会将这份遗嘱公开，并执行遗嘱中的指示。

在 MQTT 中，客户端可以在连接时在服务端中注册一个遗嘱消息，与普通消息类似，我们可以设置遗嘱消息的主题、有效载荷等等。当该客户端意外断开连接，服务端就会向其他订阅了相应主题的客户端发送此遗嘱消息。这些接收者也因此可以及时地采取行动，例如向用户发送通知、切换备用设备等等。

## 何时使用 MQTT 遗嘱消息？

假设我们有一个传感器监控一个很少变化的值，普通的实现是定期发布最新数值，但更好的实现是仅在数值发生变化时以保留消息的形式发送它。这使得任何新的订阅者总能立即获得当前值，而不必等待传感器再一次发布。不过订阅者也因此没有办法根据是否及时收到消息来判断传感器是否离线。借助遗嘱消息，我们可以立即得知传感器保持活动超时，而且不必总是获取传感器发布的值。

遗嘱消息的典型应用场景包括：

- **设备离线检测**：当设备意外断开连接时，立即通知相关系统
- **状态监控**：监控关键设备的在线状态，确保系统正常运行
- **故障处理**：当设备故障时，自动触发备用设备或告警机制
- **资源管理**：及时释放离线设备占用的资源

## 遗嘱消息的特性

- **自动触发**：当客户端异常断开连接时，服务器自动发送遗嘱消息
- **可配置性**：可以设置遗嘱消息的主题、内容、QoS 级别和保留标志
- **即时性**：客户端断开连接后，遗嘱消息会立即发送给相关订阅者
- **可靠性**：即使客户端突然掉线，也能确保重要状态信息被传递

## 通过 MQTTX 发送遗嘱消息给 RobustMQ

### 使用 MQTTX CLI

1. **连接并设置遗嘱消息**

   ```bash
   mqttx conn -h '117.72.92.117' -p 1883 --will-topic 'device/offline' --will-message 'Device disconnected unexpectedly'
   ```

2. **订阅遗嘱消息主题**

   ```bash
   mqttx sub -t 'device/offline' -h '117.72.92.117' -p 1883 -v
   ```

3. **断开连接触发遗嘱消息**

   当第一个客户端连接断开时，订阅者会收到遗嘱消息：

   ```text
   topic:  device/offline
   payload:  Device disconnected unexpectedly
   ```

### 实际应用示例

#### 设备状态监控

```bash
# 设备连接时设置遗嘱消息
mqttx conn -h '117.72.92.117' -p 1883 --will-topic 'sensor/status' --will-message '{"device_id":"sensor001","status":"offline","timestamp":"2024-01-01T12:00:00Z"}'

# 监控系统订阅设备状态
mqttx sub -t 'sensor/status' -h '117.72.92.117' -p 1883 -v
```

#### 智能家居设备

```bash
# 智能门锁连接时设置遗嘱消息
mqttx conn -h '117.72.92.117' -p 1883 --will-topic 'home/security/doorlock' --will-message '{"device":"doorlock","status":"offline","alert":"Device disconnected"}'

# 安全系统订阅门锁状态
mqttx sub -t 'home/security/+' -h '117.72.92.117' -p 1883 -v
```

#### 工业设备监控

```bash
# 工业传感器连接时设置遗嘱消息
mqttx conn -h '117.72.92.117' -p 1883 --will-topic 'factory/machine/status' --will-message '{"machine_id":"M001","status":"offline","location":"Production Line A"}'

# 监控中心订阅设备状态
mqttx sub -t 'factory/machine/status' -h '117.72.92.117' -p 1883 -v
```

#### 车辆追踪系统

```bash
# 车辆设备连接时设置遗嘱消息
mqttx conn -h '117.72.92.117' -p 1883 --will-topic 'vehicle/tracking' --will-message '{"vehicle_id":"V001","status":"offline","last_location":"GPS coordinates"}'

# 调度中心订阅车辆状态
mqttx sub -t 'vehicle/tracking' -h '117.72.92.117' -p 1883 -v
```

## 遗嘱消息配置参数

- **遗嘱主题 (Will Topic)**：遗嘱消息发布到的主题
- **遗嘱消息 (Will Message)**：遗嘱消息的内容
- **遗嘱 QoS (Will QoS)**：遗嘱消息的服务质量级别 (0, 1, 2)
- **遗嘱保留标志 (Will Retain)**：是否将遗嘱消息设置为保留消息
- **遗嘱延迟时间 (Will Delay Interval)**：遗嘱消息发送的延迟时间（MQTT 5.0 特性）
