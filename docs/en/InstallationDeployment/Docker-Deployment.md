# Docker Deployment

This guide describes how to deploy RobustMQ using Docker, suitable for development, testing, and production environments.

## Prerequisites

### System Requirements

- **Docker**: Version 20.10+
- **Docker Compose**: Version 2.0+
- **Memory**: At least 2GB available memory
- **Disk**: At least 10GB available space

### Install Docker

```bash
# Ubuntu/Debian
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# CentOS/RHEL
sudo yum install -y docker docker-compose
sudo systemctl start docker
sudo systemctl enable docker
```

## Quick Start

### Download Project

```bash
# Clone project
git clone https://github.com/robustmq/robustmq.git
cd robustmq

# Or download release version
wget https://github.com/robustmq/robustmq/archive/refs/tags/v1.0.0.tar.gz
tar -xzf v1.0.0.tar.gz
cd robustmq-1.0.0
```

### One-Click Start (Recommended)

```bash
# Start RobustMQ (all-in-one deployment)
docker-compose -f docker/docker-compose.yml up -d robustmq

# View logs
docker-compose -f docker/docker-compose.yml logs -f robustmq

# Stop service
docker-compose -f docker/docker-compose.yml down
```

## Deployment Modes

### All-in-One Deployment (Development/Testing)

All-in-one deployment runs all services in a single container, suitable for development and testing environments.

```bash
# Start all-in-one service
docker-compose -f docker/docker-compose.yml up -d robustmq

# View service status
docker-compose -f docker/docker-compose.yml ps

# View logs
docker-compose -f docker/docker-compose.yml logs robustmq
```

**Port Mapping**:

- MQTT TCP: 1883
- MQTT SSL: 1885
- MQTT WebSocket: 8083
- MQTT WebSocket SSL: 8085
- Kafka: 9092
- gRPC: 1228
- AMQP: 5672

### Microservices Deployment (Production)

Microservices deployment runs different services in independent containers, suitable for production environments.

```bash
# Start all microservices
docker-compose -f docker/docker-compose.yml --profile microservices up -d

# Or start services separately
docker-compose -f docker/docker-compose.yml up -d meta-service
docker-compose -f docker/docker-compose.yml up -d mqtt-broker
docker-compose -f docker/docker-compose.yml up -d journal-service
```

**Service Description**:

- `meta-service`: Metadata management and cluster coordination
- `mqtt-broker`: MQTT protocol handling
- `journal-service`: Persistent storage layer

### Monitoring Deployment (Optional)

Complete monitoring stack including Prometheus, Grafana, and Jaeger.

```bash
# Start RobustMQ + monitoring stack
docker-compose -f docker/docker-compose.yml --profile monitoring up -d
```

**Monitoring Services**:

- Prometheus: `http://localhost:9090` (when monitoring stack is running)
- Grafana: `http://localhost:3000` (admin/admin, when monitoring stack is running)
- Jaeger: `http://localhost:16686` (when monitoring stack is running)

## Configuration Management

### Environment Variables

| Variable Name | Default Value | Description |
|---------------|---------------|-------------|
| `ROBUSTMQ_ROLES` | `meta,broker,journal` | Service roles |
| `ROBUSTMQ_CONFIG_PATH` | `/robustmq/config/server.toml` | Configuration file path |
| `ROBUSTMQ_LOG_PATH` | `/robustmq/logs` | Log directory |
| `ROBUSTMQ_DATA_PATH` | `/robustmq/data` | Data directory |
| `RUST_LOG` | `info` | Log level |

### Configuration Files

Place configuration files in the `config/` directory:

```bash
# Create configuration directory
mkdir -p config

# Copy default configuration
cp config/server.toml.template config/server.toml

# Edit configuration
vim config/server.toml
```

### Custom Configuration

```yaml
# docker-compose.override.yml
version: '3.8'
services:
  robustmq:
    environment:
      - RUST_LOG=debug
      - ROBUSTMQ_ROLES=meta,broker
    volumes:
      - ./custom-config:/robustmq/config:ro
    ports:
      - "1883:1883"
      - "8080:8080"
```

## Verifying Deployment

### Check Service Status

```bash
# View container status
docker-compose -f docker/docker-compose.yml ps

# Check health status
curl http://localhost:8083/health

# View container logs
docker-compose -f docker/docker-compose.yml logs robustmq
```

### Test MQTT Connection

```bash
# Send message
docker run --rm --network robustmq_default \
  eclipse-mosquitto mosquitto_pub \
  -h robustmq -p 1883 \
  -t "test/topic" -m "Hello RobustMQ"

# Subscribe to messages
docker run --rm --network robustmq_default \
  eclipse-mosquitto mosquitto_sub \
  -h robustmq -p 1883 \
  -t "test/topic"
```

### Test with mqttx

```bash
# Send message
mqttx pub -h localhost -p 1883 -t "test/docker" -m "Docker deployment test"

# Subscribe to messages
mqttx sub -h localhost -p 1883 -t "test/docker"
```

## Data Management

### Data Persistence

Docker Compose uses named volumes for data persistence:

```bash
# View data volumes
docker volume ls | grep robustmq

# Backup data
docker run --rm -v robustmq_robustmq_data:/data -v $(pwd):/backup \
  alpine tar czf /backup/robustmq-data-backup.tar.gz -C /data .

# Restore data
docker run --rm -v robustmq_robustmq_data:/data -v $(pwd):/backup \
  alpine tar xzf /backup/robustmq-data-backup.tar.gz -C /data
```

### Log Management

```bash
# View application logs
docker-compose -f docker/docker-compose.yml logs robustmq

# View real-time logs
docker-compose -f docker/docker-compose.yml logs -f robustmq

# Clean logs
docker-compose -f docker/docker-compose.yml logs --tail=100 robustmq
```

## Management Commands

### Container Management

```bash
# Restart service
docker-compose -f docker/docker-compose.yml restart robustmq

# Stop service
docker-compose -f docker/docker-compose.yml stop robustmq

# Remove container
docker-compose -f docker/docker-compose.yml rm robustmq

# Enter container
docker exec -it robustmq bash
```

### Service Management

```bash
# View cluster status
docker exec robustmq ./libs/cli-command cluster info

# View MQTT status
docker exec robustmq ./libs/cli-command mqtt status

# Run performance test
docker exec robustmq ./libs/cli-bench mqtt --help
```

## Production Environment Configuration

### Resource Limits

```yaml
# docker-compose.prod.yml
version: '3.8'
services:
  robustmq:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
        reservations:
          cpus: '1.0'
          memory: 1G
    restart: unless-stopped
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

### Network Configuration

```yaml
# Custom network
networks:
  robustmq-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16

services:
  robustmq:
    networks:
      - robustmq-network
```

### Security Configuration

```yaml
# Security configuration
services:
  robustmq:
    user: "1000:1000"  # Non-root user
    read_only: true
    tmpfs:
      - /tmp
      - /var/tmp
    security_opt:
      - no-new-privileges:true
```

## Troubleshooting

### Common Issues

1. **Port Conflicts**

   ```bash
   # Check port usage
   netstat -tlnp | grep 1883

   # Modify port mapping
   ports:
     - "1885:1883"  # Use different port
   ```

2. **Permission Issues**

   ```bash
   # Fix data directory permissions
   sudo chown -R 1000:1000 ./data
   ```

3. **Configuration Errors**

   ```bash
   # Validate configuration file
   docker exec robustmq ./libs/broker-server --help
   ```

### Debug Commands

```bash
# View container details
docker inspect robustmq

# View resource usage
docker stats robustmq

# View network information
docker network ls
docker network inspect robustmq_default

# View volume information
docker volume inspect robustmq_robustmq_data
```

## Upgrade and Maintenance

### Version Upgrade

```bash
# Stop service
docker-compose -f docker/docker-compose.yml down

# Pull new image
docker-compose -f docker/docker-compose.yml pull

# Start new version
docker-compose -f docker/docker-compose.yml up -d
```

### Regular Maintenance

```bash
# Clean unused images
docker image prune -f

# Clean unused volumes
docker volume prune -f

# Clean unused networks
docker network prune -f
```
