# 构建与打包

## 环境依赖

| 工具 | 用途 | 是否必需 |
|------|------|----------|
| `make` | 执行仓库的构建和发布目标 | 必需 |
| Rust (`cargo`, `rustup`) | 编译 Rust 代码 | 必需 |
| `pnpm` + `git` | 构建前端（Web 控制台）| 构建完整包时需要 |
| `docker` | 构建 Docker 镜像 | 可选 |
| `GITHUB_TOKEN` | 发布到 GitHub Releases | 发布时需要 |

---

## 常用命令

| 命令 | 说明 |
|------|------|
| `make build` | 构建当前平台的二进制包（不含前端）|
| `make build-full` | 构建包含前端 Web 控制台的完整包 |
| `make release` | 构建并发布 Release 包（需要发布凭据） |

版本号自动从 `Cargo.toml` 读取，无需手动指定。

## 构建并联动 Web 控制台

RobustMQ Web 控制台位于独立的 [`robustmq-copilot`](https://github.com/robustmq/robustmq-copilot)
仓库中。后端的 Admin HTTP 服务提供 API；完整二进制包会将编译后的前端静态文件放入 `dist/`，
由同一个 Admin HTTP 端口提供页面和 API。

完整包构建需要 Rust（`cargo`、`rustup`）、`make`、`git`、Node.js 20 或更高版本，以及
`pnpm` 10 或更高版本。

### 构建包含前端的完整包

在 RobustMQ 仓库根目录执行：

```bash
make build-full
```

该命令会将 `robustmq-copilot` 克隆或更新到被忽略的 `build/robustmq-copilot/`，运行
`pnpm install --no-frozen-lockfile` 和 `pnpm ui:build`，编译 Rust 服务端，再将
`packages/web-ui/dist/` 复制到发布包的 `dist/` 目录。最终 tarball 位于
`build/robustmq-{version}-{platform}.tar.gz`。

解压并启动完整包（以下文件名以 macOS ARM64 的版本为例）：

```bash
tar -xzf build/robustmq-0.4.11-darwin-arm64.tar.gz
cd robustmq-0.4.11-darwin-arm64
./bin/robust-server start
```

启动脚本会读取 `config/server.toml` 中的 `http_port`，并写入 `dist/config.js`。控制台使用
当前页面的主机名和该端口访问 Admin HTTP API。默认页面地址为 `http://127.0.0.1:58080/`。

验证页面和后端：

```bash
curl -i http://127.0.0.1:58080/health/ready
curl -i http://127.0.0.1:58080/
```

`/health/ready` 应返回 `HTTP 200`，根路径应返回前端 HTML。根路径返回 `404` 通常表示运行的是
不含 `dist/` 的 API-only 包，或启动进程时当前目录不是包含 `dist/` 的包根目录。Admin API
仍可能正常工作；需要 Web 控制台时请使用 `make build-full` 并从完整包根目录启动。

### 前后端分开开发

先在 RobustMQ 仓库启动后端：

```bash
cargo run --package cmd --bin broker-server -- --conf=config/server.toml
```

然后在另一个终端克隆并启动前端：

```bash
git clone https://github.com/robustmq/robustmq-copilot.git
cd robustmq-copilot
pnpm install
ROBUSTMQ_API_PORT=58080 pnpm ui:dev
```

开发页面默认地址为 `http://127.0.0.1:4000/`。连接其他后端时可使用完整 URL：

```bash
ROBUSTMQ_API_URL=http://192.168.1.10:58080 pnpm ui:dev
```

`ROBUSTMQ_API_URL` 优先于 `ROBUSTMQ_API_PORT`。未设置完整 URL 时，前端使用当前页面主机名和
配置端口。Admin HTTP 已启用 CORS，开发时无需额外代理配置。

### 构建仅含 API 的后端包

```bash
make build
```

该包不包含 `dist/`，适合 API 或协议测试。其根路径可能返回 `404`，但这不表示 Admin HTTP
API 未启动；可以请求 `/health/ready` 验证。需要 Web 控制台时应使用 `make build-full`。

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
docker build -f docker/robustmq/Dockerfile --build-arg VERSION=v0.4.11 \
  -t robustmq:local .
```

---

## 发布到 GitHub Releases

```bash
export GITHUB_TOKEN="your_token"
make release
```

需要 Token 具备 `write:packages` 和 `contents:write` 权限。发布成功后可在 [Releases 页面](https://github.com/robustmq/robustmq/releases) 下载。

> 注意：`make release` 只构建当前系统平台，不支持交叉编译。多平台发布由 GitHub Actions CI 并行完成。
