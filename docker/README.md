# RobustMQ Docker Images

## 1. Build Dependency Image (CI Acceleration)

```bash
# Login
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# Build dependency image
cd docker/
./build-and-push.sh
```

**Purpose:** Pre-compile all Rust dependencies to accelerate CI/CD builds

**Image:** `ghcr.io/socutes/robustmq/rust-deps:latest`

## 2. Build Application Image (Production Deployment)

```bash
# 本地构建应用镜像
VERSION=0.2.0
docker build -f docker/Dockerfile -t robustmq:${VERSION} .

# 仅构建某个服务（可选：meta-service/mqtt-broker/journal-service/all-in-one）
docker build -f docker/Dockerfile --target mqtt-broker -t robustmq:${VERSION} .

# 运行容器（示例）
docker run -d --name robustmq robustmq:${VERSION}

# macOS/arm64 构建 linux/amd64 镜像（如需）
docker buildx create --use --name robustx || true
docker buildx build --platform linux/amd64 -f docker/Dockerfile -t robustmq:${VERSION} --load .
```

**Purpose:** Deploy RobustMQ service in production environment

**Image:** `robustmq:<version>`

### 推送到镜像仓库

#### 方式 A：使用一键脚本（推荐）

```bash
# 推送到 GHCR（GitHub Container Registry）
./docker/build-and-push-app.sh --version 0.2.0 --org socutes --registry ghcr --push-latest

# 推送到 Docker Hub
./docker/build-and-push-app.sh --version 0.2.0 --org your_dockerhub_user --registry dockerhub

# 仅构建并推送指定阶段（示例：仅 MQTT Broker）
./docker/build-and-push-app.sh --version 0.2.0 --org socutes --registry ghcr --target mqtt-broker

# 在 macOS/arm64 上构建 linux/amd64 并推送
./docker/build-and-push-app.sh --version 0.2.0 --org socutes --registry ghcr --platform linux/amd64
```

登录说明：
```bash
# GHCR 登录
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# Docker Hub 登录
docker login
```

脚本参数：
```text
--version <version>     镜像版本标签（默认：latest）
--org <org/user>        组织或用户名（必填）
--name <image-name>     镜像名（默认：robustmq）
--registry ghcr|dockerhub  目标仓库（默认：ghcr）
--target <stage>        指定 Dockerfile 构建阶段（可选）
--platform <platform>   目标平台（如：linux/amd64，可选）
--push-latest           额外推送 latest 标签（可选）
```

#### 方式 B：手动打标签并推送

```bash
# GHCR
ORG=socutes
docker tag robustmq:${VERSION} ghcr.io/${ORG}/robustmq:${VERSION}
docker push ghcr.io/${ORG}/robustmq:${VERSION}
docker tag robustmq:${VERSION} ghcr.io/${ORG}/robustmq:latest
docker push ghcr.io/${ORG}/robustmq:latest

# Docker Hub
USER=your_dockerhub_user
docker tag robustmq:${VERSION} ${USER}/robustmq:${VERSION}
docker push ${USER}/robustmq:${VERSION}
docker tag robustmq:${VERSION} ${USER}/robustmq:latest
docker push ${USER}/robustmq:latest
```

## Use Dependency Image in CI

```yaml
container:
  image: ghcr.io/socutes/robustmq/rust-deps:latest
  credentials:
    username: ${{ github.actor }}
    password: ${{ secrets.GITHUB_TOKEN }}
```

## When to Update Dependency Image

- Every 2-4 weeks
- Rust version upgrades
- 20+ dependency changes
- CI takes longer than 8 minutes

## Troubleshooting

```bash
# Check disk space
df -h

# Check Docker
docker info

# Re-login
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin
```