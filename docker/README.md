# RobustMQ Docker Deployment

This directory contains Docker configurations for deploying RobustMQ in various modes.

## 🚀 Quick Start

### All-in-One Deployment (Recommended for Development)

```bash
# Build and start RobustMQ with all services in one container
docker-compose up -d robustmq

# View logs
docker-compose logs -f robustmq

# Stop
docker-compose down
```

### Microservices Deployment (Recommended for Production)

```bash
# Start all microservices
docker-compose --profile microservices up -d

# Or start individual services
docker-compose up -d meta-service
docker-compose up -d mqtt-broker
docker-compose up -d journal-service
```

### With Monitoring Stack

```bash
# Start RobustMQ with Prometheus, Grafana, and Jaeger
docker-compose --profile monitoring up -d
```

## 📋 Available Services

### Core Services

| Service | Ports | Description |
|---------|-------|-------------|
| `robustmq` | 1883, 1884, 8083, 8084, 9092, 1228, 5672 | All-in-one deployment |
| `meta-service` | 1228 | Metadata management and cluster coordination |
| `mqtt-broker` | 1883, 1884, 8083, 8084 | MQTT protocol handler |
| `journal-service` | 1229 | Persistent storage layer |

### Monitoring Services (Optional)

| Service | Port | Credentials | Description |
|---------|------|-------------|-------------|
| `prometheus` | 9090 | - | Metrics collection |
| `grafana` | 3000 | admin/admin | Metrics visualization |
| `jaeger` | 16686 | - | Distributed tracing |

## 🛠️ Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ROBUSTMQ_ROLES` | `meta,broker,journal` | Comma-separated list of roles |
| `ROBUSTMQ_CONFIG_PATH` | `/robustmq/config/server.toml` | Configuration file path |
| `ROBUSTMQ_LOG_PATH` | `/robustmq/logs` | Log directory |
| `ROBUSTMQ_DATA_PATH` | `/robustmq/data` | Data directory |
| `RUST_LOG` | `info` | Log level |

### Configuration Files

Place your configuration files in the `config/` directory:

- `server.toml` - All-in-one configuration
- `meta-service.toml` - Meta service specific
- `mqtt-broker.toml` - MQTT broker specific  
- `journal-service.toml` - Journal service specific

## 🔧 Building Custom Images

### Build Specific Stages

```bash
# All-in-one (default)
docker build -f docker/Dockerfile --target all-in-one -t robustmq:latest .

# Meta service only
docker build -f docker/Dockerfile --target meta-service -t robustmq:meta .

# MQTT broker only
docker build -f docker/Dockerfile --target mqtt-broker -t robustmq:mqtt .

# Journal service only
docker build -f docker/Dockerfile --target journal-service -t robustmq:journal .

# Development image
docker build -f docker/Dockerfile --target development -t robustmq:dev .
```

### Multi-platform Build

```bash
# Build for multiple architectures
docker buildx build --platform linux/amd64,linux/arm64 \
  -f docker/Dockerfile \
  -t robustmq:latest \
  --push .
```

## 📊 Monitoring

### Accessing Monitoring Tools

1. **Grafana**: <http://localhost:3000> (admin/admin)
2. **Prometheus**: <http://localhost:9090>
3. **Jaeger**: <http://localhost:16686>

### Health Checks

```bash
# Check service health
curl http://localhost:8083/health

# Check service status via CLI
docker exec robustmq ./libs/cli-command mqtt status
```

## 🔄 Development Workflow

### Local Development

```bash
# Start development container
docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d

# Access container for debugging
docker exec -it robustmq bash

# View real-time logs
docker-compose logs -f robustmq
```

### Testing

```bash
# Run integration tests
docker-compose exec robustmq ./libs/cli-bench mqtt --help

# MQTT client test
docker run --rm -it --network docker_default \
  eclipse-mosquitto mosquitto_pub \
  -h robustmq -p 1883 \
  -t "test/topic" -m "Hello RobustMQ"
```

## 🐛 Troubleshooting

### Common Issues

1. **Port conflicts**: Ensure ports are not already in use
2. **Permission denied**: Check file permissions in mounted volumes
3. **Configuration errors**: Validate TOML syntax in config files

### Debug Commands

```bash
# Check container status
docker-compose ps

# View detailed logs
docker-compose logs --details robustmq

# Check resource usage
docker stats robustmq

# Access container shell
docker exec -it robustmq bash
```

### Log Locations

- Container logs: `docker-compose logs`
- Application logs: `./logs/` (mounted volume)
- System logs: `/robustmq/logs/` (inside container)

## 📁 Directory Structure

```text
docker/
├── Dockerfile              # Main Dockerfile with multi-stage builds
├── docker-compose.yml      # Service definitions
├── .dockerignore           # Files to exclude from build context
├── README.md               # This file
├── config/                 # Configuration templates
│   ├── server.toml.template
│   ├── meta-service.toml.template
│   ├── mqtt-broker.toml.template
│   └── journal-service.toml.template
└── monitoring/             # Monitoring configurations
    ├── prometheus.yml
    └── grafana/
        ├── dashboards/
        └── datasources/
```

## 🔒 Security Considerations

1. **Non-root user**: Containers run as `robustmq` user
2. **Read-only configs**: Configuration mounted as read-only
3. **Network isolation**: Services use dedicated Docker network
4. **Resource limits**: Configure appropriate CPU and memory limits

## 📈 Performance Tuning

### Resource Limits

Add to your `docker-compose.yml`:

```yaml
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
```

### Volume Performance

For better I/O performance, consider using:

- Local SSD storage for data volumes
- Separate volumes for logs and data
- Regular cleanup of log files

## 🔄 Backup and Recovery

### Data Backup

```bash
# Backup data volumes
docker run --rm -v docker_robustmq_data:/data -v $(pwd):/backup \
  alpine tar czf /backup/robustmq-data-backup.tar.gz -C /data .

# Restore data volumes
docker run --rm -v docker_robustmq_data:/data -v $(pwd):/backup \
  alpine tar xzf /backup/robustmq-data-backup.tar.gz -C /data
```

### Configuration Backup

```bash
# Backup configurations
cp -r config/ config-backup-$(date +%Y%m%d)/
```

## 📞 Support

For issues and questions:

- GitHub Issues: <https://github.com/robustmq/robustmq/issues>
- Documentation: <https://robustmq.com/>
- Discord: <https://discord.gg/sygeGRh5>
