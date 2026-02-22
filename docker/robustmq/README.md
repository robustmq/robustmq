# RobustMQ Docker

This directory contains the Dockerfile and Docker Compose configuration for running RobustMQ.

## Files

- `Dockerfile` — Multi-stage build: `planner` → `builder` → `runtime` → named targets (`all-in-one`, `meta-service`, `mqtt-broker`)
- `docker-compose.yml` — Local deployment with optional monitoring stack
- `monitoring/` — Prometheus + Grafana configuration for the `--profile monitoring` stack

## Quick Start

```bash
# Run from repo root or docker/robustmq/

# Pull and run pre-built image
docker run -d --name robustmq \
  -p 1883:1883 -p 8083:8083 -p 1228:1228 \
  ghcr.io/robustmq/robustmq:latest

# Or build and run with docker compose (all-in-one)
docker compose up -d robustmq

# With monitoring stack (Prometheus + Grafana)
docker compose --profile monitoring up -d
```

## Monitoring Access

| Service | URL | Credentials |
|---------|-----|-------------|
| RobustMQ HTTP API | http://localhost:8080 | — |
| Prometheus | http://localhost:9090 | — |
| Grafana | http://localhost:3000 | admin / admin |

## Build

```bash
# Build from repo root
docker build -f docker/robustmq/Dockerfile --target all-in-one -t robustmq:local .
```

## Supported Platforms

- `linux/amd64`
- `linux/arm64`
