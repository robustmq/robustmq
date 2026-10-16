# 使用 Cargo 运行

当前工作区通过一个 Broker 二进制启动 RobustMQ。`cmd` 包已经不再提供
旧文档中的 `meta-service`、`mqtt-server` 和 `journal-server` 这些可运行
目标。

## 单机 Broker

在仓库根目录执行：

```bash
make run
```

该命令等价于直接运行 Broker 二进制：

```bash
cargo run --package cmd --bin broker-server -- --conf=config/server.toml
```

仓库中的 `config/server.toml` 是单节点配置，包含 `meta`、`broker` 和
`engine` 角色，数据默认写入 `./data`。

默认本地端口如下：

| 服务 | 端口 |
|------|------|
| MQTT | 1883 |
| MQTT TLS | 1885 |
| MQTT WebSocket | 8083 |
| Kafka | 9092 |
| AMQP | 5672 |
| NATS | 4222 |
| 存储引擎 | 1779 |
| Admin HTTP API | 58080 |
| gRPC | 1228 |

## 验证

另开一个终端执行：

```bash
curl -i http://127.0.0.1:58080/health/node
curl http://127.0.0.1:58080/metrics | head
```

使用 MQTTX 验证收发：

```bash
mqttx sub -h 127.0.0.1 -p 1883 -u admin -P robustmq -t "test/topic"
mqttx pub -h 127.0.0.1 -p 1883 -u admin -P robustmq \
  -t "test/topic" -m "Hello RobustMQ!"
```

## 只构建不运行

```bash
cargo build --package cmd --bin broker-server
```

如需使用其他配置文件，在 `--conf` 后传入路径，例如：

```bash
cargo run --package cmd --bin broker-server -- \
  --conf=config/cluster/server-1.toml
```
