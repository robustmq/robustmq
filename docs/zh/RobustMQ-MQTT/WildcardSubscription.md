# MQTT 通配符订阅

## 什么是 MQTT 通配符订阅？

MQTT 主题名称是用于消息路由的 UTF-8 编码字符串。为了提供更大的灵活性，MQTT 支持分层主题命名空间。主题通常按层级分级，并使用斜杠 `/` 在级别之间进行分隔，例如 `chat/room/1`。

通配符订阅允许客户端在主题名称中包含一个或多个通配符字符，通过主题过滤器匹配多个主题，从而实现一次订阅多个主题。RobustMQ 支持两种类型的通配符：单层通配符 `+` 和多层通配符 `#`。

## 何时使用 MQTT 通配符订阅？

通配符订阅适用于以下场景：

- **批量订阅**：需要订阅多个相关主题时，使用通配符可以简化订阅操作
- **动态主题**：当主题名称包含动态部分（如设备ID、房间号等）时
- **层级订阅**：需要订阅某个层级下的所有子主题时
- **系统监控**：监控系统中所有设备或特定类型设备的状态
- **消息聚合**：收集来自多个相关主题的消息进行统一处理
- **灵活路由**：根据业务需求灵活匹配和路由消息

## 通配符类型

### 单层通配符 `+`

`+`（U+002B）是一个通配符字符，仅匹配一个主题层级。单层通配符可以在主题过滤器的任何层级中使用，包括第一个和最后一个层级。

**使用规则**：

- 必须占据整个过滤器层级
- 可以在主题过滤器的多个层级中使用
- 可以与多层通配符结合使用

**有效示例**：

```text
"+" 有效
"sensor/+" 有效
"sensor/+/temperature" 有效
"sensor/+/+/data" 有效
```

**无效示例**：

```text
"sensor+" 无效 (没有占据整个层级)
"sensor/+/" 无效 (斜杠后为空)
```

### 多层通配符 `#`

`#`（U+0023）是一个通配符字符，匹配主题中的任意层级。多层通配符表示它的父级和任意数量的子层级。

**使用规则**：

- 必须占据整个层级
- 必须是主题的最后一个字符
- 可以匹配零个或多个层级

**有效示例**：

```text
"#" 有效，匹配所有主题
"sensor/#" 有效
"home/floor1/#" 有效
```

**无效示例**：

```text
"sensor/bedroom#" 无效 (没有占据整个层级)
"sensor/#/temperature" 无效 (不是主题最后一个字符)
```

## 通配符匹配示例

### 单层通配符匹配

如果客户端订阅主题 `sensor/+/temperature`，将会收到来自以下主题的消息：

```text
sensor/1/temperature
sensor/2/temperature
sensor/room1/temperature
sensor/device001/temperature
```

但是不会匹配以下主题：

```text
sensor/temperature          (缺少中间层级)
sensor/bedroom/1/temperature (中间层级过多)
```

### 多层通配符匹配

如果客户端订阅主题 `sensor/#`，将会收到以下主题的消息：

```text
sensor
sensor/temperature
sensor/1/temperature
sensor/bedroom/1/temperature
sensor/floor1/room1/device1/data
```

## 通过 MQTTX 使用通配符订阅

### 使用 MQTTX CLI

1. **单层通配符订阅**

   ```bash
   # 订阅所有房间的温度数据
   mqttx sub -t 'sensor/+/temperature' -h '117.72.92.117' -p 1883 -v
   
   # 订阅特定设备类型的所有数据
   mqttx sub -t 'device/+/status' -h '117.72.92.117' -p 1883 -v
   ```

2. **多层通配符订阅**

   ```bash
   # 订阅所有传感器数据
   mqttx sub -t 'sensor/#' -h '117.72.92.117' -p 1883 -v
   
   # 订阅所有主题
   mqttx sub -t '#' -h '117.72.92.117' -p 1883 -v
   ```

3. **发布消息测试匹配**

   ```bash
   # 发布到匹配单层通配符的主题
   mqttx pub -t 'sensor/room1/temperature' -m '{"value":25.5}' -h '117.72.92.117' -p 1883
   mqttx pub -t 'sensor/room2/temperature' -m '{"value":26.0}' -h '117.72.92.117' -p 1883
   
   # 发布到匹配多层通配符的主题
   mqttx pub -t 'sensor/humidity' -m '{"value":60}' -h '117.72.92.117' -p 1883
   mqttx pub -t 'sensor/floor1/room1/device1/data' -m '{"data":"test"}' -h '117.72.92.117' -p 1883
   ```

### 实际应用示例

#### 智能家居监控

```bash
# 订阅所有房间的传感器数据
mqttx sub -t 'home/+/sensor/#' -h '117.72.92.117' -p 1883 -v

# 发布不同房间的传感器数据
mqttx pub -t 'home/livingroom/sensor/temperature' -m '{"value":22.5}' -h '117.72.92.117' -p 1883
mqttx pub -t 'home/bedroom/sensor/humidity' -m '{"value":55}' -h '117.72.92.117' -p 1883
mqttx pub -t 'home/kitchen/sensor/light' -m '{"value":300}' -h '117.72.92.117' -p 1883
```

#### 工业设备监控

```bash
# 订阅所有生产线的设备状态
mqttx sub -t 'factory/+/line/+/device/status' -h '117.72.92.117' -p 1883 -v

# 发布不同生产线的设备状态
mqttx pub -t 'factory/plant1/line/1/device/machine1/status' -m '{"status":"running"}' -h '117.72.92.117' -p 1883
mqttx pub -t 'factory/plant1/line/2/device/conveyor/status' -m '{"status":"idle"}' -h '117.72.92.117' -p 1883
mqttx pub -t 'factory/plant2/line/1/device/robot/status' -m '{"status":"maintenance"}' -h '117.72.92.117' -p 1883
```

#### 车辆追踪系统

```bash
# 订阅所有车辆的位置信息
mqttx sub -t 'fleet/+/location' -h '117.72.92.117' -p 1883 -v

# 发布不同车辆的位置信息
mqttx pub -t 'fleet/truck001/location' -m '{"lat":39.9042,"lng":116.4074}' -h '117.72.92.117' -p 1883
mqttx pub -t 'fleet/van002/location' -m '{"lat":39.9052,"lng":116.4084}' -h '117.72.92.117' -p 1883
mqttx pub -t 'fleet/car003/location' -m '{"lat":39.9062,"lng":116.4094}' -h '117.72.92.117' -p 1883
```

#### 多租户系统

```bash
# 订阅特定租户的所有数据
mqttx sub -t 'tenant/user1/#' -h '117.72.92.117' -p 1883 -v

# 发布不同租户的数据
mqttx pub -t 'tenant/user1/sensor/data' -m '{"value":100}' -h '117.72.92.117' -p 1883
mqttx pub -t 'tenant/user1/device/status' -m '{"online":true}' -h '117.72.92.117' -p 1883
mqttx pub -t 'tenant/user1/notification/alert' -m '{"message":"System alert"}' -h '117.72.92.117' -p 1883
```

#### 日志收集系统

```bash
# 订阅所有应用的日志
mqttx sub -t 'logs/+/#' -h '117.72.92.117' -p 1883 -v

# 发布不同应用的日志
mqttx pub -t 'logs/webapp/error' -m '{"level":"ERROR","message":"Database connection failed"}' -h '117.72.92.117' -p 1883
mqttx pub -t 'logs/mobileapp/info' -m '{"level":"INFO","message":"User login successful"}' -h '117.72.92.117' -p 1883
mqttx pub -t 'logs/backend/debug' -m '{"level":"DEBUG","message":"Processing request"}' -h '117.72.92.117' -p 1883
```

## 通配符订阅与普通订阅的区别

| 特性 | 普通订阅 | 通配符订阅 |
|------|----------|------------|
| 主题匹配 | 精确匹配 | 模式匹配 |
| 订阅数量 | 一个主题一个订阅 | 一个订阅匹配多个主题 |
| 灵活性 | 低 | 高 |
| 性能影响 | 低 | 中等 |
| 使用场景 | 特定主题 | 批量主题 |
| 维护成本 | 高 | 低 |

## 通配符订阅的限制

1. **仅用于订阅**：通配符只能用于订阅，不能用于发布
2. **性能影响**：大量客户端使用通配符订阅可能对性能造成影响
3. **匹配规则**：必须遵循严格的匹配规则，否则订阅无效
4. **层级限制**：单层通配符只能匹配一个层级
5. **位置限制**：多层通配符必须是主题的最后一个字符

## 注意事项

1. **性能考虑**：避免在大量客户端上使用通配符订阅
2. **匹配精度**：确保通配符模式准确匹配所需主题
3. **层级设计**：合理设计主题层级结构，便于通配符使用
4. **测试验证**：充分测试通配符订阅的匹配效果
5. **监控告警**：监控通配符订阅的使用情况和性能影响
6. **安全控制**：确保通配符订阅不会意外订阅敏感主题
