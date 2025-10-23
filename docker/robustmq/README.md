# RobustMQ Application Image

This directory contains all files for building the RobustMQ application image, including the application image and monitoring configuration.

## Files

- `Dockerfile` - Main application image build file
- `docker-compose.yml` - Local development environment configuration
- `monitoring/` - Monitoring configuration directory
  - `prometheus.yml` - Prometheus configuration
  - `grafana/` - Grafana dashboards and configuration

## Purpose

This directory is used to deploy RobustMQ applications, containing a complete runtime environment and monitoring stack.

## Build Commands

```bash
# Build and push to GHCR
make docker-app-ghcr ORG=yourorg VERSION=0.2.0

# Build and push to Docker Hub
make docker-app-dockerhub ORG=yourorg VERSION=0.2.0

# Custom build
make docker-app ARGS='--org yourorg --version 0.2.0 --registry ghcr'
```

## Deployment Commands

```bash
# Start RobustMQ (basic mode)
docker-compose up -d

# Start RobustMQ + monitoring
docker-compose --profile monitoring up -d

# Start microservices mode
docker-compose --profile microservices up -d
```

## Monitoring Access

- **RobustMQ Admin**: http://localhost:8080
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (admin/admin)

## Image Information

- **Multi-stage build**: meta-service, mqtt-broker, admin-server
- **Supported platforms**: linux/amd64, linux/arm64
- **Registry**: GHCR or Docker Hub
- **Monitoring stack**: Prometheus + Grafana + Jaeger
