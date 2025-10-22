# RobustMQ Dependency Cache Image

这个目录包含用于构建 Rust 依赖缓存镜像的所有文件。

## 文件说明

- `Dockerfile.deps` - 主要的依赖缓存镜像构建文件
- `install-deps.sh` - 系统依赖安装脚本（带镜像源回退）
- `install-runtime.sh` - 运行时依赖安装脚本（带镜像源回退）

## 用途

这个镜像用于加速 CI/CD 构建过程，包含所有预编译的 Rust 依赖项。

## 构建命令

```bash
make docker-deps
```

## 镜像信息

- **镜像名称**: `ghcr.io/socutes/robustmq/rust-deps:latest`
- **大小**: ~8-10GB
- **构建时间**: 20-40 分钟（首次）
- **包含**: 864 个预编译依赖项
