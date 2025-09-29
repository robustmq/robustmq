# Build Script 使用说明

## 快速开始

### 1. 基本构建

```bash
# 使用 Cargo.toml 中的版本构建当前平台的包
./scripts/build.sh

# 指定版本构建
./scripts/build.sh --version v0.1.30
```

### 2. 包含前端构建

```bash
# 构建包含前端的完整包
./scripts/build.sh --with-frontend
```

### 3. Docker 镜像构建

```bash
# 构建 Docker 镜像
./scripts/build.sh --with-docker
```

### 4. 清理构建

```bash
# 清理后重新构建
./scripts/build.sh --clean
```

## 前置条件

1. **Rust 环境**
   - `cargo` 和 `rustup` 已安装

2. **前端构建（可选）**
   - `pnpm` 已安装（仅在使用 `--with-frontend` 时需要）
   - `git` 已安装（用于自动克隆前端代码）

3. **Docker 构建（可选）**
   - `docker` 已安装并运行（仅在使用 `--with-docker` 时需要）

> **注意**: 如果缺少必要的工具，脚本会显示详细的安装指令，包括不同操作系统的安装命令。

## 完整用法

```bash
./scripts/build.sh [OPTIONS]

选项:
  -h, --help              显示帮助
  -v, --version VERSION   指定版本（默认从项目根目录 Cargo.toml 文件读取）
  --with-frontend         包含前端构建
  --with-docker           构建 Docker 镜像（基于 docker/Dockerfile）
  --clean                 清理构建目录

示例:
  # 基本构建
  ./scripts/build.sh
  
  # 构建 Docker 镜像
  ./scripts/build.sh --with-docker
  
  # 构建包含前端的完整包
  ./scripts/build.sh --with-frontend
  
  # 指定版本构建
  ./scripts/build.sh --version v0.1.31
```

## 输出结果

### 普通构建
构建完成后会在 `build/` 目录生成：
- `robustmq-{version}-{platform}.tar.gz` - 安装包

### 包结构
```text
robustmq-{version}-{platform}/
├── bin/           # 源码 bin 目录（启动脚本等）
├── libs/          # Rust 编译的二进制文件
├── config/        # 源码 config 目录（配置文件）
├── dist/          # 前端构建产物（如果包含前端）
├── LICENSE        # 许可证文件
└── package-info.txt # 包信息文件（包含版本信息）
```

### Docker 构建
构建完成后会生成 Docker 镜像：
- `robustmq/robustmq:{version}` - 版本标签镜像
- `robustmq/robustmq:latest` - 最新标签镜像

## 使用场景

### 场景1: 开发测试
```bash
# 使用 Cargo.toml 版本快速构建测试包
./scripts/build.sh
```

### 场景2: 发布准备
```bash
# 使用 Cargo.toml 版本构建完整发布包（自动克隆前端代码）
./scripts/build.sh --with-frontend

# 指定版本构建完整发布包
./scripts/build.sh --with-frontend --version v0.1.31
```

### 场景3: Docker 镜像构建
```bash
# 构建 Docker 镜像
./scripts/build.sh --with-docker

# 构建指定版本的 Docker 镜像
./scripts/build.sh --with-docker --version v0.1.31
```

### 场景4: 清理重建
```bash
# 清理后重新构建
./scripts/build.sh --clean --with-frontend
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
