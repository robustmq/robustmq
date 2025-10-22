# RobustMQ Application Image

这个目录包含用于构建 RobustMQ 应用镜像的所有文件。

## 文件说明

- `Dockerfile` - 主要的应用镜像构建文件
- `docker-compose.yml` - 本地开发环境配置

## 用途

这个镜像用于部署 RobustMQ 应用程序，包含完整的运行时环境。

## 构建命令

```bash
# 构建并推送到 GHCR
make docker-app-ghcr ORG=yourorg VERSION=0.2.0

# 构建并推送到 Docker Hub
make docker-app-dockerhub ORG=yourorg VERSION=0.2.0

# 自定义构建
make docker-app ARGS='--org yourorg --version 0.2.0 --registry ghcr'
```

## 镜像信息

- **多阶段构建**: meta-service, mqtt-broker, admin-server
- **支持平台**: linux/amd64, linux/arm64
- **注册表**: GHCR 或 Docker Hub
