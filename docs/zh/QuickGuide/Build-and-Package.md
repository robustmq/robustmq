# RobustMQ 构建打包指南

本指南介绍如何构建和打包 RobustMQ。

## 📦 构建产物概览

RobustMQ 构建过程会生成以下类型的产物：

| 产物类型 | 文件格式 | 生成命令 | 用途 |
|----------|----------|----------|------|
| **安装包** | `.tar.gz` 压缩包 | `make build` / `make build-full` | 用户下载安装的二进制包 |
| **Docker 镜像** | Docker 镜像 | `make docker-app-*` | 容器化部署 |
| **依赖镜像** | Docker 镜像 | `make docker-deps` | CI/CD 构建加速 |
| **GitHub 发布** | 在线发布页面 | `make release` | 用户下载和查看发布 |

### 产物详细说明

- **`.tar.gz` 安装包**：包含 Rust 编译的二进制文件、配置文件、启动脚本等，用户解压后即可运行
- **Docker 镜像**：容器化的 RobustMQ 应用，支持 Docker 和 Kubernetes 部署
- **依赖镜像**：预编译的 Rust 依赖缓存，用于加速 CI/CD 构建过程
- **GitHub 发布**：在线发布页面，用户可以通过浏览器下载安装包

## 🚀 快速开始

### 使用 Make 命令（推荐）

| 命令 | 功能 | 版本来源 | 说明 |
|------|------|----------|------|
| `make build` | 基本构建 | 自动读取 Cargo.toml | 构建当前平台包（不含前端） |
| `make build-full` | 完整构建 | 自动读取 Cargo.toml | 构建包含前端的完整包 |
| `make build-version VERSION=v0.1.30` | 指定版本构建 | 手动指定 | 构建指定版本的包 |
| `make build-clean` | 清理重建 | 自动读取 Cargo.toml | 清理后重新构建 |

> **版本说明**：不指定版本时，所有构建命令都会自动从项目根目录的 `Cargo.toml` 文件中读取当前版本号。

## 🐳 Docker 镜像构建

### 依赖镜像（CI/CD 优化）

| 命令 | 功能 | 版本来源 | 说明 |
|------|------|----------|------|
| `make docker-deps` | 构建依赖镜像 | 自动读取 Cargo.toml | 构建 CI/CD 依赖缓存镜像，推送到 robustmq 组织 |
| `make docker-deps-tag TAG=2025-10-20` | 构建带标签依赖镜像 | 手动指定标签 | 构建指定标签的依赖镜像 |
| `make docker-deps-force` | 强制重新构建依赖镜像 | 自动读取 Cargo.toml | 清理缓存后强制重新构建，确保完全重建 |

> **权限说明**：依赖镜像推送到固定的组织：`ghcr.io/robustmq/robustmq/rust-deps`，确保您有 robustmq 组织的写权限。

### 应用镜像

| 命令 | 功能 | 版本来源 | 说明 |
|------|------|----------|------|
| `make docker-app ARGS='--org yourorg --version 0.2.0 --registry ghcr'` | 灵活应用镜像构建 | 手动指定 | 支持自定义参数的应用镜像构建 |
| `make docker-app-ghcr ORG=yourorg VERSION=0.2.0` | GHCR 应用镜像 | 手动指定 | 构建并推送到 GitHub Container Registry |
| `make docker-app-dockerhub ORG=yourorg VERSION=0.2.0` | Docker Hub 应用镜像 | 手动指定 | 构建并推送到 Docker Hub |

### Docker 目录结构

```
docker/
├── deps/                    # 依赖镜像相关文件
│   ├── Dockerfile.deps     # 依赖镜像 Dockerfile
│   ├── install-deps.sh     # 系统依赖安装脚本
│   ├── install-runtime.sh  # 运行时依赖安装脚本
│   ├── .dockerignore       # 依赖镜像构建忽略文件
│   └── README.md           # 依赖镜像说明文档
└── robustmq/               # 应用镜像相关文件
    ├── Dockerfile          # 应用镜像 Dockerfile
    ├── docker-compose.yml  # 本地开发 Docker Compose
    ├── monitoring/         # 监控配置
    │   ├── prometheus.yml  # Prometheus 配置
    │   ├── grafana/        # Grafana 配置
    │   └── jaeger/         # Jaeger 配置
    ├── .dockerignore       # 应用镜像构建忽略文件
    └── README.md           # 应用镜像说明文档
```

## 🚀 版本发布

| 命令 | 功能 | 版本来源 | 说明 |
|------|------|----------|------|
| `make release` | 创建新发布 | 自动读取 Cargo.toml | 创建 GitHub 发布并上传包 |
| `make release-version VERSION=v0.1.30` | 指定版本发布 | 手动指定 | 创建指定版本的 GitHub 发布 |
| `make release-upload VERSION=v0.1.30` | 上传到现有发布 | 手动指定 | 上传包到已存在的 GitHub 发布 |

### 前置条件

```bash
# 设置 GitHub Token（必需）
export GITHUB_TOKEN="your_github_token_here"
```

> **权限说明**：
> - 依赖镜像推送到固定的组织：`ghcr.io/robustmq/robustmq/rust-deps`
> - 应用镜像推送到指定的组织或用户账户
> - 确保您的 `GITHUB_TOKEN` 有 `write:packages` 权限
> - 确保您有 `robustmq` 组织的写权限

## 📦 输出结果

### 构建产物说明

| 产物类型 | 文件位置 | 内容说明 | 用途 |
|----------|----------|----------|------|
| **安装包** | `build/robustmq-{version}-{platform}.tar.gz` | 压缩的二进制安装包 | 用户下载安装 RobustMQ |
| **包信息** | `build/robustmq-{version}-{platform}/package-info.txt` | 版本、平台、构建时间等元数据 | 了解包的详细信息 |
| **Docker 镜像** | `robustmq/robustmq:{version}` | 容器化的 RobustMQ 应用 | Docker 部署和运行 |
| **依赖镜像** | `ghcr.io/robustmq/robustmq/rust-deps:latest` | Rust 依赖缓存镜像 | 加速 CI/CD 构建，存储在 robustmq 组织下 |
| **GitHub 发布** | `https://github.com/robustmq/robustmq/releases/tag/{version}` | 在线发布页面 | 用户下载和查看发布说明 |

### 安装包结构详解

| 目录/文件 | 内容类型 | 具体内容 | 作用 |
|-----------|----------|----------|------|
| `bin/` | 启动脚本 | `robust-server`, `robust-ctl`, `robust-bench` | 系统启动和管理脚本 |
| `libs/` | 二进制可执行文件 | `broker-server`, `cli-command`, `cli-bench` | 核心 Rust 编译的二进制程序 |
| `config/` | 配置文件 | `server.toml`, `server-tracing.toml` | 服务配置和日志配置 |
| `dist/` | 前端静态文件 | HTML, CSS, JavaScript 文件 | Web 管理界面（如果包含前端） |
| `LICENSE` | 许可证文件 | Apache 2.0 许可证文本 | 法律许可信息 |
| `package-info.txt` | 元数据文件 | 版本、平台、构建时间、二进制列表 | 包的详细信息 |


## 📋 使用场景

| 场景 | 命令 | 产物 | 说明 |
|------|------|------|------|
| **开发测试** | `make build` | 本地 `.tar.gz` 安装包 | 快速构建测试包，用于本地开发和测试 |
| **发布准备** | `make build-full` | 本地完整 `.tar.gz` 安装包 | 构建包含前端的完整发布包，用于正式发布 |
| **CI/CD 优化** | `make docker-deps` | Docker 依赖缓存镜像 | 构建 Rust 依赖缓存镜像，推送到 robustmq 组织，加速 CI/CD 构建过程 |
| **强制重建** | `make docker-deps-force` | Docker 依赖缓存镜像 | 清理缓存后强制重新构建，解决缓存问题，确保完全重建 |
| **应用部署** | `make docker-app-ghcr ORG=yourorg VERSION=0.2.0` | Docker 应用镜像 | 构建并推送应用镜像到 GitHub Container Registry |
| **版本发布** | `make release` | GitHub 发布页面 + 安装包 | 创建 GitHub 发布并上传安装包，用户可下载 |
| **多平台发布** | `make release-upload VERSION=v0.1.31` | 更新 GitHub 发布 | 为现有 GitHub 发布添加当前平台的安装包 |


## 🔧 Docker 构建改进

### 权限问题修复

**问题**：之前构建依赖镜像时可能遇到 `permission_denied: create_package` 错误。

**解决方案**：
- 使用固定的组织名称：`ghcr.io/robustmq/robustmq/rust-deps`
- 统一镜像命名，便于 CI/CD 管理
- 确保构建者有 robustmq 组织的写权限

### 网络问题修复

**问题**：构建过程中可能遇到网络连接问题（如 502 Bad Gateway）。

**解决方案**：
- 实现了多镜像源自动切换
- 支持官方 Debian、阿里云、清华大学、中科大、163、华为云、腾讯云等镜像源
- 自动重试机制，提高构建成功率

### 构建优化

**改进**：
- 分离依赖镜像和应用镜像的构建逻辑
- 优化 `.dockerignore` 文件，减少构建上下文
- 添加预构建检查，确保基础镜像可用
- 自动登录 GitHub Container Registry
- 支持强制重新构建，解决缓存问题

### 强制重新构建

**何时使用**：
- 依赖包缓存没有生效
- 镜像构建出现问题
- 需要完全清理缓存重新构建

**使用方法**：
```bash
# 方法 1：使用 make 目标（推荐）
make docker-deps-force

# 方法 2：直接使用脚本
./scripts/build-and-push-deps.sh latest --no-cache

# 方法 3：手动清理后构建
docker builder prune -f
docker rmi ghcr.io/robustmq/robustmq/rust-deps:latest
make docker-deps
```

## ⚠️ 注意事项

### 构建脚本限制

| 特性 | 状态 | 说明 |
|------|------|------|
| 当前系统平台 | ✅ | 只构建当前系统平台 |
| 自动版本检测 | ✅ | 自动从 Cargo.toml 读取版本 |
| 发布模式构建 | ✅ | 使用 `cargo build --release` |
| 自动前端克隆 | ✅ | 自动克隆前端代码 |
| 交叉编译 | ❌ | 不支持交叉编译 |

### 发布脚本限制

| 特性 | 状态 | 说明 |
|------|------|------|
| 前端构建 | ✅ | 总是包含前端构建 |
| 当前平台 | ✅ | 总是构建当前系统平台 |
| 现有发布上传 | ❌ | `--upload-only` 需要现有发布 |

## 🔧 环境要求

| 类型 | 工具 | 用途 | 必需性 |
|------|------|------|--------|
| **基本** | Rust (`cargo`, `rustup`) | Rust 编译 | ✅ 必需 |
| **前端** | `pnpm` | 前端包管理 | 🔶 可选 |
| **前端** | `git` | 克隆前端代码 | 🔶 可选 |
| **Docker** | `docker` | Docker 环境 | 🔶 可选 |
| **发布** | `curl` | API 请求 | 🔶 可选 |
| **发布** | `jq` | JSON 解析 | 🔶 可选 |
| **发布** | `GITHUB_TOKEN` | GitHub 访问令牌 | 🔶 可选 |

## 🆘 常见问题

### 构建失败排查

| 问题 | 检查命令 | 解决方案 |
|------|----------|----------|
| Rust 环境问题 | `cargo --version` | 安装 Rust 环境 |
| Docker 环境问题 | `docker info` | 启动 Docker 服务 |
| 网络连接问题 | `ping github.com` | 检查网络连接 |

### Docker 构建失败排查

| 问题 | 检查命令 | 解决方案 |
|------|----------|----------|
| Docker 环境问题 | `docker info` | 启动 Docker 服务 |
| 权限问题 | `echo $GITHUB_TOKEN` | 设置正确的 GitHub Token |
| 网络连接问题 | `docker pull rust:1.90.0-bookworm` | 检查 Docker Hub 连接 |
| 镜像推送失败 | `docker login ghcr.io` | 手动登录 GitHub Container Registry |

### 发布失败排查

| 问题 | 检查命令 | 解决方案 |
|------|----------|----------|
| GitHub Token 问题 | `echo $GITHUB_TOKEN` | 设置正确的 Token |
| Token 权限问题 | `curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user` | 检查 Token 权限 |
| 网络连接问题 | `curl -I https://api.github.com` | 检查网络连接 |

### 查看构建产物

| 操作 | 命令 | 说明 |
|------|------|------|
| 查看构建目录 | `ls -la build/` | 查看所有构建产物 |
| 解压测试 | `tar -xzf build/robustmq-*.tar.gz` | 解压安装包 |
| 测试二进制 | `./robustmq-*/libs/broker-server --help` | 测试可执行文件 |
| 查看包信息 | `cat robustmq-*/package-info.txt` | 查看包详细信息 |

### 查看 Docker 镜像

| 操作 | 命令 | 说明 |
|------|------|------|
| 查看本地镜像 | `docker images | grep robustmq` | 查看本地构建的镜像 |
| 查看依赖镜像 | `docker images | grep rust-deps` | 查看依赖缓存镜像 |
| 测试镜像 | `docker run --rm ghcr.io/robustmq/robustmq/rust-deps:latest rustc --version` | 测试依赖镜像 |
| 查看镜像历史 | `docker history ghcr.io/robustmq/robustmq/rust-deps:latest` | 查看镜像构建历史 |
