# Docker 部署

本指南介绍如何使用 Docker 部署 RobustMQ，适用于开发测试和生产环境。

## 准备工作

### 系统要求

- **Docker**: 版本 20.10+
- **Docker Compose**: 版本 2.0+
- **内存**: 至少 2GB 可用内存
- **磁盘**: 至少 10GB 可用空间

### 安装 Docker

```bash
# Ubuntu/Debian
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# CentOS/RHEL
sudo yum install -y docker docker-compose
sudo systemctl start docker
sudo systemctl enable docker
```

## 快速开始

### 下载项目

```bash
# 克隆项目
git clone https://github.com/robustmq/robustmq.git
cd robustmq

# 或者下载发布版本
wget https://github.com/robustmq/robustmq/archive/refs/tags/v1.0.0.tar.gz
tar -xzf v1.0.0.tar.gz
cd robustmq-1.0.0
```

### 一键启动（推荐）

```bash
# 启动 RobustMQ（一体化部署）
docker-compose -f docker/docker-compose.yml up -d robustmq

# 查看日志
docker-compose -f docker/docker-compose.yml logs -f robustmq

# 停止服务
docker-compose -f docker/docker-compose.yml down
```

## 部署模式

### 一体化部署（开发测试）

一体化部署将所有服务运行在单个容器中，适合开发和测试环境。

```bash
# 启动一体化服务
docker-compose -f docker/docker-compose.yml up -d robustmq

# 查看服务状态
docker-compose -f docker/docker-compose.yml ps

# 查看日志
docker-compose -f docker/docker-compose.yml logs robustmq
```

**端口映射**:

- MQTT TCP: 1883
- MQTT SSL: 1885
- MQTT WebSocket: 8083
- MQTT WebSocket SSL: 8085
- Kafka: 9092
- gRPC: 1228
- AMQP: 5672

### 微服务部署（生产环境）

微服务部署将不同服务运行在独立容器中，适合生产环境。

```bash
# 启动所有微服务
docker-compose -f docker/docker-compose.yml --profile microservices up -d

# 或者分别启动服务
docker-compose -f docker/docker-compose.yml up -d meta-service
docker-compose -f docker/docker-compose.yml up -d mqtt-broker
docker-compose -f docker/docker-compose.yml up -d journal-service
```

**服务说明**:

- `meta-service`: 元数据管理和集群协调
- `mqtt-broker`: MQTT 协议处理
- `journal-service`: 持久化存储层

### 监控部署（可选）

包含 Prometheus、Grafana 和 Jaeger 的完整监控栈。

```bash
# 启动 RobustMQ + 监控栈
docker-compose -f docker/docker-compose.yml --profile monitoring up -d
```

**监控服务**:

- Prometheus: `http://localhost:9090` (监控栈运行时访问)
- Grafana: `http://localhost:3000` (admin/admin，监控栈运行时访问)
- Jaeger: `http://localhost:16686` (监控栈运行时访问)

## 配置管理

### 环境变量

| 变量名 | 默认值 | 描述 |
|--------|--------|------|
| `ROBUSTMQ_ROLES` | `meta,broker,journal` | 服务角色 |
| `ROBUSTMQ_CONFIG_PATH` | `/robustmq/config/server.toml` | 配置文件路径 |
| `ROBUSTMQ_LOG_PATH` | `/robustmq/logs` | 日志目录 |
| `ROBUSTMQ_DATA_PATH` | `/robustmq/data` | 数据目录 |
| `RUST_LOG` | `info` | 日志级别 |

### 配置文件

将配置文件放在 `config/` 目录下：

```bash
# 创建配置目录
mkdir -p config

# 复制默认配置
cp config/server.toml.template config/server.toml

# 编辑配置
vim config/server.toml
```

### 自定义配置

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

## 验证部署

### 检查服务状态

```bash
# 查看容器状态
docker-compose -f docker/docker-compose.yml ps

# 查看健康状态
curl http://localhost:8083/health

# 查看容器日志
docker-compose -f docker/docker-compose.yml logs robustmq
```

### 测试 MQTT 连接

```bash
# 发送消息
docker run --rm --network robustmq_default \
  eclipse-mosquitto mosquitto_pub \
  -h robustmq -p 1883 \
  -t "test/topic" -m "Hello RobustMQ"

# 订阅消息
docker run --rm --network robustmq_default \
  eclipse-mosquitto mosquitto_sub \
  -h robustmq -p 1883 \
  -t "test/topic"
```

### 使用 mqttx 测试

```bash
# 发送消息
mqttx pub -h localhost -p 1883 -t "test/docker" -m "Docker deployment test"

# 订阅消息
mqttx sub -h localhost -p 1883 -t "test/docker"
```

## 数据管理

### 数据持久化

Docker Compose 使用命名卷来持久化数据：

```bash
# 查看数据卷
docker volume ls | grep robustmq

# 备份数据
docker run --rm -v robustmq_robustmq_data:/data -v $(pwd):/backup \
  alpine tar czf /backup/robustmq-data-backup.tar.gz -C /data .

# 恢复数据
docker run --rm -v robustmq_robustmq_data:/data -v $(pwd):/backup \
  alpine tar xzf /backup/robustmq-data-backup.tar.gz -C /data
```

### 日志管理

```bash
# 查看应用日志
docker-compose -f docker/docker-compose.yml logs robustmq

# 查看实时日志
docker-compose -f docker/docker-compose.yml logs -f robustmq

# 清理日志
docker-compose -f docker/docker-compose.yml logs --tail=100 robustmq
```

## 管理命令

### 容器管理

```bash
# 重启服务
docker-compose -f docker/docker-compose.yml restart robustmq

# 停止服务
docker-compose -f docker/docker-compose.yml stop robustmq

# 删除容器
docker-compose -f docker/docker-compose.yml rm robustmq

# 进入容器
docker exec -it robustmq bash
```

### 服务管理

```bash
# 查看集群状态
docker exec robustmq ./libs/cli-command cluster info

# 查看 MQTT 状态
docker exec robustmq ./libs/cli-command mqtt status

# 运行性能测试
docker exec robustmq ./libs/cli-bench mqtt --help
```

## 生产环境配置

### 资源限制

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

### 网络配置

```yaml
# 自定义网络
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

### 安全配置

```yaml
# 安全配置
services:
  robustmq:
    user: "1000:1000"  # 非 root 用户
    read_only: true
    tmpfs:
      - /tmp
      - /var/tmp
    security_opt:
      - no-new-privileges:true
```

## 故障排除

### 常见问题

1. **端口冲突**

   ```bash
   # 检查端口占用
   netstat -tlnp | grep 1883

   # 修改端口映射
   ports:
     - "1885:1883"  # 使用不同端口
   ```

2. **权限问题**

   ```bash
   # 修复数据目录权限
   sudo chown -R 1000:1000 ./data
   ```

3. **配置错误**

   ```bash
   # 验证配置文件
   docker exec robustmq ./libs/broker-server --help
   ```

### 调试命令

```bash
# 查看容器详细信息
docker inspect robustmq

# 查看资源使用情况
docker stats robustmq

# 查看网络信息
docker network ls
docker network inspect robustmq_default

# 查看卷信息
docker volume inspect robustmq_robustmq_data
```

## 升级和维护

### 版本升级

```bash
# 停止服务
docker-compose -f docker/docker-compose.yml down

# 拉取新镜像
docker-compose -f docker/docker-compose.yml pull

# 启动新版本
docker-compose -f docker/docker-compose.yml up -d
```

### 定期维护

```bash
# 清理未使用的镜像
docker image prune -f

# 清理未使用的卷
docker volume prune -f

# 清理未使用的网络
docker network prune -f
```
