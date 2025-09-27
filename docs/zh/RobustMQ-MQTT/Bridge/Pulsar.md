# Apache Pulsar 连接器

RobustMQ 支持将 MQTT 消息桥接到 Apache Pulsar 消息系统，实现高吞吐量、低延迟的消息传输和持久化存储。

## 功能特性

- **高性能消息传输**: 基于 Pulsar 的分布式架构，支持高并发消息处理
- **多种认证方式**: 支持 Token、OAuth2、Basic 认证
- **消息持久化**: 利用 Pulsar 的持久化机制确保消息不丢失
- **自动重试机制**: 内置错误重试和故障恢复机制
- **异步非阻塞**: 采用异步发送模式，提升系统吞吐量

## 配置说明

### 连接器配置结构

```json
{
  "server": "pulsar://localhost:6650",
  "topic": "mqtt-messages",
  "token": "your-auth-token",
  "oauth": "{\"issuer_url\":\"https://auth.example.com\",\"client_id\":\"client\",\"client_secret\":\"secret\"}",
  "basic_name": "username",
  "basic_password": "password"
}
```

### 配置参数说明

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `server` | String | ✅ | Pulsar 服务器地址，格式：`pulsar://host:port` |
| `topic` | String | ✅ | 目标 Pulsar 主题名称 |
| `token` | String | ❌ | Token 认证令牌 |
| `oauth` | String | ❌ | OAuth2 认证配置（JSON 格式） |
| `basic_name` | String | ❌ | Basic 认证用户名 |
| `basic_password` | String | ❌ | Basic 认证密码 |

### 认证方式

#### 1. Token 认证
```json
{
  "server": "pulsar://localhost:6650",
  "topic": "mqtt-messages",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

#### 2. OAuth2 认证
```json
{
  "server": "pulsar://localhost:6650",
  "topic": "mqtt-messages",
  "oauth": "{\"issuer_url\":\"https://auth.example.com\",\"client_id\":\"mqtt-client\",\"client_secret\":\"secret123\"}"
}
```

#### 3. Basic 认证
```json
{
  "server": "pulsar://localhost:6650",
  "topic": "mqtt-messages",
  "basic_name": "mqtt-user",
  "basic_password": "mqtt-password"
}
```

#### 4. 无认证（开发环境）
```json
{
  "server": "pulsar://localhost:6650",
  "topic": "mqtt-messages"
}
```

## 使用示例

### 1. 创建 Pulsar 连接器

使用 RobustMQ 管理 API 创建 Pulsar 连接器：

```bash
curl -X POST http://localhost:8080/api/v1/connector \
  -H "Content-Type: application/json" \
  -d '{
    "cluster_name": "default-cluster",
    "connector_name": "pulsar-connector-1",
    "connector_type": "pulsar",
    "topic_id": "mqtt/sensor/+/data",
    "config": "{\"server\":\"pulsar://localhost:6650\",\"topic\":\"sensor-data\"}"
  }'
```

### 2. 查看连接器状态

```bash
curl -X GET http://localhost:8080/api/v1/connector/pulsar-connector-1
```

### 3. 更新连接器配置

```bash
curl -X PUT http://localhost:8080/api/v1/connector/pulsar-connector-1 \
  -H "Content-Type: application/json" \
  -d '{
    "config": "{\"server\":\"pulsar://localhost:6650\",\"topic\":\"sensor-data-v2\",\"token\":\"new-token\"}"
  }'
```

### 4. 删除连接器

```bash
curl -X DELETE http://localhost:8080/api/v1/connector/pulsar-connector-1
```

## 消息格式

### MQTT 消息转换

RobustMQ 将 MQTT 消息转换为 Pulsar 消息时，保持以下格式：

```json
{
  "topic": "mqtt/sensor/temperature/data",
  "payload": "25.6",
  "qos": 1,
  "retain": false,
  "timestamp": 1703123456789,
  "client_id": "sensor-001",
  "headers": {
    "content-type": "application/json",
    "device-id": "sensor-001"
  }
}
```

### Pulsar 消息属性

| 属性 | 说明 |
|------|------|
| `key` | 使用 MQTT 客户端 ID 作为消息键 |
| `properties` | 包含 MQTT 消息的元数据信息 |
| `payload` | MQTT 消息的原始负载数据 |
| `event_time` | MQTT 消息的发布时间戳 |

## 部署配置

### Docker Compose 示例

```yaml
version: '3.8'
services:
  pulsar:
    image: apachepulsar/pulsar:3.1.0
    command: bin/pulsar standalone
    ports:
      - "6650:6650"
      - "8080:8080"
    environment:
      - PULSAR_MEM="-Xms512m -Xmx512m -XX:MaxDirectMemorySize=256m"

  robustmq:
    image: robustmq/robustmq:latest
    ports:
      - "1883:1883"
      - "8883:8883"
      - "8080:8080"
    depends_on:
      - pulsar
    environment:
      - ROBUSTMQ_PULSAR_SERVER=pulsar://pulsar:6650
```

### Kubernetes 部署

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: robustmq-pulsar-config
data:
  connector.json: |
    {
      "server": "pulsar://pulsar-broker:6650",
      "topic": "mqtt-messages",
      "token": "your-token-here"
    }
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: robustmq-with-pulsar
spec:
  replicas: 3
  selector:
    matchLabels:
      app: robustmq
  template:
    metadata:
      labels:
        app: robustmq
    spec:
      containers:
      - name: robustmq
        image: robustmq/robustmq:latest
        ports:
        - containerPort: 1883
        - containerPort: 8080
        volumeMounts:
        - name: config
          mountPath: /app/config
      volumes:
      - name: config
        configMap:
          name: robustmq-pulsar-config
```

## 性能调优

### 连接器配置优化

```json
{
  "server": "pulsar://localhost:6650",
  "topic": "mqtt-messages",
  "batch_size": 1000,
  "batch_timeout_ms": 100,
  "compression_type": "LZ4",
  "send_timeout_ms": 30000
}
```

### Pulsar 客户端优化

- **批处理**: 启用消息批处理以提高吞吐量
- **压缩**: 使用 LZ4 或 ZSTD 压缩减少网络传输
- **连接池**: 复用 Pulsar 客户端连接
- **异步发送**: 使用异步模式避免阻塞

## 监控指标

RobustMQ 为 Pulsar 连接器提供以下监控指标：

### 消息指标
- `pulsar_messages_sent_total`: 发送到 Pulsar 的消息总数
- `pulsar_messages_failed_total`: 发送失败的消息总数
- `pulsar_send_duration_ms`: 消息发送耗时分布

### 连接指标
- `pulsar_connections_active`: 活跃的 Pulsar 连接数
- `pulsar_connection_errors_total`: 连接错误总数
- `pulsar_reconnections_total`: 重连次数

### 性能指标
- `pulsar_throughput_messages_per_sec`: 消息吞吐量（消息/秒）
- `pulsar_throughput_bytes_per_sec`: 数据吞吐量（字节/秒）

## 故障排查

### 常见问题

#### 1. 连接失败
```bash
# 检查 Pulsar 服务状态
curl http://localhost:8080/admin/v2/clusters

# 验证网络连通性
telnet localhost 6650
```

#### 2. 认证失败
```bash
# 验证 Token 有效性
curl -H "Authorization: Bearer YOUR_TOKEN" \
     http://localhost:8080/admin/v2/tenants
```

#### 3. 消息发送失败
```bash
# 检查主题是否存在
curl http://localhost:8080/admin/v2/persistent/public/default/mqtt-messages
```

### 日志分析

```bash
# 查看连接器日志
grep "PulsarBridgePlugin" /var/log/robustmq/robustmq.log

# 查看 Pulsar 相关错误
grep "pulsar" /var/log/robustmq/robustmq.log | grep ERROR
```

### 性能诊断

```bash
# 检查消息积压
curl http://localhost:8080/admin/v2/persistent/public/default/mqtt-messages/stats

# 监控连接器性能
curl http://localhost:9090/metrics | grep pulsar_
```

## 最佳实践

### 1. 主题设计
- 使用有意义的主题名称
- 考虑分区策略以提高并行度
- 设置合适的消息保留策略

### 2. 安全配置
- 生产环境必须启用认证
- 使用 TLS 加密传输
- 定期轮换认证凭据

### 3. 性能优化
- 根据消息量调整批处理大小
- 启用消息压缩节省带宽
- 监控并调整超时参数

### 4. 运维管理
- 设置完善的监控告警
- 定期备份连接器配置
- 建立故障恢复流程

通过 Apache Pulsar 连接器，RobustMQ 能够将 MQTT 消息高效地桥接到 Pulsar 生态系统，为构建大规模、高可靠的 IoT 数据管道提供强有力的支持。
