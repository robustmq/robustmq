# 构建与打包

## 环境依赖

| 工具 | 用途 | 是否必需 |
|------|------|----------|
| Rust (`cargo`, `rustup`) | 编译 Rust 代码 | 必需 |
| `pnpm` + `git` | 构建前端（Web 控制台）| 构建完整包时需要 |
| `docker` | 构建 Docker 镜像 | 可选 |
| `GITHUB_TOKEN` | 发布到 GitHub Releases | 发布时需要 |

---

## 常用命令

| 命令 | 说明 |
|------|------|
| `make build` | 构建当前平台的二进制包（不含前端）|
| `make build-full` | 构建完整包（含前端 Web 控制台）|
| `make release` | 构建完整包并发布到 GitHub Releases |

版本号自动从 `Cargo.toml` 读取，无需手动指定。

---

## 产物结构

构建产物位于 `build/` 目录，格式为 `robustmq-{version}-{platform}.tar.gz`，解压后结构如下：

```
robustmq-v0.3.0-linux-amd64/
  bin/
    robust-server     ← 用户命令（加入 PATH）
    robust-ctl
    robust-bench
  libs/
    broker-server     ← 实际二进制（由 bin/ 脚本调用）
    cli-command
    cli-bench
  config/
    server.toml
  dist/               ← 前端静态文件（仅 build-full 包含）
```

---

## Docker 镜像

镜像在每次 Release 时由 CI 自动构建并推送，支持 `linux/amd64` 和 `linux/arm64`：

```bash
docker pull ghcr.io/robustmq/robustmq:latest
docker pull ghcr.io/robustmq/robustmq:v0.3.0
```

本地手动构建：

```bash
docker build -f docker/robustmq/Dockerfile -t robustmq:local .
```

---

## 发布到 GitHub Releases

```bash
export GITHUB_TOKEN="your_token"
make release
```

需要 Token 具备 `write:packages` 和 `contents:write` 权限。发布成功后可在 [Releases 页面](https://github.com/robustmq/robustmq/releases) 下载。

> 注意：`make release` 只构建当前系统平台，不支持交叉编译。多平台发布由 GitHub Actions CI 并行完成。
