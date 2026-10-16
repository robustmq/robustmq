# Docker 部署

## 前置条件

- Docker 20.10+
- （可选）Docker Compose 2.0+

Dockerfile 会下载已发布的 RobustMQ 版本包。示例使用版本标签
`v0.4.11`。如需直接运行当前工作区代码，请参考
[Cargo 运行指南](../ContributionGuide/ContributingCode/Cargo-Running.md)。

---

## 方式一：直接运行镜像

```bash
docker run -d --name robustmq \
  -p 1883:1883 \
  -p 1885:1885 \
  -p 8083:8083 \
  -p 8085:8085 \
  -p 9092:9092 \
  -p 1228:1228 \
  -p 5672:5672 \
  -p 58080:58080 \
  ghcr.io/robustmq/robustmq:latest
```

查看日志：

```bash
docker logs -f robustmq
```

---

## 方式二：Docker Compose

```bash
# 克隆项目
git clone https://github.com/robustmq/robustmq.git
cd robustmq

# 启动
docker compose -f docker/robustmq/docker-compose.yml up -d robustmq

# 查看状态
docker compose -f docker/robustmq/docker-compose.yml ps

# 停止
docker compose -f docker/robustmq/docker-compose.yml down
```

Compose 文件会向 Dockerfile 传入发布版本，并挂载仓库顶层的 `config/`
目录。

---

## 验证

```bash
# 就绪检查和指标
curl -i http://127.0.0.1:58080/health/ready
curl http://127.0.0.1:58080/metrics | head

# 订阅（终端 1）
mqttx sub -h 127.0.0.1 -p 1883 -u admin -P robustmq -t "test/topic"

# 发布（终端 2）
mqttx pub -h 127.0.0.1 -p 1883 -u admin -P robustmq \
  -t "test/topic" -m "Hello RobustMQ!"
```

收到消息即表示运行正常。

---

## 默认端口

| 服务 | 端口 |
|------|------|
| MQTT TCP | 1883 |
| MQTT TLS | 1885 |
| MQTT WebSocket | 8083 |
| MQTT WebSocket TLS | 8085 |
| Kafka | 9092 |
| AMQP | 5672 |
| NATS | 4222 |
| 存储引擎 | 1779 |
| Placement Center gRPC | 1228 |
| HTTP API | 58080 |
