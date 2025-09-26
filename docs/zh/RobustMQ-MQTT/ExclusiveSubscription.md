# MQTT 排他订阅

## 什么是 MQTT 排他订阅？

排他订阅是 MQTT 的扩展功能，允许对主题进行互斥订阅。一个主题同一时刻仅被允许存在一个订阅者，在当前订阅者未取消订阅前，其他订阅者都将无法订阅对应主题。

排他订阅的核心特性是**互斥性**：当一个客户端成功订阅了某个排他主题后，其他客户端就无法再订阅同一个主题，直到当前订阅者取消订阅为止。

## 何时使用 MQTT 排他订阅？

排他订阅适用于以下场景：

- **资源独占访问**：确保某个资源在同一时间只能被一个客户端访问
- **任务分配**：防止多个客户端同时处理同一个任务，避免重复处理
- **设备控制**：确保某个设备在同一时间只被一个控制器管理
- **数据同步**：保证数据更新的唯一性，避免并发冲突
- **负载均衡**：在多个客户端中选择一个来处理特定任务

## 排他订阅的语法规则

要进行排他订阅，您需要为主题名称添加 `$exclusive/` 前缀：

| 示例             | 前缀          | 真实主题名 |
| -------------- | ----------- | ----- |
| $exclusive/t/1 | $exclusive/ | t/1   |

**重要说明**：

- 排他订阅必须使用 `$exclusive/` 前缀
- 其他客户端依然可以通过原始主题名（如 `t/1`）进行普通订阅
- 排他订阅和普通订阅是相互独立的

## 排他订阅的特性

- **互斥性**：同一时间只能有一个客户端订阅排他主题
- **独占性**：排他订阅者独占该主题的消息接收
- **自动释放**：当排他订阅者断开连接时，排他订阅会自动释放
- **错误反馈**：订阅失败时会返回明确的错误码

## 订阅失败错误码

| 错误码  | 原因                        |
| ---- | ------------------------- |
| 0x8F | 使用了 $exclusive/，但并未开启排他订阅 |
| 0x97 | 已经有客户端订阅了该主题              |

## 通过 MQTTX 使用排他订阅

### 使用 MQTTX CLI

1. **第一个客户端进行排他订阅**

   ```bash
   mqttx sub -t "$exclusive/sensor/temperature" -h '117.72.92.117' -p 1883 -v
   ```

2. **第二个客户端尝试排他订阅同一主题**

   ```bash
   mqttx sub -t "$exclusive/sensor/temperature" -h '117.72.92.117' -p 1883 -v
   ```

   输出结果：

   ```text
   subscription negated to $exclusive/sensor/temperature with code 151
   ```

3. **发布消息到排他主题**

   ```bash
   mqttx pub -t "$exclusive/sensor/temperature" -m "25.5°C" -h '117.72.92.117' -p 1883
   ```

   只有第一个客户端会收到消息。

4. **取消排他订阅**

   断开第一个客户端连接，然后第二个客户端就可以成功订阅了。

### 实际应用示例

#### 设备控制排他订阅

```bash
# 控制器A独占控制设备
mqttx sub -t "$exclusive/device/robot/control" -h '117.72.92.117' -p 1883 -v

# 控制器B尝试控制同一设备（会失败）
mqttx sub -t "$exclusive/device/robot/control" -h '117.72.92.117' -p 1883 -v

# 发送控制命令
mqttx pub -t "$exclusive/device/robot/control" -m '{"action":"start","speed":50}' -h '117.72.92.117' -p 1883
```

#### 任务分配排他订阅

```bash
# 工作节点1获取任务
mqttx sub -t "$exclusive/task/processing" -h '117.72.92.117' -p 1883 -v

# 工作节点2尝试获取同一任务（会失败）
mqttx sub -t "$exclusive/task/processing" -h '117.72.92.117' -p 1883 -v

# 发布新任务
mqttx pub -t "$exclusive/task/processing" -m '{"task_id":"T001","type":"data_analysis"}' -h '117.72.92.117' -p 1883
```

#### 数据同步排他订阅

```bash
# 主节点独占数据更新权限
mqttx sub -t "$exclusive/database/update" -h '117.72.92.117' -p 1883 -v

# 备用节点尝试获取更新权限（会失败）
mqttx sub -t "$exclusive/database/update" -h '117.72.92.117' -p 1883 -v

# 发送数据更新
mqttx pub -t "$exclusive/database/update" -m '{"table":"users","operation":"insert","data":{}}' -h '117.72.92.117' -p 1883
```

#### 资源管理排他订阅

```bash
# 资源管理器1独占资源分配
mqttx sub -t "$exclusive/resource/allocation" -h '117.72.92.117' -p 1883 -v

# 资源管理器2尝试分配资源（会失败）
mqttx sub -t "$exclusive/resource/allocation" -h '117.72.92.117' -p 1883 -v

# 请求资源分配
mqttx pub -t "$exclusive/resource/allocation" -m '{"resource_type":"cpu","amount":4,"duration":3600}' -h '117.72.92.117' -p 1883
```

## 排他订阅与普通订阅的区别

| 特性 | 普通订阅 | 排他订阅 |
|------|----------|----------|
| 订阅者数量 | 多个 | 一个 |
| 主题前缀 | 无 | $exclusive/ |
| 消息分发 | 广播给所有订阅者 | 只发送给排他订阅者 |
| 订阅失败 | 不会失败 | 可能失败（错误码0x97） |
| 应用场景 | 一般消息分发 | 资源独占、任务分配 |

## 注意事项

1. **前缀要求**：排他订阅必须使用 `$exclusive/` 前缀
2. **功能开启**：需要确保服务器端已开启排他订阅功能
3. **连接管理**：客户端断开连接时，排他订阅会自动释放
4. **错误处理**：客户端应正确处理订阅失败的情况
5. **主题设计**：合理设计主题结构，避免冲突
