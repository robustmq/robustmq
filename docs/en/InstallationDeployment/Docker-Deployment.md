# Docker Deployment

## Prerequisites

- Docker 20.10+
- (Optional) Docker Compose 2.0+

The Dockerfile packages a published RobustMQ release asset. The examples use
release tag `v0.4.11`. To run the current working tree directly, use Cargo as
described in the [Cargo running guide](../ContributionGuide/ContributingCode/Cargo-Running.md).

---

## Option 1: Run Image Directly

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

The Compose file supplies the release version to the Dockerfile and mounts the
repository's top-level `config/` directory.

---

## Verify

```bash
# Readiness and metrics
curl -i http://127.0.0.1:58080/health/ready
curl http://127.0.0.1:58080/metrics | head

# Subscribe (terminal 1)
mqttx sub -h 127.0.0.1 -p 1883 -u admin -P robustmq -t "test/topic"

# Publish (terminal 2)
mqttx pub -h 127.0.0.1 -p 1883 -u admin -P robustmq \
  -t "test/topic" -m "Hello RobustMQ!"
```

If the subscriber receives the message, the service is running correctly.

---

## Default Ports

| Service | Port |
|---------|------|
| MQTT TCP | 1883 |
| MQTT TLS | 1885 |
| MQTT WebSocket | 8083 |
| MQTT WebSocket TLS | 8085 |
| Kafka | 9092 |
| AMQP | 5672 |
| NATS | 4222 |
| Storage engine | 1779 |
| Placement Center gRPC | 1228 |
| HTTP API | 58080 |
