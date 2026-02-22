# Docker 部署

## 前置条件

- Docker 20.10+
- （可选）Docker Compose 2.0+

---

## 方式一：直接运行镜像

```bash
docker run -d --name robustmq \
  -p 1883:1883 \
  -p 8083:8083 \
  -p 1228:1228 \
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

---

## 验证

```bash
# 订阅（终端 1）
mqttx sub -h 127.0.0.1 -p 1883 -t "test/topic"

# 发布（终端 2）
mqttx pub -h 127.0.0.1 -p 1883 -t "test/topic" -m "Hello RobustMQ!"
```

收到消息即表示运行正常。

---

## 默认端口

| 服务 | 端口 |
|------|------|
| MQTT TCP | 1883 |
| MQTT WebSocket | 8083 |
| Placement Center gRPC | 1228 |
| HTTP API | 8080 |
