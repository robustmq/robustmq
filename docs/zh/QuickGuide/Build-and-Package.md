# RobustMQ 构建打包指南

本指南介绍如何构建和打包 RobustMQ。

## 📦 构建产物概览

RobustMQ 构建过程会生成以下类型的产物：

| 产物类型 | 文件格式 | 生成命令 | 用途 |
|----------|----------|----------|------|
| **安装包** | `.tar.gz` 压缩包 | `make build` / `make build-full` | 用户下载安装的二进制包 |
| **Docker 镜像** | Docker 镜像 | `make docker-app-*` | 容器化部署 |
| **GitHub 发布** | 在线发布页面 | `make release` | 用户下载和查看发布 |

### 产物详细说明

- **`.tar.gz` 安装包**：包含 Rust 编译的二进制文件、配置文件、启动脚本等，用户解压后即可运行
- **Docker 镜像**：容器化的 RobustMQ 应用，支持 Docker 和 Kubernetes 部署（使用 cargo-chef 优化依赖缓存）
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

### 应用镜像

| 命令 | 功能 | 版本来源 | 说明 |
|------|------|----------|------|
| `make docker-app ARGS='--org yourorg --version 0.2.0 --registry ghcr'` | 灵活应用镜像构建 | 手动指定 | 支持自定义参数的应用镜像构建 |
| `make docker-app-ghcr ORG=yourorg VERSION=0.2.0` | GHCR 应用镜像 | 手动指定 | 构建并推送到 GitHub Container Registry |
| `make docker-app-dockerhub ORG=yourorg VERSION=0.2.0` | Docker Hub 应用镜像 | 手动指定 | 构建并推送到 Docker Hub |

### Docker 目录结构

```
docker/
└── robustmq/               # 应用镜像相关文件
    ├── Dockerfile          # 应用镜像 Dockerfile（使用 cargo-chef 优化依赖缓存）
    ├── docker-compose.yml  # 本地开发 Docker Compose
    ├── monitoring/         # 监控配置
    │   ├── prometheus.yml  # Prometheus 配置
    │   ├── grafana/        # Grafana 配置
    │   └── jaeger/         # Jaeger 配置
    ├── .dockerignore       # 应用镜像构建忽略文件
    └── README.md           # 应用镜像说明文档
```

> **构建优化**：Dockerfile 使用 cargo-chef 实现依赖层缓存，无需单独的 deps 镜像。cargo-chef 会自动生成依赖配方（recipe），只有当 Cargo.lock 改变时才重新编译依赖。

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
> - 应用镜像推送到指定的组织或用户账户
> - 确保您的 `GITHUB_TOKEN` 有 `write:packages` 权限

## 📦 输出结果

### 构建产物说明

| 产物类型 | 文件位置 | 内容说明 | 用途 |
|----------|----------|----------|------|
| **安装包** | `build/robustmq-{version}-{platform}.tar.gz` | 压缩的二进制安装包 | 用户下载安装 RobustMQ |
| **包信息** | `build/robustmq-{version}-{platform}/package-info.txt` | 版本、平台、构建时间等元数据 | 了解包的详细信息 |
| **Docker 镜像** | `robustmq/robustmq:{version}` | 容器化的 RobustMQ 应用（内置 cargo-chef 优化） | Docker 部署和运行 |
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
| **应用部署** | `make docker-app-ghcr ORG=yourorg VERSION=0.2.0` | Docker 应用镜像 | 构建并推送应用镜像到 GitHub Container Registry（使用 cargo-chef 自动优化依赖缓存） |
| **版本发布** | `make release` | GitHub 发布页面 + 安装包 | 创建 GitHub 发布并上传安装包，用户可下载 |
| **多平台发布** | `make release-upload VERSION=v0.1.31` | 更新 GitHub 发布 | 为现有 GitHub 发布添加当前平台的安装包 |


## 🔧 Docker 构建优化

### cargo-chef 依赖缓存

**方案**：RobustMQ 使用 **cargo-chef** 实现高效的依赖层缓存，无需单独的 deps 镜像。

**工作原理**：
1. **Planner 阶段**：分析 Cargo.toml 和 Cargo.lock，生成依赖配方（recipe.json）
2. **Builder 阶段**：根据配方先编译依赖，再编译应用代码
3. **分层缓存**：只有 Cargo.lock 改变时才重新编译依赖，否则使用缓存层

**优势**：
- ✅ 自动化依赖缓存管理，无需手动维护
- ✅ Docker 原生层缓存，构建速度快
- ✅ 只缓存纯依赖，不包含应用代码
- ✅ 依赖改变时自动更新，应用代码改变时复用依赖缓存

### 构建加速技巧

**清理缓存重建**：
```bash
# 清理 Docker 构建缓存
docker builder prune -f

# 重新构建镜像（不使用缓存）
docker build --no-cache -f docker/robustmq/Dockerfile -t robustmq:latest .
```

**多阶段构建**：
- Development 阶段：包含调试工具，用于开发和调试
- Runtime 阶段：精简的生产镜像，只包含运行时必需组件

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
| 测试镜像 | `docker run --rm robustmq/robustmq:latest --help` | 测试应用镜像 |
| 查看镜像历史 | `docker history robustmq/robustmq:latest` | 查看镜像构建历史 |
| 查看镜像大小 | `docker images robustmq/robustmq --format "\{\{.Size\}\}"` | 查看镜像大小 |