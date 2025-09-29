# RobustMQ 编译打包指南

本指南介绍如何编译和打包 RobustMQ，包括本地构建、Docker 镜像构建和发布流程。

## 目录

- [环境准备](#环境准备)
- [快速开始](#快速开始)
- [构建选项](#构建选项)
- [Docker 构建](#docker-构建)
- [发布流程](#发布流程)
- [常见问题](#常见问题)

## 环境准备

请参考 [基础开发环境搭建](../ContributionGuide/ContributingCode/Build-Develop-Env.md) 完成开发环境的配置。

### 额外依赖（可选）

- **前端构建**: `pnpm` 和 `git` 已安装（仅在使用 `--with-frontend` 时需要）
- **Docker 构建**: `docker` 已安装并运行（仅在使用 `--with-docker` 时需要）

> **提示**: 如果缺少必要的工具，构建脚本会自动检测并显示详细的安装指令，包括适用于不同操作系统的安装命令。

## 快速开始

### 1. 克隆项目

```bash
git clone https://github.com/robustmq/robustmq.git
cd robustmq
```

### 2. 基本构建

```bash
# 构建当前平台包（使用 Cargo.toml 中的版本）
./scripts/build.sh

# 构建包含前端的完整包
./scripts/build.sh --with-frontend

# 构建 Docker 镜像
./scripts/build.sh --with-docker

# 指定版本构建
./scripts/build.sh --version v0.1.31
```

> **版本说明**: 如果不指定 `--version` 参数，构建脚本会自动从项目根目录的 `Cargo.toml` 文件中读取版本号。

### 3. 查看构建结果

```bash
ls -la build/
# 输出示例：
# robustmq-0.1.35-darwin-arm64.tar.gz
# robustmq-0.1.35-darwin-arm64.tar.gz.sha256
```

## 构建选项

### build.sh 脚本

`scripts/build.sh` 是主要的构建脚本，支持以下选项：

```bash
./scripts/build.sh [OPTIONS]

选项:
  -h, --help              显示帮助
  -v, --version VERSION   指定版本（默认从项目根目录 Cargo.toml 文件读取）
  --with-frontend         包含前端构建
  --with-docker           构建 Docker 镜像
  --clean                 清理构建目录
```

### 使用场景

#### 场景1: 开发测试
```bash
# 快速构建当前平台（自动使用 Cargo.toml 中的版本）
./scripts/build.sh
```

#### 场景2: 完整发布包
```bash
# 构建包含前端的完整包（自动克隆前端代码）
./scripts/build.sh --with-frontend

# 指定版本构建完整发布包
./scripts/build.sh --with-frontend --version v0.1.31
```

#### 场景3: Docker 镜像构建
```bash
# 构建 Docker 镜像
./scripts/build.sh --with-docker

# 构建指定版本的 Docker 镜像
./scripts/build.sh --with-docker --version v0.1.31
```

#### 场景4: 清理重建
```bash
# 清理后重新构建
./scripts/build.sh --clean --with-frontend
```

## Docker 构建

### 构建 Docker 镜像

```bash
# 构建 Docker 镜像
./scripts/build.sh --with-docker
```

### 输出结果

构建完成后会生成 Docker 镜像：
- `robustmq/robustmq:{version}` - 版本标签镜像
- `robustmq/robustmq:latest` - 最新标签镜像

### 前置条件

- `docker` 已安装并运行
- Docker 构建基于 `docker/Dockerfile` 文件
- 脚本会自动检查 Docker 命令和守护进程状态

## 输出结果

### 普通构建
构建完成后会在 `build/` 目录生成：
- `robustmq-{version}-{platform}.tar.gz` - 安装包

### 包结构

```text
robustmq-0.1.35-darwin-arm64/
├── bin/           # 源码 bin 目录（启动脚本等）
├── libs/          # Rust 编译的二进制文件
│   ├── broker-server
│   ├── cli-command
│   └── cli-bench
├── config/        # 源码 config 目录（配置文件）
├── dist/          # 前端构建产物（如果包含前端）
├── LICENSE        # 许可证文件
└── package-info.txt # 包信息文件（包含版本信息）
```

## 发布流程

### release.sh 脚本

`scripts/release.sh` 用于自动化 GitHub 发布流程：

#### 基本用法

```bash
# 发布新版本（创建 release 并上传包）
./scripts/release.sh

# 只上传当前系统的包到已存在的 release
./scripts/release.sh --upload-only

# 指定版本
./scripts/release.sh --version v0.1.31
```

#### 环境变量

```bash
# 必需：GitHub 个人访问令牌
export GITHUB_TOKEN="your_github_token"
```

#### 发布步骤

1. **设置 GitHub Token**
   ```bash
   export GITHUB_TOKEN="your_token_here"
   ```

2. **执行发布**
   ```bash
   # 发布新版本
   ./scripts/release.sh
   
   # 或只上传包到已存在的版本
   ./scripts/release.sh --upload-only
   ```

## 常见问题

### Q: Docker 构建失败

**A:** 检查以下项目：

1. Docker 是否已安装并运行
2. `docker/Dockerfile` 文件是否存在
3. Docker 守护进程是否正常运行

```bash
# 检查 Docker 状态
docker info
```

### Q: GitHub 发布失败

**A:** 检查以下项目：

1. GitHub Token 是否正确设置
2. Token 是否有足够的权限
3. 网络连接是否正常

### Q: 构建产物在哪里

**A:** 默认在 `./build/` 目录下：

```bash
ls -la build/
# 查看所有构建产物
find build/ -name "*.tar.gz"
```

### Q: 如何验证构建产物

**A:** 解压并测试：

```bash
# 解压并测试
tar -xzf robustmq-0.1.35-darwin-arm64.tar.gz
cd robustmq-0.1.35-darwin-arm64

# 测试二进制文件
./libs/broker-server --help
./libs/cli-command --help
./libs/cli-bench --help

# 查看包信息
cat package-info.txt
```

## 注意事项

- ✅ 只构建当前系统平台
- ✅ 不指定 `--version` 时自动从项目根目录的 Cargo.toml 文件中读取版本
- ✅ 使用 `cargo build --release` 进行 Rust 项目构建
- ✅ 前端构建是可选的，会自动从 GitHub 克隆 robustmq-copilot 代码
- ✅ 每次构建前端时都会自动拉取最新代码（git pull）
- ✅ Docker 构建基于 `docker/Dockerfile` 文件
- ✅ Docker 构建会检查 Docker 命令和守护进程状态
- ❌ 不支持交叉编译
