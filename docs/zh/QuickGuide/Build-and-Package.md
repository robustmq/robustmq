# RobustMQ 编译打包指南

本指南将详细介绍如何编译和打包 RobustMQ，包括本地构建、跨平台编译和发布流程。

## 目录

- [环境准备](#环境准备)
- [快速开始](#快速开始)
- [构建脚本详解](#构建脚本详解)
- [支持的平台](#支持的平台)
- [构建组件](#构建组件)
- [发布流程](#发布流程)
- [常见问题](#常见问题)

## 环境准备

### 必需依赖

#### Rust 环境（用于构建服务器组件）

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

#### Go 环境（用于构建 Kubernetes Operator）

```bash
# 安装 Go (版本 >= 1.19)
# 从 https://golang.org/dl/ 下载并安装

# 验证安装
go version
```

#### 其他工具

```bash
# 必需工具
sudo apt-get install curl jq git tar  # Ubuntu/Debian
# 或
brew install curl jq git  # macOS
```

安装 `protoc`

```bash
# Ubuntu(已验证)
PB_REL="https://github.com/protocolbuffers/protobuf/releases"
curl -LO $PB_REL/download/v26.1/protoc-26.1-linux-x86_64.zip
# 替换本机版本
unzip protoc-26.1-linux-x86_64.zip -d $HOME/.local
```

解决: `failed to run custom build command for zstd-sysv2.0.15+zstd.1.5.7`

```bash
# 解决: failed to run custom build command for zstd-sysv2.0.15+zstd.1.5.7
# Ubuntu(已验证)
sudo apt install build-essential clang pkg-config libssl-dev
# Fedora/RHEL
sudo dnf install clang pkg-config zstd-devel # Fedora/RHEL
# macOS
brew install zstd pkg-config 
```

### 可选依赖

#### 跨平台编译工具链

```bash
# 安装交叉编译目标
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-pc-windows-gnu
```

## 快速开始

### 1. 克隆项目

```bash
git clone https://github.com/robustmq/robustmq.git
cd robustmq
```

### 2. 本地构建（当前平台）

```bash
# 构建服务器组件（默认）
./scripts/build.sh

# 构建所有组件
./scripts/build.sh --component all

# 构建特定组件
./scripts/build.sh --component server
./scripts/build.sh --component operator
```

### 3. 查看构建结果

```bash
ls -la build/
# 输出示例：
# robustmq-v1.0.0-darwin-arm64.tar.gz
# robustmq-v1.0.0-darwin-arm64.tar.gz.sha256
```

## 构建脚本详解

### build.sh 脚本

`scripts/build.sh` 是主要的构建脚本，支持多种构建选项：

#### 基本用法

```bash
./scripts/build.sh [OPTIONS]
```

#### 主要选项

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `-h, --help` | 显示帮助信息 | - |
| `-v, --version VERSION` | 指定构建版本 | 自动从 git 检测 |
| `-c, --component COMP` | 构建组件：server/operator/all | server |
| `-p, --platform PLATFORM` | 目标平台 | 自动检测当前平台 |
| `-a, --all-platforms` | 构建所有支持的平台 | - |
| `-t, --type TYPE` | 构建类型：release/debug | release |
| `-o, --output DIR` | 输出目录 | ./build |
| `--clean` | 构建前清理输出目录 | false |
| `--verbose` | 启用详细输出 | false |
| `--dry-run` | 显示将要构建的内容但不实际构建 | false |
| `--no-parallel` | 禁用并行构建 | false |

#### 使用示例

```bash
# 构建当前平台的发布版本
./scripts/build.sh

# 构建特定平台的调试版本
./scripts/build.sh --platform linux-amd64 --type debug

# 构建所有平台
./scripts/build.sh --all-platforms

# 构建特定版本
./scripts/build.sh --version v1.0.0

# 清理构建
./scripts/build.sh --clean

# 查看将要构建的内容（不实际构建）
./scripts/build.sh --dry-run --all-platforms
```

## 支持的平台

### 服务器组件（Rust）

| 平台标识 | 操作系统 | 架构 | Rust Target |
|----------|----------|------|-------------|
| `linux-amd64` | Linux | x86_64 | x86_64-unknown-linux-gnu |
| `linux-amd64-musl` | Linux | x86_64 (musl) | x86_64-unknown-linux-musl |
| `linux-arm64` | Linux | ARM64 | aarch64-unknown-linux-gnu |
| `linux-arm64-musl` | Linux | ARM64 (musl) | aarch64-unknown-linux-musl |
| `linux-armv7` | Linux | ARMv7 | armv7-unknown-linux-gnueabihf |
| `darwin-amd64` | macOS | x86_64 | x86_64-apple-darwin |
| `darwin-arm64` | macOS | ARM64 (Apple Silicon) | aarch64-apple-darwin |
| `windows-amd64` | Windows | x86_64 | x86_64-pc-windows-gnu |
| `windows-386` | Windows | x86 | i686-pc-windows-gnu |
| `windows-arm64` | Windows | ARM64 | aarch64-pc-windows-gnullvm |
| `freebsd-amd64` | FreeBSD | x86_64 | x86_64-unknown-freebsd |

### Operator 组件（Go）

| 平台标识 | 操作系统 | 架构 | Go Target |
|----------|----------|------|-----------|
| `linux-amd64` | Linux | x86_64 | linux/amd64 |
| `linux-arm64` | Linux | ARM64 | linux/arm64 |
| `linux-armv7` | Linux | ARMv7 | linux/arm |
| `darwin-amd64` | macOS | x86_64 | darwin/amd64 |
| `darwin-arm64` | macOS | ARM64 | darwin/arm64 |
| `windows-amd64` | Windows | x86_64 | windows/amd64 |
| `windows-386` | Windows | x86 | windows/386 |
| `freebsd-amd64` | FreeBSD | x86_64 | freebsd/amd64 |

## 构建组件

### 服务器组件

服务器组件包含以下二进制文件：

- `broker-server` - RobustMQ 主服务器
- `cli-command` - 命令行管理工具
- `cli-bench` - 性能测试工具

#### 构建过程

1. 检查 Rust 环境和目标平台
2. 安装必要的 Rust 目标（如果未安装）
3. 使用 `cargo build` 编译二进制文件
4. 创建包结构并复制文件
5. 生成 tarball 和校验和

#### 包结构

```text
robustmq-v1.0.0-linux-amd64/
├── bin/           # 启动脚本
├── libs/          # 二进制文件
├── config/        # 配置文件
├── docs/          # 文档
├── package-info.txt  # 包信息
└── version.txt    # 版本信息
```

### Operator 组件

Operator 组件是 Kubernetes 操作器，用于在 K8s 环境中管理 RobustMQ。

#### Operator 构建过程

1. 检查 Go 环境
2. 设置交叉编译环境变量
3. 使用 `go build` 编译二进制文件
4. 创建包结构并复制相关文件
5. 生成 tarball 和校验和

#### Operator 包结构

```text
robustmq-operator-v1.0.0-linux-amd64/
├── bin/           # 二进制文件
├── config/        # 配置文件
├── manifests/     # K8s 清单文件
├── docs/          # 文档
├── package-info.txt  # 包信息
└── version.txt    # 版本信息
```

## 发布流程

### release.sh 脚本

`scripts/release.sh` 用于自动化 GitHub 发布流程：

#### 主要功能

1. 从 `Cargo.toml` 提取版本号
2. 检查或创建 GitHub Release
3. 调用 `build.sh` 构建分发包
4. 上传 tarball 文件到 GitHub Release

#### 使用方法

```bash
# 基本用法
./scripts/release.sh

# 指定版本
./scripts/release.sh --version v1.0.0

# 指定平台
./scripts/release.sh --platform linux-amd64

# 构建所有平台
./scripts/release.sh --platform all

# 干运行（查看将要执行的操作）
./scripts/release.sh --dry-run

# 强制重新创建已存在的 Release
./scripts/release.sh --force
```

#### 环境变量

```bash
# 必需：GitHub 个人访问令牌
export GITHUB_TOKEN="your_github_token"

# 可选：GitHub 仓库（默认：robustmq/robustmq）
export GITHUB_REPO="owner/repo"

# 其他选项
export VERSION="v1.0.0"
export PLATFORM="linux-amd64"
export DRY_RUN="true"
export FORCE="true"
export VERBOSE="true"
export SKIP_BUILD="true"
```

#### GitHub Token 权限

创建 GitHub 个人访问令牌时需要以下权限：

- `repo` - 完整控制私有仓库
- `public_repo` - 访问公共仓库

### 发布步骤

1. **准备环境**

   ```bash
   # 设置 GitHub Token
   export GITHUB_TOKEN="your_token_here"
   
   # 确保在项目根目录
   cd robustmq
   ```

2. **执行发布**

   ```bash
   # 发布当前版本到所有平台
   ./scripts/release.sh --platform all
   
   # 或发布特定平台
   ./scripts/release.sh --platform linux-amd64
   ```

3. **验证发布**

   - 访问 GitHub Releases 页面
   - 检查上传的文件
   - 验证下载链接

## 常见问题

### Q: 构建失败，提示缺少 Rust 目标

**A:** 安装对应的 Rust 目标：

```bash
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
# 等等...
```

### Q: 交叉编译失败

**A:** 确保安装了对应的工具链：

```bash
# Linux 上编译 Windows
sudo apt-get install gcc-mingw-w64-x86-64

# macOS 上编译 Linux
brew install FiloSottile/musl-cross/musl-cross
```

### Q: GitHub 发布失败

**A:** 检查以下项目：

1. GitHub Token 是否正确设置
2. Token 是否有足够的权限
3. 网络连接是否正常
4. 仓库是否存在且可访问

### Q: 如何跳过构建直接上传现有包

**A:** 使用 `--skip-build` 选项：

```bash
./scripts/release.sh --skip-build
```

### Q: 如何查看详细的构建日志

**A:** 使用 `--verbose` 选项：

```bash
./scripts/build.sh --verbose
./scripts/release.sh --verbose
```

### Q: 构建产物在哪里

**A:** 默认在 `./build/` 目录下：

```bash
ls -la build/
# 查看所有构建产物
find build/ -name "*.tar.gz"
```

### Q: 如何验证构建产物

**A:** 使用校验和文件：

```bash
# 验证 SHA256
sha256sum -c robustmq-v1.0.0-linux-amd64.tar.gz.sha256

# 解压并测试
tar -xzf robustmq-v1.0.0-linux-amd64.tar.gz
cd robustmq-v1.0.0-linux-amd64
./libs/broker-server --help
```

## 高级用法

### 自定义构建配置

```bash
# 使用环境变量配置
export VERSION="v1.0.0"
export BUILD_TYPE="release"
export OUTPUT_DIR="/custom/build/path"
export VERBOSE="true"

./scripts/build.sh
```

### 并行构建多个平台

```bash
# 构建多个特定平台（并行）
./scripts/build.sh --platform linux-amd64 &
./scripts/build.sh --platform darwin-arm64 &
./scripts/build.sh --platform windows-amd64 &
wait
```

### 集成到 CI/CD

```yaml
# GitHub Actions 示例
- name: Build RobustMQ
  run: |
    export GITHUB_TOKEN=${{ secrets.GITHUB_TOKEN }}
    ./scripts/release.sh --platform all
```

## 总结

RobustMQ 提供了完整的构建和发布工具链：

- **build.sh**: 灵活的构建脚本，支持多平台、多组件构建
- **release.sh**: 自动化发布脚本，集成 GitHub Releases
- **跨平台支持**: 支持主流操作系统和架构
- **组件化构建**: 可单独构建服务器或 Operator 组件
- **完整的包管理**: 自动生成包信息、校验和等

通过本指南，您可以轻松地构建和发布 RobustMQ 到各种平台，满足不同部署环境的需求。
