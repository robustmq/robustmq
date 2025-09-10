# 体验 RobustMQ MQTT

本指南将带您快速体验 RobustMQ 的 MQTT 功能，包括启动 Broker、查看集群配置、发送和消费 MQTT 消息。

## 目录

- [运行 Broker](#运行-broker)
- [查看集群配置](#查看集群配置)
- [发送 MQTT 消息](#发送-mqtt-消息)
- [消费 MQTT 消息](#消费-mqtt-消息)
- [高级功能](#高级功能)

## 运行 Broker

### 1. 下载并解压二进制包

首先，我们需要下载并解压 RobustMQ 的二进制包：

```bash
# 下载最新版本的二进制包（以 v1.0.0 为例）
wget https://github.com/robustmq/robustmq/releases/download/v1.0.0/robustmq-v1.0.0-linux-amd64.tar.gz

# 解压二进制包
tar -xzf robustmq-v1.0.0-linux-amd64.tar.gz

# 进入解压后的目录
cd robustmq-v1.0.0-linux-amd64
```

### 2. 启动 RobustMQ Broker

```bash
# 启动 Broker（使用默认配置）
./bin/broker-server start

# 或者使用配置文件启动
./bin/broker-server start config/server.toml

# 后台启动
nohup ./bin/broker-server start > broker.log 2>&1 &
```

### 3. 验证 Broker 启动状态

Broker 启动成功后，您应该看到类似以下的输出：

```bash
[INFO] RobustMQ Broker starting...
[INFO] MQTT server listening on 0.0.0.0:1883
[INFO] Admin server listening on 0.0.0.0:8080
[INFO] Broker started successfully
```

### 4. 检查服务状态

```bash
# 检查 MQTT 端口是否监听
netstat -tlnp | grep 1883

# 检查管理端口是否监听
netstat -tlnp | grep 8080

# 或者使用 ss 命令
ss -tlnp | grep :1883
ss -tlnp | grep :8080
```

## 查看集群配置

### 使用 robust-ctl 命令行工具

RobustMQ 提供了强大的命令行管理工具 `robust-ctl`，让我们来查看集群配置：

```bash
# 查看集群配置
./bin/robust-ctl cluster config get
```

### 配置信息解读

执行 `robust-ctl cluster config get` 后，您将看到类似以下的配置信息：

```json
{
  "cluster": {
    "name": "robustmq-cluster",
    "nodes": [
      {
        "id": "node-1",
        "address": "127.0.0.1:9090",
        "role": "leader",
        "status": "active"
      }
    ],
    "replication_factor": 1,
    "consensus": "raft"
  },
  "mqtt": {
    "port": 1883,
    "max_connections": 10000,
    "keep_alive": 60,
    "retain_available": true,
    "wildcard_subscription_available": true
  }
}
```

## 发送 MQTT 消息

### 使用 MQTTX 发送消息

```bash
# 发送简单消息
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"

# 发送 QoS 1 消息
mqttx pub -h localhost -p 1883 -t "test/qos1" -m "QoS 1 message" -q 1

# 发送保留消息
mqttx pub -h localhost -p 1883 -t "test/retained" -m "Retained message" -r

# 发送 JSON 格式消息
mqttx pub -h localhost -p 1883 -t "sensors/temperature" -m '{"value": 25.5, "unit": "celsius", "timestamp": "2024-01-01T12:00:00Z"}'
```

## 消费 MQTT 消息

### 使用 MQTTX 订阅消息

```bash
# 订阅单个主题
mqttx sub -h localhost -p 1883 -t "test/topic"

# 订阅通配符主题
mqttx sub -h localhost -p 1883 -t "test/+"  # 单级通配符
mqttx sub -h localhost -p 1883 -t "test/#"  # 多级通配符

# 订阅 QoS 1 消息
mqttx sub -h localhost -p 1883 -t "test/qos1" -q 1

# 订阅并显示详细信息
mqttx sub -h localhost -p 1883 -t "test/topic" --verbose
```

## 高级功能

### 性能测试

```bash
# 使用 MQTTX 进行性能测试
mqttx bench pub -h localhost -p 1883 -t "test/bench" -c 10 -C 100

# 测试订阅性能
mqttx bench sub -h localhost -p 1883 -t "test/bench" -c 50
```

## 完整示例

让我们通过一个完整的示例来体验 RobustMQ MQTT 功能：

### 步骤 1: 启动 Broker

```bash
# 终端 1: 启动 Broker
./bin/broker-server start
```

### 步骤 2: 查看集群配置

```bash
# 终端 2: 查看配置
./bin/robust-ctl cluster config get
```

### 步骤 3: 订阅消息

```bash
# 终端 3: 订阅消息
mqttx sub -h localhost -p 1883 -t "demo/temperature" --verbose
```

### 步骤 4: 发送消息

```bash
# 终端 4: 发送消息
mqttx pub -h localhost -p 1883 -t "demo/temperature" -m '{"sensor": "temp-001", "value": 23.5, "unit": "celsius"}'
```
