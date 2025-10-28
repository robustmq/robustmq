# MQTTX 测试 RobustMQ 使用手册

## 概述

MQTTX CLI 是一款强大的 MQTT 5.0 命令行客户端工具，可用于测试和验证 RobustMQ MQTT Broker 的功能和性能。本手册介绍如何使用 MQTTX CLI 对 RobustMQ 进行发布、订阅和性能测试。

## 安装 MQTTX CLI

### macOS

```bash
# 使用 Homebrew
brew install emqx/mqttx/mqttx-cli

# 或使用 npm
npm install -g @emqx/mqttx-cli
```

### Linux

```bash
# 使用 npm
npm install -g @emqx/mqttx-cli

# 或下载二进制文件
wget https://github.com/emqx/MQTTX/releases/latest/download/mqttx-cli-linux-x64
chmod +x mqttx-cli-linux-x64
sudo mv mqttx-cli-linux-x64 /usr/local/bin/mqttx
```

### Windows

```bash
# 使用 npm
npm install -g @emqx/mqttx-cli

# 或使用 Chocolatey
choco install mqttx-cli
```

### 验证安装

```bash
mqttx --version
```

## 基础连接测试

### 连接到 RobustMQ

```bash
# 基础连接测试
mqttx conn -h localhost -p 1883 --client-id robustmq_test_client

# 带认证的连接
mqttx conn -h localhost -p 1883 -u username -P password --client-id robustmq_auth_client

# SSL 连接测试
mqttx conn -h localhost -p 1885 --client-id robustmq_ssl_client --protocol mqtts
```

## 消息发布测试

### 基础发布

```bash
# 发布单条消息
mqttx pub -h localhost -p 1883 -t 'robustmq/test/hello' -m 'Hello RobustMQ!'

# 指定 QoS 发布
mqttx pub -h localhost -p 1883 -t 'robustmq/test/qos1' -m 'QoS 1 message' -q 1

# 发布保留消息
mqttx pub -h localhost -p 1883 -t 'robustmq/test/retain' -m 'Retained message' -r
```

### 批量发布

```bash
# 发布多条消息
mqttx pub -h localhost -p 1883 -t 'robustmq/test/batch' -m 'Message 1,Message 2,Message 3' -s ','

# 从文件读取消息内容
echo "Hello from file" > message.txt
mqttx pub -h localhost -p 1883 -t 'robustmq/test/file' -M message.txt

# JSON 格式消息
mqttx pub -h localhost -p 1883 -t 'robustmq/test/json' -m '{"sensor":"temp","value":25.5,"unit":"celsius"}'
```

### 定时发布

```bash
# 每5秒发布一次消息，总共发布10次
mqttx pub -h localhost -p 1883 -t 'robustmq/test/interval' -m 'Interval message' -i 5 -c 10

# 无限循环发布（按 Ctrl+C 停止）
mqttx pub -h localhost -p 1883 -t 'robustmq/test/loop' -m 'Loop message' -i 2
```

## 消息订阅测试

### 基础订阅

```bash
# 订阅单个主题
mqttx sub -h localhost -p 1883 -t 'robustmq/test/hello'

# 订阅通配符主题
mqttx sub -h localhost -p 1883 -t 'robustmq/test/+' -q 1

# 订阅多级通配符
mqttx sub -h localhost -p 1883 -t 'robustmq/test/#'
```

### 高级订阅

```bash
# 订阅多个主题
mqttx sub -h localhost -p 1883 -t 'robustmq/sensors/+/temperature,robustmq/sensors/+/humidity'

# 带认证的订阅
mqttx sub -h localhost -p 1883 -t 'robustmq/secure/+' -u username -P password

# 显示详细信息
mqttx sub -h localhost -p 1883 -t 'robustmq/test/#' --verbose

# 输出到文件
mqttx sub -h localhost -p 1883 -t 'robustmq/logs/#' --output-mode clean > mqtt_logs.txt
```

### 订阅格式化输出

```bash
# JSON 格式输出
mqttx sub -h localhost -p 1883 -t 'robustmq/test/+' --format json

# 自定义格式输出
mqttx sub -h localhost -p 1883 -t 'robustmq/test/+' --format '[%Y-%m-%d %H:%M:%S] Topic: %t, Message: %p'
```

## 性能压测

### 发布性能测试

```bash
# 基础发布压测：1000条消息，10个并发连接
mqttx bench pub -h localhost -p 1883 -t 'robustmq/bench/pub' -c 10 -C 1000

# 高并发发布测试：100个连接，每个发布100条消息
mqttx bench pub -h localhost -p 1883 -t 'robustmq/bench/high' -c 100 -C 100

# 指定消息大小和 QoS
mqttx bench pub -h localhost -p 1883 -t 'robustmq/bench/large' -c 50 -C 500 -s 1024 -q 1

# 持续压测（指定时间）
mqttx bench pub -h localhost -p 1883 -t 'robustmq/bench/duration' -c 20 -t 60s
```

### 订阅性能测试

```bash
# 基础订阅压测：50个订阅客户端
mqttx bench sub -h localhost -p 1883 -t 'robustmq/bench/sub' -c 50

# 订阅通配符主题压测
mqttx bench sub -h localhost -p 1883 -t 'robustmq/bench/+' -c 100

# 指定 QoS 的订阅压测
mqttx bench sub -h localhost -p 1883 -t 'robustmq/bench/qos2' -c 30 -q 2
```

### 综合性能测试

```bash
# 发布和订阅综合测试
# 终端1：启动订阅客户端
mqttx bench sub -h localhost -p 1883 -t 'robustmq/perf/+' -c 50

# 终端2：启动发布客户端
mqttx bench pub -h localhost -p 1883 -t 'robustmq/perf/test' -c 10 -C 1000 -i 10
```

## SSL/TLS 连接测试

### SSL 连接

```bash
# SSL 发布测试
mqttx pub -h localhost -p 1885 -t 'robustmq/ssl/test' -m 'SSL message' --protocol mqtts

# SSL 订阅测试
mqttx sub -h localhost -p 1885 -t 'robustmq/ssl/+' --protocol mqtts

# 指定证书文件
mqttx pub -h localhost -p 1885 -t 'robustmq/ssl/cert' -m 'Cert message' \
  --ca ca.crt --cert client.crt --key client.key
```

### WebSocket 连接测试

```bash
# WebSocket 发布
mqttx pub -h localhost -p 8083 -t 'robustmq/ws/test' -m 'WebSocket message' --protocol ws

# WebSocket 订阅
mqttx sub -h localhost -p 8083 -t 'robustmq/ws/+' --protocol ws

# WebSocket SSL 连接
mqttx pub -h localhost -p 8085 -t 'robustmq/wss/test' -m 'WSS message' --protocol wss
```

## MQTT 5.0 特性测试

### 用户属性测试

```bash
# 发布带用户属性的消息
mqttx pub -h localhost -p 1883 -t 'robustmq/mqtt5/props' -m 'MQTT 5.0 message' \
  --mqtt-version 5 \
  --user-properties 'region:beijing,sensor:temperature'

# 设置消息过期时间
mqttx pub -h localhost -p 1883 -t 'robustmq/mqtt5/expire' -m 'Expiring message' \
  --mqtt-version 5 \
  --message-expiry-interval 60
```

### 主题别名测试

```bash
# 使用主题别名
mqttx pub -h localhost -p 1883 -t 'robustmq/mqtt5/very/long/topic/name' -m 'Alias message' \
  --mqtt-version 5 \
  --topic-alias 1
```

## 常用测试场景

### IoT 设备模拟

```bash
# 模拟温度传感器数据发布
mqttx pub -h localhost -p 1883 \
  -t 'robustmq/sensors/temp001/data' \
  -m '{"temperature":25.5,"humidity":60,"timestamp":"2024-01-01T10:00:00Z"}' \
  -i 5 -c 100

# 模拟多个设备
for i in {1..10}; do
  mqttx pub -h localhost -p 1883 \
    -t "robustmq/sensors/device${i}/status" \
    -m "online" \
    --client-id "device${i}" &
done
```

### 消息路由测试

```bash
# 测试主题过滤
mqttx sub -h localhost -p 1883 -t 'robustmq/+/temperature' &
mqttx sub -h localhost -p 1883 -t 'robustmq/room1/+' &

# 发布到不同主题验证路由
mqttx pub -h localhost -p 1883 -t 'robustmq/room1/temperature' -m '22.5'
mqttx pub -h localhost -p 1883 -t 'robustmq/room2/temperature' -m '24.0'
mqttx pub -h localhost -p 1883 -t 'robustmq/room1/humidity' -m '65'
```

### 延迟消息测试

```bash
# 测试 RobustMQ 延迟消息功能
mqttx pub -h localhost -p 1883 \
  -t '$delayed/60/robustmq/test/delay' \
  -m 'This message will be delivered after 60 seconds'

# 订阅延迟消息
mqttx sub -h localhost -p 1883 -t 'robustmq/test/delay'
```

### 保留消息测试

```bash
# 发布保留消息
mqttx pub -h localhost -p 1883 -t 'robustmq/status/server' -m 'online' -r

# 清除保留消息
mqttx pub -h localhost -p 1883 -t 'robustmq/status/server' -m '' -r

# 订阅查看保留消息
mqttx sub -h localhost -p 1883 -t 'robustmq/status/+'
```

## 性能测试指标

### 吞吐量测试

```bash
# 高吞吐量发布测试
mqttx bench pub \
  -h localhost -p 1883 \
  -t 'robustmq/throughput/test' \
  -c 100 \
  -C 1000 \
  -i 1 \
  --verbose

# 高吞吐量订阅测试
mqttx bench sub \
  -h localhost -p 1883 \
  -t 'robustmq/throughput/+' \
  -c 200 \
  --verbose
```

### 延迟测试

```bash
# 测试消息延迟
mqttx bench pub \
  -h localhost -p 1883 \
  -t 'robustmq/latency/test' \
  -c 1 \
  -C 100 \
  -i 100 \
  --verbose
```

### 连接数测试

```bash
# 大量连接测试
mqttx bench conn \
  -h localhost -p 1883 \
  -c 1000 \
  --interval 10
```

## 脚本化测试

### 自动化测试脚本

```bash
#!/bin/bash

# RobustMQ 功能测试脚本

BROKER_HOST="localhost"
BROKER_PORT="1883"
TEST_TOPIC_PREFIX="robustmq/test"

echo "开始 RobustMQ 功能测试..."

# 1. 连接测试
echo "1. 测试基础连接..."
mqttx conn -h $BROKER_HOST -p $BROKER_PORT --client-id robustmq_conn_test
if [ $? -eq 0 ]; then
    echo "✅ 连接测试通过"
else
    echo "❌ 连接测试失败"
    exit 1
fi

# 2. 发布订阅测试
echo "2. 测试发布订阅功能..."
mqttx sub -h $BROKER_HOST -p $BROKER_PORT -t "${TEST_TOPIC_PREFIX}/pubsub" &
SUB_PID=$!
sleep 2

mqttx pub -h $BROKER_HOST -p $BROKER_PORT -t "${TEST_TOPIC_PREFIX}/pubsub" -m "Test message"
sleep 2
kill $SUB_PID
echo "✅ 发布订阅测试完成"

# 3. QoS 测试
echo "3. 测试 QoS 功能..."
for qos in 0 1 2; do
    echo "测试 QoS $qos..."
    mqttx pub -h $BROKER_HOST -p $BROKER_PORT -t "${TEST_TOPIC_PREFIX}/qos$qos" -m "QoS $qos test" -q $qos
done
echo "✅ QoS 测试完成"

# 4. 保留消息测试
echo "4. 测试保留消息..."
mqttx pub -h $BROKER_HOST -p $BROKER_PORT -t "${TEST_TOPIC_PREFIX}/retain" -m "Retained message" -r
mqttx sub -h $BROKER_HOST -p $BROKER_PORT -t "${TEST_TOPIC_PREFIX}/retain" --timeout 3
echo "✅ 保留消息测试完成"

# 5. 性能测试
echo "5. 执行性能测试..."
mqttx bench pub -h $BROKER_HOST -p $BROKER_PORT -t "${TEST_TOPIC_PREFIX}/perf" -c 10 -C 100 --verbose
echo "✅ 性能测试完成"

echo "RobustMQ 功能测试全部完成！"
```

### 压力测试脚本

```bash
#!/bin/bash

# RobustMQ 压力测试脚本

BROKER_HOST="localhost"
BROKER_PORT="1883"

echo "开始 RobustMQ 压力测试..."

# 并发连接测试
echo "1. 并发连接测试（1000个连接）..."
mqttx bench conn -h $BROKER_HOST -p $BROKER_PORT -c 1000 --interval 10

# 高并发发布测试
echo "2. 高并发发布测试（200个客户端，每个发布500条消息）..."
mqttx bench pub -h $BROKER_HOST -p $BROKER_PORT -t 'robustmq/stress/pub' -c 200 -C 500 --verbose

# 高并发订阅测试
echo "3. 高并发订阅测试（500个订阅客户端）..."
mqttx bench sub -h $BROKER_HOST -p $BROKER_PORT -t 'robustmq/stress/sub' -c 500 --verbose &
SUB_PID=$!

# 等待订阅客户端就绪
sleep 5

# 发布测试消息
mqttx bench pub -h $BROKER_HOST -p $BROKER_PORT -t 'robustmq/stress/sub' -c 50 -C 1000

# 等待消息传递完成
sleep 10
kill $SUB_PID

echo "RobustMQ 压力测试完成！"
```

## 常用参数说明

### 连接参数

| 参数 | 说明 | 示例 |
|------|------|------|
| `-h, --hostname` | Broker 主机地址 | `-h localhost` |
| `-p, --port` | Broker 端口 | `-p 1883` |
| `-u, --username` | 用户名 | `-u admin` |
| `-P, --password` | 密码 | `-P password` |
| `--client-id` | 客户端ID | `--client-id test_client` |
| `--protocol` | 协议类型 | `--protocol mqtts` |

### 消息参数

| 参数 | 说明 | 示例 |
|------|------|------|
| `-t, --topic` | 主题 | `-t robustmq/test` |
| `-m, --message` | 消息内容 | `-m 'Hello World'` |
| `-q, --qos` | QoS等级 | `-q 1` |
| `-r, --retain` | 保留消息 | `-r` |
| `-i, --interval` | 发送间隔(秒) | `-i 5` |
| `-c, --count` | 消息数量 | `-c 100` |

### 压测参数

| 参数 | 说明 | 示例 |
|------|------|------|
| `-c, --count` | 客户端数量 | `-c 100` |
| `-C, --message-count` | 每客户端消息数 | `-C 1000` |
| `-i, --interval` | 发送间隔(ms) | `-i 1000` |
| `-s, --message-size` | 消息大小(字节) | `-s 1024` |
| `--verbose` | 详细输出 | `--verbose` |

## RobustMQ 特性测试

### 多协议端口测试

```bash
# 测试不同协议端口
echo "测试 MQTT TCP 端口..."
mqttx pub -h localhost -p 1883 -t 'robustmq/tcp/test' -m 'TCP message'

echo "测试 MQTT SSL 端口..."
mqttx pub -h localhost -p 1885 -t 'robustmq/ssl/test' -m 'SSL message' --protocol mqtts

echo "测试 WebSocket 端口..."
mqttx pub -h localhost -p 8083 -t 'robustmq/ws/test' -m 'WebSocket message' --protocol ws

echo "测试 WebSocket SSL 端口..."
mqttx pub -h localhost -p 8085 -t 'robustmq/wss/test' -m 'WSS message' --protocol wss
```

### 集群测试

```bash
# 测试 RobustMQ 集群的多个节点
NODES=("localhost:1883" "node2:1883" "node3:1883")

for node in "${NODES[@]}"; do
    IFS=':' read -r host port <<< "$node"
    echo "测试节点: $host:$port"

    mqttx pub -h $host -p $port -t 'robustmq/cluster/test' -m "Message from $host" \
      --client-id "test_client_$host"
done
```

## 监控和日志

### 实时监控

```bash
# 监控所有主题的消息流
mqttx sub -h localhost -p 1883 -t '#' --verbose

# 监控特定模式的消息
mqttx sub -h localhost -p 1883 -t 'robustmq/+/+/data' --format '[%Y-%m-%d %H:%M:%S] %t: %p'

# 统计消息数量
mqttx sub -h localhost -p 1883 -t 'robustmq/stats/+' --output-mode clean | wc -l
```

### 日志记录

```bash
# 记录所有接收的消息到文件
mqttx sub -h localhost -p 1883 -t 'robustmq/logs/#' --verbose > robustmq_messages.log

# 记录性能测试结果
mqttx bench pub -h localhost -p 1883 -t 'robustmq/perf/test' -c 50 -C 1000 --verbose > perf_test.log
```

## 故障排查

### 连接问题诊断

```bash
# 详细连接诊断
mqttx conn -h localhost -p 1883 --client-id debug_client --verbose

# 测试网络连通性
mqttx conn -h localhost -p 1883 --timeout 5

# 测试认证问题
mqttx conn -h localhost -p 1883 -u test_user -P wrong_password --verbose
```

### 消息传递问题

```bash
# 测试消息是否正确传递
mqttx sub -h localhost -p 1883 -t 'robustmq/debug/+' --verbose &
mqttx pub -h localhost -p 1883 -t 'robustmq/debug/test' -m 'Debug message' --verbose

# 测试不同 QoS 的消息传递
for qos in 0 1 2; do
    mqttx pub -h localhost -p 1883 -t "robustmq/debug/qos$qos" -m "QoS $qos test" -q $qos --verbose
done
```

## 最佳实践

### 1. 测试前准备

```bash
# 检查 RobustMQ 服务状态
curl -f http://localhost:8080/health || echo "RobustMQ 服务可能未启动"

# 清理测试主题（如果需要）
mqttx pub -h localhost -p 1883 -t 'robustmq/test/cleanup' -m '' -r
```

### 2. 分阶段测试

```bash
# 阶段1：基础功能测试
mqttx pub -h localhost -p 1883 -t 'robustmq/test/basic' -m 'Basic test'

# 阶段2：中等负载测试
mqttx bench pub -h localhost -p 1883 -t 'robustmq/test/medium' -c 10 -C 100

# 阶段3：高负载测试
mqttx bench pub -h localhost -p 1883 -t 'robustmq/test/heavy' -c 100 -C 1000
```

### 3. 结果分析

```bash
# 使用 verbose 模式获取详细统计
mqttx bench pub -h localhost -p 1883 -t 'robustmq/analysis/test' -c 50 -C 500 --verbose | grep -E "(Published|Failed|Rate)"

# 测试结果保存
mqttx bench pub -h localhost -p 1883 -t 'robustmq/results/test' -c 100 -C 1000 --verbose > test_results_$(date +%Y%m%d_%H%M%S).log
```

## 总结

MQTTX CLI 是测试 RobustMQ MQTT Broker 的理想工具，提供了从基础连接到高级性能测试的完整功能。通过本手册的示例和脚本，您可以：

1. **验证基础功能**：连接、发布、订阅、认证等
2. **测试高级特性**：SSL/TLS、WebSocket、MQTT 5.0 等
3. **进行性能评估**：吞吐量、延迟、并发连接等
4. **故障诊断**：快速定位和解决问题
5. **自动化测试**：集成到 CI/CD 流程中

建议在部署 RobustMQ 到生产环境前，使用 MQTTX CLI 进行全面的功能和性能测试，确保系统稳定可靠。
