# Docker Deployment

## Prerequisites

- Docker 20.10+
- (Optional) Docker Compose 2.0+

---

## Option 1: Run Image Directly

```bash
docker run -d --name robustmq \
  -p 1883:1883 \
  -p 8083:8083 \
  -p 1228:1228 \
  ghcr.io/robustmq/robustmq:latest
```

Follow logs:

```bash
docker logs -f robustmq
```

---

## Option 2: Docker Compose

```bash
# Clone the repo
git clone https://github.com/robustmq/robustmq.git
cd robustmq

# Start
docker compose -f docker/robustmq/docker-compose.yml up -d robustmq

# Check status
docker compose -f docker/robustmq/docker-compose.yml ps

# Stop
docker compose -f docker/robustmq/docker-compose.yml down
```

---

## Verify

```bash
# Subscribe (terminal 1)
mqttx sub -h 127.0.0.1 -p 1883 -t "test/topic"

# Publish (terminal 2)
mqttx pub -h 127.0.0.1 -p 1883 -t "test/topic" -m "Hello RobustMQ!"
```

If the subscriber receives the message, the service is running correctly.

---

## Default Ports

| Service | Port |
|---------|------|
| MQTT TCP | 1883 |
| MQTT WebSocket | 8083 |
| Placement Center gRPC | 1228 |
| HTTP API | 8080 |
