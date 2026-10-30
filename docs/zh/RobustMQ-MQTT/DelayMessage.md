# MQTT 延迟发布

## 什么是 MQTT 延迟发布？

延迟发布是 RobustMQ MQTT 支持的 MQTT 扩展功能。当客户端使用特殊主题前缀 `$delayed/{DelayInterval}` 发布消息时，将触发延迟发布功能，可以实现按照用户配置的时间间隔延迟发布消息。

延迟发布的核心特性是**时间控制**：消息不会立即发布到目标主题，而是在指定的延迟时间后才会被发布，实现了消息的定时发送功能。

## 何时使用 MQTT 延迟发布？

延迟发布适用于以下场景：

- **定时任务**：需要在特定时间执行的任务，如定时提醒、定时报告
- **重试机制**：失败后的延迟重试，避免立即重试造成的系统压力
- **批量处理**：将多个消息延迟到同一时间点进行批量处理
- **流量控制**：通过延迟发布来控制消息发送的速率，避免系统过载
- **业务逻辑**：需要等待一定时间后才能执行的操作，如冷却期、等待期

## 延迟发布的语法规则

延迟发布主题的具体格式如下：

```text
$delayed/{DelayInterval}/{TopicName}
```

**参数说明**：

- `$delayed`：使用 `$delayed` 作为主题前缀的消息都将被视为需要延迟发布的消息
- `{DelayInterval}`：以秒为单位的整型延迟时间
- `{TopicName}`：MQTT 消息的目标主题名称

**重要说明**：

- 如果 `{DelayInterval}` 无法被解析为一个整型数字，服务器将丢弃该消息
- 延迟时间不能超过 `42,949,669` 秒（约 497 天）
- 仅当启用了离线消息存储或目标主题存在订阅者时，延迟消息才会进入队列；如果禁用了离线消息存储且没有订阅者，发布请求会被拒绝

## 延迟发布的特性

- **精确时间控制**：可以精确控制消息的发布时间
- **持久化存储**：被接受的延迟消息会持久化保存，并在服务器重启后恢复
- **高可靠性**：支持 QoS 级别，确保消息的可靠传递
- **灵活配置**：支持任意主题的延迟发布

当前延迟投递提供的是**至少一次投递**语义。Broker 会先将消息写入目标主题，
再删除延迟消息记录。如果 Broker 在目标主题写入成功、删除延迟消息记录之前停止，
恢复流程可能会再次投递同一条消息。因此，消费者应当以幂等方式处理延迟消息。
原子化投递清理和 Broker 端去重属于后续独立的改进项。

## 通过 MQTTX 使用延迟发布

### 使用 MQTTX CLI

1. **发布延迟消息**

   ```bash
   mqttx pub -t '$delayed/10/sensor/temperature' -m "25.5°C" -h '117.72.92.117' -p 1883
   ```

   这条消息将在 10 秒后发布到 `/sensor/temperature` 主题。

2. **订阅目标主题**

   ```bash
   mqttx sub -t "/sensor/temperature" -h '117.72.92.117' -p 1883 -v
   ```

   等待 10 秒后，订阅者会收到延迟发布的消息。

3. **发布不同延迟时间的消息**

   ```bash
   # 15 秒后发布
   mqttx pub -t '$delayed/15/device/status' -m "online" -h '117.72.92.117' -p 1883

   # 60 秒后发布
   mqttx pub -t '$delayed/60/system/backup' -m "backup completed" -h '117.72.92.117' -p 1883

   # 3600 秒（1小时）后发布
   mqttx pub -t '$delayed/3600/report/daily' -m "daily report ready" -h '117.72.92.117' -p 1883
   ```

### 实际应用示例

#### 定时提醒系统

```bash
# 设置 30 秒后的提醒
mqttx pub -t '$delayed/30/reminder/meeting' -m '{"title":"会议提醒","time":"14:00","location":"会议室A"}' -h '117.72.92.117' -p 1883

# 设置 1 小时后的提醒
mqttx pub -t '$delayed/3600/reminder/break' -m '{"title":"休息提醒","message":"该休息了"}' -h '117.72.92.117' -p 1883

# 订阅提醒主题
mqttx sub -t "/reminder/+" -h '117.72.92.117' -p 1883 -v
```

#### Broker 端投递重试

```bash
# 设置 5 秒后将消息投递到 /upload
mqttx pub -t '$delayed/5/upload' -m '{"file_id":"123"}' -h '117.72.92.117' -p 1883

# 订阅目标主题
mqttx sub -t "/upload" -h '117.72.92.117' -p 1883 -v
```

如果 Broker 向目标主题写入失败，会自动重试投递到同一个目标主题。最大重试次数和
指数退避间隔由 `[delay_message]` 配置控制。默认情况下，初次投递失败后最多重试
3 次，间隔约为 2、4、8 秒。如果所有尝试都失败，Broker 会删除延迟消息，不会将其
发送到死信主题。此重试机制处理的是 Broker 端的投递失败，不会重试订阅者报告的
应用层处理失败。重试次数会持久化保存，因此 Broker 重启不会重置配置的最大重试次数。
剩余退避时长不会持久化，Broker 重启恢复时可能会立即重试尚未执行的投递。

#### 批量处理

```bash
# 将多个消息延迟到同一时间点（10分钟后）
mqttx pub -t '$delayed/600/batch/process' -m '{"batch_id":"001","type":"data_analysis"}' -h '117.72.92.117' -p 1883
mqttx pub -t '$delayed/600/batch/process' -m '{"batch_id":"002","type":"data_analysis"}' -h '117.72.92.117' -p 1883
mqttx pub -t '$delayed/600/batch/process' -m '{"batch_id":"003","type":"data_analysis"}' -h '117.72.92.117' -p 1883

# 订阅批量处理主题
mqttx sub -t "/batch/process" -h '117.72.92.117' -p 1883 -v
```

#### 流量控制

```bash
# 控制消息发送速率，每 2 秒发送一条消息
mqttx pub -t '$delayed/2/rate/limited' -m "message 1" -h '117.72.92.117' -p 1883
mqttx pub -t '$delayed/4/rate/limited' -m "message 2" -h '117.72.92.117' -p 1883
mqttx pub -t '$delayed/6/rate/limited' -m "message 3" -h '117.72.92.117' -p 1883

# 订阅限流主题
mqttx sub -t "/rate/limited" -h '117.72.92.117' -p 1883 -v
```

#### 业务逻辑延迟

```bash
# 设备冷却期，30 分钟后才能重新启动
mqttx pub -t '$delayed/1800/device/cooldown' -m '{"device_id":"D001","action":"restart_allowed"}' -h '117.72.92.117' -p 1883

# 用户操作等待期，5 分钟后才能执行敏感操作
mqttx pub -t '$delayed/300/user/security' -m '{"user_id":"U001","operation":"sensitive_allowed"}' -h '117.72.92.117' -p 1883

# 订阅业务逻辑主题
mqttx sub -t "/device/+" -h '117.72.92.117' -p 1883 -v
mqttx sub -t "/user/+" -h '117.72.92.117' -p 1883 -v
```

## 延迟时间示例

| 延迟时间 | 示例主题 | 说明 |
|----------|----------|------|
| 15 秒 | `$delayed/15/sensor/data` | 15 秒后发布到 `/sensor/data` |
| 60 秒 | `$delayed/60/system/status` | 1 分钟后发布到 `/system/status` |
| 3600 秒 | `$delayed/3600/report/hourly` | 1 小时后发布到 `/report/hourly` |
| 86400 秒 | `$delayed/86400/report/daily` | 1 天后发布到 `/report/daily` |

## 注意事项

1. **时间格式**：延迟值必须是以秒为单位的整数，从 Broker 接受消息的时间开始计算
2. **主题格式**：必须使用 `$delayed/{DelayInterval}/{TopicName}` 格式
3. **消息持久化**：启用离线消息存储或目标主题存在订阅者时，延迟消息会被持久化。如果禁用了离线消息存储且没有订阅者，发布请求会被拒绝，消息不会进入调度队列
4. **QoS 支持**：延迟发布支持 MQTT 的 QoS 级别。当该策略拒绝消息时，MQTT 5 的 QoS 1 和 QoS 2 发布者会收到失败确认。对于 QoS 0，以及确认报文无法携带失败原因的 MQTT 3/4，Broker 会断开发布者连接，而不是发送成功确认
5. **投递重试**：Broker 端投递失败时会按照 `[delay_message]` 配置重试；重试耗尽后消息会被删除，不会进入死信主题
6. **投递保证**：延迟投递采用至少一次语义。Broker 在目标主题投递期间重启可能导致消息重复投递，因此消费者应使用幂等处理
7. **错误处理**：如果延迟时间格式错误，或无订阅者策略禁止保存延迟消息，发布请求会被拒绝
