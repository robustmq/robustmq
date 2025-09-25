# MQTT 共享订阅

## 什么是 MQTT 共享订阅？

共享订阅是 MQTT 的扩展功能，允许多个订阅者共享同一个主题的订阅。在共享订阅中，消息会被负载均衡地分发给订阅组中的不同客户端，而不是广播给所有订阅者。

共享订阅的核心特性是**负载均衡**：当有消息发布到共享订阅主题时，消息会被分发给订阅组中的一个客户端，而不是所有客户端，从而实现消息处理的负载均衡。

## 何时使用 MQTT 共享订阅？

共享订阅适用于以下场景：

- **负载均衡**：多个客户端需要处理同一类型的消息，但每个消息只需要被处理一次
- **任务分发**：将任务消息分发给多个工作节点，提高处理效率
- **消息队列**：实现类似消息队列的功能，确保消息被有序处理
- **高可用性**：当某个客户端离线时，其他客户端可以继续处理消息
- **水平扩展**：通过增加订阅者来扩展消息处理能力

## 共享订阅的语法规则

共享订阅有两种格式：

### 带群组的共享订阅

```text
$share/{group}/{topic}
```

**参数说明**：

- `$share`：共享订阅的前缀标识
- `{group}`：订阅组名称，可以是任意字符串
- `{topic}`：原始主题名称

### 不带群组的共享订阅

```text
$queue/{topic}
```

**参数说明**：

- `$queue`：队列订阅的前缀标识
- `{topic}`：原始主题名称

**重要说明**：

- 带群组的共享订阅允许创建多个订阅组，每个组独立进行负载均衡
- 不带群组的共享订阅（`$queue/`）是所有订阅者都在一个组中的特例
- 消息发布到原始主题，订阅者通过共享订阅主题接收消息

## 共享订阅的特性

- **负载均衡**：消息在订阅组内进行负载均衡分发
- **组隔离**：不同订阅组之间相互独立
- **高可用性**：支持客户端动态加入和离开
- **会话管理**：支持持久会话和临时会话
- **QoS 支持**：支持 MQTT 的 QoS 级别

## 共享订阅与会话

当客户端具有持久会话并订阅了共享订阅时，会话将在客户端断开连接时继续接收发布到共享订阅主题的消息。如果客户端长时间断开连接且消息发布速率很高，会话状态中的内部消息队列可能会溢出。

**建议**：

- 为共享订阅使用 `clean_session=true` 的会话
- 使用 MQTT v5 时，建议设置短会话过期时间
- 当会话过期时，未处理的消息会被重新分发到同组中的其他会话

## 通过 MQTTX 使用共享订阅

### 使用 MQTTX CLI

1. **创建带群组的共享订阅**

   ```bash
   # 组1的订阅者
   mqttx sub -t '$share/group1/sensor/data' -h '117.72.92.117' -p 1883 -v
   mqttx sub -t '$share/group1/sensor/data' -h '117.72.92.117' -p 1883 -v
   
   # 组2的订阅者
   mqttx sub -t '$share/group2/sensor/data' -h '117.72.92.117' -p 1883 -v
   mqttx sub -t '$share/group2/sensor/data' -h '117.72.92.117' -p 1883 -v
   ```

2. **创建不带群组的共享订阅**

   ```bash
   # 队列订阅
   mqttx sub -t '$queue/task/processing' -h '117.72.92.117' -p 1883 -v
   mqttx sub -t '$queue/task/processing' -h '117.72.92.117' -p 1883 -v
   mqttx sub -t '$queue/task/processing' -h '117.72.92.117' -p 1883 -v
   ```

3. **发布消息到原始主题**

   ```bash
   # 发布到原始主题
   mqttx pub -t 'sensor/data' -m '{"temperature":25.5,"humidity":60}' -h '117.72.92.117' -p 1883
   mqttx pub -t 'task/processing' -m '{"task_id":"T001","type":"analysis"}' -h '117.72.92.117' -p 1883
   ```

### 实际应用示例

#### 传感器数据处理

```bash
# 数据处理组1
mqttx sub -t '$share/processor1/sensor/temperature' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$share/processor1/sensor/temperature' -h '117.72.92.117' -p 1883 -v

# 数据处理组2
mqttx sub -t '$share/processor2/sensor/temperature' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$share/processor2/sensor/temperature' -h '117.72.92.117' -p 1883 -v

# 发布传感器数据
mqttx pub -t 'sensor/temperature' -m '{"value":25.5,"timestamp":"2024-01-01T12:00:00Z"}' -h '117.72.92.117' -p 1883
```

#### 任务队列处理

```bash
# 工作节点订阅任务队列
mqttx sub -t '$queue/job/queue' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$queue/job/queue' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$queue/job/queue' -h '117.72.92.117' -p 1883 -v

# 发布任务到队列
mqttx pub -t 'job/queue' -m '{"job_id":"J001","type":"image_processing","data":"base64..."}' -h '117.72.92.117' -p 1883
mqttx pub -t 'job/queue' -m '{"job_id":"J002","type":"data_analysis","data":"csv_data"}' -h '117.72.92.117' -p 1883
```

#### 消息通知系统

```bash
# 通知处理组A（高优先级）
mqttx sub -t '$share/notify_high/notification/alert' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$share/notify_high/notification/alert' -h '117.72.92.117' -p 1883 -v

# 通知处理组B（普通优先级）
mqttx sub -t '$share/notify_normal/notification/info' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$share/notify_normal/notification/info' -h '117.72.92.117' -p 1883 -v

# 发布不同类型的通知
mqttx pub -t 'notification/alert' -m '{"level":"critical","message":"System overload"}' -h '117.72.92.117' -p 1883
mqttx pub -t 'notification/info' -m '{"level":"info","message":"Daily backup completed"}' -h '117.72.92.117' -p 1883
```

#### 日志处理系统

```bash
# 日志处理组
mqttx sub -t '$share/log_processor/application/logs' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$share/log_processor/application/logs' -h '117.72.92.117' -p 1883 -v
mqttx sub -t '$share/log_processor/application/logs' -h '117.72.92.117' -p 1883 -v

# 发布日志消息
mqttx pub -t 'application/logs' -m '{"level":"INFO","message":"User login successful","user_id":"123"}' -h '117.72.92.117' -p 1883
mqttx pub -t 'application/logs' -m '{"level":"ERROR","message":"Database connection failed","error":"timeout"}' -h '117.72.92.117' -p 1883
```

## 共享订阅与普通订阅的区别

| 特性 | 普通订阅 | 共享订阅 |
|------|----------|----------|
| 消息分发 | 广播给所有订阅者 | 负载均衡分发给订阅组 |
| 主题格式 | 原始主题 | $share/{group}/{topic} 或 $queue/{topic} |
| 处理方式 | 每个订阅者都处理 | 每个消息只被一个订阅者处理 |
| 负载均衡 | 无 | 有 |
| 高可用性 | 依赖客户端 | 支持动态故障转移 |
| 应用场景 | 通知、状态同步 | 任务处理、消息队列 |

## 负载均衡策略

共享订阅支持多种负载均衡策略：

- **轮询（Round Robin）**：按顺序轮流分发给订阅者
- **随机（Random）**：随机选择订阅者
- **最少连接（Least Connections）**：分发给连接数最少的订阅者
- **哈希（Hash）**：基于消息内容哈希值分发

## 注意事项

1. **主题格式**：必须使用正确的共享订阅主题格式
2. **组管理**：合理设计订阅组，避免组内订阅者过少或过多
3. **会话管理**：建议使用 `clean_session=true` 避免消息积压
4. **QoS 级别**：根据业务需求选择合适的 QoS 级别
5. **错误处理**：客户端应正确处理消息处理失败的情况
6. **监控告警**：监控订阅组的状态和消息处理情况
